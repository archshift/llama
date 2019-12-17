#[macro_use]
mod regs;

pub mod aes;
mod config;
mod emmc;
mod i2c;
mod irq;
mod ndma;
mod otp;
mod pxi;
mod rsa;
mod sha;
pub mod timer;
mod xdma;

pub mod hid;

pub mod gpu;

mod priv11;

use std::cell::RefCell;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::default::Default;
use std::rc::Rc;

use parking_lot::Mutex;

use clock;
use cpu::irq::IrqSubsys;
use hwcore::HardwareDma9;
use io::regs::IoRegAccess;
use mem::MemoryBlock;

pub struct DmaTrigger {
    val: Arc<AtomicBool>
}
impl DmaTrigger {
    fn new() -> (Self, DmaBus) {
        let val = Arc::new(AtomicBool::new(false));
        (Self { val: val.clone() }, DmaBus { val, union: None })
    }

    fn trigger(&mut self) {
        self.val.store(true, Ordering::SeqCst)
    }
}

#[derive(Clone)]
pub struct DmaBus {
    val: Arc<AtomicBool>,
    union: Option<Box<DmaBus>>
}

impl DmaBus {
    fn observe(&self) -> bool {
        let val = self.val.load(Ordering::SeqCst);
        if let Some(ref other) = self.union {
            val && other.observe()
        } else {
            val
        }
    }

    fn flush(&mut self) {
        self.val.store(false, Ordering::SeqCst);
        if let Some(ref mut other) = self.union {
            other.flush();
        }
    }

    fn union_with(self, other: DmaBus) -> Self {
        Self {
            union: Some(Box::new(other)),
            ..self
        }
    }
}

#[derive(Clone)]
pub struct DmaBuses {
    pub null: DmaBus,
    pub aes_in: DmaBus,
    pub aes_out: DmaBus,
    pub sha_in: DmaBus,
    pub sha_out: DmaBus,
}



pub fn new_devices(irq_subsys9: IrqSubsys, irq_subsys11: IrqSubsys,
                   clk: clock::SysClock, pica_hw: gpu::HardwarePica,
                   dma9_hw: HardwareDma9)
    -> (IoRegsArm9, IoRegsShared, IoRegsArm11, IoRegsArm11Priv) {
    
    macro_rules! make_dev_uniq {
        ($type:ty) => { Rc::new(RefCell::new( <$type>::new() )) };
        ($type:ty: $($arg:expr),+) => {{ Rc::new(RefCell::new( <$type>::new($($arg),*) )) }};
    }

    macro_rules! make_dev_shared {
        ($type:ty) => { Arc::new(Mutex::new(<$type>::new())) };
        ($type:ty: $($arg:expr),+) => {{ Arc::new(Mutex::new(<$type>::new($($arg),*))) }};
    }

    let pxi_shared = pxi::PxiShared::make_channel(irq_subsys9.async_tx, irq_subsys11.async_tx);
    let dma9_shared = Rc::new(RefCell::new(dma9_hw));

    let (_, dmabus_null) = DmaTrigger::new();
    let (dmatrg_aes_in, dmabus_aes_in) = DmaTrigger::new();
    let (dmatrg_aes_out, dmabus_aes_out) = DmaTrigger::new();
    let (dmatrg_sha_in, dmabus_sha_in) = DmaTrigger::new();
    let (dmatrg_sha_out, dmabus_sha_out) = DmaTrigger::new();

    let dma_buses = DmaBuses {
        null: dmabus_null,
        aes_in: dmabus_aes_in,
        aes_out: dmabus_aes_out,
        sha_in: dmabus_sha_in,
        sha_out: dmabus_sha_out,
    };

    let cfg    = make_dev_uniq! { config::ConfigDevice };
    let irq    = make_dev_uniq! { irq::IrqDevice:     irq_subsys9.agg };
    let emmc   = make_dev_uniq! { emmc::EmmcDevice:   emmc::EmmcDeviceState::new(irq_subsys9.sync_tx) };
    let otp    = make_dev_uniq! { otp::OtpDevice:     Default::default() };
    let pxi9   = make_dev_uniq! { pxi::PxiDevice:     pxi_shared.0 };
    let timer  = make_dev_uniq! { timer::TimerDevice: clk.timer_states };
    let aes    = make_dev_uniq! { aes::AesDevice:     aes::AesDeviceState::new(dmatrg_aes_in, dmatrg_aes_out) };
    let sha    = make_dev_uniq! { sha::ShaDevice:     sha::ShaDeviceState::new(dmatrg_sha_in, dmatrg_sha_out) };
    let rsa    = make_dev_uniq! { rsa::RsaDevice:     Default::default() };
    let cfgext = make_dev_uniq! { config::ConfigExtDevice };

    let ndma   = make_dev_uniq! { ndma::NdmaDevice:   ndma::NdmaDeviceState::new(dma9_shared.clone(), dma_buses.clone()) };
    let xdma   = make_dev_uniq! { xdma::XdmaDevice:   xdma::XdmaDeviceState::new(dma9_shared, dma_buses) };

    let pxi11  = make_dev_shared! { pxi::PxiDevice:   pxi_shared.1 };
    let hid    = make_dev_shared! { hid::HidDevice };
    let i2c    = make_dev_shared! { i2c::I2cDevice:   i2c::I2cDeviceState::new(i2c::make_peripherals()) };

    let pica_hw = Rc::new(RefCell::new(pica_hw));
    let lcd    = make_dev_uniq! { gpu::LcdDevice:     pica_hw.clone() };
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
        hid:    hid,
        i2c:    i2c,
        pxi11:  pxi11.clone(),
    },
    IoRegsArm11 {
        lcd:    lcd,
        gpu:    gpu,
    },
    IoRegsArm11Priv {
        priv11: priv11,
        gid:    gid,
    })
}

