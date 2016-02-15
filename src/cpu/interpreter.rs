use cpu::{Cpu, ArmInstruction, ThumbInstruction, Psr};
use cpu::instructions;
use ram;

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
pub fn interpret_arm(cpu: &mut Cpu, mut ram: &mut ram::Ram, instr: ArmInstruction) {
    println!("Instruction {:#X}: {:?}", cpu.regs[15] - cpu.get_pc_offset(), instr);

    let bytes_advanced = match instr {
        ArmInstruction::ADD(data) => instructions::add(cpu, data),
        ArmInstruction::AND(data) => instructions::and(cpu, data),
        ArmInstruction::BIC(data) => instructions::bic(cpu, data),
        ArmInstruction::B_BL(data) => instructions::bbl(cpu, data),
        ArmInstruction::BLX(data) => instructions::blx(cpu, data),
        ArmInstruction::BX(data) => instructions::bx(cpu, data),
        ArmInstruction::CMN(data) => instructions::cmn(cpu, data),
        ArmInstruction::CMP(data) => instructions::cmp(cpu, data),
        ArmInstruction::EOR(data) => instructions::eor(cpu, data),
        ArmInstruction::LDM(data) => instructions::ldm(cpu, ram, data),
        ArmInstruction::LDR(data) => instructions::ldr(cpu, ram, data),
        ArmInstruction::LDRB(data) => instructions::ldrb(cpu, ram, data),
        ArmInstruction::MOV(data) => instructions::mov(cpu, data),
        ArmInstruction::MRS(data) => instructions::mrs(cpu, data),
        ArmInstruction::MVN(data) => instructions::mvn(cpu, data),
        ArmInstruction::ORR(data) => instructions::orr(cpu, data),
        ArmInstruction::RSB(data) => instructions::rsb(cpu, data),
        ArmInstruction::STM(data) => instructions::stm(cpu, ram, data),
        ArmInstruction::STR(data) => instructions::str(cpu, ram, data),
        ArmInstruction::STRB(data) => instructions::strb(cpu, ram, data),
        ArmInstruction::SUB(data) => instructions::sub(cpu, data),
        ArmInstruction::TEQ(data) => instructions::teq(cpu, data),
        ArmInstruction::TST(data) => instructions::tst(cpu, data),

        ArmInstruction::MOD_BLX(data) => instructions::mod_blx(cpu, data),

        _ => {
            // println!("Unimplemented instruction! {:#X}: {:?}", cpu.regs[15] - cpu.get_pc_offset(), instr);
            4
        }
    };
    cpu.regs[15] += bytes_advanced;
}

pub fn interpret_thumb(cpu: &mut Cpu, instr: ThumbInstruction) {

}
