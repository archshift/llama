use std::fmt;
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug)]
pub enum IrqType {
    Dmac1_0 = (1 << 0),
    Dmac1_1 = (1 << 1),
    Dmac1_2 = (1 << 2),
    Dmac1_3 = (1 << 3),
    Dmac1_4 = (1 << 4),
    Dmac1_5 = (1 << 5),
    Dmac1_6 = (1 << 6),
    Dmac1_7 = (1 << 7),
    Timer0  = (1 << 8),
    Timer1  = (1 << 9),
    Timer2  = (1 << 10),
    Timer3  = (1 << 11),
    PxiSync     = (1 << 12),
    PxiNotFull  = (1 << 13),
    PxiNotEmpty = (1 << 14),
    Aes        = (1 << 15),
    Sdio1      = (1 << 16),
    Sdio1Async = (1 << 17),
    Sdio3      = (1 << 18),
    Sdio3Async = (1 << 19),
    DebugRecv  = (1 << 20),
    DebugSend  = (1 << 21),
    RSA        = (1 << 22),
    CtrCard1   = (1 << 23),
    CtrCard2   = (1 << 24),
    Cgc        = (1 << 25),
    CgcDet     = (1 << 26),
    DsCard     = (1 << 27),
    Dmac2      = (1 << 28),
    Dmac2Abort = (1 << 29)
}


struct IrqRequestsInner {
    pending: u32,
    enabled: u32
}

#[derive(Clone)]
pub struct IrqRequests {
    irq_tx: IrqLine,
    inner: Arc<RwLock<IrqRequestsInner>>
}

impl fmt::Debug for IrqRequests {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "IrqRequests {{ }}")
    }
}

impl IrqRequests {
    fn new(irq_tx: IrqLine) -> IrqRequests {
        IrqRequests {
            irq_tx: irq_tx,
            inner: Arc::new(RwLock::new(IrqRequestsInner {
                pending: 0, enabled: 0
            }))
        }
    }

    fn update_line(line: &mut IrqLine, pending: u32, enabled: u32) {
        if pending & enabled != 0 {
            line.set_high();
        } else {
            line.set_low();
        }
    }

    fn mod_inner<T, F>(&mut self, f: F) -> T
        where F: FnOnce(&mut IrqRequestsInner) -> T {

        let mut inner = self.inner.write().unwrap();
        let res = f(&mut *inner);
        Self::update_line(&mut self.irq_tx, inner.pending, inner.enabled);
        res
    }

    pub fn get_pending(&self) -> u32 {
        let inner = self.inner.read().unwrap();
        inner.pending
    }

    pub fn set_enabled(&mut self, enabled: u32) {
        self.mod_inner(|inner| inner.enabled = enabled)
    }

    pub fn acknowledge(&mut self, irqs: u32) -> u32 {
        self.mod_inner(|inner| {
            inner.pending &= !irqs;
            inner.pending
        })
    }

    pub fn add(&mut self, t: IrqType) {
        trace!("Requesting interrupt {:?}", t);
        self.mod_inner(|inner| inner.pending |= t as u32)
    }

    pub fn clr(&mut self, t: IrqType) {
        self.mod_inner(|inner| inner.pending &= !(t as u32))
    }
}


#[derive(Clone)]
pub struct IrqLine {
    inner: Arc<AtomicBool>,
}

impl IrqLine {
    fn set_high(&mut self) {
        self.inner.store(true, Ordering::SeqCst);
    }
    fn set_low(&mut self) {
        self.inner.store(false, Ordering::SeqCst);
    }
    pub fn is_high(&self) -> bool {
        self.inner.load(Ordering::SeqCst)
    }
}

pub fn make_channel() -> (IrqRequests, IrqLine) {
    let line = IrqLine {
        inner: Arc::new(AtomicBool::new(false))
    };
    (IrqRequests::new(line.clone()), line)
}