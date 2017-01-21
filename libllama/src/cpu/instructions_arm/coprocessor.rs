use cpu;
use cpu::Cpu;

#[inline(always)]
pub fn mcr(cpu: &mut Cpu, data: cpu::ArmInstrCoprocData) -> cpu::InstrStatus {
    if !cpu::cond_passed(bf!(data.cond), &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    let src_val = cpu.regs[bf!(data.rd) as usize];
    let crm = bf!(data.crm);
    let opcode_1 = bf!(data.opcode_1);
    let opcode_2 = bf!(data.opcode_2);

    let coproc = cpu.get_coprocessor(bf!(data.cp_num) as usize);
    let retval = coproc.execute(src_val, opcode_1, opcode_2, crm as usize);
    coproc.set_reg(bf!(data.crn) as usize, retval);

    cpu::InstrStatus::InBlock
}

#[inline(always)]
pub fn mrc(cpu: &mut Cpu, data: cpu::ArmInstrCoprocData) -> cpu::InstrStatus {
    if !cpu::cond_passed(bf!(data.cond), &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    let crm = bf!(data.crm);
    let rd = bf!(data.rd);
    let opcode_1 = bf!(data.opcode_1);
    let opcode_2 = bf!(data.opcode_2);

    let retval = {
        let coproc = cpu.get_coprocessor(bf!(data.cp_num) as usize);
        let src_val = coproc.get_reg(bf!(data.crn) as usize);
        coproc.execute(src_val, opcode_1, opcode_2, crm as usize)
    };

    if rd == 15 {
        bf!(cpu.cpsr.n_bit = bit!(retval, 31));
        bf!(cpu.cpsr.z_bit = bit!(retval, 30));
        bf!(cpu.cpsr.c_bit = bit!(retval, 29));
        bf!(cpu.cpsr.v_bit = bit!(retval, 28));
    } else {
        cpu.regs[rd as usize] = retval;
    }

    cpu::InstrStatus::InBlock
}
