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

error_chain! {
    foreign_links {
        Io(io::Error);
        Hex(::std::num::ParseIntError);
    }
    errors {
        Parse {
            description("could not find next element to parse")
        }
        NoResponse {
            description("client should not expect a response")
        }
    }
}

fn parse_next<T, I: Iterator<Item=T>>(it: &mut I) -> Result<T> {
    it.next().ok_or(ErrorKind::Parse.into())
}

fn parse_next_hex<'a, I: Iterator<Item=&'a str>>(it: &mut I) -> Result<u32> {
    Ok(utils::from_hex(parse_next(it)?)?)
}


fn cmd_step(ctx: &mut dbgcore::DbgContext) -> Result<String> {
    ctx.hw().step();
    Ok(make_resp_signal(BreakReason::LimitReached, ctx))
}

fn cmd_continue(ctx: &mut dbgcore::DbgContext) -> Result<String> {
    ctx.resume();
    bail!(ErrorKind::NoResponse)
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

fn handle_gdb_cmd_q(cmd: &str, ctx: &mut dbgcore::DbgContext) -> Result<String> {
    let mut s = cmd.splitn(2, ':');
    let ty = parse_next(&mut &mut s)?;
    let params = parse_next(&mut &mut s)?;
    let mut out = String::new();
    match ty {
        "Supported" => {
            out += "PacketSize=400;BreakpointCommands+;swbreak+";
        }
        _ => warn!("GDB client tried to run unsupported `q` command {}", ty)
    }
    Ok(out)
}

fn handle_gdb_cmd_v(cmd: &str, ctx: &mut dbgcore::DbgContext) -> Result<String> {
    let mut s = cmd.splitn(2, |c| c == ',' || c == ':' || c == ';');
    let ty = parse_next(&mut &mut s)?;
    let params = parse_next(&mut &mut s)?;
    let mut out = String::new();
    match ty {
        "Cont" => {
            let threads = params.split(';');
            for thread in threads {
                let mut thread_data = thread.split(':');
                let action = parse_next(&mut &mut thread_data)?;
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
    Ok(out)
}


fn handle_gdb_cmd(cmd: &str, debugger: &mut dbgcore::DbgCore) -> Result<String> {
    let ty = parse_next(&mut &mut cmd.chars())?;
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
                let reg_data = utils::from_hex(&params[param_reg_range])?;
                hw.write_reg(reg, reg_data.swap_bytes());
            }
            out += "OK";
        }
        'm' => {
            let hw = hw_ctx.hw();
            let mut params = params.split(',');
            let addr = parse_next_hex(&mut params)?;
            let size = parse_next_hex(&mut params)?;
            let mut buf = [0u8];
            for b in 0..size {
                hw.read_mem(addr+b, &mut buf);
                out += &format!("{:02X}", buf[0]);
            }
        }
        'M' => {
            let mut hw = hw_ctx.hw();
            let mut params = params.split(|c| c == ',' || c == ':');
            let addr = parse_next_hex(&mut params)?;
            let size = parse_next_hex(&mut params)?;
            let data = parse_next(&mut params)?;
            for b in 0..size {
                let data_byte_range = 2*(b as usize)..2*((b as usize)+1);
                let byte = utils::from_hex(&data[data_byte_range])? as u8;
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
            let brk_ty = parse_next(&mut params)?;
            let addr = parse_next_hex(&mut params)?;
            let kind = parse_next(&mut params)?;
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
    Ok(out)
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

fn write_gdb_packet(data: &str, stream: &mut TcpStream) -> Result<()> {
    let checksum = data.bytes().fold(Checksum(0), |checksum, b| checksum + b);

    trace!("Replying with GDB packet: ${}#{:02X}", data, checksum.0);
    write!(stream, "${}#{:02X}", data, checksum.0)?;
    stream.flush()?;
    Ok(())
}

fn handle_gdb_packet(data: &[u8], stream: &mut TcpStream, debugger: &mut dbgcore::DbgCore) -> Result<()> {
    trace!("Recieving GDB packet: {}", str::from_utf8(data).unwrap());
    let mut it = data.iter().cloned();
    loop {
        match load_packet(&mut it) {
            PacketType::Command(cmd) => {
                stream.write(b"+")?;
                stream.flush()?;
                match handle_gdb_cmd(&cmd, debugger) {
                    Ok(out) => write_gdb_packet(&out, stream)?,
                    Err(e) => {
                        if let &ErrorKind::NoResponse = e.kind() {}
                        else { return Err(e) }
                    }
                }
            }
            PacketType::CtrlC => debugger.ctx().pause(),
            PacketType::Malformed => {
                stream.write(b"-")?;
                stream.flush()?;
                return Ok(())
            }
        }
    }
}


const TOKEN_LISTENER: mio::Token = mio::Token(1024);
const TOKEN_CLIENT: mio::Token = mio::Token(1025);

pub struct GdbStub {
    debugger: dbgcore::DbgCore,
    gdb_thread: Option<thread::JoinHandle<msgs::Client<Message>>>
}

impl GdbStub {
    pub fn new(msg_client: msgs::Client<Message>, debugger: dbgcore::DbgCore) -> GdbStub {
        let mut stub = GdbStub {
            debugger: debugger,
            gdb_thread: None
        };
        stub.start(msg_client);
        stub
    }

    pub fn start(&mut self, msg_client: msgs::Client<Message>) {
        let mut debugger = self.debugger.clone();
        self.gdb_thread = Some(thread::Builder::new().name("GDBStub".to_owned()).spawn(move || {
            use mio::Events;

            let poll = mio::Poll::new()
                .expect("Could not create mio polling instance!");
            let listener = TcpListener::bind(&"127.0.0.1:4567".parse().unwrap())
                .expect("Could not bind TcpListener to port!");

            poll.register(&listener, TOKEN_LISTENER, mio::Ready::readable(), mio::PollOpt::edge())
                .expect("Could not register TcpListener to mio!");

            let mut connection = Connection {
                listener: &listener,
                poll: poll,
                socket: None,
            };

            let mut events = Events::with_capacity(1024);

            info!("Starting GDB stub on port 4567...");

            't: loop {
                connection.poll.poll(&mut events, Some(Duration::from_millis(100)))
                    .expect("Could not poll for network events!");

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
            t.join().unwrap();
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
                                             mio::PollOpt::edge())
                        .expect("Could not register TCP client to mio!");
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