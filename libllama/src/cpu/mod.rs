mod cpu;

mod caches;
mod coproc;
#[macro_use] mod interpreter_dummy11;
pub mod interpreter_arm;
pub mod interpreter_thumb;

pub use self::cpu::*;
pub use self::interpreter_arm as arm;
pub use self::interpreter_thumb as thumb;
pub use self::arm::cond_passed;

pub mod dummy11;
pub mod instructions_arm;
pub mod instructions_thumb;
pub mod irq;
pub mod regs;

pub enum InstrStatus {
    InBlock, // Advance PC by instruction width
    Branched, // Do not advance PC
}
