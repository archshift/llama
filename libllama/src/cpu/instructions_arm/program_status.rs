use cpu;
use cpu::Cpu;
use cpu::decoder_arm as arm;

pub fn mrs(cpu: &mut Cpu, data: arm::mrs::InstrDesc) -> cpu::InstrStatus {
    if !cpu::cond_passed(bf!(data.cond), &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    let rd = bf!(data.rd);
    let r_bit = bf!(data.r_bit);

    if r_bit == 1 {
        cpu.regs[rd as usize] = cpu.get_current_spsr().raw();
    } else {
        cpu.regs[rd as usize] = cpu.cpsr.raw();
    }

    cpu::InstrStatus::InBlock
}

pub fn instr_msr(cpu: &mut Cpu, data: arm::msr_1::InstrDesc, immediate: bool) -> cpu::InstrStatus {
    if !cpu::cond_passed(bf!(data.cond), &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    let field_mask = bf!(data.field_mask);
    let shifter_operand = bf!(data.shifter_operand);

    let val = if immediate {
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

    if bf!(data.r_bit) == 0 {
        // CPSR
        // TODO: Check privileges
        let cleared_cpsr = cpu.cpsr.raw() & !byte_mask;
        cpu.cpsr.set_raw(cleared_cpsr | (val & byte_mask));

        if bit!(field_mask, 0) == 1 {
            // CPU mode may have been changed
            cpu.regs.swap(cpu::Mode::from_num(bf!((cpu.cpsr).mode)));
        }
    } else {
        // SPSR
        let spsr = cpu.get_current_spsr();
        byte_mask &= user_mask | priv_mask | state_mask;

        let cleared_spsr = spsr.raw() & !byte_mask;
        spsr.set_raw(cleared_spsr | (val & byte_mask));
    }

    cpu::InstrStatus::InBlock
}

pub fn msr_1(cpu: &mut Cpu, data: arm::msr_1::InstrDesc) -> cpu::InstrStatus {
    instr_msr(cpu, data, true)
}

pub fn msr_2(cpu: &mut Cpu, data: arm::msr_2::InstrDesc) -> cpu::InstrStatus {
    instr_msr(cpu, arm::msr_1::InstrDesc::new(data.raw()), false)
}