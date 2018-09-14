use cpu;
use cpu::Cpu;
use cpu::interpreter_arm as arm;
use cpu::interpreter_thumb as thumb;

pub fn ldmia(cpu: &mut Cpu, data: thumb::Ldmia::Bf) -> cpu::InstrStatus {
    // W = (rn not in register list) ? 1 : 0
    let w_bit = data.register_list.get() & (1 << data.rn.get()) == 0;
    let arminst: u32 = 0b1110100010_0_1_0000_00000000_00000000
                                    | ((w_bit as u32) << 21)
                                        | ((data.rn.get() as u32) << 16)
                                                      | ((data.register_list.get() as u32) << 0);
    cpu::instructions_arm::ldm_1(cpu, arm::Ldm1::new(arminst))
}

pub fn ldr_1(cpu: &mut Cpu, data: thumb::Ldr1::Bf) -> cpu::InstrStatus {
    let base_val = cpu.regs[data.rn.get() as usize];
    let immed_5 = data.immed_5.get() as u32;

    let addr = base_val + immed_5 * 4;
    // TODO: determine behavior based on CP15 r1 bit_U (22)
    cpu.regs[data.rd.get() as usize] = cpu.mpu.dmem_read::<u32>(addr);

    cpu::InstrStatus::InBlock
}

pub fn ldr_2(cpu: &mut Cpu, data: thumb::Ldr2::Bf) -> cpu::InstrStatus {
    let arminst: u32 = 0b111001111001_0000_0000_00000000_0000
                                      | ((data.rn.get() as u32) << 16)
                                           | ((data.rd.get() as u32) << 12)
                                                         | ((data.rm.get() as u32) << 0);
    cpu::instructions_arm::ldr(cpu, arm::Ldr::new(arminst))
}

pub fn ldr_3(cpu: &mut Cpu, data: thumb::Ldr3::Bf) -> cpu::InstrStatus {
    let immed_8 = data.immed_8.get() as u32;
    let addr = (cpu.regs[15] & 0xFFFFFFFC) + immed_8 * 4;
    cpu.regs[data.rd.get() as usize] = cpu.mpu.dmem_read::<u32>(addr);

    cpu::InstrStatus::InBlock
}

pub fn ldr_4(cpu: &mut Cpu, data: thumb::Ldr4::Bf) -> cpu::InstrStatus {
    let arminst: u32 = 0b1110010110001101_0000_00_00000000_00
                                          | ((data.rd.get() as u32) << 12)
                                                  | ((data.immed_8.get() as u32) << 2);
    cpu::instructions_arm::ldr(cpu, arm::Ldr::new(arminst))
}

pub fn ldrb_1(cpu: &mut Cpu, data: thumb::Ldrb1::Bf) -> cpu::InstrStatus {
    let arminst: u32 = 0b111001011101_0000_0000_0000000_00000
                                      | ((data.rn.get() as u32) << 16)
                                           | ((data.rd.get() as u32) << 12)
                                                        | ((data.immed_5.get() as u32) << 0);
    cpu::instructions_arm::ldrb(cpu, arm::Ldrb::new(arminst))
}

pub fn ldrb_2(cpu: &mut Cpu, data: thumb::Ldrb2::Bf) -> cpu::InstrStatus {
    let arminst: u32 = 0b111001111101_0000_0000_00000000_0000
                                      | ((data.rn.get() as u32) << 16)
                                           | ((data.rd.get() as u32) << 12)
                                                         | ((data.rm.get() as u32) << 0);
    cpu::instructions_arm::ldrb(cpu, arm::Ldrb::new(arminst))
}

pub fn ldrh_1(cpu: &mut Cpu, data: thumb::Ldrh1::Bf) -> cpu::InstrStatus {
    let arminst: u32 = 0b111000011101_0000_0000_00_00_1011_000_0
                                      | ((data.rn.get() as u32) << 16)
                                           | ((data.rd.get() as u32) << 12)
                                                   | ((data.immed_5.get() as u32 >> 3) << 8)
                                                           | ((data.immed_5.get() as u32 & 0b111) << 1);
    cpu::instructions_arm::ldrh(cpu, arm::Ldrh::new(arminst))
}

pub fn ldrh_2(cpu: &mut Cpu, data: thumb::Ldrh2::Bf) -> cpu::InstrStatus {
    let arminst: u32 = 0b111000011001_0000_0000_00001011_0000
                                      | ((data.rn.get() as u32) << 16)
                                           | ((data.rd.get() as u32) << 12)
                                                         | ((data.rm.get() as u32) << 0);
    cpu::instructions_arm::ldrh(cpu, arm::Ldrh::new(arminst))
}

pub fn ldrsb(cpu: &mut Cpu, data: thumb::Ldrsb::Bf) -> cpu::InstrStatus {
    let arminst: u32 = 0b111000011001_0000_0000_00001101_0000
                                      | ((data.rn.get() as u32) << 16)
                                           | ((data.rd.get() as u32) << 12)
                                                         | ((data.rm.get() as u32) << 0);
    cpu::instructions_arm::ldrsb(cpu, arm::Ldrsb::new(arminst))
}

