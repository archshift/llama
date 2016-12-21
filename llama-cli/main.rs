#[macro_use]
extern crate log;
extern crate capstone;
extern crate ctrlc;
extern crate env_logger;
extern crate libllama;

mod commands;

use std::env;
use std::io::{stdin, stdout, Write};
use std::sync::atomic::{AtomicBool, Ordering, ATOMIC_BOOL_INIT};
use std::time::Duration;

use libllama::{dbgcore, hwcore, ldr};

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

fn run_emulator<L: ldr::Loader>(loader: L) {
    let mut mem = hwcore::map_memory_regions();
    loader.load(&mut mem);

    let mut hwcore = hwcore::HwCore::new(loader.entrypoint(), mem);
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
            stdout().flush().unwrap();

            // Handle pause command
            let mut input = String::new();
            stdin().read_line(&mut input).unwrap();

            // Allow handling multiple commands delimited by a semicolon
            for cmd in input.split(';') {
                // Keep processing commands until unpaused
                if !commands::handle(&mut debugger, cmd.split_whitespace()) {
                    is_paused = false;
                    break
                }
            }
        } else {
            std::thread::sleep(Duration::from_millis(100));
        }
    }

    debugger.ctx().hwcore_mut().stop();
}

fn main() {
    env_logger::init().unwrap();

    let path = env::args().nth(1).unwrap();
    let loader = ldr::Ctr9Loader::from_folder(&path).unwrap();
    run_emulator(loader);
}
