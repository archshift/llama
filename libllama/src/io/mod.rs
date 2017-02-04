#[macro_use]
mod regs;

mod config;
mod emmc;
mod irq;

use std::default::Default;

pub enum IoRegion {
    Arm9(IoRegsArm9),
    Shared,
    Arm11,
}

#[derive(Default)]
pub struct IoRegsArm9 {
    config: config::ConfigDevice,
    irq: irq::IrqDevice,
    // ndma,
    // timer,
    // ctrcard,
    emmc: emmc::EmmcDevice,
    // pxi9,
    // aes,
    // sha,
    // rsa,
    // xdma,
    // spicard,
    // config_ext,
    // prng,
    // otp,
    // arm7,
}

impl IoRegsArm9 {
    pub fn new() -> IoRegsArm9 {
        Default::default()
    }

    pub unsafe fn read_reg(&mut self, offset: usize, buf: *mut u8, buf_size: usize) {
        let device: &mut regs::IoRegAccess = match offset << 8 >> 20 {
            0x000 => &mut self.config,
            0x001 => &mut self.irq,
            0x006 => &mut self.emmc,
            _ => { error!("Unimplemented IO register read at offset 0x{:X}", offset); return },
        };
        device.read_reg(offset & 0xFFF, buf, buf_size);
    }

    pub unsafe fn write_reg(&mut self, offset: usize, buf: *const u8, buf_size: usize) {
        let device: &mut regs::IoRegAccess = match offset << 8 >> 20 {
            0x000 => &mut self.config,
            0x001 => &mut self.irq,
            0x006 => &mut self.emmc,
            _ => { error!("Unimplemented IO register write at offset 0x{:X}", offset); return },
        };
        device.write_reg(offset & 0xFFF, buf, buf_size);
    }
}