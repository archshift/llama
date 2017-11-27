mod sys_control;

use cpu;
pub use self::sys_control::*;

pub type CpEffect = Box<Fn(&mut cpu::Cpu)>;

pub trait Coprocessor {
    fn move_in(&mut self, cpreg1: usize, cpreg2: usize, op1: usize, op2: usize, val: u32) -> CpEffect;
    fn move_out(&mut self, cpreg1: usize, cpreg2: usize, op1: usize, op2: usize) -> u32;
}