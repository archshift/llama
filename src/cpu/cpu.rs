use cpu;
use mem;
use system;

use std::sync::atomic::Ordering;

// Program status register
bitfield!(Psr: u32, {
    mode: 0 => 4,
    thumb_bit: 5 => 5,
    disable_fiq_bit: 6 => 6,
    disable_irq_bit: 7 => 7,
    q_bit: 27 => 27,
    v_bit: 28 => 28,
    c_bit: 29 => 29,
    z_bit: 30 => 30,
    n_bit: 31 => 31
});

pub struct Cpu {
    pub regs: [u32; 16],
    pub cpsr: Psr,
    pub spsr_fiq: Psr,
    pub spsr_irq: Psr,
    pub spsr_svc: Psr,
    pub spsr_abt: Psr,
    pub spsr_und: Psr,

    pub memory: mem::MemController,
}

pub enum BreakReason {
    LimitReached,
    Breakpoint
}

impl Cpu {
    pub fn new(memory: mem::MemController) -> Cpu {
        Cpu {
            regs: [0; 16],
            cpsr: Psr::new(0),
            spsr_fiq: Psr::new(0),
            spsr_irq: Psr::new(0),
            spsr_svc: Psr::new(0),
            spsr_abt: Psr::new(0),
            spsr_und: Psr::new(0),

            memory: memory,
        }
    }

    pub fn reset(&mut self, entry: u32) {
        self.regs[15] = entry + self.get_pc_offset();
        self.cpsr.set(Psr::mode(), 0b10011);
        self.cpsr.set(Psr::thumb_bit(), 0b0);
        self.cpsr.set(Psr::disable_fiq_bit(), 0b1);
        self.cpsr.set(Psr::disable_irq_bit(), 0b1);
    }

    pub fn get_pc_offset(&self) -> u32 {
        if self.cpsr.get(Psr::thumb_bit()) == 1 {
            4
        } else {
            8
        }
    }

    pub fn get_current_spsr(&mut self) -> &mut Psr {
        match self.cpsr.get(Psr::mode()) {
            0b10001 => &mut self.spsr_fiq,
            0b10010 => &mut self.spsr_irq,
            0b10011 => &mut self.spsr_svc,
            0b10111 => &mut self.spsr_abt,
            0b11011 => &mut self.spsr_und,
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
        for _ in num_instrs..0 {
            let addr = self.regs[15] - self.get_pc_offset();

            if self.cpsr.get(Psr::thumb_bit()) == 0 {
                let instr = cpu::decode_arm_instruction(self.memory.read::<u32>(addr));
                cpu::interpret_arm(self, instr);
            } else {
                let instr = cpu::decode_thumb_instruction(self.memory.read::<u16>(addr));
                cpu::interpret_thumb(self, instr);
            }
        }
        
        BreakReason::LimitReached
    }
}
