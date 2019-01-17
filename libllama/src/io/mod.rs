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

mod fbuf;
pub mod gpu;

mod priv11;

use std::ptr;
use std::cell::RefCell;
use std::sync::Arc;
use std::default::Default;
use std::rc::Rc;

use parking_lot::Mutex;

use clock;
use cpu::irq::IrqSubsys;
use io::regs::IoRegAccess;
use mem::MemoryBlock;

pub fn new_devices(irq_subsys9: IrqSubsys, irq_subsys11: IrqSubsys,
                   clk: clock::SysClock, pica_hw: gpu::HardwarePica)
    -> (IoRegsArm9, IoRegsShared, IoRegsArm11, IoRegsArm11Priv) {
    
    macro_rules! make_dev_uniq {
        ($type:ty) => { RefCell::new( <$type>::new() ) };
        ($type:ty: $($arg:expr),+) => {{ RefCell::new( <$type>::new($($arg),*) ) }};
    }

    macro_rules! make_dev_shared {
        ($type:ty) => { Arc::new(Mutex::new(<$type>::new())) };
        ($type:ty: $($arg:expr),+) => {{ Arc::new(Mutex::new(<$type>::new($($arg),*))) }};
    }

    let pxi_shared = pxi::PxiShared::make_channel(irq_subsys9.async_tx, irq_subsys11.async_tx);

    let cfg    = make_dev_uniq! { config::ConfigDevice };
    let irq    = make_dev_uniq! { irq::IrqDevice:     irq_subsys9.agg };
    let emmc   = make_dev_uniq! { emmc::EmmcDevice:   emmc::EmmcDeviceState::new(irq_subsys9.sync_tx) };
    let ndma   = make_dev_uniq! { ndma::NdmaDevice:   Default::default() };
    let otp    = make_dev_uniq! { otp::OtpDevice:     Default::default() };
    let pxi9   = make_dev_uniq! { pxi::PxiDevice:     pxi_shared.0 };
    let timer  = make_dev_uniq! { timer::TimerDevice: clk.timer_states };
    let aes    = make_dev_uniq! { aes::AesDevice:     Default::default() };
    let sha    = make_dev_uniq! { sha::ShaDevice:     Default::default() };
    let rsa    = make_dev_uniq! { rsa::RsaDevice:     Default::default() };
    let xdma   = make_dev_uniq! { xdma::XdmaDevice };
    let cfgext = make_dev_uniq! { config::ConfigExtDevice };

    let pxi11  = make_dev_shared! { pxi::PxiDevice:   pxi_shared.1 };
    let hid    = make_dev_shared! { hid::HidDevice };

    let fbuf   = make_dev_uniq! { fbuf::FbufDevice };
    let gpu    = make_dev_uniq! { gpu::GpuDevice:     pica_hw };

    let irq11_agg = Rc::new(RefCell::new(irq_subsys11.agg));
    let priv11 = make_dev_uniq! { priv11::Priv11Device: irq11_agg.clone() };
    let gid    = make_dev_uniq! { priv11::GidDevice:  priv11::GidState::new(irq11_agg.clone()) };

    (IoRegsArm9 {
        cfg:    cfg,
        irq:    irq,
        emmc:   emmc,
        ndma:   ndma,
        otp:    otp,
        pxi9:   pxi9,
        timer:  timer,
        aes:    aes,
        sha:    sha,
        rsa:    rsa,
        xdma:   xdma,
        cfgext: cfgext,
    },
    IoRegsShared {
        hid:    hid.clone(),
        pxi11:  pxi11.clone(),
    },
    IoRegsArm11 {
        fbuf:   fbuf,
        gpu:    gpu,
    },
    IoRegsArm11Priv {
        priv11: priv11,
        gid:    gid,
    })
}

macro_rules! impl_rw {
    ($($num:expr => $name:tt),*) => {
        pub unsafe fn read_reg(&self, offset: usize, buf: *mut u8, buf_size: usize) {
            match bits!(offset, 12:23) {
                $($num => self.$name.borrow_mut().read_reg(offset & 0xFFF, buf, buf_size),)*
                _ => {
                    error!("Unimplemented IO register read at offset 0x{:X}", offset);
                    // If we can't find a register for it, just read zero bytes
                    ptr::write_bytes(buf, 0, buf_size);
                }
            }
        }
        pub unsafe fn write_reg(&self, offset: usize, buf: *const u8, buf_size: usize) {
            match bits!(offset, 12:23) {
                $($num => self.$name.borrow_mut().write_reg(offset & 0xFFF, buf, buf_size),)*
                _ => error!("Unimplemented IO register write at offset 0x{:X}", offset),
            };
        }
    };
}

