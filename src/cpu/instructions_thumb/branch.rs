use cpu;
use cpu::Cpu;
use utils::sign_extend;

#[inline(always)]
pub fn b_1(cpu: &mut Cpu, data: cpu::ThumbInstrB_1) -> u32 {
    use cpu::ThumbInstrB_1 as ThumbInstr;

    let offset_8 = data.get(ThumbInstr::signed_imm_8());
    let cond = data.get(ThumbInstr::cond());

    if !cpu::cond_passed(cond as u32, &cpu.cpsr) {
        return 2;
    }

    let addr = (cpu.regs[15] as i32 + (sign_extend(offset_8 as u32, 8) << 1)) as u32;
    cpu.branch(addr);
    0
}

#[inline(always)]
pub fn branch(cpu: &mut Cpu, data: cpu::ThumbInstrBRANCH) -> u32 {
    use cpu::ThumbInstrBRANCH as ThumbInstr;

    let offset_11 = data.get(ThumbInstr::offset_11());

    match data.get(ThumbInstr::h_bits()) {
        0b00 => {
            let addr = (cpu.regs[15] as i32 + (sign_extend(offset_11 as u32, 11) << 1)) as u32;
            cpu.branch(addr);
            0
        },
        0b01 => {
            let addr = (cpu.regs[14] + (offset_11 << 1) as u32) & 0xFFFFFFFC;
            cpu.regs[14] = (cpu.regs[15] - 2) as u32 | 1;
            cpu.cpsr.set(cpu::Psr::thumb_bit(), 0);
            cpu.branch(addr);
            0
        },
        0b10 => {
            cpu.regs[14] = (cpu.regs[15] as i32 + (sign_extend(offset_11 as u32, 11) << 12)) as u32;
            2
        },
        0b11 => {
            let addr = cpu.regs[14] + (offset_11 << 1) as u32;
            cpu.regs[14] = (cpu.regs[15] - 2) as u32 | 1;
            cpu.branch(addr);
            0
        },
        _ => unreachable!(),
    }
}

#[inline(always)]
pub fn bx(cpu: &mut Cpu, data: cpu::ThumbInstrBranchReg) -> u32 {
    use cpu::ThumbInstrBranchReg as ThumbInstr;

    let addr = cpu.regs[data.get(ThumbInstr::rm()) as usize];
    cpu.cpsr.set(cpu::Psr::thumb_bit(), bit!(addr, 0));
    cpu.branch(addr & 0xFFFFFFFE);
    0
}