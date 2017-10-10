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
pub mod timer;
mod xdma;

pub mod hid;

use std::ptr;
use std::sync::Arc;
use std::default::Default;

use parking_lot::Mutex;

use clock;
use cpu::irq::IrqRequests;
use io::regs::IoRegAccess;
use rt_data;

pub enum IoRegion {
    Arm9(IoRegsArm9),
    Shared(IoRegsShared),
    Arm11,
}

pub fn new_devices(rt_rx: rt_data::Rx, irq_requests: IrqRequests,
                   clk: clock::SysClock) -> (IoRegsArm9, IoRegsShared) {
    let pxi_device = Arc::new(Mutex::new(pxi::PxiDevice::new()));

    (IoRegsArm9 {
        config: config::ConfigDevice::new(),
        irq: irq::IrqDevice::new(irq_requests.clone()),
        emmc: emmc::EmmcDevice::new(emmc::EmmcDeviceState::new(irq_requests.clone())),
        ndma: ndma::NdmaDevice::new(Default::default()),
        otp: otp::OtpDevice::new(Default::default()),
        pxi9: pxi_device.clone(),
        timer: timer::TimerDevice::new(clk.timer_states),
        aes: aes::AesDevice::new(aes::AesDeviceState::new(rt_rx.key_dmp)),
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
    pxi9: Arc<Mutex<pxi::PxiDevice>>,
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
            0x08 => { self.pxi9.lock().read_reg(offset & 0xFFF, buf, buf_size); return }
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
            0x08 => { self.pxi9.lock().write_reg(offset & 0xFFF, buf, buf_size); return }
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
    pxi11: Arc<Mutex<pxi::PxiDevice>>,
    // ntrcard,
    // mp,
}

impl IoRegsShared {
    pub unsafe fn read_reg(&mut self, offset: usize, buf: *mut u8, buf_size: usize) {
        let device: &mut IoRegAccess = match bits!(offset, 12 => 23) {
            0x46 => &mut self.hid,
            0x63 => { self.pxi11.lock().read_reg(offset & 0xFFF, buf, buf_size); return }
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
            0x63 => { self.pxi11.lock().write_reg(offset & 0xFFF, buf, buf_size); return }
            _ => { error!("Unimplemented IO register write at offset 0x{:X}", offset); return },
        };
        device.write_reg(offset & 0xFFF, buf, buf_size);
    }
}