use clock;
use cpu;
use cpu::caches;
use cpu::coproc;
use cpu::irq;
use cpu::regs::{GpRegs, Psr};
use mem;

use std::collections::HashSet;

#[derive(Copy, Clone, Debug)]
pub enum Mode {
    Usr = 0b10000,
    Fiq = 0b10001,
    Irq = 0b10010,
    Svc = 0b10011,
    Abt = 0b10111,
    Und = 0b11011,
    Sys = 0b11111
}

impl Mode {
    pub fn from_num(val: u32) -> Mode {
        match val {
            n if n == Mode::Usr as u32 => Mode::Usr,
            n if n == Mode::Fiq as u32 => Mode::Fiq,
            n if n == Mode::Irq as u32 => Mode::Irq,
            n if n == Mode::Svc as u32 => Mode::Svc,
            n if n == Mode::Abt as u32 => Mode::Abt,
            n if n == Mode::Und as u32 => Mode::Und,
            n if n == Mode::Sys as u32 => Mode::Sys,
            _ => unreachable!()
        }
    }
}

pub struct Cpu {
    pub regs: GpRegs,
    pub cpsr: Psr,
    pub spsr_fiq: Psr,
    pub spsr_irq: Psr,
    pub spsr_svc: Psr,
    pub spsr_abt: Psr,
    pub spsr_und: Psr,

    coproc_syscnt: coproc::SysControl,
    pub mpu: caches::Mpu,

    irq_line: irq::IrqLine,
    cycles: usize,
    sys_clk: clock::SysClock,

    pub breakpoints: HashSet<u32> // addr, is_triggered
}

#[derive(Clone)]
pub enum BreakReason {
    LimitReached,
    Breakpoint,
    Trapped,
    WFI
}


const PAUSE_CYCLES: usize = 128;

impl Cpu {
    pub fn new(memory: mem::MemController, irq_line: irq::IrqLine, clk: clock::SysClock) -> Cpu {
        Cpu {
            regs: GpRegs::new(Mode::Svc),
            cpsr: Psr::new(0),
            spsr_fiq: Psr::new(0),
            spsr_irq: Psr::new(0),
            spsr_svc: Psr::new(0),
            spsr_abt: Psr::new(0),
            spsr_und: Psr::new(0),

            coproc_syscnt: coproc::SysControl::new(),
            mpu: caches::Mpu::new(memory),

            irq_line: irq_line,
            cycles: PAUSE_CYCLES,
            sys_clk: clk,

            breakpoints: HashSet::new()
        }
    }

    pub fn reset(&mut self, entry: u32) {
        self.regs.swap(Mode::Svc);
        bf!((self.cpsr).mode = Mode::Svc as u32);
        bf!((self.cpsr).thumb_bit = 0b0);
        bf!((self.cpsr).disable_fiq_bit = 0b1);
        bf!((self.cpsr).disable_irq_bit = 0b1);

        self.regs[15] = entry + self.get_pc_offset();
    }

    pub fn get_pc_offset(&self) -> u32 {
        8 >> bf!((self.cpsr).thumb_bit)
        }

    pub fn get_coprocessor(&mut self, cp_index: usize) -> &mut coproc::Coprocessor {
        match cp_index {
            15 => &mut self.coproc_syscnt,
            _ => panic!("Tried to access unknown CP{}", cp_index),
        }
    }

    pub fn get_current_spsr(&mut self) -> &mut Psr {
        match Mode::from_num(bf!((self.cpsr).mode)) {
            Mode::Fiq => &mut self.spsr_fiq,
            Mode::Irq => &mut self.spsr_irq,
            Mode::Svc => &mut self.spsr_svc,
            Mode::Abt => &mut self.spsr_abt,
            Mode::Und => &mut self.spsr_und,
            _ => panic!("Attempted to access non-existent SPSR!"),
        }
    }

    pub fn spsr_make_current(&mut self) {
        self.cpsr = self.get_current_spsr().clone();
        self.regs.swap(cpu::Mode::from_num(bf!((self.cpsr).mode)));
    }

    #[inline(always)]
    pub fn branch(&mut self, addr: u32) {
        self.regs[15] = addr + self.get_pc_offset();
        // TODO: Invalidate pipeline once/if we have one
    }

    pub fn run(&mut self, num_instrs: u32) -> BreakReason {
        let mut cycles = self.cycles;
        let mut irq_known_pending = false;

        for _ in 0..num_instrs {
            let addr = self.regs[15] - self.get_pc_offset();

            cycles -= 1;
            // Amortize the cost of checking for IRQs, updating clock
            if cycles == 0 {
                self.sys_clk.increment(PAUSE_CYCLES * 8); // Probably speeds up time but w/e
                irq_known_pending = self.irq_line.is_high();
                cycles = PAUSE_CYCLES;
            }
            if irq_known_pending && bf!((self.cpsr).disable_irq_bit) == 0 && self.irq_line.is_high() {
                trace!("ARM9 IRQ triggered!");
                self.enter_exception(addr+4, Mode::Irq);
                irq_known_pending = false;
                continue
            }

            if self.find_toggle_breakpoint(addr) {
                return BreakReason::Breakpoint;
            }

            if bf!((self.cpsr).thumb_bit) == 0 {
                assert_eq!(addr & 0b11, 0);
                let instr = self.mpu.imem_read::<u32>(addr);
                let inst_fn = cpu::arm::decode(instr);
                cpu::arm::interpret(self, inst_fn, instr);
            } else {
                assert_eq!(addr & 0b1, 0);
                let instr = self.mpu.imem_read::<u16>(addr);
                let inst_fn = cpu::thumb::decode(instr);
                cpu::thumb::interpret(self, inst_fn, instr);
            }
        }

        self.cycles = cycles;
        BreakReason::LimitReached
    }

    pub fn enter_exception(&mut self, return_loc: u32, mode: Mode) {
        let r14_exc = return_loc;
        let spsr_exc = self.cpsr;

        self.regs.swap(mode);
        bf!((self.cpsr).mode = mode as u32);

        self.regs[14] = r14_exc;
        *self.get_current_spsr() = spsr_exc;
        bf!((self.cpsr).thumb_bit = 0);
        bf!((self.cpsr).disable_irq_bit = 1);

        // These vectors look like 0x080000XX because that's where the bootrom redirects them
        let vector_addr = match mode {
            Mode::Irq => 0x08000000,
            Mode::Fiq => unimplemented!(),
            Mode::Svc => 0x08000010,
            Mode::Und => 0x08000018,
            Mode::Abt => 0x08000028,
            Mode::Sys | Mode::Usr => panic!("No exception associated with {:?}", mode)
        };
        self.branch(vector_addr);
    }

    pub fn find_toggle_breakpoint(&mut self, addr: u32) -> bool {
        !self.breakpoints.is_empty() && self.breakpoints.remove(&addr)
    }
}
