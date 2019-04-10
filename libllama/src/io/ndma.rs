use std::cell::Cell;
use std::rc::Rc;

bf!(RegGlobalCnt[u32] {
    enabled: 0:0,
    cycle_select: 16:19,
    round_robin: 31:31
});

bf!(RegChannelCnt[u32] {
    _dst_addr_writeback_mode: 10:11,
    _dst_addr_reload: 12:12,
    _src_addr_writeback_mode: 13:14,
    _src_addr_reload: 15:15,
    _xfer_size: 16:19,
    _startup_mode: 24:27,
    _immed_mode: 28:28,
    _repeat_mode: 29:29,
    _enable_irq: 30:30,
    enabled: 31:31
});

fn reg_chan_cnt_write(dev: &mut NdmaChannel) {
    let chan_cnt = RegChannelCnt::new(dev.chan_cnt.get());
    if chan_cnt.enabled.get() == 1 {
        unimplemented!()
    }
    warn!("STUBBED: NDMA chan_cnt write {:X?}", chan_cnt);
}

iodevice!(NdmaChannel, {
    internal_state: Rc<Cell<RegGlobalCnt::Bf>>;
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

#[derive(Debug)]
pub struct NdmaDeviceState {
    global_cnt: Rc<Cell<RegGlobalCnt::Bf>>,
    channels: [NdmaChannel; 8],
}

impl Default for NdmaDeviceState {
    fn default() -> NdmaDeviceState {
        let global_cnt = Rc::new(Cell::new(RegGlobalCnt::new(0)));
        NdmaDeviceState {
            global_cnt: global_cnt.clone(),
            channels: [
                NdmaChannel::new(global_cnt.clone()), NdmaChannel::new(global_cnt.clone()),
                NdmaChannel::new(global_cnt.clone()), NdmaChannel::new(global_cnt.clone()),
                NdmaChannel::new(global_cnt.clone()), NdmaChannel::new(global_cnt.clone()),
                NdmaChannel::new(global_cnt.clone()), NdmaChannel::new(global_cnt.clone())
            ],
        }
    }
}

iodevice!(NdmaDevice, {
    internal_state: NdmaDeviceState;
    regs: {
        0x000 => global_cnt: u32 {
            write_effect = |dev: &mut NdmaDevice| {
                let new_val = RegGlobalCnt::new(dev.global_cnt.get());
                dev._internal_state.global_cnt.set(new_val);
            };
        }
    }
    ranges: {
        0x004;0xE0 => {
            // Remap addresses for individual channel registers
            read_effect = |dev: &mut NdmaDevice, buf_pos: usize, dest: &mut [u8]| {
                let channel = buf_pos / 0x1C;
                let new_buf_pos = buf_pos % 0x1C + 4; // As if the pos was for channel 0
                dev._internal_state.channels[channel].read_reg(new_buf_pos, dest.as_mut_ptr(), dest.len());
            };
            write_effect = |dev: &mut NdmaDevice, buf_pos: usize, src: &[u8]| {
                let channel = buf_pos / 0x1C;
                let new_buf_pos = buf_pos % 0x1C + 4; // As if the pos was for channel 0
                dev._internal_state.channels[channel].write_reg(new_buf_pos, src.as_ptr(), src.len());
            };
        }
    }
});
