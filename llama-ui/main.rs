#[macro_use]
extern crate log;
extern crate capstone;
extern crate lgl;
extern crate libllama;

mod commands;
mod uilog;

use std::env;

use libllama::{dbgcore, gdbstub, hwcore, ldr, msgs};

mod c {
    #![allow(warnings)]
    include!(concat!(env!("OUT_DIR"), "/qml_interop.rs"));
}

struct Backend<'a> {
    loader: &'a ldr::Loader,
    debugger: dbgcore::DbgCore,
    gdb: gdbstub::GdbStub,
    fbs: hwcore::Framebuffers,
    msg_client: msgs::Client<hwcore::Message>,
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
    use libllama::hwcore::Message;
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
        let button = match button as u32 {
            c::Button_BUTTON_A => hid::Button::A,
            c::Button_BUTTON_B => hid::Button::B,
            c::Button_BUTTON_X => hid::Button::X,
            c::Button_BUTTON_Y => hid::Button::Y,
            c::Button_BUTTON_L => hid::Button::L,
            c::Button_BUTTON_R => hid::Button::R,
            c::Button_BUTTON_UP => hid::Button::Up,
            c::Button_BUTTON_DOWN => hid::Button::Down,
            c::Button_BUTTON_LEFT => hid::Button::Left,
            c::Button_BUTTON_RIGHT => hid::Button::Right,
            c::Button_BUTTON_SELECT => hid::Button::Select,
            c::Button_BUTTON_START => hid::Button::Start,
            _ => unreachable!()
        };
        let state = if pressed { hid::ButtonState::Pressed(button) }
                    else { hid::ButtonState::Released(button) };
        backend.msg_client.send(Message::HidUpdate(state)).unwrap();
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
        backend.msg_client.send(Message::Quit).unwrap();
        backend.gdb.wait(); // Need to wait because the GDB thread owns the port
        *backend = super::load_game(backend.loader);
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

fn load_game<'a>(loader: &'a ldr::Loader) -> Backend<'a> {
    let fbs = hwcore::Framebuffers {
        top_screen: Vec::new(), bot_screen: Vec::new(),
        top_screen_size: (240, 400, 3), bot_screen_size: (240, 320, 3),
    };

    let mut pump = msgs::Pump::new();
    let client_gdb = pump.add_client(&["quit", "arm9halted"]);
    let client_user = pump.add_client(&[]);

    let hwcore = hwcore::HwCore::new(pump, loader);
    let debugger = dbgcore::DbgCore::bind(hwcore);

    let backend = Backend {
        loader: loader,
        debugger: debugger.clone(),
        gdb: gdbstub::GdbStub::new(client_gdb, debugger),
        fbs: fbs,
        msg_client: client_user,
    };

    backend
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

    let mut backend = load_game(&loader);
    unsafe { c::llama_open_gui(backend.to_c(), &callbacks) };
}