#[macro_use]
extern crate log;
extern crate capstone;
extern crate env_logger;
extern crate libc;
extern crate libllama;

mod commands;

use std::env;
use std::io::{stdin, stdout, Write};
use std::sync::atomic::{AtomicBool, Ordering, ATOMIC_BOOL_INIT};
use std::time::Duration;

use libllama::{dbgcore, hwcore, ldr};

static SIGINT_REQUESTED: AtomicBool = ATOMIC_BOOL_INIT;

#[inline(always)]
fn sigint_trap_oneshot() {
    use std::ptr;

    fn action_fn(_: libc::c_int) {
        SIGINT_REQUESTED.store(true, Ordering::SeqCst);
    }

    #[cfg(target_os="macos")]
    let action = libc::sigaction {
        sa_sigaction: action_fn as libc::size_t,
        sa_mask: 0,
        sa_flags: libc::SA_RESETHAND,
    };

    #[cfg(target_os="linux")]
    let action = {
        use std::mem;
        let mut sigset: libc::sigset_t = unsafe { mem::zeroed() };
        let mut action: libc::sigaction = unsafe { mem::zeroed() };

        unsafe { libc::sigemptyset(&mut sigset) };
        action.sa_sigaction = action_fn as libc::size_t;
        action.sa_mask = sigset;
        action.sa_flags = libc::SA_RESETHAND;
        action
    };

    unsafe { libc::sigaction(libc::SIGINT, &action, ptr::null_mut()) };
}

#[inline(always)]
fn sigint_triggered() -> bool {
    SIGINT_REQUESTED.compare_and_swap(true, false, Ordering::SeqCst)
}

fn run_emulator<L: ldr::Loader>(loader: L) {
    let mut hwcore = hwcore::HwCore::new(loader);
    let mut debugger = dbgcore::DbgCore::bind(hwcore);
    debugger.ctx().hwcore_mut().start();

    sigint_trap_oneshot();
    let mut is_paused = false;
    loop {
        if sigint_triggered() {
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
            sigint_trap_oneshot();
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
