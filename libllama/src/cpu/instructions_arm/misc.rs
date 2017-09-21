use cpu;
use cpu::Cpu;
use cpu::decoder_arm as arm;

#[inline(always)]
pub fn swi(cpu: &mut Cpu, data: arm::swi::InstrDesc) -> cpu::InstrStatus {
    if !cpu::cond_passed(bf!(data.cond), &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    let next_instr = cpu.regs[15] - cpu.get_pc_offset() / 2;
    cpu.enter_exception(next_instr, cpu::Mode::Svc);
    cpu::InstrStatus::Branched
}