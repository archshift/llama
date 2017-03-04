#[macro_use]
extern crate log;
extern crate capstone;
extern crate libc;
extern crate libllama;

mod commands;
mod uilog;

use std::env;

use libllama::{dbgcore, hwcore, ldr};

struct Backend {
    debugger: dbgcore::DbgCore
}

#[repr(C)]
struct FrontendCallbacks {
    set_running: extern fn(*mut Backend, bool),
    is_running: extern fn(*mut Backend) -> bool
}

extern {
    fn llama_open_gui(backend: *mut Backend, callbacks: *const FrontendCallbacks);
}


extern fn backend_set_running(backend: *mut Backend, state: bool) {
    if state {
        unsafe { (*backend).debugger.ctx().resume(); }
    } else {
        unsafe { (*backend).debugger.ctx().pause(); }
    }
}
extern fn backend_is_running(backend: *mut Backend) -> bool {
    unsafe { !(*backend).debugger.ctx().hwcore_mut().try_wait() }
}

fn main() {
    uilog::init().unwrap();

    let path = env::args().nth(1).unwrap();
    let loader = ldr::Ctr9Loader::from_folder(&path).unwrap();

    let callbacks = FrontendCallbacks {
        set_running: backend_set_running,
        is_running: backend_is_running
    };

    // let mut fbs = hwcore::Framebuffers {
    //     top_screen: Vec::new(), bot_screen: Vec::new(),
    //     top_screen_size: (240, 400, 3), bot_screen_size: (240, 320, 3),
    // };

    let mut hwcore = hwcore::HwCore::new(loader);
    let mut backend = Backend {
        debugger: dbgcore::DbgCore::bind(hwcore)
    };

    unsafe { llama_open_gui(&mut backend, &callbacks) };
}