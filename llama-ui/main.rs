#![deny(warnings)]

#[macro_use]
extern crate log;
extern crate capstone;
extern crate lgl;
extern crate libllama;

mod commands;
mod uilog;

use std::env;
use std::path::Path;

use libllama::{dbgcore, gdbstub, hwcore, ldr, msgs, io::gpu};

mod c {
    #![allow(warnings)]
    include!(concat!(env!("OUT_DIR"), "/qml_interop.rs"));
}

struct Backend<'a> {
    loader: &'a dyn ldr::Loader,
    debugger: dbgcore::DbgCore,
    cmd_active_cpu: dbgcore::ActiveCpu,
    gdb: gdbstub::GdbStub,
    fbs: hwcore::Framebuffers,
    fb_state: Option<gpu::FramebufState>,
    msg_client: msgs::Client<hwcore::Message>,
}

impl<'a> Backend<'a> {
    unsafe fn from_c<'b>(backend: *mut c::Backend) -> &'b mut Backend<'b> {
        &mut *(backend as *mut Backend)
    }

    fn to_c(&mut self) -> *mut c::Backend {
        (self as *mut Backend) as *mut c::Backend
    }

    fn update_fb_state(&mut self) {
        while let Ok(msg) = self.msg_client.try_recv() {
            if let hwcore::Message::FramebufState(state) = msg {
                self.fb_state = Some(state);
            } else {
                unreachable!();
            }
        }
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
    use libllama::dbgcore::ActiveCpu::Arm9;
    use libllama::io::gpu::ColorFormat;

    pub unsafe extern fn set_running(backend: *mut c::Backend, state: bool) {
        let backend = Backend::from_c(backend);
        if state {
            backend.debugger.ctx(Arm9)
                .resume();
        } else {
            backend.debugger.ctx(Arm9)
                .pause();
        }
    }

    pub unsafe extern fn is_running(backend: *mut c::Backend) -> bool {
        let backend = Backend::from_c(backend);
        backend.debugger.ctx(Arm9)
            .hwcore_mut()
            .running()
    }

    fn fix_color_rgba8(buf: &mut [u8]) {
        for window in buf.chunks_exact_mut(4) {
            let tmp = window[0];
            window[0] = window[1];
            window[1] = window[2];
            window[2] = window[3];
            window[3] = tmp;
        }
    }

    pub unsafe extern fn top_screen(backend: *mut c::Backend,
                                    buf_size_out: *mut usize,
                                    color_fmt: *mut c::ColorFormat) -> *const u8 {
        let backend = Backend::from_c(backend);

        backend.update_fb_state();
        if backend.fb_state.is_none() {
            *buf_size_out = 0;
            *color_fmt = c::ColorFormat_COLOR_RGB8;
            return std::ptr::null();
        }
        let fb_state = backend.fb_state.as_ref().unwrap();

        backend.debugger.ctx(Arm9)
            .hwcore_mut()
            .copy_framebuffers(&mut backend.fbs, fb_state);

        *color_fmt = match fb_state.color_fmt[0] {
            ColorFormat::Rgb8 => c::ColorFormat_COLOR_RGB8,
            ColorFormat::Rgba8 => {
                fix_color_rgba8(backend.fbs.top_screen.as_mut_slice());
                c::ColorFormat_COLOR_RGBA8
            },
            ColorFormat::Rgb565 => c::ColorFormat_COLOR_RGB565,
            ColorFormat::Rgb5a1 => c::ColorFormat_COLOR_RGB5A1,
            ColorFormat::Rgba4 => c::ColorFormat_COLOR_RGBA4,
        };

        *buf_size_out = backend.fbs.top_screen.len();
        backend.fbs.top_screen.as_ptr()
    }

    pub unsafe extern fn bot_screen(backend: *mut c::Backend,
                                    buf_size_out: *mut usize,
                                    color_fmt: *mut c::ColorFormat) -> *const u8 {
        let backend = Backend::from_c(backend);

        backend.update_fb_state();
        if backend.fb_state.is_none() {
            *buf_size_out = 0;
            *color_fmt = c::ColorFormat_COLOR_RGB8;
            return std::ptr::null();
        }
        let fb_state = backend.fb_state.as_ref().unwrap();

        backend.debugger.ctx(Arm9)
            .hwcore_mut()
            .copy_framebuffers(&mut backend.fbs, fb_state);

        *color_fmt = match fb_state.color_fmt[1] {
            ColorFormat::Rgb8 => c::ColorFormat_COLOR_RGB8,
            ColorFormat::Rgba8 => {
                fix_color_rgba8(backend.fbs.bot_screen.as_mut_slice());
                c::ColorFormat_COLOR_RGBA8
            },
            ColorFormat::Rgb565 => c::ColorFormat_COLOR_RGB565,
            ColorFormat::Rgb5a1 => c::ColorFormat_COLOR_RGB5A1,
            ColorFormat::Rgba4 => c::ColorFormat_COLOR_RGBA4,
        };

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
        backend.msg_client.send(Message::HidUpdate(state));
    }

    pub unsafe extern fn run_command(backend: *mut c::Backend, str_buf: *const libc::c_char, str_len: usize) {
        let backend = Backend::from_c(backend);
        let input = {
            let slice = slice::from_raw_parts(str_buf as *const u8, str_len);
            str::from_utf8(slice).unwrap()
        };

        for cmd in input.split(';') {
            lgl::log("> ");
            lgl::log(cmd);
            lgl::log("\n");
            commands::handle(&mut backend.cmd_active_cpu, &mut backend.debugger, cmd.split_whitespace());
        }
    }

    pub unsafe extern fn use_trace_logs(_: *mut c::Backend, val: bool) {
        uilog::allow_trace(val);
    }

    pub unsafe extern fn reload_game(backend: *mut c::Backend) {
        let backend = Backend::from_c(backend);
        backend.msg_client.send(Message::Quit);
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
        c::LogBufferView { buf_ptr: new_slice.as_ptr() as *const libc::c_char, buf_size: new_slice.len() }
    }

    pub extern fn buffer_size() -> usize {
        lgl::BUFFER_SIZE
    }
}

