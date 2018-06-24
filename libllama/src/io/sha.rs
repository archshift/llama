use std::fmt;

use openssl::hash::{Hasher, MessageDigest};

bfdesc!(RegCnt: u32, {
    busy: 0 => 0,
    final_round: 1 => 1,
    _enable_irq0: 2 => 2,
    big_endian: 3 => 3,
    hash_mode: 4 => 5,
    clear_fifo: 8 => 8,
    _enable_fifo: 9 => 9,
    _enable_irq1: 10 => 10
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
    let mut cnt = dev.cnt.get();
    trace!("Wrote 0x{:08X} to SHA CNT register!", cnt);

    if bf!(cnt @ RegCnt::final_round) == 1 && bf!(cnt @ RegCnt::busy) == 0 {
        info!("Reached end of final round!");
        if let Some(ref mut h) = dev._internal_state.hasher {
            let hash_slice = &*h.finish2().unwrap();
            dev._internal_state.hash = [0u8; 32];
            dev._internal_state.hash[0..hash_slice.len()].copy_from_slice(hash_slice);
        }
        bf!(cnt @ RegCnt::final_round = 0);
    }

    else if bf!(cnt @ RegCnt::clear_fifo) == 1 {
        dev._internal_state.hasher = None;
    }

    else if bf!(cnt @ RegCnt::busy) == 0 {
    }

    else if dev._internal_state.hasher.is_none() {
        // Create new hasher
        assert_eq!(bf!(cnt @ RegCnt::big_endian), 1);

        let mode = match bf!(cnt @ RegCnt::hash_mode) {
            0b00 => MessageDigest::sha256(),
            0b01 => MessageDigest::sha224(),
            _ => MessageDigest::sha1()
        };

        dev._internal_state.hasher = Some(Hasher::new(mode).unwrap());
    }

    bf!(cnt @ RegCnt::busy = 0);
    dev.cnt.set_unchecked(cnt);
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

    let _cnt = dev.cnt.get();
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