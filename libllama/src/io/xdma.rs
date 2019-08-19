use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::fmt;

use utils::fifo::Fifo;

use hwcore::HardwareDma9;
use io::{DmaBus, DmaBuses};

struct DmacVer;
pub trait Version {}
impl Version for DmacVer {}

pub type InstFn<V> = fn(&mut XdmaDevice, u64, V);
include!(concat!(env!("OUT_DIR"), "/dmac.decoder.rs"));



pub struct XdmaDeviceState {
    hw: Rc<RefCell<HardwareDma9>>,
    active_thread: Option<usize>,
    manager: XdmaThreadState,
    channels: [XdmaThreadState; 8],
    buses: HashMap<u32, Rc<dyn DmaBus>>
}

impl fmt::Debug for XdmaDeviceState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "XdmaDeviceState {{ }}")
    }
}

impl XdmaDeviceState {
    pub fn new(hw: Rc<RefCell<HardwareDma9>>, buses: DmaBuses) -> Self {
        let mut bus_map = HashMap::new();
        bus_map.insert(7, buses.sha);

        Self {
            hw,
            active_thread: None,
            manager: Default::default(),
            channels: [
                Default::default(), Default::default(), Default::default(), Default::default(),
                Default::default(), Default::default(), Default::default(), Default::default()
            ],
            buses: bus_map
        }
    }
}


bf!(ChannelCtrl[u32] {
    src_inc: 0:0,
    src_burst_size: 1:3,
    src_burst_len: 4:7,
    dst_inc: 14:14,
    dst_burst_size: 15:17,
    dst_burst_len: 18:21
});

#[derive(Copy, Clone, Debug)]
enum RequestType {
    Single, Peripheral, Burst
}

struct XdmaThreadState {
    pc: u32,
    running: bool,
    src_addr: u32,
    dst_addr: u32,
    chan_ctrl: ChannelCtrl::Bf,
    loop_ctr: [u32; 2],
    request_type: RequestType,
    data_fifo: Fifo<u8>,
}

impl Default for XdmaThreadState {
    fn default() -> Self {
        Self {
            pc: 0,
            running: false,
            src_addr: 0,
            dst_addr: 0,
            chan_ctrl: ChannelCtrl::new(0),
            loop_ctr: Default::default(),
            request_type: RequestType::Single,
            data_fifo: Fifo::new(16*16),
        }
    }
}

fn replace_active_thread(dev: &mut XdmaDevice, new: Option<usize>) -> Option<usize> {
    let res = dev._internal_state.active_thread.take();
    dev._internal_state.active_thread = new;
    res
}

fn active_thread(dev: &mut XdmaDevice) -> &mut XdmaThreadState {
    let state = &mut dev._internal_state;
    if let Some(chan) = state.active_thread {
        &mut state.channels[chan]
    } else {
        &mut state.manager
    }
}

fn increment_pc(dev: &mut XdmaDevice, by: usize) -> u32 {
    let thread = active_thread(dev);
    thread.pc += by as u32;
    thread.pc
}

mod interpreter {
    use super::*;
    pub fn run_instruction(dev: &mut XdmaDevice, inst: u64) {
        let inst_fn: InstFn<DmacVer> = decode(inst);
        inst_fn(dev, inst, DmacVer);
    }

    pub fn end<V: Version>(xdma: &mut XdmaDevice, _instr: u64, _: V) {
        warn!("STUBBED: XDMA thread END");
        active_thread(xdma).running = false;
    }

    pub fn go<V: Version>(xdma: &mut XdmaDevice, instr: Go::Bf, _: V) {
        let start_addr = instr.addr.get() as u32;
        let chan = instr.cn.get() as usize;
        let old_thread = replace_active_thread(xdma, Some(chan));

        warn!("STUBBED: XDMA starting at address {:08X} for channel {:?}", start_addr, chan);

        {
            let thread = active_thread(xdma);
            thread.pc = start_addr;
            thread.running = true;
        }

        while active_thread(xdma).running {
            let mut inst = [0u8; 8];
            let pc = active_thread(xdma).pc;
            {
                let hw = xdma._internal_state.hw.borrow();
                let mem = &hw.mem;
                mem.read_buf(pc, &mut inst);
            };

            let inst = u64::from_le_bytes(inst);
            warn!("Running XDMA instruction at {:08X}: {}", pc, disasm(inst));
            run_instruction(xdma, inst);
        }

        replace_active_thread(xdma, old_thread);
        increment_pc(xdma, 6);
    }

    pub fn kill<V: Version>(xdma: &mut XdmaDevice, _instr: u64, _: V) {
        warn!("STUBBED: XDMA thread KILL");
        active_thread(xdma).running = false;
    }

    pub fn flushp<V: Version>(xdma: &mut XdmaDevice, _instr: Flushp::Bf, _: V) {
        warn!("STUBBED: Unimplemented XDMA FLUSHP!");
        increment_pc(xdma, 2);
    }

