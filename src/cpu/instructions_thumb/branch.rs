use cpu;
use cpu::Cpu;
use utils::sign_extend;

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
