use cpu::{self, arm, thumb, Cpu, Version};

pub fn uxtb<V: Version>(cpu: &mut Cpu<V>, data: thumb::Uxtb::Bf) -> cpu::InstrStatus {
    assert!(V::is::<cpu::v6>());
    let arminst: u32 = 0b1110011011101111_000_00000111_000
                                          | ((data.rd.get() as u32) << 12)
                                                       | ((data.rm.get() as u32) << 0);
    cpu::instructions_arm::uxtb(cpu, arm::Uxtb::new(arminst))
}

pub fn uxth<V: Version>(_cpu: &mut Cpu<V>, _data: thumb::Uxth::Bf) -> cpu::InstrStatus {
    assert!(V::is::<cpu::v6>());
    /*let arminst: u32 = 0b1110011011111111_000_00000111_000
                                          | ((data.rd.get() as u32) << 12)
                                                       | ((data.rm.get() as u32) << 0);
    cpu::instructions_arm::uxth(cpu, arm::Uxth::new(arminst))*/
    unimplemented!()
}
