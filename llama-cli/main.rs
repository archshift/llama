#[macro_use]
extern crate log;
extern crate ctrlc;
extern crate env_logger;
extern crate libllama;

mod commands;
mod common;

use std::env;
use std::io::{Read, stdin, stdout, Write};
use std::sync::atomic::{AtomicBool, Ordering, ATOMIC_BOOL_INIT};
use std::time::Duration;

use libllama::{dbgcore, hwcore};

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

fn run_emulator(code: &Vec<u8>, load_offset: u32, entrypoint: u32) {
    let mem = hwcore::map_memory_regions();
    mem.write_buf(load_offset, code.as_slice());

    let mut hwcore = hwcore::HwCore::new(entrypoint, mem);
    let mut debugger = dbgcore::DbgCore::bind(hwcore);
    debugger.ctx().hwcore_mut().start();

    sigint_trap();
    let mut is_paused = false;
    loop {
        if sigint_triggered() {
            sigint_reset();
            debugger.ctx().pause();
            is_paused = true;
        }

        if is_paused {
            // Print prompt text
            print!(" > ");
            stdout().flush();

            // Handle pause command
            let mut input = String::new();
            stdin().read_line(&mut input).unwrap();
            is_paused = commands::handle(&mut debugger, input.trim_right().split_whitespace());
        } else {
            std::thread::sleep(Duration::from_millis(100));
        }
    }

    debugger.ctx().hwcore_mut().stop();
}

fn main() {
    use common::from_hex;

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
