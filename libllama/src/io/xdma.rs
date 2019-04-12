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
    running: bool
}

fn active_thread(dev: &mut XdmaDevice) -> &mut XdmaThreadState {
    let state = &mut dev._internal_state;
    if let Some(chan) = state.active_thread {
        &mut state.channels[chan]
    } else {
        &mut state.manager
    }
}

fn increment_pc(dev: &mut XdmaDevice, by: usize) {
    let thread = active_thread(dev);
    thread.pc += by as u32;
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

    pub fn go<V: Version>(xdma: &mut XdmaDevice, instr: u64, _: V) {
        let start_addr = (instr >> 16) as u32;
        warn!("STUBBED: XDMA starting at address {:08X}", start_addr);

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
    }

    pub fn kill<V: Version>(xdma: &mut XdmaDevice, _instr: u64, _: V) {
        warn!("STUBBED: XDMA thread KILL");
        active_thread(xdma).running = false;
    }

    pub fn flushp<V: Version>(xdma: &mut XdmaDevice, instr: u64, _: V) {
        warn!("STUBBED: Unimplemented XDMA instruction {}! {:08X}", disasm(instr), instr);
        increment_pc(xdma, 2);
    }

    pub fn undef<V: Version>(_xdma: &mut XdmaDevice, instr: u64, _: V) {
        panic!("Unimplemented XDMA instruction! {:08X}", instr)
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