use cpu::{self, Cpu, Version};
use cpu::interpreter_arm as arm;

pub fn uxtb<V: Version>(cpu: &mut Cpu<V>, data: arm::Uxtb::Bf) -> cpu::InstrStatus {
    assert!( V::is::<cpu::v6>() );
    if !cpu::cond_passed(data.cond.get(), &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    let rm = cpu.regs[data.rm.get() as usize];
    let rot = 8 * data.rot.get() as usize;

    let val = (rm >> rot) & 0xFF;
    cpu.regs[data.rd.get() as usize] = val;

    cpu::InstrStatus::InBlock
}

pub fn uxth<V: Version>(cpu: &mut Cpu<V>, data: arm::Uxth::Bf) -> cpu::InstrStatus {
    assert!( V::is::<cpu::v6>() );
    if !cpu::cond_passed(data.cond.get(), &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    let rm = cpu.regs[data.rm.get() as usize];
    let rot = 8 * data.rot.get();

    let val = rm.rotate_right(rot) & 0xFFFF;
    cpu.regs[data.rd.get() as usize] = val;

    cpu::InstrStatus::InBlock
}
