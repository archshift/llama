use cpu;
use cpu::Cpu;

enum ProcessInstrBitOp {
    AND,
    AND_NOT,
    OR,
    XOR,
}

#[inline(always)]
fn instr_bitwise(cpu: &mut Cpu, data: cpu::ThumbInstrBitwise, op: ProcessInstrBitOp) -> u32 {
    let base_val = cpu.regs[bf!(data.rd) as usize];
    let rm = cpu.regs[bf!(data.rm) as usize];

    let val = match op {
        ProcessInstrBitOp::AND => base_val & rm,
        ProcessInstrBitOp::AND_NOT => base_val & !rm,
        ProcessInstrBitOp::OR => base_val | rm,
        ProcessInstrBitOp::XOR => base_val ^ rm,
    };

    bf!((cpu.cpsr).n_bit = bit!(val, 31));
    bf!((cpu.cpsr).z_bit = (val == 0) as u32);
    cpu.regs[bf!(data.rd) as usize] = val;

    2
}

#[inline(always)]
pub fn and(cpu: &mut Cpu, data: cpu::ThumbInstrBitwise) -> u32 {
    instr_bitwise(cpu, data, ProcessInstrBitOp::AND)
}

#[inline(always)]
pub fn bic(cpu: &mut Cpu, data: cpu::ThumbInstrBitwise) -> u32 {
    instr_bitwise(cpu, data, ProcessInstrBitOp::AND_NOT)
}

#[inline(always)]
pub fn cmp_1(cpu: &mut Cpu, data: cpu::ThumbInstrCMP_1) -> u32 {
    let base_val = cpu.regs[bf!(data.rn) as usize];
    let immed = bf!(data.immed_8) as u32;

    let val = base_val - immed;
    let carry_bit = !base_val.checked_sub(immed).is_none();
    let overflow_bit = (base_val as i32).checked_sub(immed as i32).is_none();

    bf!((cpu.cpsr).n_bit = bit!(val, 31));
    bf!((cpu.cpsr).z_bit = (val == 0) as u32);
    bf!((cpu.cpsr).c_bit = carry_bit as u32);
    bf!((cpu.cpsr).v_bit = overflow_bit as u32);

    2
}

#[inline(always)]
pub fn cmp_3(cpu: &mut Cpu, data: cpu::ThumbInstrCMP_3) -> u32 {
    let rn = bf!(data.rn) | (bf!(data.h1) << 3);
    let rm = bf!(data.rm) | (bf!(data.h2) << 3);
    let base_val = cpu.regs[rn as usize];
    let other = cpu.regs[rm as usize];

    let val = base_val - other;
    let carry_bit = !base_val.checked_sub(other).is_none();
    let overflow_bit = (base_val as i32).checked_sub(other as i32).is_none();

    bf!((cpu.cpsr).n_bit = bit!(val, 31));
    bf!((cpu.cpsr).z_bit = (val == 0) as u32);
    bf!((cpu.cpsr).c_bit = carry_bit as u32);
    bf!((cpu.cpsr).v_bit = overflow_bit as u32);

    2
}

#[inline(always)]
pub fn eor(cpu: &mut Cpu, data: cpu::ThumbInstrBitwise) -> u32 {
    instr_bitwise(cpu, data, ProcessInstrBitOp::XOR)
}

#[inline(always)]
pub fn lsl_1(cpu: &mut Cpu, data: cpu::ThumbInstrShift_1) -> u32 {
    let base_val = cpu.regs[bf!(data.rm) as usize];
    let amount = bf!(data.immed_5) as u32;
    let val = base_val << amount;

    if amount > 0 {
        bf!((cpu.cpsr).c_bit = bit!(base_val, 32 - (amount as usize)));
    }
    bf!((cpu.cpsr).n_bit = bit!(val, 31));
    bf!((cpu.cpsr).z_bit = (val == 0) as u32);
    cpu.regs[bf!(data.rd) as usize] = val;

    2
}

#[inline(always)]
pub fn lsr_1(cpu: &mut Cpu, data: cpu::ThumbInstrShift_1) -> u32 {
    let base_val = cpu.regs[bf!(data.rm) as usize];
    let amount = bf!(data.immed_5) as u32;

    let val = if amount == 0 {
        // LSR 32
        bf!((cpu.cpsr).c_bit = bit!(base_val, 31));
        0
    } else {
        bf!((cpu.cpsr).c_bit = bit!(base_val, (amount as usize) - 1));
        base_val >> amount
    };
    bf!((cpu.cpsr).n_bit = bit!(val, 31));
    bf!((cpu.cpsr).z_bit = (val == 0) as u32);
    cpu.regs[bf!(data.rd) as usize] = val;

    2
}


#[inline(always)]
pub fn mov_1(cpu: &mut Cpu, data: cpu::ThumbInstrMOV_1) -> u32 {
    let val = bf!(data.immed_8) as u32;

    bf!((cpu.cpsr).n_bit = bit!(val, 31));
    bf!((cpu.cpsr).z_bit = (val == 0) as u32);
    cpu.regs[bf!(data.rd) as usize] = val;

    2
}

#[inline(always)]
pub fn mov_2(cpu: &mut Cpu, data: cpu::ThumbInstrMOV_2) -> u32 {
    let val = cpu.regs[bf!(data.rn) as usize];

    bf!((cpu.cpsr).n_bit = bit!(val, 31));
    bf!((cpu.cpsr).z_bit = (val == 0) as u32);
    bf!((cpu.cpsr).c_bit = 0);
    bf!((cpu.cpsr).v_bit = 0);
    cpu.regs[bf!(data.rd) as usize] = val;

    2
}

#[inline(always)]
pub fn mov_3(cpu: &mut Cpu, data: cpu::ThumbInstrMOV_3) -> u32 {
    let rd = bf!(data.rd) | (bf!(data.h1) << 3);
    let rm = bf!(data.rm) | (bf!(data.h2) << 3);
    let base_val = cpu.regs[rm as usize];

    if rd == 15 {
        cpu.branch(base_val);
        return 0
    }

    cpu.regs[rd as usize] = base_val;
    2
}

#[inline(always)]
pub fn orr(cpu: &mut Cpu, data: cpu::ThumbInstrBitwise) -> u32 {
    instr_bitwise(cpu, data, ProcessInstrBitOp::OR)
}

#[inline(always)]
pub fn tst(cpu: &mut Cpu, data: cpu::ThumbInstrBitwise) -> u32 {
    let base_val = cpu.regs[bf!(data.rd) as usize];
    let val = base_val & cpu.regs[bf!(data.rm) as usize];

    bf!((cpu.cpsr).n_bit = bit!(val, 31));
    bf!((cpu.cpsr).z_bit = (val == 0) as u32);

    2
}