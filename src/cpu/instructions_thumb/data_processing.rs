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
    use cpu::ThumbInstrBitwise as ThumbInstr;
    let base_val = cpu.regs[data.get(ThumbInstr::rd()) as usize];
    let rm = cpu.regs[data.get(ThumbInstr::rm()) as usize];

    let val = match op {
        ProcessInstrBitOp::AND => base_val & rm,
        ProcessInstrBitOp::AND_NOT => base_val & !rm,
        ProcessInstrBitOp::OR => base_val | rm,
        ProcessInstrBitOp::XOR => base_val ^ rm,
    };

    cpu.cpsr.set(cpu::Psr::n_bit(), bit!(val, 31));
    cpu.cpsr.set(cpu::Psr::z_bit(), (val == 0) as u32);
    cpu.regs[data.get(ThumbInstr::rd()) as usize] = val;

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
pub fn eor(cpu: &mut Cpu, data: cpu::ThumbInstrBitwise) -> u32 {
    instr_bitwise(cpu, data, ProcessInstrBitOp::XOR)
}

#[inline(always)]
pub fn lsl_1(cpu: &mut Cpu, data: cpu::ThumbInstrShift_1) -> u32 {
    use cpu::ThumbInstrShift_1 as ThumbInstr;

    let base_val = cpu.regs[data.get(ThumbInstr::rm()) as usize];
    let amount = data.get(ThumbInstr::immed_5()) as u32;
    let val = base_val << amount;

    if amount > 0 {
        cpu.cpsr.set(cpu::Psr::c_bit(), bit!(base_val, 32 - (amount as usize)));
    }
    cpu.cpsr.set(cpu::Psr::n_bit(), bit!(val, 31));
    cpu.cpsr.set(cpu::Psr::z_bit(), (val == 0) as u32);
    cpu.regs[data.get(ThumbInstr::rd()) as usize] = val;

    2
}

#[inline(always)]
pub fn lsr_1(cpu: &mut Cpu, data: cpu::ThumbInstrShift_1) -> u32 {
    use cpu::ThumbInstrShift_1 as ThumbInstr;

    let base_val = cpu.regs[data.get(ThumbInstr::rm()) as usize];
    let amount = data.get(ThumbInstr::immed_5()) as u32;

    let val = if amount == 0 {
        // LSR 32
        cpu.cpsr.set(cpu::Psr::c_bit(), bit!(base_val, 31));
        0
    } else {
        cpu.cpsr.set(cpu::Psr::c_bit(), bit!(base_val, (amount as usize) - 1));
        base_val >> amount
    };
    cpu.cpsr.set(cpu::Psr::n_bit(), bit!(val, 31));
    cpu.cpsr.set(cpu::Psr::z_bit(), (val == 0) as u32);
    cpu.regs[data.get(ThumbInstr::rd()) as usize] = val;

    2
}


#[inline(always)]
pub fn mov_1(cpu: &mut Cpu, data: cpu::ThumbInstrMOV_1) -> u32 {
    use cpu::ThumbInstrMOV_1 as ThumbInstr;
    let val = data.get(ThumbInstr::immed_8()) as u32;

    cpu.cpsr.set(cpu::Psr::n_bit(), bit!(val, 31));
    cpu.cpsr.set(cpu::Psr::z_bit(), (val == 0) as u32);
    cpu.regs[data.get(ThumbInstr::rd()) as usize] = val;

    2
}

#[inline(always)]
pub fn mov_2(cpu: &mut Cpu, data: cpu::ThumbInstrMOV_2) -> u32 {
    use cpu::ThumbInstrMOV_2 as ThumbInstr;
    let val = cpu.regs[data.get(ThumbInstr::rn()) as usize];

    cpu.cpsr.set(cpu::Psr::n_bit(), bit!(val, 31));
    cpu.cpsr.set(cpu::Psr::z_bit(), (val == 0) as u32);
    cpu.cpsr.set(cpu::Psr::c_bit(), 0);
    cpu.cpsr.set(cpu::Psr::v_bit(), 0);
    cpu.regs[data.get(ThumbInstr::rd()) as usize] = val;

    2
}

#[inline(always)]
pub fn mov_3(cpu: &mut Cpu, data: cpu::ThumbInstrMOV_3) -> u32 {
    use cpu::ThumbInstrMOV_3 as ThumbInstr;

    let rd = data.get(ThumbInstr::rd()) | (data.get(ThumbInstr::h1()) << 3);
    let rm = data.get(ThumbInstr::rm()) | (data.get(ThumbInstr::h2()) << 3);
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
    use cpu::ThumbInstrBitwise as ThumbInstr;
    let base_val = cpu.regs[data.get(ThumbInstr::rd()) as usize];
    let val = base_val & cpu.regs[data.get(ThumbInstr::rm()) as usize];

    cpu.cpsr.set(cpu::Psr::n_bit(), bit!(val, 31));
    cpu.cpsr.set(cpu::Psr::z_bit(), (val == 0) as u32);

    2
}