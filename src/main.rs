#[macro_use]
extern crate log;
extern crate ctrlc;
extern crate env_logger;

use std::env;
use std::io::{Read, stdin, stdout, Write};
use std::sync::atomic::{AtomicBool, Ordering, ATOMIC_BOOL_INIT};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[macro_use]
mod utils;

mod cpu;
mod dbgcore;
mod io;
mod mem;
mod hwcore;

fn from_hex(string: String) -> Result<u32, std::num::ParseIntError> {
    let slice = if string.starts_with("0x") {
        &string[2..]
    } else {
        &string[..]
    };
    u32::from_str_radix(slice, 16)
}

static SIGINT_REQUESTED: AtomicBool = ATOMIC_BOOL_INIT;

fn sigint_trap() {
    ctrlc::set_handler(|| SIGINT_REQUESTED.store(true, Ordering::SeqCst));
}

fn sigint_triggered() -> bool {
    SIGINT_REQUESTED.compare_and_swap(true, false, Ordering::SeqCst)
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
        is_paused = {
            let triggered = sigint_triggered();
            if triggered { debugger.pause(); }
            is_paused | triggered
        };

        if is_paused {
            // Print prompt text
            print!(" > ");
            stdout().flush();

            // Read pause command
            let mut input = String::new();
            stdin().read_line(&mut input).unwrap();

            // Handle pause command
            match input.as_str().trim_right() {
                "run" => { debugger.resume(); is_paused = false; },
                "regs" => {
                    let ctx = debugger.get_ctx();
                    for i in 0..16 {
                        println!("R{} = 0x{:08X}", i, ctx.read_reg(i));
                    }
                }
                "quit" => {
                    debugger.hwcore_mut().stop();
                    std::process::exit(0);
                }
                unk_cmd @ _ => println!("Error: Unrecognized command `{}`", unk_cmd),
            }
        } else {
            std::thread::sleep(Duration::from_millis(100));
        }
    }

    debugger.hwcore_mut().stop();
}

fn main() {
    env_logger::init().unwrap();

    let filename = env::args().nth(1).unwrap();
    let load_offset = from_hex(env::args().nth(2).unwrap()).unwrap();
    let entrypoint = from_hex(env::args().nth(3).unwrap()).unwrap();

    let mut file = std::fs::File::open(filename).unwrap();
    let file_data = file.metadata().unwrap();
    let file_size = file_data.len();

    let mut filebuf = Vec::<u8>::with_capacity(file_size as usize);

    let size = file.read_to_end(&mut filebuf).unwrap();
    info!("Reading {} bytes from input file", size);
    assert!(size == file_size as usize);

    run_emulator(&filebuf, load_offset, entrypoint);
}
