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
    debugger: dbgcore::DbgCore,
    fbs: hwcore::Framebuffers
}

#[repr(C)]
struct FrontendCallbacks {
    set_running: extern fn(*mut Backend, bool),
    is_running: extern fn(*mut Backend) -> bool,
    top_screen: extern fn(*mut Backend, *mut usize) -> *const u8,
    bot_screen: extern fn(*mut Backend, *mut usize) -> *const u8,
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
extern fn backend_top_screen(backend: *mut Backend, buf_size_out: *mut usize) -> *const u8 {
    let backend = unsafe { &mut *backend };
    backend.debugger.ctx().hwcore_mut().copy_framebuffers(&mut backend.fbs);
    unsafe {
        *buf_size_out = backend.fbs.top_screen.len();
        backend.fbs.top_screen.as_ptr()
    }
}
extern fn backend_bot_screen(backend: *mut Backend, buf_size_out: *mut usize) -> *const u8 {
    let backend = unsafe { &mut *backend };
    backend.debugger.ctx().hwcore_mut().copy_framebuffers(&mut backend.fbs);
    unsafe {
        *buf_size_out = backend.fbs.bot_screen.len();
        backend.fbs.bot_screen.as_ptr()
    }
}

fn main() {
    uilog::init().unwrap();

    let path = env::args().nth(1).unwrap();
    let loader = ldr::Ctr9Loader::from_folder(&path).unwrap();

    let callbacks = FrontendCallbacks {
        set_running: backend_set_running,
        is_running: backend_is_running,
        top_screen: backend_top_screen,
        bot_screen: backend_bot_screen,
    };

    let fbs = hwcore::Framebuffers {
        top_screen: Vec::new(), bot_screen: Vec::new(),
        top_screen_size: (240, 400, 3), bot_screen_size: (240, 320, 3),
    };

    let hwcore = hwcore::HwCore::new(loader);
    let mut backend = Backend {
        debugger: dbgcore::DbgCore::bind(hwcore),
        fbs: fbs
    };

    unsafe { llama_open_gui(&mut backend, &callbacks) };
}