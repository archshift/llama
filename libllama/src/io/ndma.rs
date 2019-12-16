use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::fmt;

use hwcore::HardwareDma9;
use io::{DmaBus, DmaBuses};

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
    startup_mode: 24:27,
    immed_mode: 28:28,
    repeat_mode: 29:29,
    _enable_irq: 30:30,
    enabled: 31:31
});

#[derive(Clone, Debug)]
pub struct NdmaChannelState {
    connections: Rc<RefCell<NdmaConnections>>,
    started: bool,
    src_addr: u32,
    dst_addr: u32,
    xfer_total: u32,
}

fn should_xfer(dev: &mut NdmaChannel) -> bool {
    let chan_cnt = RegChannelCnt::new(dev.chan_cnt.get());
    if chan_cnt.enabled.get() == 0 {
        return false;
    }
    let startup_dev = chan_cnt.startup_mode.get();
    let mut xns = dev._internal_state.connections.borrow_mut();

    let mode = (chan_cnt.immed_mode.get(), chan_cnt.repeat_mode.get());
    match mode {
        (0, 0) => {
            let bus = xns.buses.get_mut(&startup_dev)
                .expect(&format!("Could not find NDMA bus for device {}", startup_dev));
            bus.observe()
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

    let xns = dev._internal_state.connections.borrow_mut();
    let mut hw = xns.hw.borrow_mut();

    let chan_cnt = RegChannelCnt::new(dev.chan_cnt.get());

    let line_size = 1u32 << chan_cnt.xfer_size.get();
    let src_addr = &mut dev._internal_state.src_addr;
    let dst_addr = &mut dev._internal_state.dst_addr;
    let total_words = dev.write_cnt.get() & 0xFFFFFF;

    let mut tmp_buf = vec![0u8; 4 * line_size as usize];
    let word_fill = dev.fill_data.get().to_le_bytes();

    assert!(*src_addr % 4 == 0);
    assert!(*dst_addr % 4 == 0);
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

            hw.mem.read_buf(*src_addr, &mut tmp_buf[..]);

            *src_addr = match src_wb_mode {
                0 => *src_addr + 4 * line_size,
                1 => *src_addr - 4 * line_size,
                2 => *src_addr,
                _ => unreachable!()
            };
        }

        hw.mem.write_buf(*dst_addr, &mut tmp_buf[..]);

        *dst_addr = match dst_wb_mode {
            0 => *dst_addr + 4 * line_size,
            1 => *dst_addr - 4 * line_size,
            2 => *dst_addr,
            _ => unreachable!()
        };
    }

    dev._internal_state.xfer_total += total_words;

    let chan_cnt = RegChannelCnt::alias_mut(dev.chan_cnt.ref_mut());
    let mode = (chan_cnt.immed_mode.get(), chan_cnt.repeat_mode.get());
    match mode {
        (0, 0) => {
            // Suspend mode
            let xfer_total = dev._internal_state.xfer_total;
            let xfer_max = dev.xfer_max.get();
            assert!(xfer_total <= xfer_max);
            if xfer_total == xfer_max {
                chan_cnt.enabled.set(0);
            }
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
    let chan_cnt = RegChannelCnt::alias_mut(dev.chan_cnt.ref_mut());
    let state = &mut dev._internal_state;

    if !state.started && chan_cnt.enabled.get() == 1 {
        // Just starting transfer now, copy data down.
        state.src_addr = dev.src_addr.get();
        state.dst_addr = dev.dst_addr.get();
        state.xfer_total = 0;
    }
    state.started = chan_cnt.enabled.get() == 1;
}


iodevice!(NdmaChannel, {
    internal_state: NdmaChannelState;
    regs: {
        0x004 => src_addr: u32 { }
        0x008 => dst_addr: u32 { }
        0x00C => xfer_max: u32 {
            write_bits = 0x0FFFFFFF;
        }
        0x010 => write_cnt: u32 {
            write_bits = 0x00FFFFFF;
        }
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
    buses: HashMap<u32, DmaBus>
}

impl fmt::Debug for NdmaConnections {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "NdmaConnections {{ }}")
    }
}

#[derive(Debug)]
pub struct NdmaDeviceState {
    channels: [NdmaChannel; 8],
}

impl NdmaDeviceState {
    pub fn new(hw: Rc<RefCell<HardwareDma9>>, buses: DmaBuses) -> Self {
        let mut bus_map = HashMap::new();
        bus_map.insert(10, buses.sha_in.clone());
        bus_map.insert(11, buses.sha_out.clone());

        let chan_state = NdmaChannelState {
            connections: Rc::new(RefCell::new(
                NdmaConnections {
                    hw, buses: bus_map
                }
            )),
            started: false,
            src_addr: 0,
            dst_addr: 0,
            xfer_total: 0
        };
        
        Self {
            channels: [
                NdmaChannel::new(chan_state.clone()), NdmaChannel::new(chan_state.clone()),
                NdmaChannel::new(chan_state.clone()), NdmaChannel::new(chan_state.clone()),
                NdmaChannel::new(chan_state.clone()), NdmaChannel::new(chan_state.clone()),
                NdmaChannel::new(chan_state.clone()), NdmaChannel::new(chan_state.clone()),
            ],
        }
    }
}

pub fn schedule(dev: &mut NdmaDevice) {
    for channel in dev._internal_state.channels.iter_mut() {
        process_channel(channel);
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
