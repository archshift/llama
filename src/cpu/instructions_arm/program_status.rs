use cpu;
use cpu::Cpu;

#[inline(always)]
pub fn mrs(cpu: &mut Cpu, data: cpu::ArmInstrMoveStatusReg) -> u32 {
    use cpu::ArmInstrMoveStatusReg as ArmInstr;

    if !cpu::cond_passed(data.get(ArmInstr::cond()), &cpu.cpsr) {
        return 4;
    }

    let rd = data.get(ArmInstr::rd());
    let r_bit = data.get(ArmInstr::r_bit());

    if r_bit == 1 {
        cpu.regs[rd as usize] = cpu.get_current_spsr().raw();
    } else {
        cpu.regs[rd as usize] = cpu.cpsr.raw();
    }

    4
}
