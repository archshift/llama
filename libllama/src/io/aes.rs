use std::cmp;
use std::collections::VecDeque;
use std::fmt;
use std::mem;

use openssl::symm;

bfdesc!(RegCnt: u32, {
    fifo_in_count: 0 => 4,
    fifo_out_count: 5 => 9,
    flush_fifo_in: 10 => 10,
    flush_fifo_out: 11 => 11,
    fifo_in_dma_size: 12 => 13,
    fifo_out_dma_size: 14 => 15,
    mac_size: 16 => 18,
    mac_source_reg: 20 => 20,
    mac_verified: 21 => 21,
    out_big_endian: 22 => 22,
    in_big_endian: 23 => 23,
    out_normal_order: 24 => 24,
    in_normal_order: 25 => 25,
    update_keyslot: 26 => 26,
    mode: 27 => 29,
    enable_irq: 30 => 30,
    busy: 31 => 31
});

bfdesc!(RegKeyCnt: u8, {
    keyslot: 0 => 5,
    use_dsi_keygen: 6 => 6,
    enable_fifo_flush: 7 => 7
});

#[derive(Clone, Copy, Default)]
pub struct Key {
    data: [u8; 0x10]
}

#[derive(Default)]
pub struct KeyFifoState {
    pos: usize,
    buf: [u32; 4]
}

pub struct AesDeviceState {
    active_keyslot: usize,
    active_process: Option<symm::Crypter>,
    bytes_left: usize,

    key_slots: [Key; 0x40],
    keyx_slots: [Key; 0x40],
    keyy_slots: [Key; 0x40],
    keyfifo_state: KeyFifoState,
    keyxfifo_state: KeyFifoState,
    keyyfifo_state: KeyFifoState,

    fifo_in_buf: VecDeque<u32>,
    fifo_out_buf: VecDeque<u32>,
    reg_ctr: [u8; 0x10],
}

unsafe impl Send for AesDeviceState {} // TODO: Not good!

impl Default for AesDeviceState {
    fn default() -> AesDeviceState {
        AesDeviceState {
            active_keyslot: 0,
            active_process: None,
            bytes_left: 0,
            key_slots: [Default::default(); 0x40],
            keyx_slots: [Default::default(); 0x40],
            keyy_slots: [Default::default(); 0x40],
            keyfifo_state: Default::default(),
            keyxfifo_state: Default::default(),
            keyyfifo_state: Default::default(),
            fifo_in_buf: VecDeque::new(),
            fifo_out_buf: VecDeque::new(),
            reg_ctr: [0; 0x10],
        }
    }
}

impl fmt::Debug for AesDeviceState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "AesDeviceState {{ }}")
    }
}

fn reg_cnt_onread(dev: &mut AesDevice) {
    let mut cnt = dev.cnt.get();
    let in_count = cmp::min(16, dev._internal_state.fifo_in_buf.len());
    let out_count = cmp::min(16, dev._internal_state.fifo_out_buf.len());
    bf!(cnt @ RegCnt::fifo_in_count = in_count as u32);
    bf!(cnt @ RegCnt::fifo_out_count = out_count as u32);
    dev.cnt.set_unchecked(cnt);

    info!("Reading from AES CNT register (in_count: {}, out_count: {})", in_count, out_count);
}

fn reg_cnt_update(dev: &mut AesDevice) {
    let cnt = dev.cnt.get();
    warn!("STUBBED: Wrote 0x{:08X} to AES CNT register!", cnt);

    if bf!(cnt @ RegCnt::update_keyslot) == 1 {
        dev._internal_state.active_keyslot = dev.key_sel.get() as usize;
        info!("Setting AES active keyslot to 0x{:X}", dev._internal_state.active_keyslot);
        // Remove update_keyslot bit
        dev.cnt.set_unchecked(bf!(cnt @ RegCnt::update_keyslot as 0));
    }

    if bf!(cnt @ RegCnt::busy) == 1 {
        let mode = bf!(cnt @ RegCnt::mode);
        let keyslot = dev._internal_state.active_keyslot;
        let key = dev._internal_state.key_slots[keyslot];
        let bytes = dev.blk_cnt.get() << 4;

        // Reverse word order for CTR
        let mut ctr: [u32; 4] = unsafe { mem::transmute(dev._internal_state.reg_ctr) };
        ctr.reverse();
        let ctr: [u8; 0x10] = unsafe { mem::transmute(ctr) };

        assert!(bf!(cnt @ RegCnt::out_big_endian) == 1);
        assert!(bf!(cnt @ RegCnt::in_big_endian) == 1);
        assert!(bf!(cnt @ RegCnt::out_normal_order) == 1);
        assert!(bf!(cnt @ RegCnt::in_normal_order) == 1);

        error!("Attempted to start AES crypto! mode: {}, keyslot: 0x{:X}, bytes: 0x{:X}",
            mode, keyslot, bytes);

        match mode {
            4 | 5 => {
                let symm_mode = if mode & 1 == 1 {
                    symm::Mode::Encrypt
                } else {
                    symm::Mode::Decrypt
                };
                let mut crypter = symm::Crypter::new(symm::Cipher::aes_128_cbc(), symm_mode,
                                                     &key.data[..], Some(&ctr[..])).unwrap();
                crypter.pad(false);
                dev._internal_state.active_process = Some(crypter);
            }
            _ => unimplemented!()
        }

        dev._internal_state.bytes_left = bytes as usize;
    }
}

