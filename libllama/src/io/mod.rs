#[macro_use]
mod regs;

mod config;
mod emmc;
mod irq;
mod sha;
mod xdma;

use std::ptr;
use std::default::Default;

pub enum IoRegion {
    Arm9(IoRegsArm9),
    Shared(IoRegsShared),
    Arm11,
}

pub struct IoRegsArm9 {
    config: config::ConfigDevice,
    irq: irq::IrqDevice,
    // ndma,
    // timer,
    // ctrcard,
    emmc: emmc::EmmcDevice,
    // pxi9,
    // aes,
    sha: sha::ShaDevice,
    // rsa,
    xdma: xdma::XdmaDevice,
    // spicard,
    config_ext: config::ConfigExtDevice,
    // prng,
    // otp,
    // arm7,
}

impl IoRegsArm9 {
    pub fn new() -> IoRegsArm9 {
        IoRegsArm9 {
            config: config::ConfigDevice::new(),
            irq: irq::IrqDevice::new(),
            emmc: emmc::EmmcDevice::new(Default::default()),
            sha: sha::ShaDevice::new(Default::default()),
            xdma: xdma::XdmaDevice::new(),
            config_ext: config::ConfigExtDevice::new(),
        }
    }

    pub unsafe fn read_reg(&mut self, offset: usize, buf: *mut u8, buf_size: usize) {
        let device: &mut regs::IoRegAccess = match bits!(offset, 12 => 23) {
            0x00 => &mut self.config,
            0x01 => &mut self.irq,
            0x06 => &mut self.emmc,
            0x0A => &mut self.sha,
            0x0C => &mut self.xdma,
            0x10 => &mut self.config_ext,
            _ => { error!("Unimplemented IO register read at offset 0x{:X}", offset); return },
        };
        device.read_reg(offset & 0xFFF, buf, buf_size);
    }

    pub unsafe fn write_reg(&mut self, offset: usize, buf: *const u8, buf_size: usize) {
        let device: &mut regs::IoRegAccess = match bits!(offset, 12 => 23) {
            0x00 => &mut self.config,
            0x01 => &mut self.irq,
            0x06 => &mut self.emmc,
            0x0A => &mut self.sha,
            0x0C => &mut self.xdma,
            0x10 => &mut self.config_ext,
            _ => { error!("Unimplemented IO register write at offset 0x{:X}", offset); return },
        };
        device.write_reg(offset & 0xFFF, buf, buf_size);
    }
}


pub struct IoRegsShared {
    // sdio_wifi,
    // hash,
    // y2r,
    // csnd,
    // lgyfb0,
    // lgyfb1,
    // camera,
    // wifi,
    // mvd,
    // config11,
    // spi,
    // i2c,
    // codec,
    // hid,
    // gpio,
    // mic,
    // pxi,
    // ntrcard,
    // mp,
}

impl IoRegsShared {
    pub fn new() -> IoRegsShared {
        IoRegsShared {
        }
    }

    pub unsafe fn read_reg(&mut self, offset: usize, buf: *mut u8, buf_size: usize) {
        let device: &mut regs::IoRegAccess = match bits!(offset, 12 => 23) {
            _ => {
                error!("Unimplemented IO register read at offset 0x{:X}", offset);
                // If we can't find a register for it, just read zero bytes
                ptr::write_bytes(buf, 0, buf_size);
                return
            }
        };
        device.read_reg(offset & 0xFFF, buf, buf_size);
    }

    pub unsafe fn write_reg(&mut self, offset: usize, buf: *const u8, buf_size: usize) {
        let device: &mut regs::IoRegAccess = match bits!(offset, 12 => 19) {
            _ => { error!("Unimplemented IO register write at offset 0x{:X}", offset); return },
        };
        device.write_reg(offset & 0xFFF, buf, buf_size);
    }
}