use cpu::{self, Cpu, Version};
use cpu::interpreter_arm as arm;

pub fn uxtb<V: Version>(cpu: &mut Cpu<V>, data: arm::Uxtb::Bf) -> cpu::InstrStatus {
    assert!( V::is::<cpu::v6>() );
    if !cpu::cond_passed(data.cond.get(), &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    let rm = cpu.regs[data.rm.get() as usize];
    let rot = data.rot.get() as usize;

    let val = (rm >> rot) as u8 as u32;
    cpu.regs[data.rd.get() as usize] = val;

    cpu::InstrStatus::InBlock
}

