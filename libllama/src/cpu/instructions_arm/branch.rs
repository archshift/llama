use cpu;
use cpu::Cpu;
use bitutils::sign_extend;

#[inline(always)]
fn instr_branch_exchange(cpu: &mut Cpu, data: cpu::ArmInstrBranchExchange, link: bool) -> cpu::InstrStatus {
    if !cpu::cond_passed(bf!(data.cond), &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    let addr = cpu.regs[bf!(data.rm) as usize];

    if link {
        cpu.regs[14] = cpu.regs[15] - 4;
    }

    bf!((cpu.cpsr).thumb_bit = bit!(addr, 0));
    cpu.branch(addr & 0xFFFFFFFE);

    cpu::InstrStatus::Branched
}

#[inline(always)]
pub fn bbl(cpu: &mut Cpu, data: cpu::ArmInstrBBL) -> cpu::InstrStatus {
    if !cpu::cond_passed(bf!(data.cond), &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    let signed_imm_24 = bf!(data.signed_imm_24);

    if bf!(data.link_bit) == 1 {
        cpu.regs[14] = cpu.regs[15] - 4;
    }

    let pc = cpu.regs[15];
    cpu.branch(((pc as i32) + (sign_extend(signed_imm_24, 24) << 2)) as u32);

    cpu::InstrStatus::Branched
}

#[inline(always)]
pub fn blx(cpu: &mut Cpu, data: cpu::ArmInstrBranchExchange) -> cpu::InstrStatus {
    instr_branch_exchange(cpu, data, true)
}

#[inline(always)]
pub fn bx(cpu: &mut Cpu, data: cpu::ArmInstrBranchExchange) -> cpu::InstrStatus {
    instr_branch_exchange(cpu, data, false)
}

#[inline(always)]
pub fn mod_blx(cpu: &mut Cpu, data: cpu::ArmInstrModBLX) -> cpu::InstrStatus {
    let signed_imm_24 = bf!(data.signed_imm_24);
    let h_bit = bf!(data.h_bit);

    cpu.regs[14] = cpu.regs[15] - 4;
    bf!((cpu.cpsr).thumb_bit = 1);

    let pc = cpu.regs[15];
    cpu.branch((pc as i32 + (sign_extend(signed_imm_24, 24) << 2)) as u32 + (h_bit << 1));

    cpu::InstrStatus::Branched
}