    pub fn sev<V: Version>(xdma: &mut XdmaDevice, _instr: Sev::Bf, _: V) {
        warn!("STUBBED: Unimplemented XDMA SEV!");
        increment_pc(xdma, 2);
    }

    pub fn mov<V: Version>(xdma: &mut XdmaDevice, instr: Mov::Bf, _: V) {
        let imm = instr.imm.get() as u32;
        let reg = instr.rd.get();
        {
            let thread = active_thread(xdma);
            match reg {
                0 => thread.src_addr = imm,
                1 => thread.chan_ctrl.val = imm,
                2 => thread.dst_addr = imm,
                _ => unreachable!()
            }
        }
        increment_pc(xdma, 6);
    }

    pub fn lp<V: Version>(xdma: &mut XdmaDevice, instr: Lp::Bf, _: V) {
        let which_reg = instr.lc.get() as usize;
        let iterations = instr.iter.get() as u32;
        active_thread(xdma).loop_ctr[which_reg] = iterations + 1;

        increment_pc(xdma, 2);
    }

    pub fn lpend<V: Version>(xdma: &mut XdmaDevice, instr: Lpend::Bf, _: V) {
        let which_reg = instr.lc.get() as usize;
        let forever = instr.nf.get() == 0;
        let rel = instr.jump.get() as u32;
        {
            let thread = &mut active_thread(xdma);

            assert!(!forever);
            let done = {
                let ctr = &mut thread.loop_ctr[which_reg];
                *ctr -= 1;
                *ctr == 0
            };

            if !done {
                thread.pc -= rel;
                return
            }
        }
        increment_pc(xdma, 2);
    }

    pub fn wfp<V: Version>(xdma: &mut XdmaDevice, instr: Wfp::Bf, _: V) {
        let periph = instr.periph.get();
        let bs = instr.bs.get();
        let p = instr.p.get();

        let mode = match (bs, p) {
            (0, 0) => RequestType::Single,
            (1, 0) => RequestType::Burst,
            (0, 1) => RequestType::Peripheral,
            _ => unreachable!()
        };
        active_thread(xdma).request_type = mode;

        {
            let bus = &xdma._internal_state.buses[&(periph as u32)];
            if !bus.read_ready() && !bus.write_ready() {
                panic!("XDMA waiting for unready peripheral {}!", periph);
            }
        }
        increment_pc(xdma, 2);


        warn!("STUBBED: XDMA WFP with periph={}, mode={:?}", periph, mode);
    }

    pub fn ld<V: Version>(xdma: &mut XdmaDevice, instr: Ld::Bf, _: V) {
        let x = instr.x.get();
        let bs = instr.bs.get();

        let src_burst_len;
        let src_burst_size;
        let src_inc;
        let req_type;
        let mut src_addr;
        {
            let thread = active_thread(xdma);
            src_burst_len = thread.chan_ctrl.src_burst_len.get() + 1;
            src_burst_size = 1 << thread.chan_ctrl.src_burst_size.get();
            src_inc = src_burst_size * thread.chan_ctrl.src_inc.get();
            req_type = thread.request_type;
            src_addr = thread.src_addr;
        }

        match (req_type, bs, x) {
            (RequestType::Single, 0, 1) | (RequestType::Peripheral, 0, 1) => {
                // do single transfer
                unimplemented!()
            }
            (_, 0, 0) | (RequestType::Burst, 1, 1) | (RequestType::Peripheral, 1, 1) => {
                // do burst transfer
                let mut dst_buf = [0u8; 16];
                let state = &mut xdma._internal_state;
                let thread_num = state.active_thread.unwrap();
                let hw = state.hw.borrow_mut();
                let fifo = &mut state.channels[thread_num].data_fifo;

                warn!("STUBBED: XDMA burst read {}x{} from addr {:08X}, incrementing?={}",
                      src_burst_len, src_burst_size, src_addr, src_inc>0);

                for _ in 0..src_burst_len {
                    let buf = &mut dst_buf[..(src_burst_size as usize)];

                    hw.mem.read_buf(src_addr, buf);
                    fifo.clone_extend(buf);

                    warn!("XDMA reading {:X?} from address {:08X}", buf, src_addr);
                    src_addr += src_inc;
                }
            }
            _ => {}
        }

        active_thread(xdma).src_addr = src_addr;
        increment_pc(xdma, 1);
    }

