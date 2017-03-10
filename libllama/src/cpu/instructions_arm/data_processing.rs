use cpu;
use cpu::Cpu;
use cpu::decoder_arm as arm;

fn shifter_lsl(pre_shift: u32, amount: usize, c_bit: bool) -> (u32, bool) {
    if amount == 0 {
        (pre_shift, c_bit)
    } else if amount < 32 {
        let res = pre_shift << amount;
        (res, bit!(pre_shift, (32 - amount) as usize) == 1)
    } else if amount == 32 {
        (0, bit!(pre_shift, 0) == 1)
    } else {
        (0, false)
    }
}

fn shifter_lsr_imm(pre_shift: u32, amount: usize, c_bit: bool) -> (u32, bool) {
    if amount == 0 {
        (0, bit!(pre_shift, 31) == 1)
    } else {
        let res = pre_shift >> amount;
        (res, bit!(pre_shift, (amount - 1) as usize) == 1)
    }
}

fn shifter_lsr_reg(pre_shift: u32, amount: usize, c_bit: bool) -> (u32, bool) {
    if amount == 0 {
        (pre_shift, c_bit)
    } else if amount < 32 {
        let res = pre_shift >> amount;
        (res, bit!(pre_shift, (amount - 1) as usize) == 1)
    } else if amount == 32 {
        (0, bit!(pre_shift, 31) == 1)
    } else {
        (0, false)
    }
}

fn shifter_asr_imm(pre_shift: u32, amount: usize, c_bit: bool) -> (u32, bool) {
    if amount == 0 {
        if bit!(pre_shift, 31) == 0 {
            (0, false)
        } else {
            (0xFFFFFFFF, true)
        }
    } else {
        let res = ((pre_shift as i32) >> amount) as u32;
        (res, bit!(pre_shift, (amount - 1) as usize) == 1)
    }
}

fn shifter_asr_reg(pre_shift: u32, amount: usize, c_bit: bool) -> (u32, bool) {
    if amount == 0 {
        (pre_shift, c_bit)
    } else if amount < 32 {
        let res = ((pre_shift as i32) >> amount) as u32;
        (res, bit!(pre_shift, (amount - 1) as usize) == 1)
    } else {
        if bit!(pre_shift, 31) == 0 {
            (0, false)
        } else {
            (0xFFFFFFFF, true)
        }
    }
}

fn shifter_ror_imm(pre_shift: u32, amount: usize, c_bit: bool) -> (u32, bool) {
    if amount == 0 {
        let res = ((c_bit as u32) << 31) | (pre_shift >> 1);
        (res, bit!(pre_shift, 0) == 1)
    } else {
        let res = pre_shift.rotate_right(amount as u32);
        (res, bit!(pre_shift, (amount - 1) as usize) == 1)
    }
}

fn shifter_ror_reg(pre_shift: u32, amount: usize, c_bit: bool) -> (u32, bool) {
    if amount == 0 {
        (pre_shift, c_bit)
    } else if amount & 0xF == 0 {
        (pre_shift, bit!(pre_shift, 31) == 1)
    } else {
        let amount = amount & 0xF;
        let res = pre_shift.rotate_right(amount as u32);
        (res, bit!(pre_shift, (amount - 1) as usize) == 1)
    }
}

pub fn getreg(cpu: &Cpu, pc_advanced: bool, num: usize) -> u32 {
    if num != 15 { cpu.regs[num] }
    else if pc_advanced { cpu.regs[15] + 4 }
    else { cpu.regs[15] }
}

struct BarrelShifterOut {
    val: u32,
    has_carry: bool,
    pc_advanced: bool, // To work around the fact that PC @ +12 when shifting by a register value
}

