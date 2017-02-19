#[macro_use]
extern crate log;
extern crate capstone;
extern crate env_logger;
extern crate libc;
extern crate libllama;
extern crate sdl2;

mod commands;
mod gui;

use std::env;
use std::io::{stdin, stdout, Write};
use std::sync::atomic::{AtomicBool, Ordering, ATOMIC_BOOL_INIT};
use std::time::Duration;

use libllama::{dbgcore, hwcore, ldr};

static SIGINT_REQUESTED: AtomicBool = ATOMIC_BOOL_INIT;

unsafe fn sigint_set_action(action_fn: libc::size_t) {
    use std::ptr;
    #[cfg(target_os="macos")]
    let action = libc::sigaction {
        sa_sigaction: action_fn,
        sa_mask: 0,
        sa_flags: libc::SA_RESETHAND,
    };

    #[cfg(target_os="linux")]
    let action = {
        use std::mem;
        let mut sigset: libc::sigset_t = mem::zeroed();
        let mut action: libc::sigaction = mem::zeroed();

        libc::sigemptyset(&mut sigset);
        action.sa_sigaction = action_fn as libc::size_t;
        action.sa_mask = sigset;
        action.sa_flags = libc::SA_RESETHAND;
        action
    };

    libc::sigaction(libc::SIGINT, &action, ptr::null_mut());
}

fn sigint_trap() {
    fn action_fn(_: libc::c_int) {
        SIGINT_REQUESTED.store(true, Ordering::SeqCst);
    }
    unsafe { sigint_set_action(action_fn as libc::size_t) };
}

fn sigint_reset() {
    unsafe { sigint_set_action(libc::SIG_DFL) };
}

fn sigint_triggered() -> bool {
    SIGINT_REQUESTED.compare_and_swap(true, false, Ordering::SeqCst)
}

fn run_emulator<L: ldr::Loader>(loader: L) {
    let mut hwcore = hwcore::HwCore::new(loader);
    let mut debugger = dbgcore::DbgCore::bind(hwcore);
    debugger.ctx().hwcore_mut().start();

    let mut gui = gui::Gui::new();
    let mut fbs = hwcore::Framebuffers {
        top_screen: Vec::new(), bot_screen: Vec::new(),
        top_screen_size: (240, 400, 3), bot_screen_size: (240, 320, 3),
    };

    sigint_trap();
    let mut is_paused = false;
    loop {
        if sigint_triggered() {
            debugger.ctx().pause();
            is_paused = true;
        }

        if is_paused {
            sigint_reset();

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

            sigint_trap();
        } else {
            gui.handle_events();

            if debugger.ctx().hwcore_mut().try_wait() {
                is_paused = true;
            }

            debugger.ctx().hwcore_mut().copy_framebuffers(&mut fbs);
            gui.render_framebuffers(&fbs);

            std::thread::sleep(Duration::from_millis(10));
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
