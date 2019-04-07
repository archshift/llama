use mem;
use msgs;
use hwcore::Message;

use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;

pub struct HardwarePica {
    event_tx: msgs::Client<Message>,
    fb_state: FramebufState,
    mem: mem::MemController,
}

impl HardwarePica {
    pub fn new(client: msgs::Client<Message>, mem: mem::MemController) -> Self {
        Self {
            event_tx: client,
            fb_state: FramebufState::new(),
            mem: mem
        }
    }
}

impl fmt::Debug for HardwarePica {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "HardwarePica {{ }}")
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ColorFormat {
    Rgba8 = 0,
    Rgb8 = 1,
    Rgb565 = 2,
    Rgb5a1 = 3,
    Rgba4 = 4
}

impl ColorFormat {
    fn from_index(idx: u32) -> Self {
        match idx {
            0 => ColorFormat::Rgba8,
            1 => ColorFormat::Rgb8,
            2 => ColorFormat::Rgb565,
            3 => ColorFormat::Rgb5a1,
            4 => ColorFormat::Rgba4,
            _ => unimplemented!()
        }
    }
}

#[derive(Debug, Clone)]
pub struct FramebufState {
    pub addr_top_left: [u32; 2],
    pub addr_top_right: [u32; 2],
    pub addr_bot: [u32; 2],
    pub color_fmt: [ColorFormat; 2],
    pub bg_color: [u32; 2]
}

impl FramebufState {
    fn new() -> Self {
        Self {
            addr_top_left: [0; 2],
            addr_top_right: [0; 2],
            addr_bot: [0; 2],
            color_fmt: [ColorFormat::Rgb8; 2],
            bg_color: [0; 2],
        }
    }

    fn lcd_update(dev: &mut LcdDevice) {
        let state = &mut dev._internal_state.borrow_mut();
        {
            let this = &mut state.fb_state;
            let top_fill = dev.top_fill.get();
            let bot_fill = dev.bot_fill.get();
            this.bg_color = [
                top_fill & 0xFFFFFF * ((top_fill >> 6) & 1),
                bot_fill & 0xFFFFFF * ((bot_fill >> 6) & 1)
            ];
        }

        Self::publish(&state.fb_state, &state.event_tx);
    }

    fn gpu_update(dev: &mut GpuDevice) {
        let state = &mut dev._internal_state.borrow_mut();
        {
            let this = &mut state.fb_state;

            this.addr_top_left = [dev.top_fb_left0_addr.get(), dev.top_fb_left1_addr.get()];
            this.addr_top_right = [dev.top_fb_right0_addr.get(), dev.top_fb_right1_addr.get()];
            this.addr_bot = [dev.bot_fb0_addr.get(), dev.bot_fb1_addr.get()];
            this.color_fmt = [
                ColorFormat::from_index(FbFmt::new(dev.top_fbfmt.get()).colorFmt.get()),
                ColorFormat::from_index(FbFmt::new(dev.bot_fbfmt.get()).colorFmt.get())
            ];
        }

        Self::publish(&state.fb_state, &state.event_tx);
    }

    fn publish(&self, client: &msgs::Client<Message>) {
        let msg = Message::FramebufState(self.clone());
        client.send(msg)
    }
}

enum FillWidth {
    _16bit = 2,
    _24bit = 3,
    _32bit = 4,
}

impl FillWidth {
    fn from_index(idx: u32) -> Self {
        match idx {
            0 => FillWidth::_16bit,
            1 | 3 => FillWidth::_24bit,
            2 => FillWidth::_32bit,
            _ => unimplemented!()
        }
    }
}

bf!(FillCtrl[u32] {
    busy: 0:0,
    done: 1:1,
    width: 8:9,
});

bf!(FbFmt[u32] {
    colorFmt: 0:2
});

fn process_memfill(dev: &mut GpuDevice, which: usize) {
    let (start_word, end_word, val, ctrl) = match which {
        0 => (
            dev.memfill0_start_word.get(), dev.memfill0_end_word.get(),
            dev.memfill0_val.get(), &mut dev.memfill0_ctrl
        ),
        1 => (
            dev.memfill1_start_word.get(), dev.memfill1_end_word.get(),
            dev.memfill1_val.get(), &mut dev.memfill1_ctrl
        ),
        _ => unreachable!()
    };

    let mem = &mut dev._internal_state.borrow_mut().mem;

    let ctrl = FillCtrl::alias_mut(ctrl.ref_mut());
    if ctrl.busy.get() == 0 {
        return
    }
    
    let width = FillWidth::from_index(ctrl.width.get());
    let mut block = [0u8; 12];
    
    for chunk in block[..].chunks_mut(width as usize) {
        let mut val = val;
        for byte in chunk {
            *byte = val as u8;
            val >>= 8;
        }
    }

    let start = start_word << 3;
    let end = end_word << 3;

    let mut addr = start;
    while addr < end {
        let block_size = block.len().min((end - addr) as usize);
        mem.write_buf(addr, &block[..block_size]);
        addr += block_size as u32;
    }

    ctrl.busy.set(0);
    ctrl.done.set(1);
}



