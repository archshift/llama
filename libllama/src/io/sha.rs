use std::fmt;

use openssl::hash::{Hasher, MessageDigest};

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
    hash: [u8; 32]
}

impl fmt::Debug for ShaDeviceState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ShaDeviceState {{ }}")
    }
}
unsafe impl Send for ShaDeviceState {} // TODO: Not good!

// TODO: The following implementation does not yet completely work, and still needs
// more hardware testing to determine the source of errors.

fn reg_cnt_update(dev: &mut ShaDevice) {
    let cnt = RegCnt::alias_mut(dev.cnt.ref_mut());
    trace!("Wrote 0x{:08X} to SHA CNT register!", cnt.val);

    if cnt.final_round.get() == 1 && cnt.busy.get() == 0 {
        info!("Reached end of final round!");
        if let Some(ref mut h) = dev._internal_state.hasher {
            let hash_slice = &*h.finish().unwrap();
            dev._internal_state.hash = [0u8; 32];
            dev._internal_state.hash[0..hash_slice.len()].copy_from_slice(hash_slice);
        }
        cnt.final_round.set(0);
    }

    else if cnt.clear_fifo.get() == 1 {
        dev._internal_state.hasher = None;
    }

    else if cnt.busy.get() == 0 {
    }

    else if dev._internal_state.hasher.is_none() {
        // Create new hasher
        assert_eq!(cnt.big_endian.get(), 1);

        let mode = match cnt.hash_mode.get() {
            0b00 => MessageDigest::sha256(),
            0b01 => MessageDigest::sha224(),
            _ => MessageDigest::sha1()
        };

        dev._internal_state.hasher = Some(Hasher::new(mode).unwrap());
    }

    cnt.busy.set(0);
}

fn reg_hash_read(dev: &mut ShaDevice, buf_pos: usize, dest: &mut [u8]) {
    println!("Reading {} bytes from SHA HASH at +0x{:X}", dest.len(), buf_pos);
    let src_slice = &dev._internal_state.hash[buf_pos .. buf_pos + dest.len()];
    dest.clone_from_slice(src_slice);
}

// TODO: Does a word written to any part of the hash just add on? What about
// writing to the fifo in reverse? How does this work?
fn reg_fifo_write(dev: &mut ShaDevice, buf_pos: usize, source: &[u8]) {
    println!("Writing {} bytes to SHA FIFO at +0x{:X}", source.len(), buf_pos);

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
        0x004 => blk_cnt: u32 { }
    }
    ranges: {
        0x040;0x20 => {
            read_effect = reg_hash_read;
            write_effect = |_, _, _| unimplemented!();
        }
        0x080;0x40 => {
            read_effect = |_, _, _| unimplemented!();
            write_effect = reg_fifo_write;
        }
    }
});
