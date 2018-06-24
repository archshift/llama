use cpu::{Cpu, InstrStatus};

pub type InstFn = fn(&mut Cpu, u16) -> InstrStatus;
mod interpreter {
    use cpu;
    pub use cpu::instructions_thumb::*;
    pub fn undef(cpu: &mut cpu::Cpu, instr: u16) -> cpu::InstrStatus {
        panic!("Unimplemented instruction! {:#X}: {:?}", cpu.regs[15] - cpu.get_pc_offset(), instr)
    }
    pub const bkpt: super::InstFn = undef;
    pub const cmn: super::InstFn = undef;
    pub const swi: super::InstFn = undef;
}

include!(concat!(env!("OUT_DIR"), "/thumb.decoder.rs"));

pub fn interpret(cpu: &mut Cpu, inst_fn: InstFn, inst: u16) {
    let status = inst_fn(cpu, inst);

    match status {
        InstrStatus::InBlock => cpu.regs[15] += 2,
        InstrStatus::Branched => {},
    }
}
