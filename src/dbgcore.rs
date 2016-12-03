use std::borrow::{Borrow, BorrowMut};
use std::sync::{self, atomic};

use hwcore;

pub struct DbgCore {
    hw: hwcore::HwCore
}

impl DbgCore {
    pub fn new(hw: hwcore::HwCore) -> DbgCore {
        DbgCore {
            hw: hw,
        }
    }

    pub fn pause<'a>(&'a mut self) -> DbgContext<'a> {
        self.hw.stop();
        DbgContext {
            hw: self.hw.hardware_mut()
        }
    }

    pub fn resume(&mut self, context: DbgContext) {
        self.hw.start();
    }
}

impl Borrow<hwcore::HwCore> for DbgCore {
    fn borrow(&self) -> &hwcore::HwCore {
        &self.hw
    }
}

impl BorrowMut<hwcore::HwCore> for DbgCore {
    fn borrow_mut(&mut self) -> &mut hwcore::HwCore {
        &mut self.hw
    }
}

pub struct DbgContext<'a> {
    pub hw: sync::RwLockWriteGuard<'a, hwcore::Hardware>
}

impl<'a> DbgContext<'a> {
    pub fn read_mem() {

    }

    pub fn write_mem() {

    }

    pub fn read_reg() {

    }

    pub fn write_reg() {

    }

    fn set_breakpoint(&self) {

    }
}