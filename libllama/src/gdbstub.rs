use std::io::{self, Read, Write};
use std::ops::{Add, AddAssign};
use std::str;
use std::thread;
use std::time::Duration;

use mio;
use mio::tcp::{TcpListener, TcpStream};

use cpu::BreakReason;
use dbgcore::{self, ActiveCpu::Arm9};
use hwcore::Message;
use msgs;
use utils;

#[derive(Debug, Error)]
pub enum ErrorKind {
    Hex(::std::num::ParseIntError),
    Io(io::Error),

    /// Client should not expect a response
    NoResponse,
    /// Could not find next element to parse
    Parse,
}

pub type Result<T> = ::std::result::Result<T, ErrorKind>;

fn parse_next<T, I: Iterator<Item=T>>(it: &mut I) -> Result<T> {
    it.next().ok_or(ErrorKind::Parse.into())
}

fn parse_next_hex<'a, I: Iterator<Item=&'a str>>(it: &mut I) -> Result<u32> {
    Ok(utils::from_hex(parse_next(it)?)?)
}


fn cmd_step(ctx: &mut GdbCtx) -> Result<String> {
    ctx.dbg.hw().step();
    let break_data = BreakData::new(BreakReason::LimitReached, ctx.dbg);
    let signal = break_data.to_signal();
    *ctx.last_halt = break_data;
    Ok(signal)
}

fn cmd_continue(ctx: &mut GdbCtx) -> Result<String> {
    ctx.dbg.resume();
    Err(ErrorKind::NoResponse)
}

struct BreakData {
    reason: BreakReason,
    r15: u32,
    r13: u32
}

impl BreakData {
    fn new(reason: BreakReason, dbg: &mut dbgcore::DbgContext) -> BreakData {
        let hw = dbg.hw();
        BreakData {
            reason: reason,
            r15: hw.pause_addr(),
            r13: hw.read_reg(13),
        }
    }

    fn to_signal(&self) -> String {
        let reason_str = match self.reason {
            BreakReason::Breakpoint => format!(";{}:", "swbreak"),
            _ => String::new(),
        };
        format!("T05{:02X}:{:08X};{:02X}:{:08X}{};", 15, self.r15.swap_bytes(),
                                                     13, self.r13.swap_bytes(),
                                                     reason_str)
    }
}

fn handle_gdb_cmd_q(cmd: &str, _ctx: &mut GdbCtx) -> Result<String> {
    let mut s = cmd.splitn(2, ':');
    let ty = parse_next(&mut s)?;
    let mut out = String::new();
    match ty {
        "fThreadInfo" => out += "m0000000000000001",
        "sThreadInfo" => out += "l",
        "C" => out += "QC0000000000000001",
        "Attached" => out += "1",
        "Supported" => {
            out += "PacketSize=400;BreakpointCommands+;swbreak+;vContSupported+";
        }
        _ => warn!("GDB client tried to run unsupported `q` command {}", ty)
    }
    Ok(out)
}

