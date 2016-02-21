use cpu;
use cpu::Cpu;

#[inline(always)]
pub fn mrs(cpu: &mut Cpu, data: cpu::ArmInstrMoveStatusReg) -> u32 {
    use cpu::ArmInstrMoveStatusReg as ArmInstr;

    if !cpu::cond_passed(data.get(ArmInstr::cond()), &cpu.cpsr) {
        return 4;
    }

    let rd = data.get(ArmInstr::rd());
    let r_bit = data.get(ArmInstr::r_bit());

    if r_bit == 1 {
        cpu.regs[rd as usize] = cpu.get_current_spsr().raw();
    } else {
        cpu.regs[rd as usize] = cpu.cpsr.raw();
    }

    4
}

#[inline(always)]
pub fn msr(cpu: &mut Cpu, data: cpu::ArmInstrMoveStatusReg) -> u32 {
    use cpu::ArmInstrMoveStatusReg as ArmInstr;

    if !cpu::cond_passed(data.get(ArmInstr::cond()), &cpu.cpsr) {
        return 4;
    }

    let field_mask = data.get(ArmInstr::field_mask());
    let shifter_operand = data.get(ArmInstr::shifter_operand());

    let val = if data.get(ArmInstr::i_bit()) == 1 {
        let immed_8 = bits!(shifter_operand, 0 => 7);
        let rotate_imm = bits!(shifter_operand, 8 => 11);
        immed_8.rotate_right(rotate_imm * 2)
    } else {
        cpu.regs[bits!(shifter_operand, 0 => 3) as usize]
    };

    let unalloc_mask = 0x07FFFF00u32;
    let user_mask    = 0xF8000000u32;
    let priv_mask    = 0x0000000Fu32;
    let state_mask   = 0x00000020u32;

    if val & unalloc_mask != 0 {
        panic!("Attempted to set reserved bits!");
    }

    let mut byte_mask = 0u32;
    byte_mask |= if bit!(field_mask, 0) == 1 { 0x000000FF } else { 0 };
    byte_mask |= if bit!(field_mask, 1) == 1 { 0x0000FF00 } else { 0 };
    byte_mask |= if bit!(field_mask, 2) == 1 { 0x00FF0000 } else { 0 };
    byte_mask |= if bit!(field_mask, 3) == 1 { 0xFF000000 } else { 0 };

    if data.get(ArmInstr::r_bit()) == 0 {
        // CPSR
        // TODO: Check privileges
        let cleared_cpsr = cpu.cpsr.raw() & !byte_mask;
        cpu.cpsr.set_raw(cleared_cpsr | (val & byte_mask))
    } else {
        // SPSR
        let spsr = cpu.get_current_spsr();
        byte_mask &= user_mask | priv_mask | state_mask;

        let cleared_spsr = spsr.raw() & !byte_mask;
        spsr.set_raw(cleared_spsr | (val & byte_mask))
    }

    4
}
