use cpu;
use cpu::Cpu;
use cpu::interpreter_arm as arm;

pub fn swi(cpu: &mut Cpu, data: arm::Swi::Bf) -> cpu::InstrStatus {
    if !cpu::cond_passed(data.cond.get(), &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    let next_instr = cpu.regs[15] - cpu.get_pc_offset() / 2;
    cpu.enter_exception(next_instr, cpu::Mode::Svc);
    cpu::InstrStatus::Branched
}

pub fn bkpt(_cpu: &mut Cpu, data: arm::Bkpt::Bf) -> cpu::InstrStatus {
    let brk_num = data.immed_lo.get() | (data.immed_hi.get() << 4);
    panic!("Hit breakpoint instruction! (#{})", brk_num);
}
