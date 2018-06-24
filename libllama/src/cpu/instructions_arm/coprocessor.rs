use cpu;
use cpu::Cpu;
use cpu::interpreter_arm as arm;

pub fn mcr(cpu: &mut Cpu, data: arm::Mcr) -> cpu::InstrStatus {
    if !cpu::cond_passed(bf!(data.cond), &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    let src_val = cpu.regs[bf!(data.rd) as usize];
    let crn = bf!(data.crn) as usize;
    let crm = bf!(data.crm) as usize;
    let opcode_1 = bf!(data.opcode_1) as usize;
    let opcode_2 = bf!(data.opcode_2) as usize;

    let cp_effect = {
        let coproc = cpu.get_coprocessor(bf!(data.cp_num) as usize);
        coproc.move_in(crn, crm, opcode_1, opcode_2, src_val)
    };
    cp_effect(cpu);

    cpu::InstrStatus::InBlock
}

pub fn mrc(cpu: &mut Cpu, data: arm::Mrc) -> cpu::InstrStatus {
    if !cpu::cond_passed(bf!(data.cond), &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    let crn = bf!(data.crn) as usize;
    let crm = bf!(data.crm) as usize;
    let opcode_1 = bf!(data.opcode_1) as usize;
    let opcode_2 = bf!(data.opcode_2) as usize;
    let rd = bf!(data.rd);

    let retval = {
        let coproc = cpu.get_coprocessor(bf!(data.cp_num) as usize);
        coproc.move_out(crn, crm, opcode_1, opcode_2)
    };

    if rd == 15 {
        bf!((cpu.cpsr).n_bit = bit!(retval, 31));
        bf!((cpu.cpsr).z_bit = bit!(retval, 30));
        bf!((cpu.cpsr).c_bit = bit!(retval, 29));
        bf!((cpu.cpsr).v_bit = bit!(retval, 28));
    } else {
        cpu.regs[rd as usize] = retval;
    }

    cpu::InstrStatus::InBlock
}
