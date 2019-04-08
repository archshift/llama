use std::fmt;

use openssl::bn;

bf!(RegCnt[u32] {
    busy: 0:0,
    keyslot: 4:5,
    little_endian: 8:8,
    normal_order: 9:9
});

bf!(RegSlotCnt[u32] {
    key_set: 0:0,
    key_prot: 1:1
});

pub struct RsaKeyslot {
    write_pos: usize,
    buf: [u8; 0x100],
    modulus: [u8; 0x100],
    ready: bool,
}

impl Clone for RsaKeyslot {
    fn clone(&self) -> RsaKeyslot {
        let mut new_buf = [0u8; 0x100];
        new_buf.copy_from_slice(&self.buf);
        let mut new_mod = [0u8; 0x100];
        new_mod.copy_from_slice(&self.modulus);

        RsaKeyslot {
            write_pos: self.write_pos,
            buf: new_buf,
            modulus: new_mod,
            ready: self.ready
        }
    }
}

pub struct RsaDeviceState {
    slots: [RsaKeyslot; 4],
    message: [u8; 0x100],
}

impl Default for RsaDeviceState {
    fn default() -> RsaDeviceState {
        let new_keyslot = RsaKeyslot {
            write_pos: 0,
            buf: [0u8; 0x100],
            modulus: [0u8; 0x100],
            ready: false,
        };
        RsaDeviceState {
            slots: [new_keyslot.clone(), new_keyslot.clone(),
                    new_keyslot.clone(), new_keyslot],
            message: [0; 0x100],
        }
    }
}

impl fmt::Debug for RsaDeviceState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "RsaDeviceState {{ }}")
    }
}

fn get_keydata(dev: &RsaDevice, keyslot: usize) -> (RegSlotCnt::Bf, u32) {
    match keyslot {
        0 => (RegSlotCnt::new(dev.slot0_cnt.get()), dev.slot0_len.get()),
        1 => (RegSlotCnt::new(dev.slot1_cnt.get()), dev.slot1_len.get()),
        2 => (RegSlotCnt::new(dev.slot2_cnt.get()), dev.slot2_len.get()),
        3 => (RegSlotCnt::new(dev.slot3_cnt.get()), dev.slot3_len.get()),
        _ => unreachable!()
    }
}

fn reg_slot_cnt_read(dev: &mut RsaDevice, keyslot: usize) {
    let cnt = match keyslot {
        0 => dev.slot0_cnt.ref_mut(),
        1 => dev.slot1_cnt.ref_mut(),
        2 => dev.slot2_cnt.ref_mut(),
        3 => dev.slot3_cnt.ref_mut(),
        _ => unreachable!()
    };
    let ready = dev._internal_state.slots[keyslot].ready;
    trace!("Reading from RSA keyslot {} CNT; ready == {:?}", keyslot, ready);
    RegSlotCnt::alias_mut(cnt).key_set.set(ready as u32);
}

fn reg_slot_cnt_update(dev: &mut RsaDevice, keyslot: usize) {
    let (slot_cnt, _) = get_keydata(dev, keyslot);
    let prev_ready = dev._internal_state.slots[keyslot].ready;
    if prev_ready && slot_cnt.key_set.get() == 0 {
        dev._internal_state.slots[keyslot].ready = false;
        dev._internal_state.slots[keyslot].write_pos = 0;
        trace!("Reset RSA keyslot {}!", keyslot);
    }
}

fn word_swap(buf: &mut [u8;256]) -> &mut [u8;256] {
    {
        let mut chunks = buf.chunks_exact_mut(4);
        loop {
            match (chunks.next(), chunks.next_back()) {
                (Some(front), Some(back)) => {
                    let mut tmp = [0u8;4];
                    tmp.copy_from_slice(back);
                    back.copy_from_slice(front);
                    front.copy_from_slice(&tmp);
                }
                _ => break
            }
        }
    }
    buf
}

fn byte_swap_inner(buf: &mut [u8;256]) -> &mut [u8;256] {
    for chunk in buf.chunks_exact_mut(4) {
        chunk.reverse();
    }
    buf
}

