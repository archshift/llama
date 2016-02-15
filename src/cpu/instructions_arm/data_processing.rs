use cpu;
use cpu::Cpu;

#[inline(always)]
fn get_shifter_val(instr_data: &cpu::ArmInstrDProc, cpu: &Cpu) -> (u32, bool) {
    use cpu::ArmInstrDProc as ArmInstr;

    let shifter_bits = instr_data.get(ArmInstr::shifter_operand());
    let c_bit = cpu.cpsr.get(cpu::Psr::c_bit()) == 1;

    if instr_data.get(ArmInstr::i_bit()) == 1 {
        let immed_8 = bits!(shifter_bits, 0 => 7);
        let rotate_imm = bits!(shifter_bits, 8 => 11);

        let res = immed_8.rotate_right(rotate_imm * 2);
        if rotate_imm == 0 {
            return (res, c_bit);
        } else {
            return (res, bit!(res, 31) == 1);
        }
    }

    let pre_shift: u32 = cpu.regs[bits!(shifter_bits, 0 => 3) as usize];
    let amount = if bit!(shifter_bits, 4) == 0 {
        bits!(shifter_bits, 7 => 11) as u8
    } else {
        cpu.regs[bits!(shifter_bits, 8 => 11) as usize] as u8
    };

    match bits!(shifter_bits, 4 => 6) {
        0b000 | 0b001 => { // LSL
            if amount == 0 {
                return (pre_shift, c_bit)
            } else if amount < 32 {
                let res = pre_shift << amount;
                return (res, bit!(pre_shift, (32 - amount) as usize) == 1)
            } else if amount == 32 {
                return (0, bit!(pre_shift, 0) == 1)
            } else {
                return (0, false)
            }
        },
        0b010 | 0b011 => { // LSR
            if amount == 0 {
                return (pre_shift, c_bit)
            } else if amount < 32 {
                let res = pre_shift >> amount;
                return (res, bit!(pre_shift, (amount - 1) as usize) == 1)
            } else if amount == 32 {
                return (0, bit!(pre_shift, 31) == 1)
            } else {
                return (0, false)
            }
        },
        0b100 => { // ASR immedate
            if amount == 0 {
                if bit!(pre_shift, 31) == 0 {
                    return (0, false)
                } else {
                    return (0xFFFFFFFF, true)
                }
            } else {
                let res = ((pre_shift as i32) >> amount) as u32;
                return (res, bit!(pre_shift, (amount - 1) as usize) == 1)
            }
        },
        0b101 => { // ASR register
            if amount == 0 {
                return (pre_shift, c_bit)
            } else if amount < 32 {
                let res = ((pre_shift as i32) >> amount) as u32;
                return (res, bit!(pre_shift, (amount - 1) as usize) == 1)
            } else {
                if bit!(pre_shift, 31) == 0 {
                    return (0, false)
                } else {
                    return (0xFFFFFFFF, true)
                }
            }
        },
        0b110 => { // ROR immediate
            if amount == 0 {
                let res = ((c_bit as u32) << 31) | (pre_shift >> 1);
                return (res, bit!(pre_shift, 0) == 1)
            } else {
                let res = pre_shift.rotate_right(amount as u32);
                return (res, bit!(pre_shift, (amount - 1) as usize) == 1)
            }
        },
        0b111 => { // ROR register
            if amount == 0 {
                return (pre_shift, c_bit)
            } else if amount & 0xF == 0 {
                return (pre_shift, bit!(pre_shift, 31) == 1)
            } else {
                let amount = amount & 0xF;
                let res = pre_shift.rotate_right(amount as u32);
                return (res, bit!(pre_shift, (amount - 1) as usize) == 1)
            }
        }
        _ => {
            panic!("Unhandled shifter operation!");
        }
    }
}

enum ProcessInstrBitOp {
    AND,
    AND_NOT,
    OR,
    XOR,
}

