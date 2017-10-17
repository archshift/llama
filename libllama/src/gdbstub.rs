use std::io::{self, Read, Write};
use std::ops::{Add, AddAssign};
use std::str;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;

use mio;
use mio::tcp::{TcpListener, TcpStream};

use cpu::BreakReason;
use dbgcore;
use hwcore::Message;
use msgs;
use utils;

fn cmd_step(ctx: &mut dbgcore::DbgContext) -> Option<String> {
    ctx.hw().step();
    Some(make_resp_signal(BreakReason::LimitReached, ctx))
}

fn cmd_continue(ctx: &mut dbgcore::DbgContext) -> Option<String> {
    ctx.resume();
    None
}

fn make_resp_signal(reason: BreakReason, ctx: &mut dbgcore::DbgContext) -> String {
    let hw = ctx.hw();
    let reason_str = match reason {
        BreakReason::Breakpoint => format!(";{}:", "swbreak"),
        _ => String::new(),
    };
    format!("T05{:02X}:{:08X};{:02X}:{:08X}{}", 15, hw.read_reg(15).swap_bytes(),
                                                13, hw.read_reg(13).swap_bytes(),
                                                reason_str)
}

fn handle_gdb_cmd_q(cmd: &str, ctx: &mut dbgcore::DbgContext) -> Option<String> {
    let mut s = cmd.splitn(2, ':');
    let ty = s.next().unwrap();
    let params = s.next().unwrap_or("");
    let mut out = String::new();
    match ty {
        "Supported" => {
            out += "PacketSize=400;BreakpointCommands+;swbreak+";
        }
        _ => warn!("GDB client tried to run unsupported `q` command {}", ty)
    }
    Some(out)
}

fn handle_gdb_cmd_v(cmd: &str, ctx: &mut dbgcore::DbgContext) -> Option<String> {
    let mut s = cmd.splitn(2, |c| c == ',' || c == ':' || c == ';');
    let ty = s.next().unwrap();
    let params = s.next().unwrap_or("");
    let mut out = String::new();
    match ty {
        "Cont" => {
            let threads = params.split(';');
            for thread in threads {
                let mut thread_data = thread.split(':');
                let action = thread_data.next().unwrap();
                let thread_name = thread_data.next();
                if let Some(name) = thread_name {
                    assert!(name == "-1");
                }
                match action {
                    "c" => return cmd_continue(ctx),
                    "s" => return cmd_step(ctx),
                    _ => warn!("GDB client tried to run unsupported `vCont` action {}", action)
                }
            }
        }
        "Cont?" => {
            let supported = ["c", "s"];
            out += "vCont";
            for ty in supported.iter() {
                out += ";";
                out += ty;
            }
        }
        _ => warn!("GDB client tried to run unsupported `v` command {}", ty)
    }
    Some(out)
}


fn handle_gdb_cmd(cmd: &str, debugger: &mut dbgcore::DbgCore) -> Option<String> {
    let ty = cmd.chars().next().unwrap();
    let params = &cmd[1..];
    let mut out = String::new();
    let mut hw_ctx = debugger.ctx();
    hw_ctx.pause();
    match ty {
        'g' => {
            let hw = hw_ctx.hw();
            for reg in 0..16 {
                out += &format!("{:08X}", hw.read_reg(reg).swap_bytes());
            }
        }
        'G' => {
            let mut hw = hw_ctx.hw();
            for reg in 0..16 {
                let param_reg_range = 8*reg..8*(reg+1);
                let reg_data = utils::from_hex(&params[param_reg_range]).unwrap();
                hw.write_reg(reg, reg_data.swap_bytes());
            }
            out += "OK";
        }
        'm' => {
            let hw = hw_ctx.hw();
            let mut params = params.split(',');
            let addr = utils::from_hex(params.next().unwrap()).unwrap();
            let size = utils::from_hex(params.next().unwrap()).unwrap();
            let mut buf = [0u8];
            for b in 0..size {
                hw.read_mem(addr+b, &mut buf);
                out += &format!("{:02X}", buf[0]);
            }
        }
        'M' => {
            let mut hw = hw_ctx.hw();
            let mut params = params.split(|c| c == ',' || c == ':');
            let addr = utils::from_hex(params.next().unwrap()).unwrap();
            let size = utils::from_hex(params.next().unwrap()).unwrap();
            let data = params.next().unwrap();
            for b in 0..size {
                let data_byte_range = 2*(b as usize)..2*((b as usize)+1);
                let byte = utils::from_hex(&data[data_byte_range]).unwrap() as u8;
                hw.write_mem(addr+b, &[byte]);
            }
            out += "OK";
        }
        'q' => {
            return handle_gdb_cmd_q(params, &mut hw_ctx);
        }
        's' => {
            return cmd_step(&mut hw_ctx);
        }
        'c' => {
            return cmd_continue(&mut hw_ctx);
        }
        'v' => {
            return handle_gdb_cmd_v(params, &mut hw_ctx);
        }
        'z' | 'Z' => {
            let mut hw = hw_ctx.hw();
            let mut params = params.split(',');
            let brk_ty = params.next().unwrap();
            let addr = utils::from_hex(params.next().unwrap()).unwrap();
            let kind = params.next().unwrap();
            assert!(brk_ty == "0");
            if ty == 'Z' {
                hw.set_breakpoint(addr);
            } else {
                hw.del_breakpoint(addr);
            }
            out += "OK";
        }
        // '?' => {
        //     out += &make_resp_signal(&mut hw_ctx);
        // }
        x => {
            warn!("GDB client tried to run unsupported command {}", x);
        }
    }
    Some(out)
}

