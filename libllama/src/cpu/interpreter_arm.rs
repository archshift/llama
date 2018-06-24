use cpu::Cpu;
use cpu::regs::Psr;
use cpu::InstrStatus;

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

pub type InstFn = fn(&mut Cpu, u32) -> InstrStatus;

mod interpreter {
    use cpu;
    pub use cpu::instructions_arm::*;

    pub fn undef(cpu: &mut cpu::Cpu, instr: u32) -> cpu::InstrStatus {
        panic!("Unimplemented instruction! {:#X}: {:?}", cpu.regs[15] - cpu.get_pc_offset(), instr)
    }

    #[inline(always)]
    pub fn blx_2(cpu: &mut cpu::Cpu, instr: super::Blx2) -> cpu::InstrStatus {
        blx(cpu, instr)
    }
}

include!(concat!(env!("OUT_DIR"), "/arm.decoder.rs"));

pub fn interpret(cpu: &mut Cpu, inst_fn: InstFn, inst: u32) {
    let status = inst_fn(cpu, inst);

    match status {
        InstrStatus::InBlock => cpu.regs[15] += 4,
        InstrStatus::Branched => {},
    }
}
