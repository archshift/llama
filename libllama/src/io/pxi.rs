use std::sync::{Arc, atomic, mpsc};
use std::fmt;

use cpu::irq::{self, IrqClient};

bf!(RegCnt[u16] {
    send_empty: 0:0,
    send_full: 1:1,
    flush_send: 3:3,
    recv_empty: 8:8,
    recv_full: 9:9,
    cannot_rw: 14:14
});

bf!(RegSyncCtrl[u8] {
    trigger_irq11: 5:5,
    trigger_irq9: 6:6,
    irq_enabled: 7:7
});

fn reg_sync_read(dev: &mut PxiDevice) {
    let byte = dev._internal_state.sync_rx.load(atomic::Ordering::SeqCst) as u8;
    trace!("Read {:X} from PXI_SYNC", byte);
    dev.sync_recv.set_unchecked(byte);
}

fn reg_sync_write(dev: &mut PxiDevice) {
    let byte = dev.sync_send.get();
    trace!("Wrote {:X} to PXI_SYNC", byte);
    dev._internal_state.sync_tx.store(byte as usize, atomic::Ordering::SeqCst);
}

fn reg_cnt_read(dev: &mut PxiDevice) {
    let cnt = RegCnt::alias_mut(dev.cnt.ref_mut());
    let tx_count = dev._internal_state.tx_count.load(atomic::Ordering::SeqCst);
    let rx_count = dev._internal_state.rx_count.load(atomic::Ordering::SeqCst);
    cnt.send_empty.set((tx_count == 0) as u16);
    cnt.send_full.set((tx_count == 4) as u16);
    cnt.recv_empty.set((rx_count == 0) as u16);
    cnt.recv_full.set((rx_count == 4) as u16);
}

fn reg_cnt_write(dev: &mut PxiDevice) {
    let cnt = RegCnt::alias_mut(dev.cnt.ref_mut());
    if (cnt.flush_send.get() == 1) {
        warn!("STUBBED: cannot flush PXI tx channel!");
        cnt.flush_send.set(0);
    }
    if (cnt.cannot_rw.get() == 1) {
        cnt.cannot_rw.set(0);
    }
    warn!("STUBBED: Write to PXI_CNT");
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
enum PxiEnd {
    Arm9,
    Arm11
}

pub struct PxiShared {
    end: PxiEnd,

    tx_count: Arc<atomic::AtomicUsize>,
    rx_count: Arc<atomic::AtomicUsize>,
    tx: mpsc::SyncSender<u32>,
    rx: mpsc::Receiver<u32>,
    sync_tx: Arc<atomic::AtomicUsize>,
    sync_rx: Arc<atomic::AtomicUsize>,

    irq_enabled: Arc<atomic::AtomicBool>,
    other_irq_enabled: Arc<atomic::AtomicBool>,
    irq_client: irq::IrqAsyncClient,
}

impl PxiShared {
    pub fn make_channel(irq9: irq::IrqAsyncClient, irq11: irq::IrqAsyncClient) -> (PxiShared, PxiShared) {
        let count_1rx_2tx = Arc::new(atomic::AtomicUsize::new(0));
        let count_2rx_1tx = Arc::new(atomic::AtomicUsize::new(0));
        let sync_1rx_2tx = Arc::new(atomic::AtomicUsize::new(0));
        let sync_2rx_1tx = Arc::new(atomic::AtomicUsize::new(0));
        let (pxi1_tx, pxi2_rx) = mpsc::sync_channel(4);
        let (pxi2_tx, pxi1_rx) = mpsc::sync_channel(4);
        let irq_1enabled = Arc::new(atomic::AtomicBool::new(false));
        let irq_2enabled = Arc::new(atomic::AtomicBool::new(false));

        let pxi11 = PxiShared {
            end: PxiEnd::Arm11,

            tx_count: count_2rx_1tx.clone(),
            rx_count: count_1rx_2tx.clone(),
            tx: pxi1_tx,
            rx: pxi1_rx,
            sync_tx: sync_2rx_1tx.clone(),
            sync_rx: sync_1rx_2tx.clone(),

            irq_enabled: irq_1enabled.clone(),
            other_irq_enabled: irq_2enabled.clone(),
            irq_client: irq9,
        };

        let pxi9 = PxiShared {
            end: PxiEnd::Arm9,

            tx_count: count_1rx_2tx,
            rx_count: count_2rx_1tx,
            tx: pxi2_tx,
            rx: pxi2_rx,
            sync_tx: sync_1rx_2tx,
            sync_rx: sync_2rx_1tx,

            irq_enabled: irq_2enabled,
            other_irq_enabled: irq_1enabled,
            irq_client: irq11,
        };
        (pxi9, pxi11)
    }
}

impl fmt::Debug for PxiShared {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.debug_struct("PxiShared")
            .field("end", &self.end)
            .field("tx_count", &self.tx_count)
            .field("rx_count", &self.rx_count)
            .field("tx", &self.tx)
            .field("rx", &self.rx)
            .field("sync_tx", &self.sync_tx)
            .field("sync_rx", &self.sync_rx)
            .field("irq_enabled", &self.irq_enabled)
            .finish()
    }
}