#[derive(Clone, Copy)]
struct Checksum(pub u32);
impl Add<u8> for Checksum {
    type Output = Checksum;
    fn add(self, b: u8) -> Checksum {
        Checksum((self.0 + (b as u32)) % 256)
    }
}
impl AddAssign<u8> for Checksum {
    fn add_assign(&mut self, b: u8) {
        self.0 = (*self + b).0;
    }
}

enum PacketType {
    Command(String),
    CtrlC,
    Malformed
}

fn load_packet<I: Iterator<Item = u8>>(it: &mut I) -> PacketType {
    let mut it = it.skip_while(|b| *b != 0x03 && *b != b'$');
    match it.next() {
        Some(0x3) => return PacketType::CtrlC,
        Some(b'$') => {}
        _ => return PacketType::Malformed
    }

    let mut string = String::new();
    let mut checksum = Checksum(0);
    for b in it.by_ref().take_while(|b| *b != b'#') {
        string.push(b as char);
        checksum += b;
    }

    if let (Some(top), Some(bot)) = (it.next(), it.next()) {
        let packet_checksum = str::from_utf8(&[top, bot]).ok()
            .and_then(|s| utils::from_hex(s).ok());
        if Some(checksum.0) == packet_checksum {
            return PacketType::Command(string)
        }
    }
    return PacketType::Malformed
}

fn write_gdb_packet(data: &str, stream: &mut TcpStream) {
    let checksum = data.bytes().fold(Checksum(0), |checksum, b| checksum + b);
    write!(stream, "${}#{:02X}", data, checksum.0).unwrap();
    stream.flush().unwrap();
}

fn handle_gdb_packet(data: &[u8], stream: &mut TcpStream, debugger: &mut dbgcore::DbgCore) {
    trace!("Recieving GDB packet: {}", str::from_utf8(data).unwrap());
    let mut it = data.iter().cloned();
    loop {
        match load_packet(&mut it) {
            PacketType::Command(cmd) => {
                stream.write(b"+").unwrap();
                stream.flush().unwrap();
                if let Some(out) = handle_gdb_cmd(&cmd, debugger) {
                    write_gdb_packet(&out, stream);
                }
            }
            PacketType::CtrlC => debugger.ctx().pause(),
            PacketType::Malformed => {
                stream.write(b"-").unwrap();
                stream.flush().unwrap();
                return
            }
        }
    }
}


const TOKEN_LISTENER: mio::Token = mio::Token(1024);
const TOKEN_CLIENT: mio::Token = mio::Token(1025);

pub struct GdbStub {
    msg_client: Option<msgs::Client<Message>>,
    debugger: dbgcore::DbgCore,
    gdb_thread: Option<thread::JoinHandle<msgs::Client<Message>>>
}

impl GdbStub {
    pub fn new(msg_client: msgs::Client<Message>, debugger: dbgcore::DbgCore) -> GdbStub {
        GdbStub {
            msg_client: Some(msg_client),
            debugger: debugger,
            gdb_thread: None
        }
    }

    pub fn start(&mut self) {
        let msg_client = self.msg_client.take().unwrap();
        let mut debugger = self.debugger.clone();
        self.gdb_thread = Some(thread::Builder::new().name("GDBStub".to_owned()).spawn(move || {
            use mio::Events;

            let poll = mio::Poll::new().unwrap();
            let listener = TcpListener::bind(&"127.0.0.1:4567".parse().unwrap()).unwrap();
            poll.register(&listener, TOKEN_LISTENER, mio::Ready::readable(), mio::PollOpt::edge()).unwrap();

            let mut connection = Connection {
                listener: &listener,
                poll: poll,
                socket: None,
            };

            let mut events = Events::with_capacity(1024);

            info!("Starting GDB stub on port 4567...");

            't: loop {
                connection.poll.poll(&mut events, Some(Duration::from_millis(100))).unwrap();
                for event in &events {
                    handle_event(&event, &mut connection, |buf, stream| {
                        handle_gdb_packet(buf, stream, &mut debugger);
                    });
                }

                for msg in msg_client.try_iter() {
                    match msg {
                        Message::Quit => break 't,
                        Message::Arm9Halted(reason) => {
                            if let Some(ref mut stream) = connection.socket {
                                write_gdb_packet(&make_resp_signal(reason, &mut debugger.ctx()), stream);
                            }
                        }
                        _ => {}
                    }
                }
            }
            msg_client
        }).unwrap())
    }

    pub fn wait(&mut self) {
        if let Some(t) = self.gdb_thread.take() {
            self.msg_client = Some(t.join().unwrap());
        }
    }
}

struct Connection<'a> {
    listener: &'a TcpListener,
    poll: mio::Poll,
    socket: Option<TcpStream>,
}

fn handle_event<F>(event: &mio::Event, connection: &mut Connection, mut client_responder: F)
        where F: FnMut(&[u8], &mut TcpStream) {

    let mut buf = [0u8; 1024];
    match event.token() {
        TOKEN_LISTENER => {
            match connection.listener.accept() {
                Ok((socket, _)) => {
                    info!("GDB stub accepting connection");
                    connection.poll.register(&socket, TOKEN_CLIENT, mio::Ready::readable(),
                                             mio::PollOpt::edge()).unwrap();
                    connection.socket = Some(socket);
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    return; // Socket is not ready anymore, stop accepting
                }
                e => panic!("GDB stub IO error! {:?}", e)
            }
        }
        TOKEN_CLIENT => for i in 0..128 {
            match connection.socket.as_mut().unwrap().read(&mut buf) {
                Ok(0) => {
                    connection.socket = None;
                    break;
                }
                Ok(l) => client_responder(&buf[..l], connection.socket.as_mut().unwrap()),
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    continue; // Socket is not ready anymore, stop reading
                }
                e => panic!("GDB stub IO error! {:?}", e), // Unexpected error
            }
        },
        _ => unimplemented!()
    }
}