#[inline(always)]
fn instr_bitwise(cpu: &mut Cpu, data: cpu::ArmInstrDProc, op: ProcessInstrBitOp) -> u32 {
    use cpu::ArmInstrDProc as ArmInstr;

    if !cpu::cond_passed(data.get(ArmInstr::cond()), &cpu.cpsr) {
        return 4;
    }

    let dst_reg = data.get(ArmInstr::rd());
    let s_bit = data.get(ArmInstr::s_bit()) == 1;
    let (shifter_val, shifter_carry) = get_shifter_val(&data, cpu);
    let rn = data.get(ArmInstr::rn());

    let val = match op {
        ProcessInstrBitOp::AND => cpu.regs[rn as usize] & shifter_val,
        ProcessInstrBitOp::AND_NOT => cpu.regs[rn as usize] & !shifter_val,
        ProcessInstrBitOp::OR => cpu.regs[rn as usize] | shifter_val,
        ProcessInstrBitOp::XOR => cpu.regs[rn as usize] ^ shifter_val,
    };

    if s_bit {
        if dst_reg == 15 {
            cpu.spsr_make_current();
        } else {
            cpu.cpsr.set(cpu::Psr::n_bit(), bit!(val, 31));
            cpu.cpsr.set(cpu::Psr::z_bit(), (val == 0) as u32);
            cpu.cpsr.set(cpu::Psr::c_bit(), shifter_carry as u32);
        }
    }

    if dst_reg == 15 {
        cpu.branch(val);
        return 0;
    } else {
        cpu.regs[dst_reg as usize] = val;
        return 4;
    }
}

#[inline(always)]
fn instr_compare(cpu: &mut Cpu, data: cpu::ArmInstrDProc, negative: bool) -> u32 {
    use cpu::ArmInstrDProc as ArmInstr;

    if !cpu::cond_passed(data.get(ArmInstr::cond()), &cpu.cpsr) {
        return 4;
    }

    let base_val = cpu.regs[data.get(ArmInstr::rn()) as usize];
    let (shifter_val, _) = get_shifter_val(&data, cpu);

    let (val, carry_bit, overflow_bit) = if negative {
        let val = base_val - shifter_val;
        let u_overflow = base_val.checked_sub(shifter_val).is_none();
        let s_overflow = (base_val as i32).checked_sub(shifter_val as i32).is_none();
        (val, !u_overflow, s_overflow)
    } else {
        let val = base_val + shifter_val;
        let u_overflow = base_val.checked_add(shifter_val).is_none();
        let s_overflow = (base_val as i32).checked_add(shifter_val as i32).is_none();
        (val, u_overflow, s_overflow)
    };

    cpu.cpsr.set(cpu::Psr::n_bit(), bit!(val, 31));
    cpu.cpsr.set(cpu::Psr::z_bit(), (val == 0) as u32);
    cpu.cpsr.set(cpu::Psr::c_bit(), carry_bit as u32);
    cpu.cpsr.set(cpu::Psr::v_bit(), overflow_bit as u32);

    4
}

enum ProcessInstrLogicalOp {
    ADD,
    REVERSE_SUB,
    SUB,
}

#[inline(always)]
fn instr_logical(cpu: &mut Cpu, data: cpu::ArmInstrDProc, op: ProcessInstrLogicalOp) -> u32 {
    use cpu::ArmInstrDProc as ArmInstr;

    if !cpu::cond_passed(data.get(ArmInstr::cond()), &cpu.cpsr) {
        return 4;
    }

    let dst_reg = data.get(ArmInstr::rd());
    let s_bit = data.get(ArmInstr::s_bit()) == 1;

    let base_val = cpu.regs[data.get(ArmInstr::rn()) as usize];
    let (shifter_val, _) = get_shifter_val(&data, cpu);

    let (val, carry_bit, overflow_bit) = match op {
        ProcessInstrLogicalOp::ADD => {
            let val = base_val + shifter_val;
            let u_overflow = base_val.checked_add(shifter_val).is_none();
            let s_overflow = (base_val as i32).checked_add(shifter_val as i32).is_none();
            (val, u_overflow, s_overflow)
        },
        ProcessInstrLogicalOp::REVERSE_SUB => {
            let val = shifter_val - base_val;
            let u_overflow = shifter_val.checked_sub(base_val).is_none();
            let s_overflow = (shifter_val as i32).checked_sub(base_val as i32).is_none();
            (val, !u_overflow, s_overflow)
        }
        ProcessInstrLogicalOp::SUB => {
            let val = base_val - shifter_val;
            let u_overflow = base_val.checked_sub(shifter_val).is_none();
            let s_overflow = (base_val as i32).checked_sub(shifter_val as i32).is_none();
            (val, !u_overflow, s_overflow)
        }
    };

    if s_bit {
        if dst_reg == 15 {
            cpu.spsr_make_current();
        } else {
            cpu.cpsr.set(cpu::Psr::n_bit(), bit!(val, 31));
            cpu.cpsr.set(cpu::Psr::z_bit(), (val == 0) as u32);
            cpu.cpsr.set(cpu::Psr::c_bit(), carry_bit as u32);
            cpu.cpsr.set(cpu::Psr::v_bit(), overflow_bit as u32);
        }
    }

    if dst_reg == 15 {
        cpu.branch(val);
        return 0;
    } else {
        cpu.regs[dst_reg as usize] = val;
        return 4;
    }
}

