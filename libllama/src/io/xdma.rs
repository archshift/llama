use std::cell::RefCell;
use std::rc::Rc;
use std::fmt;

use hwcore::HardwareDma9;

struct DmacVer;
pub trait Version {}
impl Version for DmacVer {}

pub type InstFn<V> = fn(&mut XdmaDevice, u64, V);
include!(concat!(env!("OUT_DIR"), "/dmac.decoder.rs"));



pub struct XdmaDeviceState {
    hw: Rc<RefCell<HardwareDma9>>,
    active_thread: Option<usize>,
    manager: XdmaThreadState,
    channels: [XdmaThreadState; 8]
}

impl fmt::Debug for XdmaDeviceState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "XdmaDeviceState {{ }}")
    }
}

impl XdmaDeviceState {
    pub fn new(hw: Rc<RefCell<HardwareDma9>>) -> Self {
        Self {
            hw,
            active_thread: None,
            manager: Default::default(),
            channels: [
                Default::default(), Default::default(), Default::default(), Default::default(),
                Default::default(), Default::default(), Default::default(), Default::default()
            ]
        }
    }
}


#[derive(Default)]
struct XdmaThreadState {
    pc: u32,
    running: bool,
    src_addr: u32,
    dst_addr: u32,
    chan_ctrl: u32,
    loop_ctr: [u32; 2],
    loop_start_pc: [u32; 2],
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
            run_instruction(xdma, inst);
        }

        replace_active_thread(xdma, old_thread);
        increment_pc(xdma, 6);
    }

    pub fn kill<V: Version>(xdma: &mut XdmaDevice, _instr: u64, _: V) {
        warn!("STUBBED: XDMA thread KILL");
        active_thread(xdma).running = false;
    }

    pub fn flushp<V: Version>(xdma: &mut XdmaDevice, instr: Flushp::Bf, _: V) {
        warn!("STUBBED: Unimplemented XDMA instruction {:?}!", instr);
        increment_pc(xdma, 2);
    }

    pub fn mov<V: Version>(xdma: &mut XdmaDevice, instr: Mov::Bf, _: V) {
        let imm = instr.imm.get() as u32;
        let reg = instr.rd.get();
        {
            let thread = active_thread(xdma);
            match reg {
                0 => thread.src_addr = imm,
                1 => thread.chan_ctrl = imm,
                2 => thread.dst_addr = imm,
                _ => unreachable!()
            }
        }
        increment_pc(xdma, 6);
    }

    pub fn lp<V: Version>(xdma: &mut XdmaDevice, instr: Lp::Bf, _: V) {
        let which_reg = instr.lc.get() as usize;
        let iterations = instr.iter.get() as u32;
        active_thread(xdma).loop_ctr[which_reg] = iterations;

        let next_pc = increment_pc(xdma, 2);
        active_thread(xdma).loop_start_pc[which_reg] = next_pc;
    }

    pub fn wfp<V: Version>(xdma: &mut XdmaDevice, instr: Wfp::Bf, _: V) {
        let periph = instr.periph.get();
        let bs = instr.bs.get();
        let p = instr.p.get();
        #[derive(Debug)]
        enum Mode {
            Single, Peripheral, Burst
        }
        let mode = match (bs, p) {
            (0, 0) => Mode::Single,
            (1, 0) => Mode::Burst,
            (0, 1) => Mode::Peripheral,
            _ => unreachable!()
        };

        increment_pc(xdma, 2);

        warn!("STUBBED: XDMA WFP with periph={}, mode={:?}", periph, mode);
    }

    pub fn ldp<V: Version>(_xdma: &mut XdmaDevice, instr: Ldp::Bf, _: V) {
        let periph = instr.periph.get();
        let bs = instr.bs.get();

        panic!("Unimplmented XDMA LDP{} from periph {}",
               if bs == 1 {"B"} else {"S"}, periph);
    }
    
    pub fn nop<V: Version>(xdma: &mut XdmaDevice, _instr: u64, _: V) {
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