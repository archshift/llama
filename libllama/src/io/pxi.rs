use std::sync::{Arc, atomic, mpsc};

bf!(RegSync[u32] {
    data_recv: 0:7,
    data_sent: 8:15
});

bf!(RegCnt[u16] {
    send_empty: 0:0,
    send_full: 1:1,
    flush_send: 3:3,
    recv_empty: 8:8,
    recv_full: 9:9,
    cannot_rw: 14:14
});

fn reg_sync_read(dev: &mut PxiDevice) {
    let byte = dev._internal_state.sync_rx.load(atomic::Ordering::SeqCst) as u32;
    let sync_ref = RegSync::alias_mut(dev.sync.ref_mut());
    sync_ref.data_recv.set(byte);
}

fn reg_sync_write(dev: &mut PxiDevice) {
    let sync = RegSync::new(dev.sync.get());
    let byte = sync.data_sent.get();
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

#[derive(Debug)]
pub struct PxiShared {
    tx_count: Arc<atomic::AtomicUsize>,
    rx_count: Arc<atomic::AtomicUsize>,
    tx: mpsc::SyncSender<u32>,
    rx: mpsc::Receiver<u32>,
    sync_tx: Arc<atomic::AtomicUsize>,
    sync_rx: Arc<atomic::AtomicUsize>,
}

impl PxiShared {
    pub fn make_channel() -> (PxiShared, PxiShared) {
        let count_1rx_2tx = Arc::new(atomic::AtomicUsize::new(0));
        let count_2rx_1tx = Arc::new(atomic::AtomicUsize::new(0));
        let sync_1rx_2tx = Arc::new(atomic::AtomicUsize::new(0));
        let sync_2rx_1tx = Arc::new(atomic::AtomicUsize::new(0));
        let (pxi1_tx, pxi2_rx) = mpsc::sync_channel(4);
        let (pxi2_tx, pxi1_rx) = mpsc::sync_channel(4);

        let pxi1 = PxiShared {
            tx_count: count_2rx_1tx.clone(),
            rx_count: count_1rx_2tx.clone(),
            tx: pxi1_tx,
            rx: pxi1_rx,
            sync_tx: sync_2rx_1tx.clone(),
            sync_rx: sync_1rx_2tx.clone(),
        };

        let pxi2 = PxiShared {
            tx_count: count_1rx_2tx,
            rx_count: count_2rx_1tx,
            tx: pxi2_tx,
            rx: pxi2_rx,
            sync_tx: sync_1rx_2tx,
            sync_rx: sync_2rx_1tx,
        };
        (pxi1, pxi2)
    }
}

iodevice!(PxiDevice, {
    internal_state: PxiShared;
    regs: {
        0x000 => sync: u32 {
            // write_bits = 0xFFFFFF00;
            read_effect = reg_sync_read;
            write_effect = reg_sync_write;
        }
        0x004 => cnt: u16 {
            write_bits = 0b11000100_00001100;
            read_effect = reg_cnt_read;
            write_effect = reg_cnt_write;
        }
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
                    Err(mpsc::TryRecvError::Empty) => panic!("Attempted to receive PXI word while FIFO empty"),
                    Err(e) => panic!("{:?}", e),
                };
                dev.recv.set_unchecked(dat);
            };
        }
    }
});
