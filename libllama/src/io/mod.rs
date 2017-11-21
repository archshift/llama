#[macro_use]
mod regs;

pub mod aes;
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

#[derive(Clone)]
pub enum IoRegion {
    Arm9(IoRegsArm9),
    Shared(IoRegsShared),
    Arm11,
}

pub fn new_devices(irq_requests: IrqRequests, clk: clock::SysClock) -> (IoRegsArm9, IoRegsShared) {
    macro_rules! make_dev {
        ($type:ty) => { Arc::new(Mutex::new(<$type>::new())) };
        ($type:ty: $($arg:expr),+) => {{ Arc::new(Mutex::new(<$type>::new($($arg),*))) }};
    }

    let cfg    = make_dev! { config::ConfigDevice };
    let irq    = make_dev! { irq::IrqDevice:     irq_requests.clone() };
    let emmc   = make_dev! { emmc::EmmcDevice:   emmc::EmmcDeviceState::new(irq_requests.clone()) };
    let ndma   = make_dev! { ndma::NdmaDevice:   Default::default() };
    let otp    = make_dev! { otp::OtpDevice:     Default::default() };
    let pxi    = make_dev! { pxi::PxiDevice };
    let timer  = make_dev! { timer::TimerDevice: clk.timer_states };
    let aes    = make_dev! { aes::AesDevice:     Default::default() };
    let sha    = make_dev! { sha::ShaDevice:     Default::default() };
    let rsa    = make_dev! { rsa::RsaDevice:     Default::default() };
    let xdma   = make_dev! { xdma::XdmaDevice };
    let cfgext = make_dev! { config::ConfigExtDevice };

    let hid    = make_dev! { hid::HidDevice };

    (IoRegsArm9 {
        cfg:    cfg.clone(),
        irq:    irq.clone(),
        emmc:   emmc.clone(),
        ndma:   ndma.clone(),
        otp:    otp.clone(),
        pxi9:   pxi.clone(),
        timer:  timer.clone(),
        aes:    aes.clone(),
        sha:    sha.clone(),
        rsa:    rsa.clone(),
        xdma:   xdma.clone(),
        cfgext: cfgext.clone(),
    },
    IoRegsShared {
        hid:    hid.clone(),
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

#[derive(Clone)]
pub struct IoRegsArm9 {
    pub cfg:    Arc<Mutex< config::ConfigDevice >>,
    pub irq:    Arc<Mutex< irq::IrqDevice >>,
    pub ndma:   Arc<Mutex< ndma::NdmaDevice >>,
    pub timer:  Arc<Mutex< timer::TimerDevice >>,
    // ctrcard,
    pub emmc:   Arc<Mutex< emmc::EmmcDevice >>,
    pub pxi9:   Arc<Mutex< pxi::PxiDevice >>,
    pub aes:    Arc<Mutex< aes::AesDevice >>,
    pub sha:    Arc<Mutex< sha::ShaDevice >>,
    pub rsa:    Arc<Mutex< rsa::RsaDevice >>,
    pub xdma:   Arc<Mutex< xdma::XdmaDevice >>,
    // spicard,
    pub cfgext: Arc<Mutex< config::ConfigExtDevice >>,
    // prng,
    pub otp:    Arc<Mutex< otp::OtpDevice >>,
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


#[derive(Clone)]
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
    pub hid: Arc<Mutex< hid::HidDevice >>,
    // gpio,
    // mic,
    pub pxi11: Arc<Mutex< pxi::PxiDevice >>,
    // ntrcard,
    // mp,
}

impl IoRegsShared {
    impl_rw! {
        0x46 => hid,
        0x63 => pxi11
    }
}