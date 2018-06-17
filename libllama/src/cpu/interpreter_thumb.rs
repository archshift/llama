use cpu::{Cpu, InstrStatus};
use cpu::decoder_thumb::InstFn;
use cpu::instructions_thumb;

pub fn interpret_thumb(cpu: &mut Cpu, inst_fn: InstFn, inst: u16) {
    #[cfg(feature = "trace_instructions")]
    trace!("Instruction {:#X}: {:?}", cpu.regs[15] - cpu.get_pc_offset(), instr);

    let status = inst_fn(cpu, inst);

    match status {
        InstrStatus::InBlock => cpu.regs[15] += 2,
        InstrStatus::Branched => {},
    }
}
