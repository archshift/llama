use std::sync;

use hwcore;

#[derive(Clone)]
pub struct DbgCore {
    hw: sync::Arc<sync::Mutex<hwcore::HwCore>>
}

impl DbgCore {
    pub fn bind(hw: hwcore::HwCore) -> DbgCore {
        DbgCore {
            hw: sync::Arc::new(sync::Mutex::new(hw)),
        }
    }

    pub fn ctx<'a>(&'a mut self) -> DbgContext<'a> {
        DbgContext {
            hwcore: self.hw.lock().unwrap()
        }
    }
}

pub struct DbgContext<'a> {
    hwcore: sync::MutexGuard<'a, hwcore::HwCore>
}

impl<'a> DbgContext<'a> {
    pub fn pause(&mut self) {
        self.hwcore.stop();
    }

    pub fn resume(&mut self) {
        self.hwcore.start();
    }

    pub fn hwcore(&self) -> &hwcore::HwCore {
        &*self.hwcore
    }

    pub fn hwcore_mut(&mut self) -> &mut hwcore::HwCore {
        &mut *self.hwcore
    }

    pub fn hw<'b>(&'b mut self) -> DbgHwContext<'b> {
        DbgHwContext {
            // Will panic if still running
            hw: self.hwcore.hardware_mut()
        }
    }
}

pub struct DbgHwContext<'a> {
    hw: sync::RwLockWriteGuard<'a, hwcore::Hardware>
}

impl<'a> DbgHwContext<'a> {
    pub fn read_mem(&self, address: u32, bytes: &mut [u8]) {
        self.hw.arm9.memory.read_buf(address, bytes)
    }

    pub fn write_mem(&mut self, address: u32, bytes: &[u8]) {
        self.hw.arm9.memory.write_buf(address, bytes)
    }

    pub fn read_reg(&self, reg: usize) -> u32 {
        self.hw.arm9.regs[reg]
    }

    pub fn write_reg(&mut self, reg: usize, value: u32) {
        self.hw.arm9.regs[reg] = value;
    }

    pub fn pause_addr(&self) -> u32 {
        self.hw.arm9.regs[15] - self.hw.arm9.get_pc_offset()
    }

    pub fn is_thumb(&self) -> bool {
        bf!((self.hw.arm9.cpsr).thumb_bit) == 1
    }

    pub fn step(&mut self) {
        self.hw.arm9.run(1);
    }

    pub fn set_breakpoint(&mut self, addr: u32) {
        self.hw.arm9.breakpoints.insert(addr, false);
    }

    pub fn has_breakpoint(&mut self, addr: u32) -> bool {
        self.hw.arm9.breakpoints.get(&addr).is_some()
    }

    pub fn del_breakpoint(&mut self, addr: u32) {
        self.hw.arm9.breakpoints.remove(&addr);
    }
}