fn reg_key_cnt_update(dev: &mut AesDevice) {
    let key_cnt = dev.key_cnt.get();
    let flush_fifo = bf!(key_cnt @ RegKeyCnt::enable_fifo_flush) == 1;

    info!("Wrote to AES KEYCNT register; keyslot: 0x{:X}, Mode: {}, FIFO flush: {}",
        bf!(key_cnt @ RegKeyCnt::keyslot),
        if bf!(key_cnt @ RegKeyCnt::use_dsi_keygen) == 1 { "DSi" } else { "3DS" },
        flush_fifo
    );

    if flush_fifo {
        warn!("STUBBED: Flushing AES key FIFOs");
        // TODO: verify?
        dev._internal_state.keyfifo_state.pos = 0;
        dev._internal_state.keyxfifo_state.pos = 0;
        dev._internal_state.keyyfifo_state.pos = 0;
    }
}

fn reg_fifo_in_update(dev: &mut AesDevice) {
    {
        let active_process = dev._internal_state.active_process.as_mut()
            .expect("Attempted to write to AES FIFO-IN when not started!");

        let word = dev.fifo_in.get();
        dev._internal_state.fifo_in_buf.push_back(word);

        if dev._internal_state.fifo_in_buf.len() == 4 {
            let words = [
                dev._internal_state.fifo_in_buf.pop_front().unwrap(),
                dev._internal_state.fifo_in_buf.pop_front().unwrap(),
                dev._internal_state.fifo_in_buf.pop_front().unwrap(),
                dev._internal_state.fifo_in_buf.pop_front().unwrap()
            ];
            let bytes: [u8; 0x10] = unsafe { mem::transmute(words) };

            let mut dec_bytes = [0u8; 0x20]; // Double size because of library silliness
            active_process.update(&bytes[..], &mut dec_bytes[..]);

            let dec_words: [u32; 8] = unsafe { mem::transmute(dec_bytes) };
            dev._internal_state.fifo_out_buf.push_back(dec_words[0]);
            dev._internal_state.fifo_out_buf.push_back(dec_words[1]);
            dev._internal_state.fifo_out_buf.push_back(dec_words[2]);
            dev._internal_state.fifo_out_buf.push_back(dec_words[3]);
        }
    }

    dev._internal_state.bytes_left -= 4;
    if dev._internal_state.bytes_left == 0 {
        dev._internal_state.active_process = None;
        let cnt = dev.cnt.get();
        dev.cnt.set_unchecked(bf!(cnt @ RegCnt::busy as 0));
    }
}

fn reg_fifo_out_onread(dev: &mut AesDevice) {
    if let Some(word) = dev._internal_state.fifo_out_buf.pop_front() {
        dev.fifo_out.set_unchecked(word);
    }
}

fn reg_key_fifo_update(dev: &mut AesDevice) {
    let word = dev.key_fifo.get();
    info!("Wrote 0x{:08X} to AES KEYFIFO register!", word);

    let cnt = dev.cnt.get();
    let state = &mut dev._internal_state.keyfifo_state;

    state.buf[state.pos / 4] = word;
    state.pos += 4;
    if state.pos >= 0x10 {
        // Done updating the key
        let keyslot = bf!((dev.key_cnt.get()) @ RegKeyCnt::keyslot) as usize;
        dev._internal_state.key_slots[keyslot] = Key {
            data: unsafe { mem::transmute(state.buf) }
        }
    }
}

fn reg_ctr_write(dev: &mut AesDevice, buf_pos: usize, src: &[u8]) {
    trace!("Writing {} bytes to AES CTR at +0x{:X}", src.len(), buf_pos);
    let dst_slice = &mut dev._internal_state.reg_ctr[buf_pos .. buf_pos + src.len()];
    dst_slice.clone_from_slice(src);
}

iodevice!(AesDevice, {
    internal_state: AesDeviceState;
    regs: {
        0x000 => cnt: u32 {
            write_bits = 0b11111111_11011111_11111100_00000000;
            read_effect = reg_cnt_onread;
            write_effect = reg_cnt_update;
        }
        0x004 => mac_blk_cnt: u16 { }
        0x006 => blk_cnt: u16 { }
        0x008 => fifo_in: u32 { write_effect = reg_fifo_in_update; }
        0x00C => fifo_out: u32 { read_effect = reg_fifo_out_onread; }
        0x010 => key_sel: u8 { }
        0x011 => key_cnt: u8 { write_effect = reg_key_cnt_update; }
        0x100 => key_fifo: u32 {
            read_effect = |_| unimplemented!();
            write_effect = reg_key_fifo_update;
        }
        0x104 => keyx_fifo: u32 { write_effect = |_| unimplemented!(); }
        0x108 => keyy_fifo: u32 { write_effect = |_| unimplemented!(); }
    }
    ranges: {
        0x020;0x10 => {  // CTR
            read_effect = |_, _, _| unimplemented!();
            write_effect = reg_ctr_write;
        }
        0x030;0x10 => {  // MAC
            read_effect = |_, _, _| unimplemented!();
            write_effect = |_, _, _| unimplemented!();
        }
        0x040;0x30 => {  // KEY0
            read_effect = |_, _, _| unimplemented!();
            write_effect = |_, _, _| unimplemented!();
        }
        0x070;0x30 => {  // KEY1
            read_effect = |_, _, _| unimplemented!();
            write_effect = |_, _, _| unimplemented!();
        }
        0x0A0;0x30 => {  // KEY2
            read_effect = |_, _, _| unimplemented!();
            write_effect = |_, _, _| unimplemented!();
        }
        0x0D0;0x30 => {  // KEY3
            read_effect = |_, _, _| unimplemented!();
            write_effect = |_, _, _| unimplemented!();
        }
    }
});