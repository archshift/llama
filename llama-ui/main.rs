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

mod c {
    #![allow(warnings)]
    include!(concat!(env!("OUT_DIR"), "/qml_interop.rs"));
}

struct Backend<'a> {
    loader: &'a ldr::Loader,
    debugger: dbgcore::DbgCore,
    fbs: hwcore::Framebuffers
}

impl<'a> Backend<'a> {
    unsafe fn from_c<'b>(backend: *mut c::Backend) -> &'b mut Backend<'b> {
        &mut *(backend as *mut Backend)
    }

    fn to_c(&mut self) -> *mut c::Backend {
        (self as *mut Backend) as *mut c::Backend
    }
}

mod cbs {
    use std::slice;
    use std::str;

    use commands;
    use uilog;
    use {Backend, c};

    use lgl;
    use libllama::io::hid;

    pub unsafe extern fn set_running(backend: *mut c::Backend, state: bool) {
        let backend = Backend::from_c(backend);
        if state {
            backend.debugger.ctx().resume();
        } else {
            backend.debugger.ctx().pause();
        }
    }

    pub unsafe extern fn is_running(backend: *mut c::Backend) -> bool {
        let backend = Backend::from_c(backend);
        backend.debugger.ctx().hwcore_mut().running()
    }

    pub unsafe extern fn top_screen(backend: *mut c::Backend, buf_size_out: *mut usize) -> *const u8 {
        let backend = Backend::from_c(backend);
        backend.debugger.ctx().hwcore_mut().copy_framebuffers(&mut backend.fbs);
        *buf_size_out = backend.fbs.top_screen.len();
        backend.fbs.top_screen.as_ptr()
    }

    pub unsafe extern fn bot_screen(backend: *mut c::Backend, buf_size_out: *mut usize) -> *const u8 {
        let backend = Backend::from_c(backend);
        backend.debugger.ctx().hwcore_mut().copy_framebuffers(&mut backend.fbs);
        *buf_size_out = backend.fbs.bot_screen.len();
        backend.fbs.bot_screen.as_ptr()
    }

    pub unsafe extern fn mod_button(backend: *mut c::Backend, button: i32, pressed: bool) {
        let backend = Backend::from_c(backend);
        let button = match button {
            _ if button == c::Button::BUTTON_A as i32 => hid::Button::A,
            _ if button == c::Button::BUTTON_B as i32 => hid::Button::B,
            _ if button == c::Button::BUTTON_X as i32 => hid::Button::X,
            _ if button == c::Button::BUTTON_Y as i32 => hid::Button::Y,
            _ if button == c::Button::BUTTON_L as i32 => hid::Button::L,
            _ if button == c::Button::BUTTON_R as i32 => hid::Button::R,
            _ if button == c::Button::BUTTON_UP as i32 => hid::Button::Up,
            _ if button == c::Button::BUTTON_DOWN as i32 => hid::Button::Down,
            _ if button == c::Button::BUTTON_LEFT as i32 => hid::Button::Left,
            _ if button == c::Button::BUTTON_RIGHT as i32 => hid::Button::Right,
            _ if button == c::Button::BUTTON_SELECT as i32 => hid::Button::Select,
            _ if button == c::Button::BUTTON_START as i32 => hid::Button::Start,
            _ => unreachable!()
        };
        let state = if pressed { hid::ButtonState::Pressed(button) }
                    else { hid::ButtonState::Released(button) };
        backend.debugger.ctx().hwcore_mut().rt_tx.hid_btn.send(state).unwrap();
    }

    pub unsafe extern fn run_command(backend: *mut c::Backend, str_buf: *const i8, str_len: usize) {
        let backend = Backend::from_c(backend);
        let input = {
            let slice = slice::from_raw_parts(str_buf as *const u8, str_len);
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

    pub unsafe extern fn use_trace_logs(_: *mut c::Backend, val: bool) {
        uilog::allow_trace(val);
    }

    pub unsafe extern fn reload_game(backend: *mut c::Backend) {
        let backend = Backend::from_c(backend);
        backend.debugger = super::load_game(backend.loader);
    }

    pub unsafe extern fn log(buf: c::LogBufferView) {
        let s = {
            let slice = slice::from_raw_parts(buf.buf_ptr as *const u8, buf.buf_size);
            str::from_utf8(slice).unwrap()
        };
        lgl::log(s)
    }

    pub unsafe extern fn buffer(buf: c::LogBufferMutView) -> c::LogBufferView {
        let new_slice = lgl::buffer(slice::from_raw_parts_mut(buf.buf_ptr as *mut u8, buf.buf_size));
        c::LogBufferView { buf_ptr: new_slice.as_ptr() as *const i8, buf_size: new_slice.len() }
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
    let _logger = uilog::init().unwrap();

    let path = env::args().nth(1).unwrap();
    let loader = ldr::Ctr9Loader::from_folder(&path).unwrap();

    let callbacks = c::FrontendCallbacks {
        set_running: Some(cbs::set_running),
        is_running: Some(cbs::is_running),
        reload_game: Some(cbs::reload_game),

        top_screen: Some(cbs::top_screen),
        bot_screen: Some(cbs::bot_screen),
        mod_button: Some(cbs::mod_button),

        run_command: Some(cbs::run_command),
        use_trace_logs: Some(cbs::use_trace_logs),
        log: Some(cbs::log),
        buffer: Some(cbs::buffer),
        buffer_size: Some(cbs::buffer_size),
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

    unsafe { c::llama_open_gui(backend.to_c(), &callbacks) };
}