fn reg_sync_ctrl_write(dev: &mut PxiDevice) {
    let cntl = RegSyncCtrl::alias_mut(dev.sync_ctrl.ref_mut());
    let irq_enabled = cntl.irq_enabled.get() == 1;

    let state = &mut dev._internal_state;

    if cntl.trigger_irq9.get() == 1 {
        // Assume for now we're only targeting opposite PXI end
        assert!(state.end == PxiEnd::Arm11);
        if state.other_irq_enabled.load(atomic::Ordering::Relaxed) {
            info!("Triggering PXI sync on ARM9");
            state.irq_client.assert(irq::IrqType9::PxiSync);
        }
    }
    if cntl.trigger_irq11.get() == 1 {
        // Assume for now we're only targeting opposite PXI end
        assert!(state.end == PxiEnd::Arm9);
        if state.other_irq_enabled.load(atomic::Ordering::Relaxed) {
            info!("Triggering PXI sync on ARM11");
            state.irq_client.assert(irq::IrqType11::PxiSync);
        }
    }

    cntl.trigger_irq9.set(0);
    cntl.trigger_irq11.set(0);

    state.irq_enabled.store(irq_enabled, atomic::Ordering::Relaxed);
}

iodevice!(PxiDevice, {
    internal_state: PxiShared;
    regs: {
        0x000 => sync_recv: u8 {
            // write_bits = 0xFFFFFF00;
            read_effect = reg_sync_read;
        }
        0x001 => sync_send: u8 {
            write_effect = reg_sync_write;
        }
        0x002 => sync_unk: u8 {
            write_effect = |_| trace!("STUBBED: Write to PXI unknown sync ctrl register!");
        }
        0x003 => sync_ctrl: u8 {
            write_effect = reg_sync_ctrl_write;
        }
        0x004 => cnt: u16 {
            write_bits = 0b11000100_00001100;
            read_effect = reg_cnt_read;
            write_effect = reg_cnt_write;
        }
        0x006 => cnt_ext: u16 { }
        0x008 => send: u32 {
            write_effect = |dev: &mut PxiDevice| {
                let dat = dev.send.get();
                match dev._internal_state.tx.try_send(dat) {
                    Ok(_) => dev._internal_state.tx_count.fetch_add(1, atomic::Ordering::SeqCst),
                    Err(mpsc::TrySendError::Full(_)) => panic!("Attempted to send PXI word while FIFO full"),
                    Err(e) => panic!("{:?}", e),
                }
            };
        }
        0x00C => recv: u32 {
            read_effect = |dev: &mut PxiDevice| {
                let dat = match dev._internal_state.rx.try_recv() {
                    Ok(dat) => {
                        dev._internal_state.rx_count.fetch_sub(1, atomic::Ordering::SeqCst);
                        dat
                    }
                    Err(mpsc::TryRecvError::Empty) => {
                        debug!("Attempted to receive PXI word while FIFO empty");
                        return
                    }
                    Err(e) => panic!("{:?}", e),
                };
                dev.recv.set_unchecked(dat);
            };
        }
    }
});
