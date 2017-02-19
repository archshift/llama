use cpu;
use cpu::Cpu;
use cpu::decoder_arm as arm;

#[inline(always)]
pub fn swi(cpu: &mut Cpu, data: arm::swi::InstrDesc) -> cpu::InstrStatus {
    if !cpu::cond_passed(bf!(data.cond), &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    let ret_addr = cpu.regs[15] - 4;

    cpu.spsr_svc = cpu.cpsr;
    bf!((cpu.cpsr).mode = cpu::Mode::Svc as u32);
    cpu.regs.swap(cpu::Mode::Svc);
    cpu.regs[14] = ret_addr;

    bf!((cpu.cpsr).thumb_bit = 0);
    bf!((cpu.cpsr).disable_irq_bit = 1);
    cpu.branch(0x08000010); // This is where bootrom points the SWI vector
    cpu::InstrStatus::Branched
}