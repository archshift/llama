#[macro_use]
extern crate log;
extern crate ctrlc;
extern crate env_logger;
extern crate libllama;

use std::env;
use std::io::{Read, stdin, stdout, Write};
use std::sync::atomic::{AtomicBool, Ordering, ATOMIC_BOOL_INIT};
use std::sync::Arc;
use std::time::{Duration, Instant};

use libllama::{dbgcore, hwcore};

fn from_hex(string: &str) -> Result<u32, std::num::ParseIntError> {
    let slice = if string.starts_with("0x") {
        &string[2..]
    } else {
        &string[..]
    };
    u32::from_str_radix(slice, 16)
}

static SIGINT_REQUESTED: AtomicBool = ATOMIC_BOOL_INIT;

#[inline(always)]
fn sigint_trap() {
    ctrlc::set_handler(|| SIGINT_REQUESTED.store(true, Ordering::SeqCst));
}

#[inline(always)]
fn sigint_reset() {
    ctrlc::set_handler(|| std::process::exit(0));
}

#[inline(always)]
fn sigint_triggered() -> bool {
    SIGINT_REQUESTED.compare_and_swap(true, false, Ordering::SeqCst)
}

/// Prints memory to the screen based on provided address, number of bytes
/// Command format: "mem <start address hex> [# bytes hex]"
///
/// `args`: Iterator over &str items
fn handle_mem_command<'a, It>(debugger: &mut dbgcore::DbgCore, mut args: It)
    where It: Iterator<Item=&'a str> {

    let (start_str, num_str) = match (args.next(), args.next()) {
        (Some(ss), Some(ns)) => (ss, ns),
        (Some(ss), None) => (ss, ss),
        (None, _) => { println!("Usage: `mem <start> [num]"); return }
    };

    let (start, num) = match from_hex(start_str).and_then(|s| Ok((s, from_hex(num_str)?))) {
        Ok((s, n)) if n > 0 => (s, n),
        Ok((s, _)) => (s, 1),
        _ => { println!("Error: could not parse hex value!"); return }
    };

    trace!("Printing {} bytes of RAM starting at 0x{:08X}", num, start);

    let ctx = debugger.get_ctx();
    print!("{:02X}", ctx.read_mem(start));
    for addr in start+1 .. (start + num) {
        print!(" {:02X}", ctx.read_mem(addr));
    }
    println!("");
}

/// Controls debugger behavior based on user-provided commands
///
/// `command`: Iterator over &str items
fn handle_command<'a, It>(debugger: &mut dbgcore::DbgCore, mut command: It) -> bool
    where It: Iterator<Item=&'a str> {
    let mut is_paused = true;

    match command.next() {
        Some("run") => { debugger.resume(); is_paused = false; },
        Some("mem") => handle_mem_command(debugger, command),
        Some("regs") => {
            let ctx = debugger.get_ctx();
            for i in 0..16 {
                println!("R{} = 0x{:08X}", i, ctx.read_reg(i));
            }
        }
        Some("quit") | Some("exit") => {
            debugger.hwcore_mut().stop();
            // TODO: Cleaner exit?
            std::process::exit(0);
        }
        None => println!("Error: No command"),
        Some(unk_cmd @ _) => println!("Error: Unrecognized command `{}`", unk_cmd),
    }

    return is_paused;
}

fn run_emulator(code: &Vec<u8>, load_offset: u32, entrypoint: u32) {
    let mem = hwcore::map_memory_regions();
    mem.write_buf(load_offset, code.as_slice());

    let mut hwcore = hwcore::HwCore::new(entrypoint, mem);
    let mut debugger = dbgcore::DbgCore::bind(hwcore);
    debugger.hwcore_mut().start();

    sigint_trap();
    let mut is_paused = false;
    loop {
        if sigint_triggered() {
            sigint_reset();
            debugger.pause();
            is_paused = true;
        }

        if is_paused {
            // Print prompt text
            print!(" > ");
            stdout().flush();

            // Handle pause command
            let mut input = String::new();
            stdin().read_line(&mut input).unwrap();
            is_paused = handle_command(&mut debugger, input.trim_right().split_whitespace());
        } else {
            std::thread::sleep(Duration::from_millis(100));
        }
    }

    debugger.hwcore_mut().stop();
}

fn main() {
    env_logger::init().unwrap();

    let filename = env::args().nth(1).unwrap();
    let load_offset = from_hex(&env::args().nth(2).unwrap()).unwrap();
    let entrypoint = from_hex(&env::args().nth(3).unwrap()).unwrap();

    let mut file = std::fs::File::open(filename).unwrap();
    let file_data = file.metadata().unwrap();
    let file_size = file_data.len();

    let mut filebuf = Vec::<u8>::with_capacity(file_size as usize);

    let size = file.read_to_end(&mut filebuf).unwrap();
    info!("Reading {} bytes from input file", size);
    assert!(size == file_size as usize);

    run_emulator(&filebuf, load_offset, entrypoint);
}
