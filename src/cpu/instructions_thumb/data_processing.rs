use cpu;
use cpu::Cpu;

#[inline(always)]
pub fn lsl_1(cpu: &mut Cpu, data: cpu::ThumbInstrLSL_1) -> u32 {
    use cpu::ThumbInstrLSL_1 as ThumbInstr;

    let base_val = cpu.regs[data.get(ThumbInstr::rm()) as usize];
    let amount = data.get(ThumbInstr::immed_5()) as u32;
    let val = base_val << amount;

    if amount > 0 {
        cpu.cpsr.set(cpu::Psr::c_bit(), bit!(val, 32 - (amount as usize)));
    }
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
