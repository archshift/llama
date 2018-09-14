use cpu;
use cpu::Cpu;
use cpu::interpreter_thumb as thumb;
use bitutils::sign_extend32;

pub fn b_1(cpu: &mut Cpu, data: thumb::B1::Bf) -> cpu::InstrStatus {
    let offset_8 = data.signed_imm_8.get();
    let cond = data.cond.get();

    if !cpu::cond_passed(cond as u32, &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    let addr = (cpu.regs[15] as i32 + (sign_extend32(offset_8 as u32, 8) << 1)) as u32;
    cpu.branch(addr);
    cpu::InstrStatus::Branched
}

pub fn branch(cpu: &mut Cpu, data: thumb::Branch::Bf) -> cpu::InstrStatus {
    let offset_11 = data.offset_11.get();

    match data.h_bits.get() {
        0b00 => {
            let addr = (cpu.regs[15] as i32 + (sign_extend32(offset_11 as u32, 11) << 1)) as u32;
            cpu.branch(addr);
            cpu::InstrStatus::Branched
        },
        0b01 => {
            let addr = (cpu.regs[14] + (offset_11 << 1) as u32) & 0xFFFFFFFC;
            cpu.regs[14] = (cpu.regs[15] - 2) as u32 | 1;
            cpu.cpsr.thumb_bit.set(0);
            cpu.branch(addr);
            cpu::InstrStatus::Branched
        },
        0b10 => {
            cpu.regs[14] = (cpu.regs[15] as i32 + (sign_extend32(offset_11 as u32, 11) << 12)) as u32;
            cpu::InstrStatus::InBlock
        },
        0b11 => {
            let addr = cpu.regs[14] + (offset_11 << 1) as u32;
            cpu.regs[14] = (cpu.regs[15] - 2) as u32 | 1;
            cpu.branch(addr);
            cpu::InstrStatus::Branched
        },
        _ => unreachable!(),
    }
}

pub fn blx_2(cpu: &mut Cpu, data: thumb::Blx2::Bf) -> cpu::InstrStatus {
    let rm = data.rm.get() | (data.h2.get() << 3);
    let addr = cpu.regs[rm as usize];

    cpu.regs[14] = (cpu.regs[15] - 2) as u32 | 1;
    cpu.cpsr.thumb_bit.set(bit!(addr, 0));

    cpu.branch(addr & 0xFFFFFFFE);
    cpu::InstrStatus::Branched
}

pub fn bx(cpu: &mut Cpu, data: thumb::Bx::Bf) -> cpu::InstrStatus {
    let addr = cpu.regs[((data.h2.get() << 3) | data.rm.get()) as usize];
    cpu.cpsr.thumb_bit.set(bit!(addr, 0));
    cpu.branch(addr & 0xFFFFFFFE);
    cpu::InstrStatus::Branched
}
