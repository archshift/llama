use std::sync::{self, atomic};

use hwcore;

pub struct DbgCore {
    hw: hwcore::HwCore
}

impl DbgCore {
    pub fn bind(hw: hwcore::HwCore) -> DbgCore {
        DbgCore {
            hw: hw,
        }
    }

    pub fn release(dbgcore: DbgCore) -> hwcore::HwCore {
        dbgcore.hw
    }

    pub fn pause(&mut self) {
        self.hw.stop();
    }

    pub fn resume(&mut self) {
        self.hw.start();
    }

    pub fn get_ctx<'a>(&'a mut self) -> DbgContext<'a> {
        DbgContext {
            // Will panic if still running
            hw: self.hw.hardware_mut()
        }
    }

    pub fn hwcore(&self) -> &hwcore::HwCore {
        &self.hw
    }

    pub fn hwcore_mut(&mut self) -> &mut hwcore::HwCore {
        &mut self.hw
    }
}

pub struct DbgContext<'a> {
    hw: sync::RwLockWriteGuard<'a, hwcore::Hardware>
}

impl<'a> DbgContext<'a> {
    pub fn read_mem(&self, address: u32) -> u32 {
        self.hw.arm9.memory.read(address)
    }

    pub fn write_mem(&mut self, address: u32, value: u32) {
        self.hw.arm9.memory.write(address, value)
    }

    pub fn read_reg(&self, reg: usize) -> u32 {
        self.hw.arm9.regs[reg]
    }

    pub fn write_reg(&self, reg: usize, value: u32) {

    }

    fn set_breakpoint(&self) {

    }
}