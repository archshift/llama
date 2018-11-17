use cpu::{self, Cpu, Version};
use cpu::interpreter_arm as arm;
use bitutils::sign_extend32;

fn instr_branch_exchange<V: Version>(cpu: &mut Cpu<V>, data: arm::Bx::Bf, link: bool) -> cpu::InstrStatus {
    if !cpu::cond_passed(data.cond.get(), &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    let addr = cpu.regs[data.rm.get() as usize];

    if link {
        cpu.regs[14] = cpu.regs[15] - 4;
    }

    cpu.cpsr.thumb_bit.set(bit!(addr, 0));
    cpu.branch(addr & 0xFFFFFFFE);

    cpu::InstrStatus::Branched
}

pub fn bbl<V: Version>(cpu: &mut Cpu<V>, data: arm::Bbl::Bf) -> cpu::InstrStatus {
    assert!(V::is::<cpu::v5>());
    if !cpu::cond_passed(data.cond.get(), &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    let signed_imm_24 = data.signed_imm_24.get();

    if data.link_bit.get() == 1 {
        cpu.regs[14] = cpu.regs[15] - 4;
    }

    let pc = cpu.regs[15];
    cpu.branch(((pc as i32) + (sign_extend32(signed_imm_24, 24) << 2)) as u32);

    cpu::InstrStatus::Branched
}

pub fn blx<V: Version>(cpu: &mut Cpu<V>, data: arm::Blx2::Bf) -> cpu::InstrStatus {
    assert!(V::is::<cpu::v5>());
    instr_branch_exchange(cpu, arm::Bx::new(data.val), true)
}

pub fn bx<V: Version>(cpu: &mut Cpu<V>, data: arm::Bx::Bf) -> cpu::InstrStatus {
    assert!(V::is::<cpu::v5>());
    instr_branch_exchange(cpu, data, false)
}

pub fn mod_blx<V: Version>(cpu: &mut Cpu<V>, data: arm::ModBlx::Bf) -> cpu::InstrStatus {
    assert!(V::is::<cpu::v5>());
    let signed_imm_24 = data.signed_imm_24.get();
    let h_bit = data.h_bit.get();

    cpu.regs[14] = cpu.regs[15] - 4;
    cpu.cpsr.thumb_bit.set(1);

    let pc = cpu.regs[15];
    cpu.branch((pc as i32 + (sign_extend32(signed_imm_24, 24) << 2)) as u32 + (h_bit << 1));

    cpu::InstrStatus::Branched
}
