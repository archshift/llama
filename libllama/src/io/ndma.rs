use std::cell::RefCell;
use std::rc::Rc;
use std::fmt;

use hwcore::HardwareDma9;
use io::DmaBuses;

bf!(RegGlobalCnt[u32] {
    enabled: 0:0,
    cycle_select: 16:19,
    round_robin: 31:31
});

bf!(RegChannelCnt[u32] {
    dst_addr_writeback_mode: 10:11,
    _dst_addr_reload: 12:12,
    src_addr_writeback_mode: 13:14,
    _src_addr_reload: 15:15,
    xfer_size: 16:19,
    _startup_mode: 24:27,
    immed_mode: 28:28,
    repeat_mode: 29:29,
    _enable_irq: 30:30,
    enabled: 31:31
});

fn should_xfer(dev: &mut NdmaChannel) -> bool {
    let chan_cnt = RegChannelCnt::new(dev.chan_cnt.get());
    if chan_cnt.enabled.get() == 0 {
        return false;
    }

    let mode = (chan_cnt.immed_mode.get(), chan_cnt.repeat_mode.get());
    match mode {
        (0, 0) => {
            unimplemented!()
        }
        (1, 0) => {
            // Immediate mode
            true
        }
        (0, 1) => {
            unimplemented!()
        }
        _ => panic!("Attempted to use impossible channel mode immed+repeat")
    }
}

fn process_channel(dev: &mut NdmaChannel) {
    if !should_xfer(dev) {
        let chan_cnt = RegChannelCnt::alias_mut(dev.chan_cnt.ref_mut());
        chan_cnt.enabled.set(0);
        return;
    }

    let xns = &mut dev._internal_state;
    let mut hw = xns.hw.borrow_mut();

    let chan_cnt = RegChannelCnt::new(dev.chan_cnt.get());

    let line_size = 1u32 << chan_cnt.xfer_size.get();
    let mut src_addr = dev.src_addr.get();
    let mut dst_addr = dev.dst_addr.get();
    let total_words = dev.write_cnt.get() & 0xFFFFFF;

    let mut tmp_buf = vec![0u8; 4 * line_size as usize];
    let word_fill = dev.fill_data.get().to_le_bytes();

    assert!(src_addr % 4 == 0);
    assert!(dst_addr % 4 == 0);
    assert!(total_words % line_size == 0);

    let src_wb_mode = chan_cnt.src_addr_writeback_mode.get();
    let dst_wb_mode = chan_cnt.dst_addr_writeback_mode.get();

    for _burst in 0..(total_words / line_size) {

        info!("Processing NDMA 0x{:X}-word burst to address {:08X}!", line_size, dst_addr);

        if src_wb_mode == 3 {
            info!("NDMA using fill source {:08X}", dev.fill_data.get());

            // Constant data fill
            for word in tmp_buf.chunks_exact_mut(4) {
                word.copy_from_slice(&word_fill);
            }
        } else {
            info!("NDMA using address source {:08X}", dev.fill_data.get());

            hw.mem.read_buf(src_addr, &mut tmp_buf[..]);

            src_addr = match src_wb_mode {
                0 => src_addr + 4 * line_size,
                1 => src_addr - 4 * line_size,
                2 => src_addr,
                _ => unreachable!()
            };
        }

        hw.mem.write_buf(dst_addr, &mut tmp_buf[..]);

        dst_addr = match dst_wb_mode {
            0 => dst_addr + 4 * line_size,
            1 => dst_addr - 4 * line_size,
            2 => dst_addr,
            _ => unreachable!()
        };
    }

    let chan_cnt = RegChannelCnt::alias_mut(dev.chan_cnt.ref_mut());
    let mode = (chan_cnt.immed_mode.get(), chan_cnt.repeat_mode.get());
    match mode {
        (0, 0) => {
            unimplemented!()
        }
        (1, 0) => {
            // Immediate mode
            chan_cnt.enabled.set(0);
        }
        (0, 1) => {
            unimplemented!()
        }
        _ => panic!("Attempted to use impossible channel mode immed+repeat")
    }
}


fn reg_chan_cnt_write(dev: &mut NdmaChannel) {
    process_channel(dev);
}

iodevice!(NdmaChannel, {
    internal_state: NdmaConnections;
    regs: {
        0x004 => src_addr: u32 { }
        0x008 => dst_addr: u32 { }
        0x00C => xfer_pos: u32 { }
        0x010 => write_cnt: u32 { }
        0x014 => block_cnt: u32 { }
        0x018 => fill_data: u32 { }
        0x01C => chan_cnt: u32 {
            write_effect = reg_chan_cnt_write;
        }
    }
});

#[derive(Clone)]
pub struct NdmaConnections {
    hw: Rc<RefCell<HardwareDma9>>,
    buses: DmaBuses,
}

impl fmt::Debug for NdmaConnections {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "NdmaConnections {{ }}")
    }
}

pub struct NdmaDeviceState {
    _connections: NdmaConnections,
    channels: [NdmaChannel; 8],
}

impl fmt::Debug for NdmaDeviceState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "NdmaConnections {{ }}")
    }
}

impl NdmaDeviceState {
    pub fn new(hw: Rc<RefCell<HardwareDma9>>, buses: DmaBuses) -> Self {
        let connections = NdmaConnections {
            hw, buses
        };
        Self {
            _connections: connections.clone(),
            channels: [
                NdmaChannel::new(connections.clone()), NdmaChannel::new(connections.clone()),
                NdmaChannel::new(connections.clone()), NdmaChannel::new(connections.clone()),
                NdmaChannel::new(connections.clone()), NdmaChannel::new(connections.clone()),
                NdmaChannel::new(connections.clone()), NdmaChannel::new(connections.clone()),
            ],
        }
    }
}

iodevice!(NdmaDevice, {
    internal_state: NdmaDeviceState;
    regs: {
        0x000 => global_cnt: u32 { }
    }
    ranges: {
        0x004;0xE0 => {
            // Remap addresses for individual channel registers
            read_effect = |dev: &mut NdmaDevice, buf_pos: usize, dest: &mut [u8]| {
                let channel = buf_pos / 0x1C;
                let new_buf_pos = buf_pos % 0x1C + 4; // As if the pos was for channel 0
                dev._internal_state.channels[channel].read_reg(new_buf_pos, dest);
            };
            write_effect = |dev: &mut NdmaDevice, buf_pos: usize, src: &[u8]| {
                let channel = buf_pos / 0x1C;
                let new_buf_pos = buf_pos % 0x1C + 4; // As if the pos was for channel 0
                dev._internal_state.channels[channel].write_reg(new_buf_pos, src);
            };
        }
    }
});
