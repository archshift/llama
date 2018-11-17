use cpu::{self, Cpu, Version};
use cpu::interpreter_arm as arm;

pub fn mcr<V: Version>(cpu: &mut Cpu<V>, data: arm::Mcr::Bf) -> cpu::InstrStatus {
    assert!(V::is::<cpu::v5>());
    if !cpu::cond_passed(data.cond.get(), &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    let src_val = cpu.regs[data.rd.get() as usize];
    let crn = data.crn.get() as usize;
    let crm = data.crm.get() as usize;
    let opcode_1 = data.opcode_1.get() as usize;
    let opcode_2 = data.opcode_2.get() as usize;

    let cp_effect = {
        let coproc = cpu.get_coprocessor(data.cp_num.get() as usize);
        coproc.move_in(crn, crm, opcode_1, opcode_2, src_val)
    };
    cp_effect(cpu);

    cpu::InstrStatus::InBlock
}

pub fn mrc<V: Version>(cpu: &mut Cpu<V>, data: arm::Mrc::Bf) -> cpu::InstrStatus {
    assert!(V::is::<cpu::v5>());
    if !cpu::cond_passed(data.cond.get(), &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    let crn = data.crn.get() as usize;
    let crm = data.crm.get() as usize;
    let opcode_1 = data.opcode_1.get() as usize;
    let opcode_2 = data.opcode_2.get() as usize;
    let rd = data.rd.get();

    let retval = {
        let coproc = cpu.get_coprocessor(data.cp_num.get() as usize);
        coproc.move_out(crn, crm, opcode_1, opcode_2)
    };

    if rd == 15 {
        cpu.cpsr.n_bit.set(bit!(retval, 31));
        cpu.cpsr.z_bit.set(bit!(retval, 30));
        cpu.cpsr.c_bit.set(bit!(retval, 29));
        cpu.cpsr.v_bit.set(bit!(retval, 28));
    } else {
        cpu.regs[rd as usize] = retval;
    }

    cpu::InstrStatus::InBlock
}
