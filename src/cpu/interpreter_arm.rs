use cpu::{Cpu, ArmInstruction, Psr};
use cpu::instructions_arm;

#[inline(always)]
pub fn cond_passed(cond_opcode: u32, cpsr: &Psr) -> bool {
    match cond_opcode {
        0b0000 => return cpsr.get(Psr::z_bit()) == 1, // EQ
        0b0001 => return cpsr.get(Psr::z_bit()) == 0, // NE
        0b0010 => return cpsr.get(Psr::c_bit()) == 1, // CS
        0b0011 => return cpsr.get(Psr::c_bit()) == 0, // CC
        0b0100 => return cpsr.get(Psr::n_bit()) == 1, // MI
        0b0101 => return cpsr.get(Psr::n_bit()) == 0, // PL
        0b0110 => return cpsr.get(Psr::v_bit()) == 1, // VS
        0b0111 => return cpsr.get(Psr::v_bit()) == 0, // VC
        0b1000 => { // HI
            return (cpsr.get(Psr::c_bit()) == 1) &&
                (cpsr.get(Psr::z_bit()) == 0)
        },
        0b1001 => { // LS
            return (cpsr.get(Psr::c_bit()) == 0) ||
                (cpsr.get(Psr::z_bit()) == 1)
        },
        0b1010 => { // GE
            return cpsr.get(Psr::n_bit()) ==
                cpsr.get(Psr::v_bit())
        },
        0b1011 => { // LT
            return cpsr.get(Psr::n_bit()) !=
                cpsr.get(Psr::v_bit())
        },
        0b1100 => { // GT
            return (cpsr.get(Psr::z_bit()) == 0) &&
                (cpsr.get(Psr::n_bit()) ==
                 cpsr.get(Psr::v_bit()))
        },
        0b1101 => { // LE
            return (cpsr.get(Psr::z_bit()) == 1) ||
                (cpsr.get(Psr::n_bit()) !=
                cpsr.get(Psr::v_bit()))
        },
        0b1110 => return true, // AL
        _ => panic!("Unhandled condition code {:#b}!", cond_opcode),
    }
}

#[inline(always)]
pub fn interpret_arm(cpu: &mut Cpu, instr: ArmInstruction) {
    trace!("Instruction {:#X}: {:?}", cpu.regs[15] - cpu.get_pc_offset(), instr);

    let bytes_advanced = match instr {
        ArmInstruction::ADD(data) => instructions_arm::add(cpu, data),
        ArmInstruction::AND(data) => instructions_arm::and(cpu, data),
        ArmInstruction::BIC(data) => instructions_arm::bic(cpu, data),
        ArmInstruction::B_BL(data) => instructions_arm::bbl(cpu, data),
        ArmInstruction::BLX(data) => instructions_arm::blx(cpu, data),
        ArmInstruction::BX(data) => instructions_arm::bx(cpu, data),
        ArmInstruction::CMN(data) => instructions_arm::cmn(cpu, data),
        ArmInstruction::CMP(data) => instructions_arm::cmp(cpu, data),
        ArmInstruction::EOR(data) => instructions_arm::eor(cpu, data),
        ArmInstruction::LDM(data) => instructions_arm::ldm(cpu, data),
        ArmInstruction::LDR(data) => instructions_arm::ldr(cpu, data),
        ArmInstruction::LDRB(data) => instructions_arm::ldrb(cpu, data),
        ArmInstruction::MOV(data) => instructions_arm::mov(cpu, data),
        ArmInstruction::MRS(data) => instructions_arm::mrs(cpu, data),
        ArmInstruction::MSR(data) => instructions_arm::msr(cpu, data),
        ArmInstruction::MVN(data) => instructions_arm::mvn(cpu, data),
        ArmInstruction::ORR(data) => instructions_arm::orr(cpu, data),
        ArmInstruction::RSB(data) => instructions_arm::rsb(cpu, data),
        ArmInstruction::STM(data) => instructions_arm::stm(cpu, data),
        ArmInstruction::STR(data) => instructions_arm::str(cpu, data),
        ArmInstruction::STRB(data) => instructions_arm::strb(cpu, data),
        ArmInstruction::SUB(data) => instructions_arm::sub(cpu, data),
        ArmInstruction::TEQ(data) => instructions_arm::teq(cpu, data),
        ArmInstruction::TST(data) => instructions_arm::tst(cpu, data),

        ArmInstruction::MOD_BLX(data) => instructions_arm::mod_blx(cpu, data),

        _ => {
            warn!("Unimplemented instruction! {:#X}: {:?}", cpu.regs[15] - cpu.get_pc_offset(), instr);
            4
        }
    };
    cpu.regs[15] += bytes_advanced;
}
