use std::fmt;
use std::sync::Arc;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::cell::Cell;
use std::sync::mpsc;

pub trait IrqType: fmt::Debug + Copy + Clone {
    fn index(&self) -> u32;
}

#[derive(Debug, Copy, Clone)]
pub enum IrqType9 {
    Dmac1_0 = 0,
    Dmac1_1 = 1,
    Dmac1_2 = 2,
    Dmac1_3 = 3,
    Dmac1_4 = 4,
    Dmac1_5 = 5,
    Dmac1_6 = 6,
    Dmac1_7 = 7,
    Timer0  = 8,
    Timer1  = 9,
    Timer2  = 10,
    Timer3  = 11,
    PxiSync     = 12,
    PxiNotFull  = 13,
    PxiNotEmpty = 14,
    Aes        = 15,
    Sdio1      = 16,
    Sdio1Async = 17,
    Sdio3      = 18,
    Sdio3Async = 19,
    DebugRecv  = 20,
    DebugSend  = 21,
    RSA        = 22,
    CtrCard1   = 23,
    CtrCard2   = 24,
    Cgc        = 25,
    CgcDet     = 26,
    DsCard     = 27,
    Dmac2      = 28,
    Dmac2Abort = 29
}

impl IrqType for IrqType9 {
    fn index(&self) -> u32 {
        *self as u32
    }
}

#[derive(Debug, Copy, Clone)]
pub enum IrqType11 {
    PxiSync = 80,
}

impl IrqType for IrqType11 {
    fn index(&self) -> u32 {
        *self as u32
    }
}





type RcMut<T> = Rc<Cell<T>>;

type AsyncLine = Arc<AtomicBool>;
type SyncLine = RcMut<bool>;
type AsyncEnabled = Arc<[AtomicUsize; 4]>;
type SyncEnabled = RcMut<u128>;

/// CPU end of the IRQ subsystem, just a simple triggered/not triggered API
#[derive(Clone)]
pub struct IrqLine {
    pub(crate) async: AsyncLine,
    pub(crate) sync: SyncLine, 
}

impl IrqLine {
    pub fn is_high(&self) -> bool {
        self.async.load(Ordering::SeqCst) || self.sync.get()
    }
}




pub trait IrqClient: Clone {
    fn assert<IRQ: IrqType>(&mut self, irq: IRQ);
    fn is_enabled(&self, u32) -> bool;
}

pub trait IrqServer {
    fn pop(&mut self) -> Option<u32>;
}



#[derive(Clone)]
pub struct IrqSyncClient {
    irqs: RcMut<u128>,
    enabled: SyncEnabled, 
    line: SyncLine,
}

impl IrqClient for IrqSyncClient {
    fn assert<IRQ: IrqType>(&mut self, irq: IRQ) {
        let index = irq.index();
        if !self.is_enabled(index) {
            return
        }

        let set = self.irqs.get();
        self.irqs.set(set | 1u128 << (index as u128));
        self.line.set(true);
    }

    fn is_enabled(&self, index: u32) -> bool {
        self.enabled.get() & (1u128 << (index as u128)) != 0
    }
}



pub struct IrqSyncServer {
    irqs: RcMut<u128>
}

impl IrqServer for IrqSyncServer {
    fn pop(&mut self) -> Option<u32> {
        let set = self.irqs.get();
        if set == 0 {
            return None
        }
        let out = set.trailing_zeros();
        self.irqs.set(set & !(1 << out));
        Some(out)
    }
}


#[derive(Clone)]
pub struct IrqAsyncClient {
    irqs: mpsc::Sender<u32>,
    enabled: AsyncEnabled, 
    line: AsyncLine,
}

impl IrqClient for IrqAsyncClient {
    fn assert<IRQ: IrqType>(&mut self, irq: IRQ) {
        let index = irq.index();
        if !self.is_enabled(index) {
            return
        }
        self.irqs.send(index).unwrap();
        self.line.store(true, Ordering::SeqCst);
    }

    fn is_enabled(&self, index: u32) -> bool {
        let which_word = (index / 32) as usize;
        let enabled = self.enabled[which_word].load(Ordering::SeqCst) as u32;
        enabled & (1 << (index % 32)) != 0
    }
}

pub struct IrqAsyncServer {
    irqs: mpsc::Receiver<u32>
}

impl IrqServer for IrqAsyncServer {
    fn pop(&mut self) -> Option<u32> {
        self.irqs.try_recv().ok()
    }
}


pub struct Aggregator {
    async_rx: IrqAsyncServer,
    async_enabled: AsyncEnabled,
    sync_rx: IrqSyncServer,
    sync_enabled: SyncEnabled,
    line: IrqLine,
    pending: u128,
}

impl fmt::Debug for Aggregator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Aggregator")
    }
}

impl Aggregator {
    pub(crate) fn set_enabled(&self, new: u128) {
        self.sync_enabled.set(new);
        self.async_enabled[0].store(new as u32 as usize, Ordering::SeqCst);
        self.async_enabled[1].store((new >> 32) as u32 as usize, Ordering::SeqCst);
        self.async_enabled[2].store((new >> 64) as u32 as usize, Ordering::SeqCst);
        self.async_enabled[3].store((new >> 96) as u32 as usize, Ordering::SeqCst);
    }

    pub(crate) fn drain_asserts(&mut self) -> u128 {
        let mut pending = self.pending;
        while let Some(x) = self.sync_rx.pop() {
            pending |= 1 << x;
        }
        while let Some(x) = self.async_rx.pop() {
            pending |= 1 << x;
        }
        self.pending = pending;
        pending
    }

    pub(crate) fn acknowledge(&mut self, which: u128) -> u128 {
        self.pending = self.drain_asserts() & !which;
        self.line.sync.set(self.pending != 0);
        self.line.async.store(self.pending != 0, Ordering::SeqCst);
        self.pending
    }
}

pub struct IrqSubsys {
    pub(crate) agg: Aggregator,
    pub(crate) sync_tx: IrqSyncClient,
    pub(crate) async_tx: IrqAsyncClient,
    pub(crate) line: IrqLine
}

impl IrqSubsys {
    pub(crate) fn create() -> Self {
        let line = IrqLine {
            sync: Rc::new(Cell::new(false)),
            async: Arc::new(AtomicBool::new(false))
        };
        let sync_enabled = Rc::new(Cell::new(0));
        let async_enabled = Arc::new([AtomicUsize::new(0), AtomicUsize::new(0), AtomicUsize::new(0), AtomicUsize::new(0)]);
        let sync_irqs = Rc::new(Cell::new(0));
        let (async_irqs_tx, async_irqs_rx) = mpsc::channel();
        Self {
            agg: Aggregator {
                async_rx: IrqAsyncServer {
                    irqs: async_irqs_rx,
                },
                async_enabled: async_enabled.clone(), 
                sync_rx: IrqSyncServer {
                    irqs: sync_irqs.clone()
                },
                sync_enabled: sync_enabled.clone(),
                line: line.clone(),
                pending: 0
            },
            sync_tx: IrqSyncClient {
                line: line.sync.clone(),
                enabled: sync_enabled,
                irqs: sync_irqs,
            },
            async_tx: IrqAsyncClient {
                line: line.async.clone(),
                enabled: async_enabled,
                irqs: async_irqs_tx,
            },
            line: line
        }
    }
}

