use cpu::{Cpu, InstrStatus};
use cpu::decoder_thumb::ThumbInstruction;
use cpu::instructions_thumb;

#[inline(always)]
pub fn interpret_thumb(cpu: &mut Cpu, instr: ThumbInstruction) {
    trace!("Instruction {:#X}: {:?}", cpu.regs[15] - cpu.get_pc_offset(), instr);

    let status = match instr {
        ThumbInstruction::adc(data) => instructions_thumb::adc(cpu, data),
        ThumbInstruction::add_1(data) => instructions_thumb::add_1(cpu, data),
        ThumbInstruction::add_2(data) => instructions_thumb::add_2(cpu, data),
        ThumbInstruction::add_3(data) => instructions_thumb::add_3(cpu, data),
        ThumbInstruction::add_4(data) => instructions_thumb::add_4(cpu, data),
        ThumbInstruction::add_5(data) => instructions_thumb::add_5(cpu, data),
        ThumbInstruction::add_6(data) => instructions_thumb::add_6(cpu, data),
        ThumbInstruction::add_7(data) => instructions_thumb::add_7(cpu, data),
        ThumbInstruction::and(data) => instructions_thumb::and(cpu, data),
        ThumbInstruction::asr_1(data) => instructions_thumb::asr_1(cpu, data),
        ThumbInstruction::b_1(data) => instructions_thumb::b_1(cpu, data),
        ThumbInstruction::bic(data) => instructions_thumb::bic(cpu, data),
        ThumbInstruction::branch(data) => instructions_thumb::branch(cpu, data),
        ThumbInstruction::bx(data) => instructions_thumb::bx(cpu, data),
        ThumbInstruction::cmp_1(data) => instructions_thumb::cmp_1(cpu, data),
        ThumbInstruction::cmp_2(data) => instructions_thumb::cmp_2(cpu, data),
        ThumbInstruction::cmp_3(data) => instructions_thumb::cmp_3(cpu, data),
        ThumbInstruction::eor(data) => instructions_thumb::eor(cpu, data),
        ThumbInstruction::ldr_1(data) => instructions_thumb::ldr_1(cpu, data),
        ThumbInstruction::ldr_2(data) => instructions_thumb::ldr_2(cpu, data),
        ThumbInstruction::ldr_3(data) => instructions_thumb::ldr_3(cpu, data),
        ThumbInstruction::ldr_4(data) => instructions_thumb::ldr_4(cpu, data),
        ThumbInstruction::ldrb_1(data) => instructions_thumb::ldrb_1(cpu, data),
        ThumbInstruction::ldrb_2(data) => instructions_thumb::ldrb_2(cpu, data),
        ThumbInstruction::ldrh_1(data) => instructions_thumb::ldrh_1(cpu, data),
        ThumbInstruction::ldrh_2(data) => instructions_thumb::ldrh_2(cpu, data),
        ThumbInstruction::lsl_1(data) => instructions_thumb::lsl_1(cpu, data),
        ThumbInstruction::lsl_2(data) => instructions_thumb::lsl_2(cpu, data),
        ThumbInstruction::lsr_1(data) => instructions_thumb::lsr_1(cpu, data),
        ThumbInstruction::mov_1(data) => instructions_thumb::mov_1(cpu, data),
        ThumbInstruction::mov_2(data) => instructions_thumb::mov_2(cpu, data),
        ThumbInstruction::mov_3(data) => instructions_thumb::mov_3(cpu, data),
        ThumbInstruction::mul(data) => instructions_thumb::mul(cpu, data),
        ThumbInstruction::mvn(data) => instructions_thumb::mvn(cpu, data),
        ThumbInstruction::neg(data) => instructions_thumb::neg(cpu, data),
        ThumbInstruction::orr(data) => instructions_thumb::orr(cpu, data),
        ThumbInstruction::pop(data) => instructions_thumb::pop(cpu, data),
        ThumbInstruction::push(data) => instructions_thumb::push(cpu, data),
        ThumbInstruction::sub_1(data) => instructions_thumb::sub_1(cpu, data),
        ThumbInstruction::sub_2(data) => instructions_thumb::sub_2(cpu, data),
        ThumbInstruction::sub_3(data) => instructions_thumb::sub_3(cpu, data),
        ThumbInstruction::sub_4(data) => instructions_thumb::sub_4(cpu, data),
        ThumbInstruction::sbc(data) => instructions_thumb::sbc(cpu, data),
        ThumbInstruction::str_1(data) => instructions_thumb::str_1(cpu, data),
        ThumbInstruction::str_2(data) => instructions_thumb::str_2(cpu, data),
        ThumbInstruction::str_3(data) => instructions_thumb::str_3(cpu, data),
        ThumbInstruction::strb_1(data) => instructions_thumb::strb_1(cpu, data),
        ThumbInstruction::strb_2(data) => instructions_thumb::strb_2(cpu, data),
        ThumbInstruction::strh_1(data) => instructions_thumb::strh_1(cpu, data),
        ThumbInstruction::strh_2(data) => instructions_thumb::strh_2(cpu, data),
        ThumbInstruction::tst(data) => instructions_thumb::tst(cpu, data),
        _ => panic!("Unimplemented instruction! {:#X}: {:?}", cpu.regs[15] - cpu.get_pc_offset(), instr)
    };

    match status {
        InstrStatus::InBlock => cpu.regs[15] += 2,
        InstrStatus::Branched => {},
    }
}
