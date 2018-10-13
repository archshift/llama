use cpu::BreakReason;
use cpu::interpreter_dummy11::*;

use mem;


pub mod modes {
    use std::thread;

    use super::*;

    pub fn idle() -> BoxedSteppable {
        let program = Program::<()>::new(());
        program.build()
    }

    pub fn boot() -> BoxedSteppable {
        const PXI_SYNC_ADDR: u32 = 0x10163000;
        fn pxisync_read(hw: &Dummy11HW) -> u8 {
            hw.memory.read::<u32>(PXI_SYNC_ADDR) as u8
        }
        fn pxisync_write(hw: &mut Dummy11HW, val: u8) {
            let current = hw.memory.read::<u32>(PXI_SYNC_ADDR);
            let new = current & 0xFFFF00FF | ((val as u32) << 8);
            hw.memory.write::<u32>(PXI_SYNC_ADDR, new);
        }

        const PXI_CNT_ADDR: u32 = 0x10163004;
        const PXI_RECV_ADDR: u32 = 0x1016300C;
        fn pxi_read(hw: &Dummy11HW) -> Option<u32> {
            let cnt = hw.memory.read::<u16>(PXI_CNT_ADDR);
            if cnt & 0x100 != 0 {
                None // Recv fifo empty
            } else {
                Some(hw.memory.read::<u32>(PXI_RECV_ADDR))
            }
        } 

        let mut program = Program::<()>::new(());
        dmprog!(in program; with _, hw;
            while +{ pxisync_read(hw) != 9 } {
                +{ thread::yield_now() };
            }
            +{ pxisync_write(hw, 11) };

            while +{ pxi_read(hw).is_none() } {
                +{ #[allow(deprecated)] { thread::sleep_ms(100) } };
            }

            +{ pxisync_write(hw, 1) };
            while +{ pxisync_read(hw) != 1 } {
                +{ thread::yield_now() };
            }

            +{ pxisync_write(hw, 14) };
            while +{ pxisync_read(hw) != 14 } {
                +{ thread::yield_now() };
            }

            while +{ pxisync_read(hw) != 1 } {
                +{ thread::yield_now() };
            }
            +{ pxisync_write(hw, 1) };

            while +{ pxisync_read(hw) != 0 } {
                +{ thread::yield_now() };
            }
            +{ pxisync_write(hw, 0) };
        );
        program.build()
    }

    pub fn kernel() -> BoxedSteppable {
        const SYNC_ADDR: u32 = 0x1FFFFFF0;

        let mut program = Program::<()>::new(());
        dmprog!(
            in program; with _, hw;
            +{ hw.memory.write::<u8>(SYNC_ADDR, 1) };
            while +{ hw.memory.read::<u8>(SYNC_ADDR) != 2 } {
                +{ thread::yield_now() };
            }
            +{ hw.memory.write::<u8>(SYNC_ADDR, 3) };
        );
        program.build()
    }
}

pub(crate) struct Dummy11HW {
    pub(crate) memory: mem::MemController
}

pub struct Dummy11 {
    pub(crate) hw: Dummy11HW,
    program: BoxedSteppable
}

impl Dummy11 {
    pub fn new(memory: mem::MemController, program: BoxedSteppable) -> Dummy11 {
        Dummy11 {
            hw: Dummy11HW {
                memory: memory
            },
            program: program
        }
    }

    pub fn step(&mut self) -> BreakReason {
        self.program.0.step(&mut self.hw)
    }
}
