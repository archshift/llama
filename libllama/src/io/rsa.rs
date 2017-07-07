use io::regs;

use std::fmt;
use std::mem;

use openssl::bn;

bfdesc!(RegCnt: u32, {
    busy: 0 => 0,
    keyslot: 4 => 5,
    little_endian: 8 => 8,
    normal_order: 9 => 9
});

bfdesc!(RegSlotCnt: u32, {
    key_set: 0 => 0,
    key_prot: 1 => 1
});

pub struct RsaKeyslot {
    write_pos: usize,
    buf: [u8; 0x100]
}

impl Clone for RsaKeyslot {
    fn clone(&self) -> RsaKeyslot {
        let mut new_buf = [0u8; 0x100];
        new_buf.copy_from_slice(&self.buf[..]);

        RsaKeyslot {
            write_pos: self.write_pos,
            buf: new_buf
        }
    }
}

pub struct RsaDeviceState {
    slots: [RsaKeyslot; 4],
    modulus: [u8; 0x100],
    message: [u8; 0x100],
}

impl Default for RsaDeviceState {
    fn default() -> RsaDeviceState {
        let new_keyslot = RsaKeyslot {
            write_pos: 0,
            buf: [0; 0x100],
        };
        RsaDeviceState {
            slots: [new_keyslot.clone(), new_keyslot.clone(),
                    new_keyslot.clone(), new_keyslot],
            modulus: [0; 0x100],
            message: [0; 0x100],
        }
    }
}

impl fmt::Debug for RsaDeviceState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "RsaDeviceState {{ }}")
    }
}

fn get_keydata(dev: &RsaDevice, keyslot: usize) -> (u32, u32) {
    match keyslot {
        0 => (dev.slot0_cnt.get(), dev.slot0_len.get()),
        1 => (dev.slot1_cnt.get(), dev.slot1_len.get()),
        2 => (dev.slot2_cnt.get(), dev.slot2_len.get()),
        3 => (dev.slot3_cnt.get(), dev.slot3_len.get()),
        _ => unreachable!()
    }
}

fn reg_slot_cnt_update(dev: &mut RsaDevice, keyslot: usize) {
    let (slot_cnt, _) = get_keydata(dev, keyslot);
    if bf!(slot_cnt @ RegSlotCnt::key_set) == 1 {
        assert_eq!(dev._internal_state.slots[keyslot].write_pos, 0x100);
    } else {
        dev._internal_state.slots[keyslot].write_pos = 0;
    }
}

fn reg_cnt_update(dev: &mut RsaDevice) {
    let cnt = dev.cnt.get();
    trace!("Wrote 0x{:08X} to RSA CNT register!", cnt);

    if bf!(cnt @ RegCnt::busy) == 1 {
        let keyslot = bf!(cnt @ RegCnt::keyslot) as usize;
        let (slot_cnt, _) = get_keydata(dev, keyslot);
        assert_eq!(bf!(slot_cnt @ RegSlotCnt::key_set), 1);

        info!("Performing RSA arithmetic!");

        let mut base_buf = [0u8; 0x100];
        base_buf.copy_from_slice(&dev._internal_state.message[..]);
        let mut modulus_buf = [0u8; 0x100];
        modulus_buf.copy_from_slice(&dev._internal_state.modulus[..]);

        match (bf!(cnt @ RegCnt::little_endian), bf!(cnt @ RegCnt::normal_order)) {
            (1, 1) => {},
            (0, 0) => {
                base_buf.reverse();
                modulus_buf.reverse();
            }
            _ => unimplemented!()
        }

        let base = bn::BigNum::from_slice(&base_buf[..]).unwrap();
        let exponent = bn::BigNum::from_slice(&dev._internal_state.slots[keyslot].buf[..]).unwrap();
        let modulus = bn::BigNum::from_slice(&modulus_buf[..]).unwrap();

        let mut res = bn::BigNum::new().unwrap();
        res.mod_exp(&base, &exponent, &modulus, &mut bn::BigNumContext::new().unwrap()).unwrap();

        for b in dev._internal_state.message.iter_mut() {
            *b = 0;
        }
        let res_vec = res.to_vec();
        // Copy result to the back of the buffer
        dev._internal_state.message[0x100 - res_vec.len() .. 0x100].copy_from_slice(res_vec.as_slice());

        dev.cnt.set_unchecked(bf!(cnt @ RegCnt::busy as 0));
    }
}