fn load_game<'a>(loader: &'a dyn ldr::Loader) -> Backend<'a> {
    let fbs = hwcore::Framebuffers::default();

    let mut hwcore = hwcore::HwCore::new(loader);
    let client_gdb = hwcore.take_client_gdb().unwrap();
    let client_user = hwcore.take_client_user().unwrap();

    let debugger = dbgcore::DbgCore::bind(hwcore);

    info!("Using ARM9 as active debug CPU");

    let backend = Backend {
        loader: loader,
        debugger: debugger.clone(),
        cmd_active_cpu: dbgcore::ActiveCpu::Arm9,
        gdb: gdbstub::GdbStub::new(client_gdb, debugger),
        fbs: fbs,
        fb_state: None,
        msg_client: client_user,
    };

    backend
}

fn c_args() -> Vec<Box<[u8]>> {
    let into_c_str = |arg: String| {
        let mut vec = arg.into_bytes();
        vec.push(b'\0');
        vec.into_boxed_slice()
    };

    let args = env::args();
    args.map(into_c_str)
        .collect()
}

fn main() {
    let _logger = uilog::init().unwrap();

    let path = env::args().nth(1).unwrap();
    let loader = ldr::make_loader(Path::new(&path));

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

    let mut backend = load_game(loader.as_ref());
    let mut args = c_args();
    let mut c_args: Vec<*mut u8> = args.iter_mut()
        .map(|arg| &mut arg[0] as *mut u8)
        .collect();

    unsafe {
        c::llama_open_gui(
            c_args.len() as i32,
            c_args.as_mut_ptr() as *mut *mut libc::c_char,
            backend.to_c(),
            &callbacks)
    };
}
