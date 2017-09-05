#[macro_use]
mod regs;

mod aes;
mod config;
mod emmc;
mod irq;
mod ndma;
mod otp;
mod pxi;
mod rsa;
mod sha;
mod timer;
mod xdma;

pub mod hid;

use std::ptr;
use std::sync;
use std::default::Default;

use rt_data;
use io::regs::IoRegAccess;

pub enum IoRegion {
    Arm9(IoRegsArm9),
    Shared(IoRegsShared),
    Arm11,
}

pub fn new_devices(rt_rx: rt_data::Rx) -> (IoRegsArm9, IoRegsShared) {
    let pxi_device = sync::Arc::new(sync::Mutex::new(pxi::PxiDevice::new()));

    (IoRegsArm9 {
        config: config::ConfigDevice::new(),
        irq: irq::IrqDevice::new(),
        emmc: emmc::EmmcDevice::new(Default::default()),
        ndma: ndma::NdmaDevice::new(Default::default()),
        otp: otp::OtpDevice::new(Default::default()),
        pxi9: pxi_device.clone(),
        timer: timer::TimerDevice::new(Default::default()),
        aes: aes::AesDevice::new(Default::default()),
        sha: sha::ShaDevice::new(Default::default()),
        rsa: rsa::RsaDevice::new(Default::default()),
        xdma: xdma::XdmaDevice::new(),
        config_ext: config::ConfigExtDevice::new(),
    },
    IoRegsShared {
        hid: hid::HidDevice::new(rt_rx.hid_btn),
        pxi11: pxi_device.clone(),
    })
}

pub struct IoRegsArm9 {
    config: config::ConfigDevice,
    irq: irq::IrqDevice,
    ndma: ndma::NdmaDevice,
    timer: timer::TimerDevice,
    // ctrcard,
    emmc: emmc::EmmcDevice,
    pxi9: sync::Arc<sync::Mutex<pxi::PxiDevice>>,
    aes: aes::AesDevice,
    sha: sha::ShaDevice,
    rsa: rsa::RsaDevice,
    xdma: xdma::XdmaDevice,
    // spicard,
    config_ext: config::ConfigExtDevice,
    // prng,
    otp: otp::OtpDevice,
    // arm7,
}

impl IoRegsArm9 {
    pub unsafe fn read_reg(&mut self, offset: usize, buf: *mut u8, buf_size: usize) {
        let device: &mut IoRegAccess = match bits!(offset, 12 => 23) {
            0x00 => &mut self.config,
            0x01 => &mut self.irq,
            0x02 => &mut self.ndma,
            0x03 => &mut self.timer,
            0x06 => &mut self.emmc,
            0x08 => { self.pxi9.lock().unwrap().read_reg(offset & 0xFFF, buf, buf_size); return }
            0x09 => &mut self.aes,
            0x0A => &mut self.sha,
            0x0B => &mut self.rsa,
            0x0C => &mut self.xdma,
            0x10 => &mut self.config_ext,
            0x12 => &mut self.otp,
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
        let device: &mut IoRegAccess = match bits!(offset, 12 => 23) {
            0x00 => &mut self.config,
            0x01 => &mut self.irq,
            0x02 => &mut self.ndma,
            0x03 => &mut self.timer,
            0x06 => &mut self.emmc,
            0x08 => { self.pxi9.lock().unwrap().write_reg(offset & 0xFFF, buf, buf_size); return }
            0x09 => &mut self.aes,
            0x0A => &mut self.sha,
            0x0B => &mut self.rsa,
            0x0C => &mut self.xdma,
            0x10 => &mut self.config_ext,
            0x12 => &mut self.otp,
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
    hid: hid::HidDevice,
    // gpio,
    // mic,
    pxi11: sync::Arc<sync::Mutex<pxi::PxiDevice>>,
    // ntrcard,
    // mp,
}

impl IoRegsShared {
    pub unsafe fn read_reg(&mut self, offset: usize, buf: *mut u8, buf_size: usize) {
        let device: &mut IoRegAccess = match bits!(offset, 12 => 23) {
            0x46 => &mut self.hid,
            0x63 => { self.pxi11.lock().unwrap().read_reg(offset & 0xFFF, buf, buf_size); return }
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
        let device: &mut IoRegAccess = match bits!(offset, 12 => 23) {
            0x46 => &mut self.hid,
            0x63 => { self.pxi11.lock().unwrap().write_reg(offset & 0xFFF, buf, buf_size); return }
            _ => { error!("Unimplemented IO register write at offset 0x{:X}", offset); return },
        };
        device.write_reg(offset & 0xFFF, buf, buf_size);
    }
}