macro_rules! impl_rw {
    ($($num:expr => $name:tt),*) => {
        pub fn read_reg(&self, offset: usize, buf: &mut [u8]) {
            match bits!(offset, 12:23) {
                $($num => self.$name.borrow_mut().read_reg(offset & 0xFFF, buf),)*
                _ => {
                    error!("Unimplemented IO register read at offset 0x{:X}", offset);
                    // If we can't find a register for it, just read zero bytes
                    buf.iter_mut().for_each(|x| *x = 0);
                }
            }
        }
        pub fn write_reg(&mut self, offset: usize, buf: &[u8]) {
            match bits!(offset, 12:23) {
                $($num => self.$name.borrow_mut().write_reg(offset & 0xFFF, buf),)*
                _ => error!("Unimplemented IO register write at offset 0x{:X}", offset),
            };
        }
    };
}

macro_rules! impl_rw_locked {
    ($($num:expr => $name:tt),*) => {
        pub fn read_reg(&self, offset: usize, buf: &mut [u8]) {
            match bits!(offset, 12:23) {
                $($num => self.$name.lock().read_reg(offset & 0xFFF, buf),)*
                _ => {
                    error!("Unimplemented IO register read at offset 0x{:X}", offset);
                    // If we can't find a register for it, just read zero bytes
                    buf.iter_mut().for_each(|x| *x = 0);
                }
            }
        }
        pub fn write_reg(&mut self, offset: usize, buf: &[u8]) {
            match bits!(offset, 12:23) {
                $($num => self.$name.lock().write_reg(offset & 0xFFF, buf),)*
                _ => error!("Unimplemented IO register write at offset 0x{:X}", offset),
            };
        }
    };
}


#[derive(Clone)]
pub struct IoRegsArm9 {
    pub cfg:    Rc<RefCell< config::ConfigDevice >>,
    pub irq:    Rc<RefCell< irq::IrqDevice >>,
    pub ndma:   Rc<RefCell< ndma::NdmaDevice >>,
    pub timer:  Rc<RefCell< timer::TimerDevice >>,
    // ctrcard,
    pub emmc:   Rc<RefCell< emmc::EmmcDevice >>,
    pub pxi9:   Rc<RefCell< pxi::PxiDevice >>,
    pub aes:    Rc<RefCell< aes::AesDevice >>,
    pub sha:    Rc<RefCell< sha::ShaDevice >>,
    pub rsa:    Rc<RefCell< rsa::RsaDevice >>,
    pub xdma:   Rc<RefCell< xdma::XdmaDevice >>,
    // spicard,
    pub cfgext: Rc<RefCell< config::ConfigExtDevice >>,
    // prng,
    pub otp:    Rc<RefCell< otp::OtpDevice >>,
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
        self.read_reg(offset, buf);

        let mut xdma = self.xdma.borrow_mut();
        xdma::schedule(&mut *xdma);
        let mut ndma = self.ndma.borrow_mut();
        ndma::schedule(&mut *ndma);
    }

    fn write_buf(&mut self, offset: usize, buf: &[u8]) {
        self.write_reg(offset, buf);

        let mut xdma = self.xdma.borrow_mut();
        xdma::schedule(&mut *xdma);
        let mut ndma = self.ndma.borrow_mut();
        ndma::schedule(&mut *ndma);
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
    pub i2c: Arc<Mutex< i2c::I2cDevice >>,
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
        0x44 => i2c,
        0x46 => hid,
        0x63 => pxi11
    }
}


impl MemoryBlock for IoRegsShared {
    fn get_bytes(&self) -> u32 {
        (0x400 * 0x400) as u32
    }

    fn read_buf(&self, offset: usize, buf: &mut [u8]) {
        self.read_reg(offset, buf)
    }

    fn write_buf(&mut self, offset: usize, buf: &[u8]) {
        self.write_reg(offset, buf)
    }
}


#[derive(Clone)]
pub struct IoRegsArm11 {
    pub lcd:  Rc<RefCell< gpu::LcdDevice >>,
    pub gpu:  Rc<RefCell< gpu::GpuDevice >>,
}

impl IoRegsArm11 {
    impl_rw! {
        0x002 => lcd,
        0x200 => gpu
    }
}

impl MemoryBlock for IoRegsArm11 {
    fn get_bytes(&self) -> u32 {
        (0xC00 * 0x400) as u32
    }

    fn read_buf(&self, offset: usize, buf: &mut [u8]) {
        self.read_reg(offset, buf)
    }

    fn write_buf(&mut self, offset: usize, buf: &[u8]) {
        self.write_reg(offset, buf)
    }
}


#[derive(Clone)]
pub struct IoRegsArm11Priv {
    pub priv11: Rc<RefCell< priv11::Priv11Device >>,
    pub gid:    Rc<RefCell< priv11::GidDevice >>,
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
        self.read_reg(offset, buf)
    }

    fn write_buf(&mut self, offset: usize, buf: &[u8]) {
        self.write_reg(offset, buf)
    }
}
