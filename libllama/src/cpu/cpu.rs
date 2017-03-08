use cpu;
use cpu::regs::{GpRegs, Psr};
use mem;

use std::collections::HashMap;

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

    pub memory: mem::MemController,
    pub breakpoints: HashMap<u32, bool> // addr, is_triggered
}

pub enum BreakReason {
    LimitReached,
    Breakpoint
}

impl Cpu {
    pub fn new(memory: mem::MemController) -> Cpu {
        Cpu {
            regs: GpRegs::new(Mode::Svc),
            cpsr: Psr::new(0),
            spsr_fiq: Psr::new(0),
            spsr_irq: Psr::new(0),
            spsr_svc: Psr::new(0),
            spsr_abt: Psr::new(0),
            spsr_und: Psr::new(0),

            memory: memory,
            breakpoints: HashMap::new()
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
        if bf!((self.cpsr).thumb_bit) == 1 {
            4
        } else {
            8
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
    }

    #[inline(always)]
    pub fn branch(&mut self, addr: u32) {
        self.regs[15] = addr + self.get_pc_offset();
        // TODO: Invalidate pipeline once/if we have one
    }

    pub fn run(&mut self, num_instrs: u32) -> BreakReason {
        use cpu::decoder_arm::ArmInstruction;
        use cpu::decoder_thumb::ThumbInstruction;

        for _ in 0..num_instrs {
            let addr = self.regs[15] - self.get_pc_offset();
            if self.find_toggle_breakpoint(addr) {
                return BreakReason::Breakpoint;
            }

            if bf!((self.cpsr).thumb_bit) == 0 {
                assert_eq!(addr & 0b11, 0);
                let instr = ArmInstruction::decode(self.memory.read::<u32>(addr));
                cpu::interpret_arm(self, instr);
            } else {
                assert_eq!(addr & 0b1, 0);
                let instr = ThumbInstruction::decode(self.memory.read::<u16>(addr));
                cpu::interpret_thumb(self, instr);
            }
        }

        BreakReason::LimitReached
    }

    pub fn find_toggle_breakpoint(&mut self, addr: u32) -> bool {
        if let Some(triggered) = self.breakpoints.get_mut(&addr) {
            *triggered ^= true;
            *triggered
        } else {
            false
        }
    }
}
