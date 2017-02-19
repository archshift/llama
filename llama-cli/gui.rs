use std::process;

use libllama::hwcore::Framebuffers;
use sdl2;
use sdl2::pixels::PixelFormatEnum;

pub struct Gui {
    sdl_ctx: sdl2::Sdl,
    video_ctx: sdl2::VideoSubsystem,
    events: sdl2::EventPump,
    renderer: sdl2::render::Renderer<'static>,

    top_texture: sdl2::render::Texture,
    bot_texture: sdl2::render::Texture
}

impl Gui {
    pub fn new() -> Gui {
        let sdl_ctx = sdl2::init().unwrap();
        let video = sdl_ctx.video().unwrap();
        let events = sdl_ctx.event_pump().unwrap();

        let window = video.window("llama", 466, 240*2).build().unwrap();
        let renderer = window.renderer()
                             .accelerated()
                             .present_vsync()
                             .build().unwrap();

        let texture_top = renderer.create_texture_streaming(PixelFormatEnum::RGB888, 240, 400).unwrap();
        let texture_bot = renderer.create_texture_streaming(PixelFormatEnum::RGB888, 240, 320).unwrap();

        Gui {
            sdl_ctx: sdl_ctx,
            video_ctx: video,
            events: events,
            renderer: renderer,

            top_texture: texture_top,
            bot_texture: texture_bot,
        }
    }

    pub fn handle_events(&mut self) {
        for event in self.events.poll_iter() {
            use sdl2::event::Event;
            match event {
                Event::Quit{..} => process::exit(0),
                _ => {}
            }
        }
    }

    pub fn render_framebuffers(&mut self, fbs: &Framebuffers) {
        self.renderer.set_draw_color(sdl2::pixels::Color::RGB(0xE1, 0xE1, 0xE1));
        self.renderer.clear();

        fn fill_tex(dst: &mut [u8], src: &Vec<u8>, src_size: &(usize, usize, usize)) {
            let (w, h, d) = *src_size;
            for index in 0..w*h {
                dst[index * 4 + 0] = src[index * d + 0];
                dst[index * 4 + 1] = src[index * d + 1];
                dst[index * 4 + 2] = src[index * d + 2];
            }
        }

        self.top_texture.with_lock(None, |b, _| fill_tex(b, &fbs.top_screen, &fbs.top_screen_size)).unwrap();
        self.bot_texture.with_lock(None, |b, _| fill_tex(b, &fbs.bot_screen, &fbs.bot_screen_size)).unwrap();

        self.renderer.set_draw_color(sdl2::pixels::Color::RGB(0x00, 0x72, 0xBC));
        self.renderer.fill_rect(Some(sdl2::rect::Rect::new(0, 0, 466, 256)));

        self.renderer.copy_ex(&self.top_texture, None, Some(sdl2::rect::Rect::new(33, 240, 240, 400)),
                              -90.0, Some(sdl2::rect::Point::new(0, 0)), false, false).unwrap();
        self.renderer.copy_ex(&self.bot_texture, None, Some(sdl2::rect::Rect::new(73, 480, 240, 320)),
                              -90.0, Some(sdl2::rect::Point::new(0, 0)), false, false).unwrap();
        self.renderer.present();
    }
}