macro_rules! impl_rw_locked {
    ($($num:expr => $name:tt),*) => {
        pub unsafe fn read_reg(&self, offset: usize, buf: *mut u8, buf_size: usize) {
            match bits!(offset, 12:23) {
                $($num => self.$name.lock().read_reg(offset & 0xFFF, buf, buf_size),)*
                _ => {
                    error!("Unimplemented IO register read at offset 0x{:X}", offset);
                    // If we can't find a register for it, just read zero bytes
                    ptr::write_bytes(buf, 0, buf_size);
                }
            }
        }
        pub unsafe fn write_reg(&mut self, offset: usize, buf: *const u8, buf_size: usize) {
            match bits!(offset, 12:23) {
                $($num => self.$name.lock().write_reg(offset & 0xFFF, buf, buf_size),)*
                _ => error!("Unimplemented IO register write at offset 0x{:X}", offset),
            };
        }
    };
}


pub struct IoRegsArm9 {
    pub cfg:    RefCell< config::ConfigDevice >,
    pub irq:    RefCell< irq::IrqDevice >,
    pub ndma:   RefCell< ndma::NdmaDevice >,
    pub timer:  RefCell< timer::TimerDevice >,
    // ctrcard,
    pub emmc:   RefCell< emmc::EmmcDevice >,
    pub pxi9:   RefCell< pxi::PxiDevice >,
    pub aes:    RefCell< aes::AesDevice >,
    pub sha:    RefCell< sha::ShaDevice >,
    pub rsa:    RefCell< rsa::RsaDevice >,
    pub xdma:   RefCell< xdma::XdmaDevice >,
    // spicard,
    pub cfgext: RefCell< config::ConfigExtDevice >,
    // prng,
    pub otp:    RefCell< otp::OtpDevice >,
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

impl MemoryBlock for IoRegsArm9 {
    fn get_bytes(&self) -> u32 {
        (0x400 * 0x400) as u32
    }

    fn read_buf(&self, offset: usize, buf: &mut [u8]) {
        unsafe { self.read_reg(offset, buf.as_mut_ptr(), buf.len()) }
    }

    fn write_buf(&mut self, offset: usize, buf: &[u8]) {
        unsafe { self.write_reg(offset, buf.as_ptr(), buf.len()) }
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
    impl_rw_locked! {
        0x46 => hid,
        0x63 => pxi11
    }
}


impl MemoryBlock for IoRegsShared {
    fn get_bytes(&self) -> u32 {
        (0x400 * 0x400) as u32
    }

    fn read_buf(&self, offset: usize, buf: &mut [u8]) {
        unsafe { self.read_reg(offset, buf.as_mut_ptr(), buf.len()) }
    }

    fn write_buf(&mut self, offset: usize, buf: &[u8]) {
        unsafe { self.write_reg(offset, buf.as_ptr(), buf.len()) }
    }
}


pub struct IoRegsArm11 {
    pub fbuf: RefCell< fbuf::FbufDevice >,
    pub gpu:  RefCell< gpu::GpuDevice >,
}

impl IoRegsArm11 {
    impl_rw! {
        0x002 => fbuf,
        0x200 => gpu
    }
}

impl MemoryBlock for IoRegsArm11 {
    fn get_bytes(&self) -> u32 {
        (0xC00 * 0x400) as u32
    }

    fn read_buf(&self, offset: usize, buf: &mut [u8]) {
        unsafe { self.read_reg(offset, buf.as_mut_ptr(), buf.len()) }
    }

    fn write_buf(&mut self, offset: usize, buf: &[u8]) {
        unsafe { self.write_reg(offset, buf.as_ptr(), buf.len()) }
    }
}


pub struct IoRegsArm11Priv {
    pub priv11: RefCell< priv11::Priv11Device >,
    pub gid:    RefCell< priv11::GidDevice >,
}

impl IoRegsArm11Priv {
    impl_rw! {
        0x0 => priv11,
        0x1 => gid
    }
}

impl MemoryBlock for IoRegsArm11Priv {
    fn get_bytes(&self) -> u32 {
        (8 * 0x400) as u32
    }

    fn read_buf(&self, offset: usize, buf: &mut [u8]) {
        unsafe { self.read_reg(offset, buf.as_mut_ptr(), buf.len()) }
    }

    fn write_buf(&mut self, offset: usize, buf: &[u8]) {
        unsafe { self.write_reg(offset, buf.as_ptr(), buf.len()) }
    }
}