fn reg_cnt_update(dev: &mut RsaDevice) {
    let cnt = RegCnt::new(dev.cnt.get());
    trace!("Wrote 0x{:08X} to RSA CNT register!", cnt.val);

    if cnt.busy.get() == 1 {
        let keyslot = cnt.keyslot.get() as usize;
        let keyslot_data = &dev._internal_state.slots[keyslot];

        assert!(keyslot_data.ready);

        trace!("Performing RSA arithmetic!");

        let mut base_buf = [0u8; 0x100];
        base_buf.copy_from_slice(&dev._internal_state.message[..]);
        let mut exponent_buf = [0u8; 0x100];
        exponent_buf.copy_from_slice(&keyslot_data.buf[..]);
        let mut modulus_buf = [0u8; 0x100];
        modulus_buf.copy_from_slice(&keyslot_data.modulus[..]);

        if cnt.little_endian.get() == 0 {
            byte_swap_inner(&mut modulus_buf);
            byte_swap_inner(&mut base_buf);
            byte_swap_inner(&mut exponent_buf);
        }
        if cnt.normal_order.get() == 0 {
            word_swap(&mut modulus_buf);
            word_swap(&mut base_buf);
        }

        let mut base = bn::BigNum::from_slice(&base_buf[..]).unwrap();
        let exponent = bn::BigNum::from_slice(&exponent_buf[..]).unwrap();
        let modulus = bn::BigNum::from_slice(&modulus_buf[..]).unwrap();

        // The AES hardware doesn't like even moduli, and will output 0 when it finds them
        if !modulus.is_bit_set(0) {
            base.clear();
        }

        let mut res = bn::BigNum::new().unwrap();
        res.mod_exp(&base, &exponent, &modulus, &mut bn::BigNumContext::new().unwrap()).unwrap();

        for b in dev._internal_state.message.iter_mut() {
            *b = 0;
        }
        let res_vec = res.to_vec();

        // Copy result to the back of the buffer
        dev._internal_state.message[0x100 - res_vec.len() .. 0x100].copy_from_slice(res_vec.as_slice());


        if cnt.little_endian.get() == 0 {
            byte_swap_inner(&mut dev._internal_state.message);
        }
        if cnt.normal_order.get() == 0 {
            word_swap(&mut dev._internal_state.message);
        }

        let cnt_ref = RegCnt::alias_mut(dev.cnt.ref_mut());
        cnt_ref.busy.set(0);
    }
}

fn reg_mod_read(dev: &mut RsaDevice, buf_pos: usize, dest: &mut [u8]) {
    let cnt = RegCnt::new(dev.cnt.get());
    let keyslot = cnt.keyslot.get() as usize;
    trace!("Reading {} bytes from RSA keysot {} MOD at +0x{:X}", dest.len(), keyslot, buf_pos);

    let mod_buf = &dev._internal_state.slots[keyslot].modulus;
    let src_slice = &mod_buf[buf_pos .. buf_pos + dest.len()];
    dest.clone_from_slice(src_slice);
}

fn reg_mod_write(dev: &mut RsaDevice, buf_pos: usize, src: &[u8]) {
    let cnt = RegCnt::new(dev.cnt.get());
    let keyslot = cnt.keyslot.get() as usize;
    trace!("Writing {} bytes to RSA keyslot {} MOD at +0x{:X}", src.len(), keyslot, buf_pos);

    let mod_buf = &mut dev._internal_state.slots[keyslot].modulus;
    let dst_slice = &mut mod_buf[buf_pos .. buf_pos + src.len()];
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

fn reg_exp_write(dev: &mut RsaDevice, _: usize, src: &[u8]) {
    let cnt = RegCnt::new(dev.cnt.get());
    let keyslot = cnt.keyslot.get() as usize;
    let (slot_cnt, _) = get_keydata(dev, keyslot);

    assert_eq!(slot_cnt.key_prot.get(), 0);

    let keyslot_state = &mut dev._internal_state.slots[keyslot];
    if keyslot_state.write_pos == 0 { // Just starting to update key, clear previous key
        keyslot_state.buf = [0u8; 0x100];
    }

    let write_pos = keyslot_state.write_pos;
    assert!(write_pos < 0x100);
    assert!(src.len() == 4);

    keyslot_state.buf[write_pos .. write_pos + 4].copy_from_slice(&src[..]);

    trace!("Writing bytes {:02X},{:02X},{:02X},{:02X} to RSA exponent FIFO!",
        src[0], src[1], src[2], src[3]);

    keyslot_state.write_pos += 4;

    if (keyslot_state.write_pos == 0x100) {
        trace!("Committing RSA keyslot {} exponent", keyslot);
        keyslot_state.ready = true;
    }
}

iodevice!(RsaDevice, {
    internal_state: RsaDeviceState;
    regs: {
        0x000 => cnt: u32 { write_effect = reg_cnt_update; }
        0x0F0 => unk: u32 { }
        0x100 => slot0_cnt: u32 {
            read_effect  = |dev: &mut RsaDevice| reg_slot_cnt_read(dev, 0);
            write_effect = |dev: &mut RsaDevice| reg_slot_cnt_update(dev, 0);
        }
        0x104 => slot0_len: u32 {
            default = 0x40;
        }
        0x110 => slot1_cnt: u32 {
            read_effect  = |dev: &mut RsaDevice| reg_slot_cnt_read(dev, 1);
            write_effect = |dev: &mut RsaDevice| reg_slot_cnt_update(dev, 1);
        }
        0x114 => slot1_len: u32 {
            default = 0x40;
        }
        0x120 => slot2_cnt: u32 {
            read_effect  = |dev: &mut RsaDevice| reg_slot_cnt_read(dev, 2);
            write_effect = |dev: &mut RsaDevice| reg_slot_cnt_update(dev, 2);
        }
        0x124 => slot2_len: u32 {
            default = 0x40;
        }
        0x130 => slot3_cnt: u32 {
            read_effect  = |dev: &mut RsaDevice| reg_slot_cnt_read(dev, 3);
            write_effect = |dev: &mut RsaDevice| reg_slot_cnt_update(dev, 3);
        }
        0x134 => slot3_len: u32 {
            default = 0x40;
        }
    }
    ranges: {
        0x200;0x100 => {
            read_effect = |_, _, _| panic!("Attempted read from RSA EXP_FIFO!");
            write_effect = reg_exp_write;
        }
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