    pub fn ldp<V: Version>(xdma: &mut XdmaDevice, instr: Ldp::Bf, _: V) {
        let periph = instr.periph.get() as u32;
        let bs = instr.bs.get();

        let src_burst_len;
        let src_burst_size;
        let src_inc;
        let req_type;
        let mut src_addr;
        {
            let thread = active_thread(xdma);
            src_burst_len = thread.chan_ctrl.src_burst_len.get() + 1;
            src_burst_size = 1 << thread.chan_ctrl.src_burst_size.get();
            src_inc = src_burst_size * thread.chan_ctrl.src_inc.get();
            req_type = thread.request_type;
            src_addr = thread.src_addr;
        }

        match (req_type, bs) {
            (RequestType::Single, 0) | (RequestType::Peripheral, 0) => {
                // do single transfer
                unimplemented!()
            }
            (RequestType::Burst, 1) | (RequestType::Peripheral, 1) => {
                // do burst transfer
                let mut dst_buf = [0u8; 16];
                let state = &mut xdma._internal_state;
                let peripheral = &state.buses[&periph];
                let thread_num = state.active_thread.unwrap();
                let fifo = &mut state.channels[thread_num].data_fifo;

                warn!("STUBBED: XDMA burst read {}x{} from addr {:08X}, incrementing?={}",
                      src_burst_len, src_burst_size, src_addr, src_inc>0);

                for _ in 0..src_burst_len {
                    let buf = &mut dst_buf[..(src_burst_size as usize)];

                    peripheral.read_addr(src_addr, buf);
                    fifo.clone_extend(buf);
                    warn!("XDMA reading {:X?} from peripheral at address {:08X}", buf, src_addr);
                    src_addr += src_inc;
                }
            }
            _ => {}
        }

        active_thread(xdma).src_addr = src_addr;
        increment_pc(xdma, 2);
    }

    pub fn st<V: Version>(xdma: &mut XdmaDevice, instr: St::Bf, _: V) {
        let x = instr.x.get();
        let bs = instr.bs.get();

        let dst_burst_len;
        let dst_burst_size;
        let dst_inc;
        let req_type;
        let mut dst_addr;
        {
            let thread = active_thread(xdma);
            dst_burst_len = thread.chan_ctrl.dst_burst_len.get() + 1;
            dst_burst_size = 1 << thread.chan_ctrl.dst_burst_size.get();
            dst_inc = dst_burst_size * thread.chan_ctrl.dst_inc.get();
            req_type = thread.request_type;
            dst_addr = thread.dst_addr;
        }

        match (req_type, bs, x) {
            (RequestType::Single, 0, 1) | (RequestType::Peripheral, 0, 1) => {
                // do single transfer
                unimplemented!()
            }
            (_, 0, 0) | (RequestType::Burst, 1, 1) | (RequestType::Peripheral, 1, 1) => {
                // do burst transfer
                let mut src_buf = [0u8; 16];
                let state = &mut xdma._internal_state;
                let thread_num = state.active_thread.unwrap();
                let mut hw = state.hw.borrow_mut();
                let fifo = &mut state.channels[thread_num].data_fifo;

                warn!("STUBBED: XDMA burst write {}x{} to addr {:08X}, incrementing?={}",
                      dst_burst_len, dst_burst_size, dst_addr, dst_inc>0);

                for _ in 0..dst_burst_len {
                    let buf = &mut src_buf[..(dst_burst_size as usize)];
                    let amount = fifo.drain(buf);
                    assert_eq!(amount, buf.len());

                    hw.mem.write_buf(dst_addr, buf);
                    warn!("XDMA writing {:X?} to address {:08X}", buf, dst_addr);
                    dst_addr += dst_inc;
                }
            }
            _ => {}
        }

        active_thread(xdma).dst_addr = dst_addr;
        increment_pc(xdma, 1);
    }
    
    pub fn nop<V: Version>(xdma: &mut XdmaDevice, _instr: u64, _: V) {
        increment_pc(xdma, 1);
    }

    pub fn rmb<V: Version>(xdma: &mut XdmaDevice, _instr: u64, _: V) {
        warn!("STUBBED: XDMA RMB");
        increment_pc(xdma, 1);
    }

    pub fn wmb<V: Version>(xdma: &mut XdmaDevice, _instr: u64, _: V) {
        warn!("STUBBED: XDMA WMB");
        increment_pc(xdma, 1);
    }

    pub fn undef<V: Version>(_xdma: &mut XdmaDevice, instr: u64, _: V) {
        panic!("Unimplemented XDMA instruction! {:012X}", instr & 0xFFFFFFFFFFFF);
    }
}

iodevice!(XdmaDevice, {
    internal_state: XdmaDeviceState;
    regs: {
        0x000 => dm_status: u32 { }
        0x020 => int_enable: u32 { }
        0x02C => int_clr: u32 { }
        0x040 => fault_type0: u32 { }
        0x100 => csr0: u32 { }
        0xD00 => dbg_status: u32 { }
        0xD04 => dbg_cmd: u32 {
            write_effect = |dev: &mut XdmaDevice| {
                if dev.dbg_cmd.get() & 0b11 == 0 {
                    let inst0 = dev.dbg_inst0.get();
                    let inst = ((inst0 as u64) >> 16) | ((dev.dbg_inst1.get() as u64) << 16);
                    let active_thread = if inst0 & 1 == 1 {
                        Some(((inst0 >> 8) & 0b111) as usize)
                    } else {
                        None
                    };

                    warn!("STUBBED: Running XDMA standalone instruction {:016X} on thread {:?}!", inst, active_thread);
                    dev._internal_state.active_thread = active_thread;
                    interpreter::run_instruction(dev, inst);
                }
            };
        }
        0xD08 => dbg_inst0: u32 { }
        0xD0C => dbg_inst1: u32 { }
    }
});
