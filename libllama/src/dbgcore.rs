use std::sync;

use cpu::{self, v5, v6};
pub use cpu::irq::{IrqType9, IrqClient};
use hwcore;
use io;

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

    pub fn ctx<'a>(&'a mut self, which: ActiveCpu) -> DbgContext<'a> {
        DbgContext {
            active_cpu: which,
            hwcore: self.hw.lock().unwrap()
        }
    }
}

pub struct DbgContext<'a> {
    active_cpu: ActiveCpu,
    hwcore: sync::MutexGuard<'a, hwcore::HwCore>
}

impl<'a> DbgContext<'a> {
    pub fn pause(&mut self) {
        self.hwcore.stop();
    }

    pub fn resume(&mut self) {
        self.hwcore.start();
    }

    pub fn running(&mut self) -> bool {
        self.hwcore.running()
    }

    pub fn hwcore(&self) -> &hwcore::HwCore {
        &*self.hwcore
    }

    pub fn hwcore_mut(&mut self) -> &mut hwcore::HwCore {
        &mut *self.hwcore
    }

    pub fn hw9<'b>(&'b mut self) -> DbgHw9Context<'b> {
        use std::sync::PoisonError;
        use hwcore::Hardware9;

        let print_regs = |p: PoisonError<sync::MutexGuard<'_, Hardware9>>| {
            let hw9 = p.into_inner();
            let s = format!("Internal error!\nCPU register state:\n\
                             gpregs: {:#X?}\n\
                             cpsr: {:#X?}\n\
                             last 1024 instruction addresses:\n\
                             {:#X?}", hw9.arm9.regs, hw9.arm9.cpsr.val, hw9.arm9.last_instructions);
            panic!("{}", s);
        };
        DbgHw9Context {
            // Will panic if still running
            hw: self.hwcore.hardware9.lock().unwrap_or_else(print_regs)
        }
    }

    pub fn hw11<'b>(&'b mut self) -> DbgHw11Context<'b> {
        use std::sync::PoisonError;
        use hwcore::Hardware11;

        let print_regs = |p: PoisonError<sync::MutexGuard<'_, Hardware11>>| {
            let hw11 = p.into_inner();
            let s = format!("Internal error!\nCPU register state:\n\
                             gpregs: {:#X?}\n\
                             cpsr: {:#X?}\n\
                             last 1024 instruction addresses:\n\
                             {:#X?}", hw11.arm11.regs, hw11.arm11.cpsr.val, hw11.arm11.last_instructions);
            panic!("{}", s);
        };
        DbgHw11Context {
            // Will panic if still running
            hw: self.hwcore.hardware11.lock().unwrap_or_else(print_regs)
        }
    }

    pub fn hw<'b>(&'b mut self) -> Box<HwCtx + 'b> {
        match self.active_cpu {
            ActiveCpu::Arm9 => Box::new(self.hw9()),
            ActiveCpu::Arm11 => Box::new(self.hw11())
        }
    }

    pub fn trigger_irq(&mut self, irq: IrqType9) {
        self.hwcore_mut().irq_tx.assert(irq);
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum ActiveCpu {
    Arm9, Arm11
}

#[allow(non_camel_case_types)]
pub enum CpuRef<'a> {
    v5(&'a cpu::Cpu<v5>),
    v6(&'a cpu::Cpu<v6>),
}

#[allow(non_camel_case_types)]
pub enum CpuMut<'a> {
    v5(&'a mut cpu::Cpu<v5>),
    v6(&'a mut cpu::Cpu<v6>),
}

macro_rules! any_cpu {
    ($self:expr, mut $ident:ident; $code:block) => {
        match $self.cpu_mut() {
            CpuMut::v5($ident) => $code,
            CpuMut::v6($ident) => $code
        }
    };
    ($self:expr, ref $ident:ident; $code:block) => {
        match $self.cpu_ref() {
            CpuRef::v5($ident) => $code,
            CpuRef::v6($ident) => $code
        }
    };
}

pub trait HwCtx {
    fn cpu_ref(&self) -> CpuRef;
    fn cpu_mut(&mut self) -> CpuMut;

    fn read_mem(&mut self, address: u32, bytes: &mut [u8]) -> Result<(), String> {
        any_cpu!(self, mut cpu; {
            cpu.mpu.icache_invalidate();
            cpu.mpu.dcache_invalidate();
            cpu.mpu.memory.debug_read_buf(address, bytes)
        })
    }

    fn write_mem(&mut self, address: u32, bytes: &[u8]) {
        any_cpu!(self, mut cpu; {
            cpu.mpu.icache_invalidate();
            cpu.mpu.dcache_invalidate();
            cpu.mpu.memory.write_buf(address, bytes);
        })
    }

    fn read_reg(&self, reg: usize) -> u32 {
        any_cpu!(self, ref cpu; {
            cpu.regs[reg]
        })
    }

    fn write_reg(&mut self, reg: usize, value: u32) {
        any_cpu!(self, mut cpu; {
            cpu.regs[reg] = value;
        })
    }

    fn read_cpsr(&self) -> u32 {
        any_cpu!(self, ref cpu; {
            cpu.cpsr.val
        })
    }

    fn write_cpsr(&mut self, value: u32) {
        any_cpu!(self, mut cpu; {
            cpu.cpsr.val = value;
            let mode_num = cpu.cpsr.mode.get();
            cpu.regs.swap(cpu::Mode::from_num(mode_num));
        })
    }

    fn pause_addr(&self) -> u32 {
        any_cpu!(self, ref cpu; {
            cpu.regs[15] - cpu.get_pc_offset()
        })
    }

    fn branch_to(&mut self, addr: u32) {
        any_cpu!(self, mut cpu; {
            cpu.branch(addr);
        })
    }

    fn is_thumb(&self) -> bool {
        any_cpu!(self, ref cpu; {
            cpu.cpsr.thumb_bit.get() == 1
        })
    }

    fn step(&mut self) {
        any_cpu!(self, mut cpu; {
            cpu.run(1);
        })
    }

    fn set_breakpoint(&mut self, addr: u32) {
        any_cpu!(self, mut cpu; {
            cpu.breakpoints.insert(addr);
        })
    }

    fn has_breakpoint(&mut self, addr: u32) -> bool {
        any_cpu!(self, ref cpu; {
            cpu.breakpoints.contains(&addr)
        })
    }

    fn del_breakpoint(&mut self, addr: u32) {
        any_cpu!(self, mut cpu; {
            cpu.breakpoints.remove(&addr);
        })
    }
}

pub struct DbgHw9Context<'a> {
    hw: sync::MutexGuard<'a, hwcore::Hardware9>
}

impl<'a> DbgHw9Context<'a> {
    pub fn io9_devices(&self) -> &io::IoRegsArm9 {
        self.hw.io9()
    }
}

impl<'a> HwCtx for DbgHw9Context<'a> {
    fn cpu_ref(&self) -> CpuRef {
        CpuRef::v5(&self.hw.arm9)
    }
    fn cpu_mut(&mut self) -> CpuMut {
        CpuMut::v5(&mut self.hw.arm9)
    }
}

pub struct DbgHw11Context<'a> {
    hw: sync::MutexGuard<'a, hwcore::Hardware11>
}

impl<'a> HwCtx for DbgHw11Context<'a> {
    fn cpu_ref(&self) -> CpuRef {
        CpuRef::v6(&self.hw.arm11)
    }
    fn cpu_mut(&mut self) -> CpuMut {
        CpuMut::v6(&mut self.hw.arm11)
    }
}
