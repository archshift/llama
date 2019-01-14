use mem;
use msgs;
use hwcore::Message;

use std::fmt;

pub struct HardwarePica {
    event_tx: msgs::Client<Message>,
    mem: mem::MemController,
}

impl HardwarePica {
    pub fn new(client: msgs::Client<Message>, mem: mem::MemController) -> Self {
        Self {
            event_tx: client,
            mem: mem
        }
    }
}

impl fmt::Debug for HardwarePica {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "HardwarePica {{ }}")
    }
}

#[derive(Clone)]
pub struct FramebufState {
    pub addr_top_left: [u32; 2],
    pub addr_top_right: [u32; 2],
    pub addr_bot: [u32; 2]
}

impl FramebufState {
    fn from_device(dev: &GpuDevice) -> Self {
        Self {
            addr_top_left:  [dev.top_fb_left0_addr.get(),  dev.top_fb_left1_addr.get()],
            addr_top_right: [dev.top_fb_right0_addr.get(), dev.top_fb_right1_addr.get()],
            addr_bot:       [dev.bot_fb0_addr.get(),       dev.bot_fb1_addr.get()],
        }
    }

    fn publish(dev: &GpuDevice) {
        let state = Self::from_device(dev);
        let msg = Message::FramebufState(state);
        dev._internal_state.event_tx.send(msg)
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

    let mem = &mut dev._internal_state.mem;

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

iodevice!(GpuDevice, {
    internal_state: HardwarePica;

    regs: {
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
        0x45C => top_fbsize0: u32 { }
        0x460 => top_fbsize1: u32 { }
        0x464 => top_fbsize2: u32 { }
        0x468 => top_fb_left0_addr: u32 {
            write_effect = |dev: &mut GpuDevice| {
                info!("Set top left FB0 to 0x{:08X}!", dev.top_fb_left0_addr.get());
                FramebufState::publish(dev);
            };
        }
        0x46C => top_fb_left1_addr: u32 {
            write_effect = |dev: &mut GpuDevice| {
                info!("Set top left FB1 to 0x{:08X}!", dev.top_fb_left1_addr.get());
                FramebufState::publish(dev);
            };
        }
        0x470 => top_fbfmt: u32 { }
        0x474 => top_unk10: u32 { }
        0x478 => top_fb_sel: u32 { }
        0x484 => top_fb_color_lut: u32 { }
        0x490 => top_fb_stride: u32 { }
        0x494 => top_fb_right0_addr: u32 {
            write_effect = |dev: &mut GpuDevice| {
                info!("Set top right FB0 to 0x{:08X}!", dev.top_fb_right0_addr.get());
                FramebufState::publish(dev);
            };
        }
        0x498 => top_fb_right1_addr: u32 {
            write_effect = |dev: &mut GpuDevice| {
                info!("Set top right FB1 to 0x{:08X}!", dev.top_fb_right1_addr.get());
                FramebufState::publish(dev);
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
        0x55C => bot_fbsize0: u32 { }
        0x560 => bot_fbsize1: u32 { }
        0x564 => bot_fbsize2: u32 { }
        0x568 => bot_fb0_addr: u32 {
            write_effect = |dev: &mut GpuDevice| {
                info!("Set bottom FB0 to 0x{:08X}!", dev.bot_fb0_addr.get());
                FramebufState::publish(dev);
            };
        }
        0x56C => bot_fb1_addr: u32 {
            write_effect = |dev: &mut GpuDevice| {
                info!("Set bottom FB1 to 0x{:08X}!", dev.bot_fb1_addr.get());
                FramebufState::publish(dev);
            };
        }
        0x570 => bot_fbfmt: u32 { }
        0x574 => bot_unk10: u32 { }
        0x578 => bot_fb_sel: u32 { }
        0x584 => bot_fb_color_lut: u32 { }
        0x590 => bot_fb_stride: u32 { }
        0x594 => bot_fb_unused0_addr: u32 { }
        0x598 => bot_fb_unused1_addr: u32 { }
        0x59C => bot_unk11: u32 { }
    }
});
