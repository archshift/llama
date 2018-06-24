use cpu;
use cpu::Cpu;
use cpu::interpreter_arm as arm;

pub fn swi(cpu: &mut Cpu, data: arm::Swi) -> cpu::InstrStatus {
    if !cpu::cond_passed(bf!(data.cond), &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    let next_instr = cpu.regs[15] - cpu.get_pc_offset() / 2;
    cpu.enter_exception(next_instr, cpu::Mode::Svc);
    cpu::InstrStatus::Branched
}

pub fn bkpt(_cpu: &mut Cpu, data: arm::Bkpt) -> cpu::InstrStatus {
    let brk_num = bf!(data.immed_lo) | (bf!(data.immed_hi) << 4);
    panic!("Hit breakpoint instruction! (#{})", brk_num);
}
