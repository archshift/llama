mod sys_control;

use cpu;
pub use self::sys_control::*;

pub type CpEffect<V> = Box<dyn Fn(&mut cpu::Cpu<V>)>;

pub trait Coprocessor<V: cpu::Version> {
    fn move_in(&mut self, cpreg1: usize, cpreg2: usize, op1: usize, op2: usize, val: u32) -> CpEffect<V>;
    fn move_out(&mut self, cpreg1: usize, cpreg2: usize, op1: usize, op2: usize) -> u32;
}
