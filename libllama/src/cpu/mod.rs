mod cpu;

#[macro_use]
mod define_inst;
pub mod decoder_arm;
pub mod decoder_thumb;

mod coproc;
mod interpreter_arm;
mod interpreter_thumb;

pub use self::cpu::*;
pub use self::interpreter_arm::*;
pub use self::interpreter_thumb::*;

pub mod dummy11;
pub mod instructions_arm;
pub mod instructions_thumb;
pub mod irq;
pub mod regs;