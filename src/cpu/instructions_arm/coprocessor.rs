use cpu;
use cpu::Cpu;

#[inline(always)]
pub fn mcr(cpu: &mut Cpu, data: cpu::ArmInstrCoprocData) -> u32 {
    use cpu::ArmInstrCoprocData as ArmInstr;

    if !cpu::cond_passed(data.get(ArmInstr::cond()), &cpu.cpsr) {
        return 4;
    }

    let src_val = cpu.regs[data.get(ArmInstr::rd()) as usize];
    let crm = data.get(ArmInstr::crm());
    let opcode_1 = data.get(ArmInstr::opcode_1());
    let opcode_2 = data.get(ArmInstr::opcode_2());

    let coproc = cpu.get_coprocessor(data.get(ArmInstr::cp_num()) as usize);
    let retval = coproc.execute(src_val, opcode_1, opcode_2, crm as usize);
    coproc.set_reg(data.get(ArmInstr::crn()) as usize, retval);

    4
}

#[inline(always)]
pub fn mrc(cpu: &mut Cpu, data: cpu::ArmInstrCoprocData) -> u32 {
    use cpu::ArmInstrCoprocData as ArmInstr;

    if !cpu::cond_passed(data.get(ArmInstr::cond()), &cpu.cpsr) {
        return 4;
    }

    let crm = data.get(ArmInstr::crm());
    let rd = data.get(ArmInstr::rd());
    let opcode_1 = data.get(ArmInstr::opcode_1());
    let opcode_2 = data.get(ArmInstr::opcode_2());

    let retval = {
        let coproc = cpu.get_coprocessor(data.get(ArmInstr::cp_num()) as usize);
        let src_val = coproc.get_reg(data.get(ArmInstr::crn()) as usize);
        coproc.execute(src_val, opcode_1, opcode_2, crm as usize)
    };

    if rd == 15 {
        cpu.cpsr.set(cpu::Psr::n_bit(), bit!(retval, 31));
        cpu.cpsr.set(cpu::Psr::z_bit(), bit!(retval, 30));
        cpu.cpsr.set(cpu::Psr::c_bit(), bit!(retval, 29));
        cpu.cpsr.set(cpu::Psr::v_bit(), bit!(retval, 28));
    } else {
        cpu.regs[rd as usize] = retval;
    }

    4
}
