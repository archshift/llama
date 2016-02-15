use cpu::{Cpu, ThumbInstruction, Psr};
use cpu::instructions_thumb;
use ram;

#[inline(always)]
pub fn interpret_thumb(cpu: &mut Cpu, mut ram: &mut ram::Ram, instr: ThumbInstruction) {
    println!("Instruction {:#X}: {:?}", cpu.regs[15] - cpu.get_pc_offset(), instr);

    let bytes_advanced = match instr {
        ThumbInstruction::LDR_1(data) => instructions_thumb::ldr_1(cpu, ram, data),
        ThumbInstruction::LDR_3(data) => instructions_thumb::ldr_3(cpu, ram, data),
        ThumbInstruction::LSL_1(data) => instructions_thumb::lsl_1(cpu, data),
        ThumbInstruction::MOV_1(data) => instructions_thumb::mov_1(cpu, data),
        ThumbInstruction::MOV_2(data) => instructions_thumb::mov_2(cpu, data),
        _ => {
            // println!("Unimplemented instruction! {:#X}: {:?}", cpu.regs[15] - cpu.get_pc_offset(), instr);
            2
        }
    };
    cpu.regs[15] += bytes_advanced;
}