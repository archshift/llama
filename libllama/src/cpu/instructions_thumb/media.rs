use cpu::{self, arm, thumb, Cpu, Version};

pub fn rev<V: Version>(cpu: &mut Cpu<V>, data: thumb::Rev::Bf) -> cpu::InstrStatus {
    assert!(V::is::<cpu::v6>());
    let arminst: u32 = 0b1110011010111111_0000_11110011_0000
                                          | ((data.rd.get() as u32) << 12)
                                                        | ((data.rn.get() as u32) << 0);
    cpu::instructions_arm::rev(cpu, arm::Rev::new(arminst))
}

pub fn uxtb<V: Version>(cpu: &mut Cpu<V>, data: thumb::Uxtb::Bf) -> cpu::InstrStatus {
    assert!(V::is::<cpu::v6>());
    let arminst: u32 = 0b1110011011101111_0000_00000111_0000
                                          | ((data.rd.get() as u32) << 12)
                                                        | ((data.rm.get() as u32) << 0);
    cpu::instructions_arm::uxtb(cpu, arm::Uxtb::new(arminst))
}

pub fn uxth<V: Version>(cpu: &mut Cpu<V>, data: thumb::Uxth::Bf) -> cpu::InstrStatus {
    assert!(V::is::<cpu::v6>());
    let arminst: u32 = 0b1110011011111111_0000_00000111_0000
                                          | ((data.rd.get() as u32) << 12)
                                                        | ((data.rm.get() as u32) << 0);
    cpu::instructions_arm::uxth(cpu, arm::Uxth::new(arminst))
}
