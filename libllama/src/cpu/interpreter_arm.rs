use cpu::Cpu;
use cpu::decoder_arm::ArmInstruction;
use cpu::instructions_arm;
use cpu::regs::Psr;

pub enum InstrStatus {
    InBlock, // Advance PC by instruction width
    Branched, // Do not advance PC
}

#[inline(always)]
pub fn cond_passed(cond_opcode: u32, cpsr: &Psr) -> bool {
    match cond_opcode {
        0b0000 => return bf!(cpsr.z_bit) == 1, // EQ
        0b0001 => return bf!(cpsr.z_bit) == 0, // NE
        0b0010 => return bf!(cpsr.c_bit) == 1, // CS
        0b0011 => return bf!(cpsr.c_bit) == 0, // CC
        0b0100 => return bf!(cpsr.n_bit) == 1, // MI
        0b0101 => return bf!(cpsr.n_bit) == 0, // PL
        0b0110 => return bf!(cpsr.v_bit) == 1, // VS
        0b0111 => return bf!(cpsr.v_bit) == 0, // VC
        0b1000 => { // HI
            return (bf!(cpsr.c_bit) == 1) && (bf!(cpsr.z_bit) == 0)
        },
        0b1001 => { // LS
            return (bf!(cpsr.c_bit) == 0) || (bf!(cpsr.z_bit) == 1)
        },
        0b1010 => { // GE
            return bf!(cpsr.n_bit) == bf!(cpsr.v_bit)
        },
        0b1011 => { // LT
            return bf!(cpsr.n_bit) != bf!(cpsr.v_bit)
        },
        0b1100 => { // GT
            return (bf!(cpsr.z_bit) == 0) &&
                (bf!(cpsr.n_bit) == bf!(cpsr.v_bit))
        },
        0b1101 => { // LE
            return (bf!(cpsr.z_bit) == 1) ||
                (bf!(cpsr.n_bit) != bf!(cpsr.v_bit))
        },
        0b1110 => return true, // AL
        _ => panic!("Unhandled condition code {:#b}!", cond_opcode),
    }
}

#[inline(always)]
pub fn interpret_arm(cpu: &mut Cpu, instr: ArmInstruction) {
    trace!("Instruction {:#X}: {:?}", cpu.regs[15] - cpu.get_pc_offset(), instr);

    let status = match instr {
        ArmInstruction::adc(data) => instructions_arm::adc(cpu, data),
        ArmInstruction::add(data) => instructions_arm::add(cpu, data),
        ArmInstruction::and(data) => instructions_arm::and(cpu, data),
        ArmInstruction::bic(data) => instructions_arm::bic(cpu, data),
        ArmInstruction::bbl(data) => instructions_arm::bbl(cpu, data),
        ArmInstruction::blx_2(data) => instructions_arm::blx(cpu, data),
        ArmInstruction::bx(data) => instructions_arm::bx(cpu, data),
        ArmInstruction::clz(data) => instructions_arm::clz(cpu, data),
        ArmInstruction::cmn(data) => instructions_arm::cmn(cpu, data),
        ArmInstruction::cmp(data) => instructions_arm::cmp(cpu, data),
        ArmInstruction::eor(data) => instructions_arm::eor(cpu, data),
        ArmInstruction::ldm_1(data) => instructions_arm::ldm_1(cpu, data),
        ArmInstruction::ldm_2(data) => instructions_arm::ldm_2(cpu, data),
        ArmInstruction::ldm_3(data) => instructions_arm::ldm_3(cpu, data),
        ArmInstruction::ldr(data) => instructions_arm::ldr(cpu, data),
        ArmInstruction::ldrb(data) => instructions_arm::ldrb(cpu, data),
        ArmInstruction::ldrd(data) => instructions_arm::ldrd(cpu, data),
        ArmInstruction::ldrh(data) => instructions_arm::ldrh(cpu, data),
        ArmInstruction::ldrsb(data) => instructions_arm::ldrsb(cpu, data),
        ArmInstruction::ldrsh(data) => instructions_arm::ldrsh(cpu, data),
        ArmInstruction::mcr(data) => InstrStatus::InBlock,
        ArmInstruction::mrc(data) => InstrStatus::InBlock,
        ArmInstruction::mov(data) => instructions_arm::mov(cpu, data),
        ArmInstruction::mrs(data) => instructions_arm::mrs(cpu, data),
        ArmInstruction::msr_1(data) => instructions_arm::msr_1(cpu, data),
        ArmInstruction::msr_2(data) => instructions_arm::msr_2(cpu, data),
        ArmInstruction::mul(data) => instructions_arm::mul(cpu, data),
        ArmInstruction::mvn(data) => instructions_arm::mvn(cpu, data),
        ArmInstruction::orr(data) => instructions_arm::orr(cpu, data),
        ArmInstruction::rsb(data) => instructions_arm::rsb(cpu, data),
        ArmInstruction::sbc(data) => instructions_arm::sbc(cpu, data),
        ArmInstruction::stm_1(data) => instructions_arm::stm_1(cpu, data),
        ArmInstruction::str(data) => instructions_arm::str(cpu, data),
        ArmInstruction::strb(data) => instructions_arm::strb(cpu, data),
        ArmInstruction::strd(data) => instructions_arm::strd(cpu, data),
        ArmInstruction::strh(data) => instructions_arm::strh(cpu, data),
        ArmInstruction::sub(data) => instructions_arm::sub(cpu, data),
        ArmInstruction::teq(data) => instructions_arm::teq(cpu, data),
        ArmInstruction::tst(data) => instructions_arm::tst(cpu, data),

        ArmInstruction::mod_blx(data) => instructions_arm::mod_blx(cpu, data),

        _ => panic!("Unimplemented instruction! {:#X}: {:?}", cpu.regs[15] - cpu.get_pc_offset(), instr)
    };

    match status {
        InstrStatus::InBlock => cpu.regs[15] += 4,
        InstrStatus::Branched => {},
    }
}
