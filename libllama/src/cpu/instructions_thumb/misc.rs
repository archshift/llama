use cpu::{self, arm, thumb, Cpu};

pub fn bkpt(cpu: &mut Cpu, data: thumb::Bkpt::Bf) -> cpu::InstrStatus {
    let immed_lo = data.immed_8.get() as u32 & 0b1111;
    let immed_hi = data.immed_8.get() as u32 >> 4;
    let arminst: u32 = 0b11100001001000000000_0000_0111_0000
                                              | (immed_hi << 8)
                                                        | (immed_lo << 0);
    cpu::instructions_arm::bkpt(cpu, arm::Bkpt::new(arminst))
}

pub fn swi(cpu: &mut Cpu, data: thumb::Swi::Bf) -> cpu::InstrStatus {
    let arminst: u32 = 0b111011110000000000000000_00000000
                                                  | ((data.immed_8.get() as u32) << 0);
    cpu::instructions_arm::swi(cpu, arm::Swi::new(arminst))
}