fn reg_mod_read(dev: &mut RsaDevice, buf_pos: usize, dest: &mut [u8]) {
    trace!("Reading {} bytes from RSA MOD at +0x{:X}", dest.len(), buf_pos);
    let src_slice = &dev._internal_state.modulus[buf_pos .. buf_pos + dest.len()];
    dest.clone_from_slice(src_slice);
}

fn reg_mod_write(dev: &mut RsaDevice, buf_pos: usize, src: &[u8]) {
    trace!("Writing {} bytes to RSA MOD at +0x{:X}", src.len(), buf_pos);
    let dst_slice = &mut dev._internal_state.modulus[buf_pos .. buf_pos + src.len()];
    dst_slice.clone_from_slice(src);
}

fn reg_txt_read(dev: &mut RsaDevice, buf_pos: usize, dest: &mut [u8]) {
    trace!("Reading {} bytes from RSA TXT at +0x{:X}", dest.len(), buf_pos);
    let src_slice = &dev._internal_state.message[buf_pos .. buf_pos + dest.len()];
    dest.clone_from_slice(src_slice);
}

fn reg_txt_write(dev: &mut RsaDevice, buf_pos: usize, src: &[u8]) {
    trace!("Writing {} bytes to RSA TXT at +0x{:X}", src.len(), buf_pos);
    let dst_slice = &mut dev._internal_state.message[buf_pos .. buf_pos + src.len()];
    dst_slice.clone_from_slice(src);
}

fn reg_exp_fifo_write(dev: &mut RsaDevice) {
    let keyslot = bf!((dev.cnt.get()) @ RegCnt::keyslot) as usize;
    let (slot_cnt, _) = get_keydata(dev, keyslot);

    assert_eq!(bf!(slot_cnt @ RegSlotCnt::key_set), 0);
    assert_eq!(bf!(slot_cnt @ RegSlotCnt::key_prot), 0);

    let write_pos = dev._internal_state.slots[keyslot].write_pos;
    if write_pos == 0 { // Just starting to update key, clear previous key
        for b in dev._internal_state.slots[keyslot].buf.iter_mut() {
            *b = 0;
        }
    }
    let word_bytes: [u8; 4] = unsafe { mem::transmute(dev.exp_fifo.get()) };
    dev._internal_state.slots[keyslot].buf[write_pos .. write_pos + 4].copy_from_slice(&word_bytes[..]);

    trace!("Writing bytes {:02X},{:02X},{:02X},{:02X} to RSA exponent FIFO!",
        word_bytes[0], word_bytes[1], word_bytes[2], word_bytes[3]);

    dev._internal_state.slots[keyslot].write_pos += 4;
}

iodevice!(RsaDevice, {
    internal_state: RsaDeviceState;
    regs: {
        0x000 => cnt: u32 { write_effect = reg_cnt_update; }
        0x0F0 => unk: u32 { }
        0x100 => slot0_cnt: u32 { write_effect = |dev: &mut RsaDevice| reg_slot_cnt_update(dev, 0); }
        0x104 => slot0_len: u32 { }
        0x110 => slot1_cnt: u32 { write_effect = |dev: &mut RsaDevice| reg_slot_cnt_update(dev, 1); }
        0x114 => slot1_len: u32 { }
        0x120 => slot2_cnt: u32 { write_effect = |dev: &mut RsaDevice| reg_slot_cnt_update(dev, 2); }
        0x124 => slot2_len: u32 { }
        0x130 => slot3_cnt: u32 { write_effect = |dev: &mut RsaDevice| reg_slot_cnt_update(dev, 3); }
        0x134 => slot3_len: u32 { }
        0x200 => exp_fifo: u32 { write_effect = reg_exp_fifo_write; }
    }
    ranges: {
        0x204;0xFC => { } // A bug in bootrom causes it to write all over this area
        0x400;0x100 => {
            read_effect = reg_mod_read;
            write_effect = reg_mod_write;
        }
        0x800;0x100 => {
            read_effect = reg_txt_read;
            write_effect = reg_txt_write;
        }
    }
});