mod cpu;

mod caches;
mod coproc;
pub mod interpreter_arm;
pub mod interpreter_thumb;

pub use self::cpu::*;
pub use self::interpreter_arm as arm;
pub use self::interpreter_thumb as thumb;
pub use self::arm::cond_passed;

pub mod instructions_arm;
pub mod instructions_thumb;
pub mod irq;
pub mod regs;

pub enum InstrStatus {
    InBlock, // Advance PC by instruction width
    Branched, // Do not advance PC
}
