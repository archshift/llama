use std::fmt;
use std::cell::RefCell;

use openssl::hash::{Hasher, MessageDigest};

use io::xdma::DmaBus;

bf!(RegCnt[u32] {
    busy: 0:0,
    final_round: 1:1,
    _enable_irq0: 2:2,
    big_endian: 3:3,
    hash_mode: 4:5,
    clear_fifo: 8:8,
    _enable_fifo: 9:9,
    _enable_irq1: 10:10
});

#[derive(Default)]
pub struct ShaDeviceState {
    hasher: Option<Hasher>,
    hash: [u8; 32],
}

impl fmt::Debug for ShaDeviceState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ShaDeviceState {{ }}")
    }
}

// TODO: The following implementation does not yet completely work, and still needs
// more hardware testing to determine the source of errors.

fn reg_cnt_update(dev: &mut ShaDevice) {
    let cnt = RegCnt::alias_mut(dev.cnt.ref_mut());
    let state = &mut dev._internal_state;
    trace!("Wrote 0x{:08X} to SHA CNT register!", cnt.val);
    trace!("SHA hasher state: {}", if state.hasher.is_some() { "active" } else { "inactive" });
    if cnt.val & 0xFFFF00C0 != 0 {
        warn!("Wrote UNIMPLEMENTED bits 0x{:08X} to SHA CNT register!", cnt.val);
    }

    if cnt.final_round.get() == 1 {
        trace!("Reached end of SHA final round!");
        if let Some(ref mut h) = state.hasher {
            let hash_slice = &*h.finish().unwrap();
            state.hash = [0u8; 32];
            trace!("SHA output hash: {:X?}", hash_slice);
            state.hash[0..hash_slice.len()].copy_from_slice(hash_slice);
        }

        cnt.final_round.set(0);
    }

    if cnt.busy.get() == 0 {
        state.hasher = None;
    } else if state.hasher.is_none() {
        // Create new hasher
        trace!("Starting SHA hasher");

        let mode = match cnt.hash_mode.get() {
            0b00 => MessageDigest::sha256(),
            0b01 => MessageDigest::sha224(),
            _ => MessageDigest::sha1()
        };
        state.hasher = Some(Hasher::new(mode).unwrap());
    }

    cnt.busy.set(0);
}

fn reg_hash_read(dev: &mut ShaDevice, buf_pos: usize, dest: &mut [u8]) {
    let src_slice = &dev._internal_state.hash[buf_pos .. buf_pos + dest.len()];
    dest.copy_from_slice(src_slice);
    trace!("Reading {} bytes from SHA HASH at +0x{:X}: {:X?}", dest.len(), buf_pos, dest);

    let cnt = RegCnt::new(dev.cnt.get());
    assert_eq!(cnt.big_endian.get(), 1);
}

// TODO: Does a word written to any part of the hash just add on? What about
// writing to the fifo in reverse? How does this work?
fn reg_fifo_write(dev: &mut ShaDevice, buf_pos: usize, source: &[u8]) {
    trace!("Writing {} bytes to SHA FIFO at +0x{:X}: {:X?}", source.len(), buf_pos, source);

    let cnt = RegCnt::new(dev.cnt.get());
    assert_eq!(cnt.big_endian.get(), 1);

    let hasher = match dev._internal_state.hasher {
        Some(ref mut h) => h,
        None => return
    };

    hasher.update(source).unwrap();
}

iodevice!(ShaDevice, {
    internal_state: ShaDeviceState;
    regs: {
        0x000 => cnt: u32 { write_effect = reg_cnt_update; }
        0x004 => blk_cnt: u32 {
            read_effect = |_| {
                warn!("STUBBED: read from SHA BLK_CNT register");
            };
            write_effect = |dev: &ShaDevice| {
                warn!("STUBBED: Write to SHA BLK_CNT register: 0x{:X}", dev.blk_cnt.get());
            };
        }
    }
    ranges: {
        0x040;0x20 => {
            read_effect = reg_hash_read;
            write_effect = |_, _, _| unimplemented!();
        }
        0x080;0x40 => {
            read_effect = |_, _, buf: &mut [u8]| {
                // Boot9 XDMAs with this register as a source, into DSP memory.
                // Might reading from this register be a way to clear the FIFO?
                for b in buf {
                    *b = 0;
                }
                warn!("STUBBED: read from SHA FIFO register");
            };
            write_effect = reg_fifo_write;
        }
    }
});



impl DmaBus for RefCell<ShaDevice> {
    fn read_ready(&self) -> bool {
        let dev = self.borrow();
        dev._internal_state.hasher.is_none()
    }

    fn write_ready(&self) -> bool {
        let dev = self.borrow();
        dev._internal_state.hasher.is_some()
    }

    fn read_addr(&self, addr: u32, buf: &mut [u8]) {
        use io::regs::IoRegAccess;
        assert!(addr >> 12 == 0x1000A);
        let src = addr & ((1 << 12) - 1);
        self.borrow_mut().read_reg(src as usize, buf);
    }
}
