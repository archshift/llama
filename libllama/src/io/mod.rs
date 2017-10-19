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
    let cfg = config::ConfigDevice::new();
    let irq = irq::IrqDevice::new(irq_requests.clone());
    let emmc = emmc::EmmcDevice::new(emmc::EmmcDeviceState::new(irq_requests.clone()));
    let ndma = ndma::NdmaDevice::new(Default::default());
    let otp = otp::OtpDevice::new(Default::default());
    let pxi = Arc::new(Mutex::new(pxi::PxiDevice::new()));
    let timer = timer::TimerDevice::new(clk.timer_states);
    let aes = aes::AesDevice::new(aes::AesDeviceState::new(rt_rx.key_dmp));
    let sha = sha::ShaDevice::new(Default::default());
    let rsa = rsa::RsaDevice::new(Default::default());
    let xdma = xdma::XdmaDevice::new();
    let cfgext = config::ConfigExtDevice::new();

    let hid = hid::HidDevice::new(hid::HidDeviceState(rt_rx.hid_btn));

    (IoRegsArm9 {
        cfg:    Mutex::new(cfg),
        irq:    Mutex::new(irq),
        emmc:   Mutex::new(emmc),
        ndma:   Mutex::new(ndma),
        otp:    Mutex::new(otp),
        pxi9:   pxi.clone(),
        timer:  Mutex::new(timer),
        aes:    Mutex::new(aes),
        sha:    Mutex::new(sha),
        rsa:    Mutex::new(rsa),
        xdma:   Mutex::new(xdma),
        cfgext: Mutex::new(cfgext),
    },
    IoRegsShared {
        hid:    Mutex::new(hid),
        pxi11:  pxi.clone(),
    })
}

macro_rules! impl_rw {
    ($($num:expr => $name:tt),*) => {
        pub unsafe fn read_reg(&self, offset: usize, buf: *mut u8, buf_size: usize) {
            match bits!(offset, 12 => 23) {
                $($num => self.$name.lock().read_reg(offset & 0xFFF, buf, buf_size),)*
                _ => {
                    error!("Unimplemented IO register read at offset 0x{:X}", offset);
                    // If we can't find a register for it, just read zero bytes
                    ptr::write_bytes(buf, 0, buf_size);
                }
            }
        }
        pub unsafe fn write_reg(&self, offset: usize, buf: *const u8, buf_size: usize) {
            match bits!(offset, 12 => 23) {
                $($num => self.$name.lock().write_reg(offset & 0xFFF, buf, buf_size),)*
                _ => error!("Unimplemented IO register write at offset 0x{:X}", offset),
            };
        }
    };
}

pub struct IoRegsArm9 {
    cfg:    Mutex<config::ConfigDevice>,
    irq:    Mutex<irq::IrqDevice>,
    ndma:   Mutex<ndma::NdmaDevice>,
    timer:  Mutex<timer::TimerDevice>,
    // ctrcard,
    emmc:   Mutex<emmc::EmmcDevice>,
    pxi9:   Arc<Mutex<pxi::PxiDevice>>,
    aes:    Mutex<aes::AesDevice>,
    sha:    Mutex<sha::ShaDevice>,
    rsa:    Mutex<rsa::RsaDevice>,
    xdma:   Mutex<xdma::XdmaDevice>,
    // spicard,
    cfgext: Mutex<config::ConfigExtDevice>,
    // prng,
    otp:    Mutex<otp::OtpDevice>,
    // arm7,
}

impl IoRegsArm9 {
    impl_rw! {
        0x00 => cfg,
        0x01 => irq,
        0x02 => ndma,
        0x03 => timer,
        0x06 => emmc,
        0x08 => pxi9,
        0x09 => aes,
        0x0A => sha,
        0x0B => rsa,
        0x0C => xdma,
        0x10 => cfgext,
        0x12 => otp
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
    hid: Mutex<hid::HidDevice>,
    // gpio,
    // mic,
    pxi11: Arc<Mutex<pxi::PxiDevice>>,
    // ntrcard,
    // mp,
}

impl IoRegsShared {
    impl_rw! {
        0x46 => hid,
        0x63 => pxi11
    }
}