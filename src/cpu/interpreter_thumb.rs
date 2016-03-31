use cpu::{Cpu, ThumbInstruction};
use cpu::instructions_thumb;

#[inline(always)]
pub fn interpret_thumb(cpu: &mut Cpu, instr: ThumbInstruction) {
    trace!("Instruction {:#X}: {:?}", cpu.regs[15] - cpu.get_pc_offset(), instr);

    let bytes_advanced = match instr {
        ThumbInstruction::AND(data) => instructions_thumb::and(cpu, data),
        ThumbInstruction::B_1(data) => instructions_thumb::b_1(cpu, data),
        ThumbInstruction::BIC(data) => instructions_thumb::bic(cpu, data),
        ThumbInstruction::BRANCH(data) => instructions_thumb::branch(cpu, data),
        ThumbInstruction::BX(data) => instructions_thumb::bx(cpu, data),
        ThumbInstruction::EOR(data) => instructions_thumb::eor(cpu, data),
        ThumbInstruction::LDR_1(data) => instructions_thumb::ldr_1(cpu, data),
        ThumbInstruction::LDR_3(data) => instructions_thumb::ldr_3(cpu, data),
        ThumbInstruction::LDRH_1(data) => instructions_thumb::ldrh_1(cpu, data),
        ThumbInstruction::LSL_1(data) => instructions_thumb::lsl_1(cpu, data),
        ThumbInstruction::MOV_1(data) => instructions_thumb::mov_1(cpu, data),
        ThumbInstruction::MOV_2(data) => instructions_thumb::mov_2(cpu, data),
        ThumbInstruction::MOV_3(data) => instructions_thumb::mov_3(cpu, data),
        ThumbInstruction::ORR(data) => instructions_thumb::orr(cpu, data),
        ThumbInstruction::POP(data) => instructions_thumb::pop(cpu, data),
        ThumbInstruction::PUSH(data) => instructions_thumb::push(cpu, data),
        ThumbInstruction::STR_1(data) => instructions_thumb::str_1(cpu, data),
        ThumbInstruction::STRH_1(data) => instructions_thumb::strh_1(cpu, data),
        ThumbInstruction::TST(data) => instructions_thumb::tst(cpu, data),
        _ => {
            warn!("Unimplemented instruction! {:#X}: {:?}", cpu.regs[15] - cpu.get_pc_offset(), instr);
            2
        }
    };
    cpu.regs[15] += bytes_advanced;
}