#[inline(always)]
fn instr_move(cpu: &mut Cpu, data: cpu::ArmInstrDProc, negate: bool) -> u32 {
    use cpu::ArmInstrDProc as ArmInstr;

    if !cpu::cond_passed(data.get(ArmInstr::cond()), &cpu.cpsr) {
        return 4;
    }

    let dst_reg = data.get(ArmInstr::rd());
    let s_bit = data.get(ArmInstr::s_bit()) == 1;
    let (mut src_val, shifter_carry) = get_shifter_val(&data, cpu);
    if negate {
        src_val = !src_val;
    }

    if s_bit {
        if dst_reg == 15 {
            cpu.spsr_make_current();
        } else {
            cpu.cpsr.set(cpu::Psr::n_bit(), bit!(src_val, 31));
            cpu.cpsr.set(cpu::Psr::z_bit(), (src_val == 0) as u32);
            cpu.cpsr.set(cpu::Psr::c_bit(), shifter_carry as u32);
        }
    }

    if dst_reg == 15 {
        cpu.branch(src_val);
        return 0;
    } else {
        cpu.regs[dst_reg as usize] = src_val;
        return 4;
    }
}

#[inline(always)]
fn instr_test(cpu: &mut Cpu, data: cpu::ArmInstrDProc, equiv: bool) -> u32 {
    use cpu::ArmInstrDProc as ArmInstr;

    if !cpu::cond_passed(data.get(ArmInstr::cond()), &cpu.cpsr) {
        return 4;
    }

    let (shifter_val, shifter_carry) = get_shifter_val(&data, cpu);
    let rn = data.get(ArmInstr::rn());
    let val = if equiv {
        cpu.regs[rn as usize] ^ shifter_val
    } else {
        cpu.regs[rn as usize] & shifter_val
    };

    cpu.cpsr.set(cpu::Psr::n_bit(), bit!(val, 31));
    cpu.cpsr.set(cpu::Psr::z_bit(), (val == 0) as u32);
    cpu.cpsr.set(cpu::Psr::c_bit(), shifter_carry as u32);

    4
}

#[inline(always)]
pub fn add(cpu: &mut Cpu, data: cpu::ArmInstrDProc) -> u32 {
    instr_logical(cpu, data, ProcessInstrLogicalOp::ADD)
}

#[inline(always)]
pub fn and(cpu: &mut Cpu, data: cpu::ArmInstrDProc) -> u32 {
    instr_bitwise(cpu, data, ProcessInstrBitOp::AND)
}

#[inline(always)]
pub fn bic(cpu: &mut Cpu, data: cpu::ArmInstrDProc) -> u32 {
    instr_bitwise(cpu, data, ProcessInstrBitOp::AND_NOT)
}

#[inline(always)]
pub fn cmn(cpu: &mut Cpu, data: cpu::ArmInstrDProc) -> u32 {
    instr_compare(cpu, data, true)
}

#[inline(always)]
pub fn cmp(cpu: &mut Cpu, data: cpu::ArmInstrDProc) -> u32 {
    instr_compare(cpu, data, false)
}

#[inline(always)]
pub fn eor(cpu: &mut Cpu, data: cpu::ArmInstrDProc) -> u32 {
    instr_bitwise(cpu, data, ProcessInstrBitOp::XOR)
}

#[inline(always)]
pub fn orr(cpu: &mut Cpu, data: cpu::ArmInstrDProc) -> u32 {
    instr_bitwise(cpu, data, ProcessInstrBitOp::OR)
}

#[inline(always)]
pub fn mov(cpu: &mut Cpu, data: cpu::ArmInstrDProc) -> u32 {
    instr_move(cpu, data, false)
}

#[inline(always)]
pub fn mvn(cpu: &mut Cpu, data: cpu::ArmInstrDProc) -> u32 {
    instr_move(cpu, data, true)
}

#[inline(always)]
pub fn rsb(cpu: &mut Cpu, data: cpu::ArmInstrDProc) -> u32 {
    instr_logical(cpu, data, ProcessInstrLogicalOp::REVERSE_SUB)
}

#[inline(always)]
pub fn sub(cpu: &mut Cpu, data: cpu::ArmInstrDProc) -> u32 {
    instr_logical(cpu, data, ProcessInstrLogicalOp::SUB)
}

#[inline(always)]
pub fn teq(cpu: &mut Cpu, data: cpu::ArmInstrDProc) -> u32 {
    instr_test(cpu, data, true)
}

#[inline(always)]
pub fn tst(cpu: &mut Cpu, data: cpu::ArmInstrDProc) -> u32 {
    instr_test(cpu, data, false)
}
