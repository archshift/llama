use clock;
use cpu;
use cpu::InstrStatus;
use cpu::caches;
use cpu::coproc;
use cpu::irq;
use cpu::regs::{GpRegs, Psr};
use mem;

use utils::cache::TinyCache;

use std::collections::HashSet;

use arraydeque::{ArrayDeque, Wrapping};

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

#[allow(non_camel_case_types)] pub struct v5;
#[allow(non_camel_case_types)] pub struct v6;
pub trait Version: 'static {
    #[inline(always)]
    fn is<T: Version>() -> bool {
        use std::any::TypeId;
        TypeId::of::<T>() == TypeId::of::<Self>()
    }
}
impl Version for v5 {}
impl Version for v6 {}

pub struct Cpu<V: Version> {
    pub regs: GpRegs,
    pub cpsr: Psr::Bf,
    pub spsr_fiq: Psr::Bf,
    pub spsr_irq: Psr::Bf,
    pub spsr_svc: Psr::Bf,
    pub spsr_abt: Psr::Bf,
    pub spsr_und: Psr::Bf,

    coproc_syscnt: coproc::SysControl,
    pub mpu: caches::Mpu,

    irq_line: irq::IrqLine,
    cycles: usize,
    sys_clk: clock::SysClock,

    pub(crate) thumb_decode_cache: TinyCache<cpu::thumb::InstFn<V>, ()>,
    pub(crate) arm_decode_cache: TinyCache<cpu::arm::InstFn<V>, ()>,

    pub last_instructions: ArrayDeque<[u32; 1024], Wrapping>,

    pub breakpoints: HashSet<u32>, // addr, is_triggered

    _version: V
}

#[derive(Clone)]
pub enum BreakReason {
    LimitReached,
    Breakpoint,
    Trapped,
    WFI
}

const PAUSE_CYCLES: usize = 128;

impl<V: Version> Cpu<V> {
    pub fn new(version: V, memory: mem::MemController, irq_line: irq::IrqLine, clk: clock::SysClock) -> Cpu<V> {
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
 
            thumb_decode_cache: TinyCache::new(
                |_, k| cpu::thumb::decode(k as u16), |_, _, _| {}
            ),
            arm_decode_cache: TinyCache::new(
                |_, k| cpu::arm::decode(k), |_, _, _| {}
            ),

            last_instructions: ArrayDeque::new(),

            breakpoints: HashSet::new(),
            _version: version
        }
    }

    pub fn reset(&mut self, entry: u32) {
        self.regs.swap(Mode::Svc);
        self.cpsr.mode.set(Mode::Svc as u32);
        self.cpsr.thumb_bit.set(0b0);
        self.cpsr.disable_fiq_bit.set(0b1);
        self.cpsr.disable_irq_bit.set(0b1);

        self.regs[15] = entry + Self::pc_offset(0);
    }

    #[inline]
    fn instr_size(thumb_bit: u32) -> u32 {
        4 >> thumb_bit
    }

    #[inline]
    fn pc_offset(thumb_bit: u32) -> u32 {
        Self::instr_size(thumb_bit) * 2
    }

    // For external interface
    pub fn get_pc_offset(&self) -> u32 {
        let thumb_bit = self.cpsr.thumb_bit.get();
        Self::pc_offset(thumb_bit)
    }

    pub fn check_alignment(&self, thumb_bit: u32) {
        assert_eq!(self.regs[15] & (Self::instr_size(thumb_bit) - 1), 0);
    }

    pub fn get_coprocessor(&mut self, cp_index: usize) -> &mut coproc::Coprocessor<V> {
        match cp_index {
            15 => &mut self.coproc_syscnt,
            _ => panic!("Tried to access unknown CP{}", cp_index),
        }
    }

    pub fn get_current_spsr(&mut self) -> &mut Psr::Bf {
        match Mode::from_num(self.cpsr.mode.get()) {
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
        self.regs.swap(cpu::Mode::from_num(self.cpsr.mode.get()));
    }

    #[inline(always)]
    pub fn branch(&mut self, addr: u32) {
        let thumb_bit = self.cpsr.thumb_bit.get();
        self.regs[15] = addr + Self::pc_offset(thumb_bit);
        self.check_alignment(thumb_bit);
    }

    pub fn run(&mut self, num_instrs: u32) -> BreakReason {
        let mut cycles = self.cycles;
        let mut irq_known_pending = false;
        let mut thumb_bit = self.cpsr.thumb_bit.get();
        self.check_alignment(thumb_bit);

        for _ in 0..num_instrs {
            let addr = self.regs[15] - Self::pc_offset(thumb_bit);

            cycles -= 1;
            // Amortize the cost of checking for IRQs, updating clock
            if cycles == 0 {
                self.sys_clk.increment(PAUSE_CYCLES * 8); // Probably speeds up time but w/e
                irq_known_pending = self.irq_line.is_high();
                cycles = PAUSE_CYCLES;
            }
            if irq_known_pending && self.cpsr.disable_irq_bit.get() == 0 && self.irq_line.is_high() {
                trace!("ARM9 IRQ triggered!");
                self.enter_exception(addr+4, Mode::Irq);
                thumb_bit = 0;
                irq_known_pending = false;
                continue
            }

            if self.find_toggle_breakpoint(addr) {
                return BreakReason::Breakpoint;
            }

            self.last_instructions.push_back(addr);

            let status = if thumb_bit == 0 {
                cpu::arm::interpret_next(self, addr)
            } else {
                cpu::thumb::interpret_next(self, addr)
            };
            match status {
                InstrStatus::InBlock => self.regs[15] += Self::instr_size(thumb_bit),
                InstrStatus::Branched => thumb_bit = self.cpsr.thumb_bit.get(),
            }
        }

        self.cycles = cycles;
        BreakReason::LimitReached
    }

    pub fn enter_exception(&mut self, return_loc: u32, mode: Mode) {
        let r14_exc = return_loc;
        let spsr_exc = self.cpsr;

        self.regs.swap(mode);
        self.cpsr.mode.set(mode as u32);

        self.regs[14] = r14_exc;
        *self.get_current_spsr() = spsr_exc;
        self.cpsr.thumb_bit.set(0);
        self.cpsr.disable_irq_bit.set(1);

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
