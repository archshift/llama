#[macro_use]
extern crate log;
extern crate capstone;
extern crate lgl;
extern crate libc;
extern crate libllama;

mod commands;
mod uilog;

use std::env;

use libllama::{dbgcore, hwcore, ldr};

pub struct Backend<'a> {
    loader: &'a ldr::Loader,
    debugger: dbgcore::DbgCore,
    fbs: hwcore::Framebuffers
}

#[repr(C)]
pub struct FrontendCallbacks {
    set_running: extern fn(*mut Backend, bool),
    is_running: extern fn(*mut Backend) -> bool,
    top_screen: extern fn(*mut Backend, *mut usize) -> *const u8,
    bot_screen: extern fn(*mut Backend, *mut usize) -> *const u8,
    run_command: extern fn(*mut Backend, *const u8, usize),
    use_trace_logs: extern fn(*mut Backend, bool),

    reload_game: extern fn(*mut Backend),

    log: extern fn(LogBufferView),
    buffer: extern fn(LogBufferMutView) -> LogBufferView,
    buffer_size: extern fn() -> usize,
}

#[repr(C)]
pub struct LogBufferView {
    buf_ptr: *const u8,
    buf_size: usize
}

#[repr(C)]
pub struct LogBufferMutView {
    buf_ptr: *mut u8,
    buf_size: usize
}

extern {
    fn llama_open_gui(backend: *mut Backend, callbacks: *const FrontendCallbacks);
}


mod cbs {
    use std::slice;
    use std::str;

    use commands;
    use uilog;
    use Backend;

    use lgl;
    use {LogBufferView, LogBufferMutView};

    pub extern fn set_running(backend: *mut Backend, state: bool) {
        if state {
            unsafe { (*backend).debugger.ctx().resume(); }
        } else {
            unsafe { (*backend).debugger.ctx().pause(); }
        }
    }

    pub extern fn is_running(backend: *mut Backend) -> bool {
        unsafe { !(*backend).debugger.ctx().hwcore_mut().try_wait() }
    }

    pub extern fn top_screen(backend: *mut Backend, buf_size_out: *mut usize) -> *const u8 {
        let backend = unsafe { &mut *backend };
        backend.debugger.ctx().hwcore_mut().copy_framebuffers(&mut backend.fbs);
        unsafe {
            *buf_size_out = backend.fbs.top_screen.len();
            backend.fbs.top_screen.as_ptr()
        }
    }

    pub extern fn bot_screen(backend: *mut Backend, buf_size_out: *mut usize) -> *const u8 {
        let backend = unsafe { &mut *backend };
        backend.debugger.ctx().hwcore_mut().copy_framebuffers(&mut backend.fbs);
        unsafe {
            *buf_size_out = backend.fbs.bot_screen.len();
            backend.fbs.bot_screen.as_ptr()
        }
    }

    pub extern fn run_command(backend: *mut Backend, str_buf: *const u8, str_len: usize) {
        let backend = unsafe { &mut *backend };
        let input = unsafe {
            let slice = slice::from_raw_parts(str_buf, str_len);
            str::from_utf8(slice).unwrap()
        };

        for cmd in input.split(';') {
            use lgl;
            lgl::log("> ");
            lgl::log(cmd);
            lgl::log("\n");
            commands::handle(&mut backend.debugger, cmd.split_whitespace());
        }
    }

    pub extern fn use_trace_logs(_: *mut Backend, val: bool) {
        uilog::allow_trace(val);
    }

    pub extern fn reload_game(backend: *mut Backend) {
        let backend = unsafe { &mut *backend };
        backend.debugger = super::load_game(backend.loader);
    }

    pub extern fn log(buf: LogBufferView) {
        let s = unsafe {
            let slice = slice::from_raw_parts(buf.buf_ptr, buf.buf_size);
            str::from_utf8(slice).unwrap()
        };
        lgl::log(s)
    }

    pub extern fn buffer(buf: LogBufferMutView) -> LogBufferView {
        let new_slice = unsafe {
            lgl::buffer(slice::from_raw_parts_mut(buf.buf_ptr, buf.buf_size))
        };
        LogBufferView { buf_ptr: new_slice.as_ptr(), buf_size: new_slice.len() }
    }

    pub extern fn buffer_size() -> usize {
        lgl::BUFFER_SIZE
    }
}

fn load_game(loader: &ldr::Loader) -> dbgcore::DbgCore {
    let hwcore = hwcore::HwCore::new(loader);
    dbgcore::DbgCore::bind(hwcore)
}

fn main() {
    let logger = uilog::init().unwrap();

    let path = env::args().nth(1).unwrap();
    let loader = ldr::Ctr9Loader::from_folder(&path).unwrap();

    let callbacks = FrontendCallbacks {
        set_running: cbs::set_running,
        is_running: cbs::is_running,
        top_screen: cbs::top_screen,
        bot_screen: cbs::bot_screen,
        run_command: cbs::run_command,
        use_trace_logs: cbs::use_trace_logs,

        reload_game: cbs::reload_game,

        log: cbs::log,
        buffer: cbs::buffer,
        buffer_size: cbs::buffer_size,
    };

    let fbs = hwcore::Framebuffers {
        top_screen: Vec::new(), bot_screen: Vec::new(),
        top_screen_size: (240, 400, 3), bot_screen_size: (240, 320, 3),
    };

    let mut backend = Backend {
        loader: &loader,
        debugger: load_game(&loader),
        fbs: fbs
    };

    unsafe { llama_open_gui(&mut backend, &callbacks) };
}