#[inline(always)]
fn get_shifter_val(instr_data: u32, cpu: &Cpu) -> BarrelShifterOut {
    // Just to make it a little bit easier to use this
    let instr_data = arm::add::InstrDesc::new(instr_data);

    let shifter_bits = bf!(instr_data.shifter_operand);
    let c_bit = bf!((cpu.cpsr).c_bit) == 1;

    if bf!(instr_data.i_bit) == 1 {
        let immed_8 = bits!(shifter_bits, 0 => 7);
        let rotate_imm = bits!(shifter_bits, 8 => 11);

        let res = immed_8.rotate_right(rotate_imm * 2);
        let carry = if rotate_imm == 0 { c_bit }
                    else { bit!(res, 31) == 1 };
        return BarrelShifterOut { val: res, has_carry: carry, pc_advanced: false }
    }

    let is_reg_shift = bit!(shifter_bits, 4) == 1;

    let amount = if !is_reg_shift {
        bits!(shifter_bits, 7 => 11) as usize
    } else {
        let reg = bits!(shifter_bits, 8 => 11) as usize;
        getreg(cpu, is_reg_shift, reg) as usize
    };
    let pre_shift = getreg(cpu, is_reg_shift, bits!(shifter_bits, 0 => 3) as usize);

    let shift_fn = match bits!(shifter_bits, 4 => 6) {
        0b000 | 0b001 => shifter_lsl,
        0b010 => shifter_lsr_imm,
        0b011 => shifter_lsr_reg,
        0b100 => shifter_asr_imm,
        0b101 => shifter_asr_reg,
        0b110 => shifter_ror_imm,
        0b111 => shifter_ror_reg,
        _ => panic!("Unhandled shifter operation!")
    };

    let (res, carry) = shift_fn(pre_shift, amount, c_bit);
    BarrelShifterOut { val: res, has_carry: carry, pc_advanced: is_reg_shift }
}

enum ProcessInstrBitOp {
    AND,
    AND_NOT,
    OR,
    XOR,
}

