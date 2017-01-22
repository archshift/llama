use cpu::{Cpu, InstrStatus, ThumbInstruction};
use cpu::instructions_thumb;

#[inline(always)]
pub fn interpret_thumb(cpu: &mut Cpu, instr: ThumbInstruction) {
    trace!("Instruction {:#X}: {:?}", cpu.regs[15] - cpu.get_pc_offset(), instr);

    let status = match instr {
        ThumbInstruction::AND(data) => instructions_thumb::and(cpu, data),
        ThumbInstruction::B_1(data) => instructions_thumb::b_1(cpu, data),
        ThumbInstruction::BIC(data) => instructions_thumb::bic(cpu, data),
        ThumbInstruction::BRANCH(data) => instructions_thumb::branch(cpu, data),
        ThumbInstruction::BX(data) => instructions_thumb::bx(cpu, data),
        ThumbInstruction::CMP_1(data) => instructions_thumb::cmp_1(cpu, data),
        ThumbInstruction::CMP_3(data) => instructions_thumb::cmp_3(cpu, data),
        ThumbInstruction::EOR(data) => instructions_thumb::eor(cpu, data),
        ThumbInstruction::LDR_1(data) => instructions_thumb::ldr_1(cpu, data),
        ThumbInstruction::LDR_3(data) => instructions_thumb::ldr_3(cpu, data),
        ThumbInstruction::LDRH_1(data) => instructions_thumb::ldrh_1(cpu, data),
        ThumbInstruction::LSL_1(data) => instructions_thumb::lsl_1(cpu, data),
        ThumbInstruction::LSR_1(data) => instructions_thumb::lsr_1(cpu, data),
        ThumbInstruction::MOV_1(data) => instructions_thumb::mov_1(cpu, data),
        ThumbInstruction::MOV_2(data) => instructions_thumb::mov_2(cpu, data),
        ThumbInstruction::MOV_3(data) => instructions_thumb::mov_3(cpu, data),
        ThumbInstruction::ORR(data) => instructions_thumb::orr(cpu, data),
        ThumbInstruction::POP(data) => instructions_thumb::pop(cpu, data),
        ThumbInstruction::PUSH(data) => instructions_thumb::push(cpu, data),
        ThumbInstruction::SUB_1(data) => instructions_thumb::sub_1(cpu, data),
        ThumbInstruction::SBC(data) => instructions_thumb::sbc(cpu, data),
        ThumbInstruction::STR_1(data) => instructions_thumb::str_1(cpu, data),
        ThumbInstruction::STRH_1(data) => instructions_thumb::strh_1(cpu, data),
        ThumbInstruction::TST(data) => instructions_thumb::tst(cpu, data),
        _ => {
            warn!("Unimplemented instruction! {:#X}: {:?}", cpu.regs[15] - cpu.get_pc_offset(), instr);
            InstrStatus::InBlock
        }
    };

    match status {
        InstrStatus::InBlock => cpu.regs[15] += 2,
        InstrStatus::Branched => {},
    }
}
