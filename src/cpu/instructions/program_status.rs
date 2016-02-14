use cpu;
use cpu::Cpu;

#[inline(always)]
pub fn mrs(cpu: &mut Cpu, data: cpu::InstrDataMoveStatusReg::Type) -> u32 {
    use cpu::InstrDataMoveStatusReg as InstrData;

    if !cpu::cond_passed(data.get::<InstrData::cond>(), &cpu.cpsr) {
        return 4;
    }

    let rd = data.get::<InstrData::rd>();
    let r_bit = data.get::<InstrData::r_bit>();

    if r_bit == 1 {
        cpu.regs[rd as usize] = cpu.get_current_spsr().raw();
    } else {
        cpu.regs[rd as usize] = cpu.cpsr.raw();
    }

    4
}
