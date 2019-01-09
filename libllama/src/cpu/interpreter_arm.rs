use cpu::{Cpu, Version};
use cpu::regs::Psr;
use cpu::InstrStatus;

pub fn cond_passed(cond_opcode: u32, cpsr: &Psr::Bf) -> bool {
    match cond_opcode {
        0b0000 => return cpsr.z_bit.get() == 1, // EQ
        0b0001 => return cpsr.z_bit.get() == 0, // NE
        0b0010 => return cpsr.c_bit.get() == 1, // CS
        0b0011 => return cpsr.c_bit.get() == 0, // CC
        0b0100 => return cpsr.n_bit.get() == 1, // MI
        0b0101 => return cpsr.n_bit.get() == 0, // PL
        0b0110 => return cpsr.v_bit.get() == 1, // VS
        0b0111 => return cpsr.v_bit.get() == 0, // VC
        0b1000 => { // HI
            return (cpsr.c_bit.get() == 1) && (cpsr.z_bit.get() == 0)
        },
        0b1001 => { // LS
            return (cpsr.c_bit.get() == 0) || (cpsr.z_bit.get() == 1)
        },
        0b1010 => { // GE
            return cpsr.n_bit.get() == cpsr.v_bit.get()
        },
        0b1011 => { // LT
            return cpsr.n_bit.get() != cpsr.v_bit.get()
        },
        0b1100 => { // GT
            return (cpsr.z_bit.get() == 0) &&
                (cpsr.n_bit.get() == cpsr.v_bit.get())
        },
        0b1101 => { // LE
            return (cpsr.z_bit.get() == 1) ||
                (cpsr.n_bit.get() != cpsr.v_bit.get())
        },
        0b1110 => return true, // AL
        _ => panic!("Unhandled condition code {:#b}!", cond_opcode),
    }
}

pub type InstFn<V> = fn(&mut Cpu<V>, u32) -> InstrStatus;

mod interpreter {
    use cpu;
    pub use cpu::instructions_arm::*;

    pub fn undef<V: cpu::Version>(cpu: &mut cpu::Cpu<V>, instr: u32) -> cpu::InstrStatus {
        panic!("Unimplemented instruction! {:#X}: {:#08X}", cpu.regs[15] - cpu.get_pc_offset(), instr)
    }

    #[inline(always)]
    pub fn blx_2<V: cpu::Version>(cpu: &mut cpu::Cpu<V>, instr: super::Blx2::Bf) -> cpu::InstrStatus {
        blx(cpu, instr)
    }
}

include!(concat!(env!("OUT_DIR"), "/arm.decoder.rs"));

#[inline]
pub fn interpret_next<V: Version>(cpu: &mut Cpu<V>, addr: u32) -> InstrStatus {
    let instr = cpu.mpu.imem_read::<u32>(addr);
    let inst_fn = *cpu.arm_decode_cache.get_or(instr, &mut ());
    
    trace!("ARM{:?} @ {:08X}: {} ({:08X})", cpu._version, addr, ::cpu::arm::disasm(instr), instr);
    inst_fn(cpu, instr)
}