iodevice!(LcdDevice, {
    internal_state: Rc<RefCell<HardwarePica>>;

    regs: {
        0x000 => parallax_enable: u32 { }
        0x004 => unk0: u32 { }
        0x008 => unk1: u32 { }
        0x00C => unk2: u32 { }
        0x014 => unk3: u32 { }
        0x200 => top_unk0: u32 { }
        0x204 => top_fill: u32 {
            write_effect = |dev: &mut LcdDevice| {
                let fill = dev.top_fill.get();
                if (fill & (1 << 24) != 0) {
                    warn!("Enabling top screen pixel fill! Color: #{:06X}", fill & 0xFFFFFF);
                }
                FramebufState::lcd_update(dev);
            };
        }
        0x238 => top_unk1: u32 { }
        0x240 => top_backlight: u32 { }
        0x244 => top_unk2: u32 { }
        0xA00 => bot_unk0: u32 { }
        0xA04 => bot_fill: u32 {
            write_effect = |dev: &mut LcdDevice| {
                let fill = dev.bot_fill.get();
                if (fill & (1 << 24) != 0) {
                    warn!("Enabling bottom screen pixel fill! Color: #{:06X}", fill & 0xFFFFFF);
                }
                FramebufState::lcd_update(dev);
            };
        }
        0xA38 => bot_unk1: u32 { }
        0xA40 => bot_backlight: u32 { }
        0xA44 => bot_unk2: u32 { }
    }
    ranges: {
        0x400;0x400 => { } // top unknown region
        0xC00;0x400 => { } // bottom unknown region
    }

});