pub fn ldrsh(cpu: &mut Cpu, data: thumb::Ldrsh::Bf) -> cpu::InstrStatus {
    let arminst: u32 = 0b111000011001_0000_0000_00001111_0000
                                      | ((data.rn.get() as u32) << 16)
                                           | ((data.rd.get() as u32) << 12)
                                                         | ((data.rm.get() as u32) << 0);
    cpu::instructions_arm::ldrsh(cpu, arm::Ldrsh::new(arminst))
}

pub fn pop(cpu: &mut Cpu, data: thumb::Pop::Bf) -> cpu::InstrStatus {
    let arminst: u32 = 0b1110100010111101_0_0000000_00000000
                                          | ((data.r_bit.get() as u32) << 15)
                                                    | ((data.register_list.get() as u32) << 0);
    cpu::instructions_arm::ldm_1(cpu, arm::Ldm1::new(arminst))
}

pub fn push(cpu: &mut Cpu, data: thumb::Push::Bf) -> cpu::InstrStatus {
    let arminst: u32 = 0b11101001001011010_0_000000_00000000
                                           | ((data.r_bit.get() as u32) << 14)
                                                    | ((data.register_list.get() as u32) << 0);
    cpu::instructions_arm::stm_1(cpu, arm::Stm1::new(arminst))
}

pub fn stmia(cpu: &mut Cpu, data: thumb::Stmia::Bf) -> cpu::InstrStatus {
    let arminst: u32 = 0b111010001010_0000_00000000_00000000
                                      | ((data.rn.get() as u32) << 16)
                                                    | ((data.register_list.get() as u32) << 0);
    cpu::instructions_arm::stm_1(cpu, arm::Stm1::new(arminst))
}

pub fn str_1(cpu: &mut Cpu, data: thumb::Str1::Bf) -> cpu::InstrStatus {
    let base_val = cpu.regs[data.rn.get() as usize];
    let immed_5 = data.immed_5.get() as u32;

    let addr = base_val + immed_5 * 4;
    // TODO: determine behavior based on CP15 r1 bit_U (22)
    cpu.mpu.dmem_write::<u32>(addr, cpu.regs[data.rd.get() as usize]);

    cpu::InstrStatus::InBlock
}

pub fn str_2(cpu: &mut Cpu, data: thumb::Str2::Bf) -> cpu::InstrStatus {
    let arminst: u32 = 0b111001111000_0000_0000_00000000_0000
                                      | ((data.rn.get() as u32) << 16)
                                           | ((data.rd.get() as u32) << 12)
                                                         | ((data.rm.get() as u32) << 0);
    cpu::instructions_arm::str(cpu, arm::Str::new(arminst))
}

pub fn str_3(cpu: &mut Cpu, data: thumb::Str3::Bf) -> cpu::InstrStatus {
    let arminst: u32 = 0b1110010110001101_0000_00_00000000_00
                                          | ((data.rd.get() as u32) << 12)
                                                  | ((data.immed_8.get() as u32) << 2);
    cpu::instructions_arm::str(cpu, arm::Str::new(arminst))
}

pub fn strb_1(cpu: &mut Cpu, data: thumb::Strb1::Bf) -> cpu::InstrStatus {
    let arminst: u32 = 0b111001011100_0000_0000_0000000_00000
                                      | ((data.rn.get() as u32) << 16)
                                           | ((data.rd.get() as u32) << 12)
                                                        | ((data.immed_5.get() as u32) << 0);
    cpu::instructions_arm::strb(cpu, arm::Strb::new(arminst))
}

pub fn strb_2(cpu: &mut Cpu, data: thumb::Strb2::Bf) -> cpu::InstrStatus {
    let arminst: u32 = 0b111001111100_0000_0000_00000000_0000
                                      | ((data.rn.get() as u32) << 16)
                                           | ((data.rd.get() as u32) << 12)
                                                         | ((data.rm.get() as u32) << 0);
    cpu::instructions_arm::strb(cpu, arm::Strb::new(arminst))
}

pub fn strh_1(cpu: &mut Cpu, data: thumb::Strh1::Bf) -> cpu::InstrStatus {
    let arminst: u32 = 0b111000011100_0000_0000_00_00_1011_000_0
                                      | ((data.rn.get() as u32) << 16)
                                           | ((data.rd.get() as u32) << 12)
                                                   | ((data.immed_5.get() as u32 >> 3) << 8)
                                                           | ((data.immed_5.get() as u32 & 0b111) << 1);
    cpu::instructions_arm::strh(cpu, arm::Strh::new(arminst))
}

pub fn strh_2(cpu: &mut Cpu, data: thumb::Strh2::Bf) -> cpu::InstrStatus {
    let arminst: u32 = 0b111000011000_0000_0000_00001011_0000
                                      | ((data.rn.get() as u32) << 16)
                                           | ((data.rd.get() as u32) << 12)
                                                         | ((data.rm.get() as u32) << 0);
    cpu::instructions_arm::strh(cpu, arm::Strh::new(arminst))
}