#[inline(always)]
fn instr_bitwise(cpu: &mut Cpu, data: arm::and::InstrDesc, op: ProcessInstrBitOp) -> cpu::InstrStatus {
    if !cpu::cond_passed(bf!(data.cond), &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    let dst_reg = bf!(data.rd);
    let s_bit = bf!(data.s_bit) == 1;
    let shift_out = get_shifter_val(data.raw(), cpu);
    let base_val = getreg(cpu, shift_out.pc_advanced, bf!(data.rn) as usize);

    let val = match op {
        ProcessInstrBitOp::AND => base_val & shift_out.val,
        ProcessInstrBitOp::AND_NOT => base_val & !shift_out.val,
        ProcessInstrBitOp::OR => base_val | shift_out.val,
        ProcessInstrBitOp::XOR => base_val ^ shift_out.val,
    };

    if s_bit {
        if dst_reg == 15 {
            cpu.spsr_make_current();
        } else {
            bf!((cpu.cpsr).n_bit = bit!(val, 31));
            bf!((cpu.cpsr).z_bit = (val == 0) as u32);
            bf!((cpu.cpsr).c_bit = shift_out.has_carry as u32);
        }
    }

    if dst_reg == 15 {
        assert!(!shift_out.pc_advanced);
        cpu.branch(val);
        return cpu::InstrStatus::Branched;
    } else {
        cpu.regs[dst_reg as usize] = val;
        return cpu::InstrStatus::InBlock;
    }
}

#[inline(always)]
fn instr_compare(cpu: &mut Cpu, data: arm::cmp::InstrDesc, negative: bool) -> cpu::InstrStatus {
    if !cpu::cond_passed(bf!(data.cond), &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    let shift_out = get_shifter_val(data.raw(), cpu);
    let base_val = getreg(cpu, shift_out.pc_advanced, bf!(data.rn) as usize);

    let (val, carry_bit, overflow_bit) = if !negative {
        let val = base_val.wrapping_sub(shift_out.val);
        let u_overflow = base_val.checked_sub(shift_out.val).is_none();
        let s_overflow = (base_val as i32).checked_sub(shift_out.val as i32).is_none();
        (val, !u_overflow, s_overflow)
    } else {
        let val = base_val.wrapping_add(shift_out.val);
        let u_overflow = base_val.checked_add(shift_out.val).is_none();
        let s_overflow = (base_val as i32).checked_add(shift_out.val as i32).is_none();
        (val, u_overflow, s_overflow)
    };

    bf!((cpu.cpsr).n_bit = bit!(val, 31));
    bf!((cpu.cpsr).z_bit = (val == 0) as u32);
    bf!((cpu.cpsr).c_bit = carry_bit as u32);
    bf!((cpu.cpsr).v_bit = overflow_bit as u32);

    cpu::InstrStatus::InBlock
}

enum ProcessInstrLogicalOp {
    ADD,
    ADD_CARRY,
    REVERSE_SUB,
    REVERSE_SUB_CARRY,
    SUB,
    SUB_CARRY,
}

#[inline(always)]
fn instr_logical(cpu: &mut Cpu, data: arm::add::InstrDesc, op: ProcessInstrLogicalOp) -> cpu::InstrStatus {
    if !cpu::cond_passed(bf!(data.cond), &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    let dst_reg = bf!(data.rd);
    let s_bit = bf!(data.s_bit) == 1;

    let shift_out = get_shifter_val(data.raw(), cpu);
    let base_val = getreg(cpu, shift_out.pc_advanced, bf!(data.rn) as usize);

    let (val, carry_bit, overflow_bit) = match op {
        ProcessInstrLogicalOp::ADD => {
            let val = wrapping_sum!(base_val, shift_out.val);
            let u_overflow = checked_sum!(base_val, shift_out.val).is_none();
            let s_overflow = checked_sum!(base_val as i32, shift_out.val as i32).is_none();
            (val, u_overflow, s_overflow)
        },
        ProcessInstrLogicalOp::ADD_CARRY => {
            let carry = bf!((cpu.cpsr).c_bit) as u32;
            let val = wrapping_sum!(base_val, shift_out.val, carry);
            let u_overflow = checked_sum!(base_val, shift_out.val, carry).is_none();
            let s_overflow = checked_sum!(base_val as i32, shift_out.val as i32, carry as i32).is_none();
            (val, u_overflow, s_overflow)
        }
        ProcessInstrLogicalOp::REVERSE_SUB => {
            let val = wrapping_diff!(shift_out.val, base_val);
            let u_overflow = checked_diff!(shift_out.val, base_val).is_none();
            let s_overflow = checked_diff!(shift_out.val as i32, base_val as i32).is_none();
            (val, !u_overflow, s_overflow)
        }
        ProcessInstrLogicalOp::REVERSE_SUB_CARRY => {
            let ncarry = bf!((cpu.cpsr).c_bit) as u32 ^ 1;
            let val = wrapping_diff!(shift_out.val, base_val, ncarry);
            let u_overflow = checked_diff!(shift_out.val, base_val, ncarry).is_none();
            let s_overflow = checked_diff!(shift_out.val as i32, base_val as i32, ncarry as i32).is_none();
            (val, !u_overflow, s_overflow)
        }
        ProcessInstrLogicalOp::SUB => {
            let val = wrapping_diff!(base_val, shift_out.val);
            let u_overflow = checked_diff!(base_val, shift_out.val).is_none();
            let s_overflow = checked_diff!(base_val as i32, shift_out.val as i32).is_none();
            (val, !u_overflow, s_overflow)
        }
        ProcessInstrLogicalOp::SUB_CARRY => {
            let ncarry = bf!((cpu.cpsr).c_bit) as u32 ^ 1;
            let val = wrapping_diff!(base_val, shift_out.val, ncarry);
            let u_overflow = checked_diff!(base_val, shift_out.val, ncarry).is_none();
            let s_overflow = checked_diff!(base_val as i32, shift_out.val as i32, ncarry as i32).is_none();
            (val, !u_overflow, s_overflow)
        }
    };

    if s_bit {
        if dst_reg == 15 {
            cpu.spsr_make_current();
        } else {
            bf!((cpu.cpsr).n_bit = bit!(val, 31));
            bf!((cpu.cpsr).z_bit = (val == 0) as u32);
            bf!((cpu.cpsr).c_bit = carry_bit as u32);
            bf!((cpu.cpsr).v_bit = overflow_bit as u32);
        }
    }

    if dst_reg == 15 {
        assert!(!shift_out.pc_advanced);
        cpu.branch(val);
        return cpu::InstrStatus::Branched;
    } else {
        cpu.regs[dst_reg as usize] = val;
        return cpu::InstrStatus::InBlock;
    }
}

#[inline(always)]
fn instr_move(cpu: &mut Cpu, data: arm::mov::InstrDesc, negate: bool) -> cpu::InstrStatus {
    if !cpu::cond_passed(bf!(data.cond), &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    let dst_reg = bf!(data.rd) as usize;
    let s_bit = bf!(data.s_bit) == 1;
    let shift_out = get_shifter_val(data.raw(), cpu);

    let src_val = if negate { !shift_out.val }
                  else { shift_out.val };

    if s_bit {
        if dst_reg == 15 {
            cpu.spsr_make_current();
        } else {
            bf!((cpu.cpsr).n_bit = bit!(src_val, 31));
            bf!((cpu.cpsr).z_bit = (src_val == 0) as u32);
            bf!((cpu.cpsr).c_bit = shift_out.has_carry as u32);
        }
    }

    if dst_reg == 15 {
        assert!(!shift_out.pc_advanced);
        cpu.branch(src_val);
        return cpu::InstrStatus::Branched;
    } else {
        cpu.regs[dst_reg] = src_val;
        return cpu::InstrStatus::InBlock;
    }
}

#[inline(always)]
fn instr_test(cpu: &mut Cpu, data: arm::tst::InstrDesc, equiv: bool) -> cpu::InstrStatus {
    if !cpu::cond_passed(bf!(data.cond), &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    let shift_out = get_shifter_val(data.raw(), cpu);
    let base_val = getreg(cpu, shift_out.pc_advanced, bf!(data.rn) as usize);

    let val = if equiv {
        base_val ^ shift_out.val
    } else {
        base_val & shift_out.val
    };

    bf!((cpu.cpsr).n_bit = bit!(val, 31));
    bf!((cpu.cpsr).z_bit = (val == 0) as u32);
    bf!((cpu.cpsr).c_bit = shift_out.has_carry as u32);

    cpu::InstrStatus::InBlock
}

#[inline(always)]
pub fn adc(cpu: &mut Cpu, data: arm::adc::InstrDesc) -> cpu::InstrStatus {
    instr_logical(cpu, arm::add::InstrDesc::new(data.raw()), ProcessInstrLogicalOp::ADD_CARRY)
}

#[inline(always)]
pub fn add(cpu: &mut Cpu, data: arm::add::InstrDesc) -> cpu::InstrStatus {
    instr_logical(cpu, data, ProcessInstrLogicalOp::ADD)
}

#[inline(always)]
pub fn and(cpu: &mut Cpu, data: arm::and::InstrDesc) -> cpu::InstrStatus {
    instr_bitwise(cpu, data, ProcessInstrBitOp::AND)
}

#[inline(always)]
pub fn bic(cpu: &mut Cpu, data: arm::bic::InstrDesc) -> cpu::InstrStatus {
    instr_bitwise(cpu, arm::and::InstrDesc::new(data.raw()), ProcessInstrBitOp::AND_NOT)
}

#[inline(always)]
pub fn clz(cpu: &mut Cpu, data: arm::clz::InstrDesc) -> cpu::InstrStatus {
    if !cpu::cond_passed(bf!(data.cond), &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    let base_val = cpu.regs[bf!(data.rm) as usize];
    cpu.regs[bf!(data.rd) as usize] = base_val.leading_zeros();

    cpu::InstrStatus::InBlock
}

#[inline(always)]
pub fn cmn(cpu: &mut Cpu, data: arm::cmn::InstrDesc) -> cpu::InstrStatus {
    instr_compare(cpu, arm::cmp::InstrDesc::new(data.raw()), true)
}

#[inline(always)]
pub fn cmp(cpu: &mut Cpu, data: arm::cmp::InstrDesc) -> cpu::InstrStatus {
    instr_compare(cpu, data, false)
}

#[inline(always)]
pub fn eor(cpu: &mut Cpu, data: arm::eor::InstrDesc) -> cpu::InstrStatus {
    instr_bitwise(cpu, arm::and::InstrDesc::new(data.raw()), ProcessInstrBitOp::XOR)
}

#[inline(always)]
pub fn orr(cpu: &mut Cpu, data: arm::orr::InstrDesc) -> cpu::InstrStatus {
    instr_bitwise(cpu, arm::and::InstrDesc::new(data.raw()), ProcessInstrBitOp::OR)
}

#[inline(always)]
pub fn mla(cpu: &mut Cpu, data: arm::mla::InstrDesc) -> cpu::InstrStatus {
    if !cpu::cond_passed(bf!(data.cond), &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    let base_val = cpu.regs[bf!(data.rm) as usize] as u64;
    let multiplier = cpu.regs[bf!(data.rs) as usize] as u64;
    let accumulated = cpu.regs[bf!(data.rn) as usize] as u64;
    let val = (base_val * multiplier + accumulated) as u32;

    cpu.regs[bf!(data.rd) as usize] = val;

    if bf!(data.s_bit) == 1 {
        bf!((cpu.cpsr).n_bit = bit!(val, 31));
        bf!((cpu.cpsr).z_bit = (val == 0) as u32);
    };

    cpu::InstrStatus::InBlock
}

#[inline(always)]
pub fn mov(cpu: &mut Cpu, data: arm::mov::InstrDesc) -> cpu::InstrStatus {
    instr_move(cpu, data, false)
}

#[inline(always)]
pub fn mul(cpu: &mut Cpu, data: arm::mul::InstrDesc) -> cpu::InstrStatus {
    if !cpu::cond_passed(bf!(data.cond), &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    let base_val = cpu.regs[bf!(data.rm) as usize] as u64;
    let multiplier = cpu.regs[bf!(data.rs) as usize] as u64;
    let val = (base_val * multiplier) as u32;

    cpu.regs[bf!(data.rd) as usize] = val;

    if bf!(data.s_bit) == 1 {
        bf!((cpu.cpsr).n_bit = bit!(val, 31));
        bf!((cpu.cpsr).z_bit = (val == 0) as u32);
    };

    cpu::InstrStatus::InBlock
}

#[inline(always)]
pub fn mvn(cpu: &mut Cpu, data: arm::mvn::InstrDesc) -> cpu::InstrStatus {
    instr_move(cpu, arm::mov::InstrDesc::new(data.raw()), true)
}

#[inline(always)]
pub fn rsb(cpu: &mut Cpu, data: arm::rsb::InstrDesc) -> cpu::InstrStatus {
    instr_logical(cpu, arm::add::InstrDesc::new(data.raw()), ProcessInstrLogicalOp::REVERSE_SUB)
}

#[inline(always)]
pub fn rsc(cpu: &mut Cpu, data: arm::rsc::InstrDesc) -> cpu::InstrStatus {
    instr_logical(cpu, arm::add::InstrDesc::new(data.raw()), ProcessInstrLogicalOp::REVERSE_SUB_CARRY)
}

#[inline(always)]
pub fn sbc(cpu: &mut Cpu, data: arm::sbc::InstrDesc) -> cpu::InstrStatus {
    instr_logical(cpu, arm::add::InstrDesc::new(data.raw()), ProcessInstrLogicalOp::SUB_CARRY)
}

#[inline(always)]
pub fn smull(cpu: &mut Cpu, data: arm::smull::InstrDesc) -> cpu::InstrStatus {
    if !cpu::cond_passed(bf!(data.cond), &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    let base_val = cpu.regs[bf!(data.rm) as usize] as i32;
    let multiplier = cpu.regs[bf!(data.rs) as usize] as i32;
    let val = (base_val as i64).wrapping_mul(multiplier as i64);

    cpu.regs[bf!(data.rd_hi) as usize] = (val >> 32) as u32;
    cpu.regs[bf!(data.rd_lo) as usize] = val as u32;

    if bf!(data.s_bit) == 1 {
        bf!((cpu.cpsr).n_bit = bit!(val, 63) as u32);
        bf!((cpu.cpsr).z_bit = (val == 0) as u32);
    };

    cpu::InstrStatus::InBlock
}

#[inline(always)]
pub fn sub(cpu: &mut Cpu, data: arm::sub::InstrDesc) -> cpu::InstrStatus {
    instr_logical(cpu, arm::add::InstrDesc::new(data.raw()), ProcessInstrLogicalOp::SUB)
}

#[inline(always)]
pub fn teq(cpu: &mut Cpu, data: arm::teq::InstrDesc) -> cpu::InstrStatus {
    instr_test(cpu, arm::tst::InstrDesc::new(data.raw()), true)
}

#[inline(always)]
pub fn tst(cpu: &mut Cpu, data: arm::tst::InstrDesc) -> cpu::InstrStatus {
    instr_test(cpu, data, false)
}

#[inline(always)]
pub fn umull(cpu: &mut Cpu, data: arm::umull::InstrDesc) -> cpu::InstrStatus {
    if !cpu::cond_passed(bf!(data.cond), &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    let base_val = cpu.regs[bf!(data.rm) as usize] as u64;
    let multiplier = cpu.regs[bf!(data.rs) as usize] as u64;
    let val = base_val.wrapping_mul(multiplier);

    cpu.regs[bf!(data.rd_hi) as usize] = (val >> 32) as u32;
    cpu.regs[bf!(data.rd_lo) as usize] = val as u32;

    if bf!(data.s_bit) == 1 {
        bf!((cpu.cpsr).n_bit = bit!(val, 63) as u32);
        bf!((cpu.cpsr).z_bit = (val == 0) as u32);
    };

    cpu::InstrStatus::InBlock
}