iodevice!(GpuDevice, {
    internal_state: Rc<RefCell<HardwarePica>>;

    regs: {
        0x004 => unk0: u32 { }
        0x010 => memfill0_start_word: u32 { }
        0x014 => memfill0_end_word: u32 { }
        0x018 => memfill0_val: u32 { }
        0x01C => memfill0_ctrl: u32 {
            write_effect = |dev: &mut GpuDevice| {
                process_memfill(dev, 0);
            };
        }
        0x020 => memfill1_start_word: u32 { }
        0x024 => memfill1_end_word: u32 { }
        0x028 => memfill1_val: u32 { }
        0x02C => memfill1_ctrl: u32 {
            write_effect = |dev: &mut GpuDevice| {
                process_memfill(dev, 1);
            };
        }
        0x030 => unk1: u32 { }

        0x400 => top_pix_clk: u32 { }
        0x404 => top_hblank_timer: u32 { }
        0x408 => top_unk0: u32 { }
        0x40C => top_unk1: u32 { }
        0x410 => top_vpix_interp: u32 { }
        0x414 => top_hdata_offs: u32 { }
        0x418 => top_unk2: u32 { }
        0x41C => top_unk3: u32 { }
        0x420 => top_hpix_offs: u32 { }
        0x424 => top_unk4: u32 { }
        0x428 => top_vblank_timer: u32 { }
        0x42C => top_unk5: u32 { }
        0x430 => top_vtotal: u32 { }
        0x434 => top_vdisp: u32 { }
        0x438 => top_vdata_offs: u32 { }
        0x43C => top_unk6: u32 { }
        0x440 => top_unk7: u32 { }
        0x444 => top_unk8: u32 { }
        0x448 => top_unk9: u32 { }
        0x44C => top_overscan_color: u32 { }
        0x45C => top_fbsize0: u32 { }
        0x460 => top_fbsize1: u32 { }
        0x464 => top_fbsize2: u32 { }
        0x468 => top_fb_left0_addr: u32 {
            write_effect = |dev: &mut GpuDevice| {
                info!("Set top left FB0 to 0x{:08X}!", dev.top_fb_left0_addr.get());
                FramebufState::gpu_update(dev);
            };
        }
        0x46C => top_fb_left1_addr: u32 {
            write_effect = |dev: &mut GpuDevice| {
                info!("Set top left FB1 to 0x{:08X}!", dev.top_fb_left1_addr.get());
                FramebufState::gpu_update(dev);
            };
        }
        0x470 => top_fbfmt: u32 {
            default = 1;
            write_effect = |dev: &mut GpuDevice| {
                let fmt = FbFmt::new(dev.top_fbfmt.get());
                let colorfmt = ColorFormat::from_index(fmt.colorFmt.get());
                info!("Set top FB color format to {:?}!", colorfmt);
                FramebufState::gpu_update(dev);
            };
        }
        0x474 => top_unk10: u32 { }
        0x478 => top_fb_sel: u32 { }
        0x480 => top_fb_color_lut_idx: u32 {
            write_bits = 0xFF;
            write_effect = |dev: &mut GpuDevice| {
                info!("Updated top FB color LUT index to 0x{:02X}", dev.top_fb_color_lut_idx.get());
            };
        }
        0x484 => top_fb_color_lut_out: u32 {
            write_bits = 0;
            read_effect = |dev: &mut GpuDevice| {
                let idx = dev.top_fb_color_lut_idx.get();
                info!("STUBBED: Reading top FB color LUT at index 0x{:02X}", idx);
                dev.top_fb_color_lut_out.set_unchecked(idx << 16 | idx << 8 | idx);
            };
        }
        0x490 => top_fb_stride: u32 { }
        0x494 => top_fb_right0_addr: u32 {
            write_effect = |dev: &mut GpuDevice| {
                info!("Set top right FB0 to 0x{:08X}!", dev.top_fb_right0_addr.get());
                FramebufState::gpu_update(dev);
            };
        }
        0x498 => top_fb_right1_addr: u32 {
            write_effect = |dev: &mut GpuDevice| {
                info!("Set top right FB1 to 0x{:08X}!", dev.top_fb_right1_addr.get());
                FramebufState::gpu_update(dev);
            };
        }
        0x49C => top_unk11: u32 { }


        0x500 => bot_pix_clk: u32 { }
        0x504 => bot_hblank_timer: u32 { }
        0x508 => bot_unk0: u32 { }
        0x50C => bot_unk1: u32 { }
        0x510 => bot_vpix_interp: u32 { }
        0x514 => bot_hdata_offs: u32 { }
        0x518 => bot_unk2: u32 { }
        0x51C => bot_unk3: u32 { }
        0x520 => bot_hpix_offs: u32 { }
        0x524 => bot_unk4: u32 { }
        0x528 => bot_vblank_timer: u32 { }
        0x52C => bot_unk5: u32 { }
        0x530 => bot_vtotal: u32 { }
        0x534 => bot_vdisp: u32 { }
        0x538 => bot_vdata_offs: u32 { }
        0x53C => bot_unk6: u32 { }
        0x540 => bot_unk7: u32 { }
        0x544 => bot_unk8: u32 { }
        0x548 => bot_unk9: u32 { }
        0x54C => bot_overscan_color: u32 { }
        0x55C => bot_fbsize0: u32 { }
        0x560 => bot_fbsize1: u32 { }
        0x564 => bot_fbsize2: u32 { }
        0x568 => bot_fb0_addr: u32 {
            write_effect = |dev: &mut GpuDevice| {
                info!("Set bottom FB0 to 0x{:08X}!", dev.bot_fb0_addr.get());
                FramebufState::gpu_update(dev);
            };
        }
        0x56C => bot_fb1_addr: u32 {
            write_effect = |dev: &mut GpuDevice| {
                info!("Set bottom FB1 to 0x{:08X}!", dev.bot_fb1_addr.get());
                FramebufState::gpu_update(dev);
            };
        }
        0x570 => bot_fbfmt: u32 {
            write_effect = |dev: &mut GpuDevice| {
                let fmt = FbFmt::new(dev.bot_fbfmt.get());
                let colorfmt = ColorFormat::from_index(fmt.colorFmt.get());
                info!("Set bottom FB color format to {:?}!", colorfmt);
                FramebufState::gpu_update(dev);
            };
        }
        0x574 => bot_unk10: u32 { }
        0x578 => bot_fb_sel: u32 { }
        0x580 => bot_fb_color_lut_idx: u32 {
            write_bits = 0xFF;
            write_effect = |dev: &mut GpuDevice| {
                info!("Updated bot FB color LUT index to 0x{:02X}", dev.bot_fb_color_lut_idx.get());
            };
        }
        0x584 => bot_fb_color_lut_out: u32 {
            write_bits = 0;
            read_effect = |dev: &mut GpuDevice| {
                let idx = dev.bot_fb_color_lut_idx.get();
                info!("STUBBED: Reading bot FB color LUT at index 0x{:02X}", idx);
                dev.bot_fb_color_lut_out.set_unchecked(idx << 16 | idx << 8 | idx);
            };
        }
        0x590 => bot_fb_stride: u32 { }
        0x594 => bot_fb_unused0_addr: u32 { }
        0x598 => bot_fb_unused1_addr: u32 { }
        0x59C => bot_unk11: u32 { }
    }
});

pub fn fb_state(dev: &GpuDevice) -> FramebufState {
    dev._internal_state.borrow().fb_state.clone()
}