fn handle_gdb_cmd_v(cmd: &str, ctx: &mut GdbCtx) -> Result<String> {
    let mut s = cmd.splitn(2, |c| c == ',' || c == ':' || c == ';');
    let ty = parse_next(&mut s)?;
    let mut out = String::new();
    match ty {
        "Cont" => {
            let params = parse_next(&mut s)?;
            let threads = params.split(';');
            for thread in threads {
                let mut thread_data = thread.split(':');
                let action = parse_next(&mut thread_data)?;
                let thread_name = thread_data.next();
                if let Some(name) = thread_name {
                    match (name, utils::from_hex(name)) {
                        | ("-1", _)
                        | (_, Ok(0))
                        | (_, Ok(1)) => {}
                        
                        | (s, _) => panic!("Attempted to issue command on invalid thread id {}", s)
                    }
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


fn handle_gdb_cmd(cmd: &str, ctx: &mut GdbCtx) -> Result<String> {
    let ty = parse_next(&mut cmd.chars())?;
    let params = &cmd[1..];
    let mut out = String::new();
    ctx.dbg.pause();
    match ty {
        'g' => {
            let hw = ctx.dbg.hw();
            for reg in 0..15 {
                out += &format!("{:08X}", hw.read_reg(reg).swap_bytes());
            }
            out += &format!("{:08X}", hw.pause_addr().swap_bytes());
            for _ in 0..8 {
                out += "xxxxxxxxxxxxxxxxxxxxxxxx"; // fX registers (12 bytes each)
            }
            out += "xxxxxxxx"; // fps register            
            out += &format!("{:08X}", hw.read_cpsr().swap_bytes());
        }
        'G' => {
            let mut hw = ctx.dbg.hw();
            let mut regs = params;
            let next_reg = |regstr: &str| -> Result<u32> {
                let val = utils::from_hex(&regstr[..8])?;
                Ok(val.swap_bytes())
            };

            for reg in 0..15 {
                hw.write_reg(reg, next_reg(regs)?);
                regs = &regs[8..];
            }
            // register at 15: PC
            hw.branch_to(next_reg(regs)?);
            regs = &regs[8..];

            // Skip 8 fX registers
            regs = &regs[8 * 24..];

            // Skip fps register
            regs = &regs[8..];

            // register at 25: CPSR
            hw.write_cpsr(next_reg(regs)?);
            out += "OK";
        }
        'H' => {
            out += "OK";
        }
        'm' => {
            let mut hw = ctx.dbg.hw();
            let mut params = params.split(',');
            let addr = parse_next_hex(&mut params)?;
            let size = parse_next_hex(&mut params)?;
            let mut buf = [0u8];
            for b in 0..size {
                if let Err(_) = hw.read_mem(addr+b, &mut buf) {
                    out += "00";
                } else {
                    out += &format!("{:02X}", buf[0]);
                }
            }
        }
        'M' => {
            let mut hw = ctx.dbg.hw();
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
        'p' => {
            let hw = ctx.dbg.hw();
            let reg = utils::from_hex(&params)? as usize;
            let regval = match reg {
                0 ... 14 => hw.read_reg(reg),
                15 => hw.pause_addr(),
                25 => hw.read_cpsr(),
                n => {
                    warn!("GDB requested bad register value {}", n);
                    0
                }
            };
            out += &format!("{:08X}", regval.swap_bytes());
        }
        'q' => {
            return handle_gdb_cmd_q(params, ctx);
        }
        's' => {
            return cmd_step(ctx);
        }
        'c' => {
            return cmd_continue(ctx);
        }
        'v' => {
            return handle_gdb_cmd_v(params, ctx);
        }
        'z' | 'Z' => {
            let mut hw = ctx.dbg.hw();
            let mut params = params.split(',');
            let brk_ty = parse_next(&mut params)?;
            let addr = parse_next_hex(&mut params)?;
            let _kind = parse_next(&mut params)?;
            assert!(brk_ty == "0");
            if ty == 'Z' {
                hw.set_breakpoint(addr);
            } else {
                hw.del_breakpoint(addr);
            }
            out += "OK";
        }
        '?' => {
            out += &ctx.last_halt.to_signal();
        }
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
    AckOk,
    AckErr,
    EndOfPacket,
    Malformed,
}

fn load_packet<I: Iterator<Item = u8>>(it: &mut I) -> PacketType {
    let mut it = it.skip_while(|b| *b != 0x03 && *b != b'$' && *b != b'-' && *b != b'+');
    match it.next() {
        Some(0x3) => return PacketType::CtrlC,
        Some(b'$') => {}
        Some(b'+') => return PacketType::AckOk,
        Some(b'-') => return PacketType::AckErr,
        None => return PacketType::EndOfPacket,
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

fn handle_gdb_packet(data: &[u8], stream: &mut TcpStream, ctx: &mut GdbCtx) -> Result<()> {
    trace!("Recieving GDB packet: {}", str::from_utf8(data).unwrap());
    let mut it = data.iter().cloned();
    loop {
        match load_packet(&mut it) {
            PacketType::Command(cmd) => {
                stream.write(b"+")?;
                stream.flush()?;
                match handle_gdb_cmd(&cmd, ctx) {
                    Ok(out) => write_gdb_packet(&out, stream)?,
                    Err(e) => {
                        if let ErrorKind::NoResponse = e {}
                        else { return Err(e) }
                    }
                }

            }
            PacketType::CtrlC => {
                ctx.dbg.pause();
                trace!("Recieved GDB packet with CTRL-C signal!");
            }
            PacketType::AckOk => {},
            PacketType::AckErr => error!("GDB client replied with error packet!"),
            PacketType::EndOfPacket => {
                return Ok(())
            }
            PacketType::Malformed => {
                trace!("Recieved malformed data {:?}", data);
                stream.write(b"-")?;
                stream.flush()?;
                return Ok(())
            }
        }
    }
}

struct GdbCtx<'a, 'b: 'a> {
    dbg: &'a mut dbgcore::DbgContext<'b>,
    last_halt: &'a mut BreakData,
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

            let mut last_halt = BreakData::new(BreakReason::Trapped, &mut debugger.ctx(Arm9));
            't: loop {
                connection.poll.poll(&mut events, Some(Duration::from_millis(100)))
                    .expect("Could not poll for network events!");

                let mut ctx = GdbCtx {
                    dbg: &mut debugger.ctx(Arm9),
                    last_halt: &mut last_halt
                };

                for event in &events {
                    handle_event(&event, &mut connection, |buf, stream| {
                        handle_gdb_packet(buf, stream, &mut ctx).unwrap();
                    });
                }

                for msg in msg_client.try_iter() {
                    match msg {
                        Message::Quit => break 't,
                        Message::Arm9Halted(reason) => {
                            if let Some(ref mut stream) = connection.socket {
                                let break_data = BreakData::new(reason, ctx.dbg);
                                write_gdb_packet(&break_data.to_signal(), stream).unwrap();
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
        TOKEN_CLIENT => for _ in 0..128 {
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
