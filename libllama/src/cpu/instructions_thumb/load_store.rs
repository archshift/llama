use cpu;
use cpu::Cpu;
use cpu::decoder_arm as arm;
use cpu::decoder_thumb as thumb;

pub fn ldmia(cpu: &mut Cpu, data: thumb::ldmia::InstrDesc) -> cpu::InstrStatus {
    // W = (rn not in register list) ? 1 : 0
    let w_bit = bf!(data.register_list) & (1 << bf!(data.rn)) == 0;
    let arminst: u32 = 0b1110100010_0_1_0000_00000000_00000000
                                    | ((w_bit as u32) << 21)
                                        | ((bf!(data.rn) as u32) << 16)
                                                      | ((bf!(data.register_list) as u32) << 0);
    cpu::instructions_arm::ldm_1(cpu, arm::ldm_1::InstrDesc::new(arminst))
}

pub fn ldr_1(cpu: &mut Cpu, data: thumb::ldr_1::InstrDesc) -> cpu::InstrStatus {
    let base_val = cpu.regs[bf!(data.rn) as usize];
    let immed_5 = bf!(data.immed_5) as u32;

    let addr = base_val + immed_5 * 4;
    // TODO: determine behavior based on CP15 r1 bit_U (22)
    cpu.regs[bf!(data.rd) as usize] = cpu.memory.read::<u32>(addr);

    cpu::InstrStatus::InBlock
}

pub fn ldr_2(cpu: &mut Cpu, data: thumb::ldr_2::InstrDesc) -> cpu::InstrStatus {
    let arminst: u32 = 0b111001111001_0000_0000_00000000_0000
                                      | ((bf!(data.rn) as u32) << 16)
                                           | ((bf!(data.rd) as u32) << 12)
                                                         | ((bf!(data.rm) as u32) << 0);
    cpu::instructions_arm::ldr(cpu, arm::ldr::InstrDesc::new(arminst))
}

pub fn ldr_3(cpu: &mut Cpu, data: thumb::ldr_3::InstrDesc) -> cpu::InstrStatus {
    let immed_8 = bf!(data.immed_8) as u32;
    let addr = (cpu.regs[15] & 0xFFFFFFFC) + immed_8 * 4;
    cpu.regs[bf!(data.rd) as usize] = cpu.memory.read::<u32>(addr);

    cpu::InstrStatus::InBlock
}

pub fn ldr_4(cpu: &mut Cpu, data: thumb::ldr_4::InstrDesc) -> cpu::InstrStatus {
    let arminst: u32 = 0b1110010110001101_0000_00_00000000_00
                                          | ((bf!(data.rd) as u32) << 12)
                                                  | ((bf!(data.immed_8) as u32) << 2);
    cpu::instructions_arm::ldr(cpu, arm::ldr::InstrDesc::new(arminst))
}

pub fn ldrb_1(cpu: &mut Cpu, data: thumb::ldrb_1::InstrDesc) -> cpu::InstrStatus {
    let arminst: u32 = 0b111001011101_0000_0000_0000000_00000
                                      | ((bf!(data.rn) as u32) << 16)
                                           | ((bf!(data.rd) as u32) << 12)
                                                        | ((bf!(data.immed_5) as u32) << 0);
    cpu::instructions_arm::ldrb(cpu, arm::ldrb::InstrDesc::new(arminst))
}

pub fn ldrb_2(cpu: &mut Cpu, data: thumb::ldrb_2::InstrDesc) -> cpu::InstrStatus {
    let arminst: u32 = 0b111001111101_0000_0000_00000000_0000
                                      | ((bf!(data.rn) as u32) << 16)
                                           | ((bf!(data.rd) as u32) << 12)
                                                         | ((bf!(data.rm) as u32) << 0);
    cpu::instructions_arm::ldrb(cpu, arm::ldrb::InstrDesc::new(arminst))
}

pub fn ldrh_1(cpu: &mut Cpu, data: thumb::ldrh_1::InstrDesc) -> cpu::InstrStatus {
    let arminst: u32 = 0b111000011101_0000_0000_00_00_1011_000_0
                                      | ((bf!(data.rn) as u32) << 16)
                                           | ((bf!(data.rd) as u32) << 12)
                                                   | ((bf!(data.immed_5) as u32 >> 3) << 8)
                                                           | ((bf!(data.immed_5) as u32 & 0b111) << 1);
    cpu::instructions_arm::ldrh(cpu, arm::ldrh::InstrDesc::new(arminst))
}

pub fn ldrh_2(cpu: &mut Cpu, data: thumb::ldrh_2::InstrDesc) -> cpu::InstrStatus {
    let arminst: u32 = 0b111000011001_0000_0000_00001011_0000
                                      | ((bf!(data.rn) as u32) << 16)
                                           | ((bf!(data.rd) as u32) << 12)
                                                         | ((bf!(data.rm) as u32) << 0);
    cpu::instructions_arm::ldrh(cpu, arm::ldrh::InstrDesc::new(arminst))
}

pub fn ldrsb(cpu: &mut Cpu, data: thumb::ldrsb::InstrDesc) -> cpu::InstrStatus {
    let arminst: u32 = 0b111000011001_0000_0000_00001101_0000
                                      | ((bf!(data.rn) as u32) << 16)
                                           | ((bf!(data.rd) as u32) << 12)
                                                         | ((bf!(data.rm) as u32) << 0);
    cpu::instructions_arm::ldrsb(cpu, arm::ldrsb::InstrDesc::new(arminst))
}

pub fn ldrsh(cpu: &mut Cpu, data: thumb::ldrsh::InstrDesc) -> cpu::InstrStatus {
    let arminst: u32 = 0b111000011001_0000_0000_00001111_0000
                                      | ((bf!(data.rn) as u32) << 16)
                                           | ((bf!(data.rd) as u32) << 12)
                                                         | ((bf!(data.rm) as u32) << 0);
    cpu::instructions_arm::ldrsh(cpu, arm::ldrsh::InstrDesc::new(arminst))
}

pub fn pop(cpu: &mut Cpu, data: thumb::pop::InstrDesc) -> cpu::InstrStatus {
    let arminst: u32 = 0b1110100010111101_0_0000000_00000000
                                          | ((bf!(data.r_bit) as u32) << 15)
                                                    | ((bf!(data.register_list) as u32) << 0);
    cpu::instructions_arm::ldm_1(cpu, arm::ldm_1::InstrDesc::new(arminst))
}

pub fn push(cpu: &mut Cpu, data: thumb::push::InstrDesc) -> cpu::InstrStatus {
    let arminst: u32 = 0b11101001001011010_0_000000_00000000
                                           | ((bf!(data.r_bit) as u32) << 14)
                                                    | ((bf!(data.register_list) as u32) << 0);
    cpu::instructions_arm::stm_1(cpu, arm::stm_1::InstrDesc::new(arminst))
}

pub fn stmia(cpu: &mut Cpu, data: thumb::stmia::InstrDesc) -> cpu::InstrStatus {
    let arminst: u32 = 0b111010001010_0000_00000000_00000000
                                      | ((bf!(data.rn) as u32) << 16)
                                                    | ((bf!(data.register_list) as u32) << 0);
    cpu::instructions_arm::stm_1(cpu, arm::stm_1::InstrDesc::new(arminst))
}

pub fn str_1(cpu: &mut Cpu, data: thumb::str_1::InstrDesc) -> cpu::InstrStatus {
    let base_val = cpu.regs[bf!(data.rn) as usize];
    let immed_5 = bf!(data.immed_5) as u32;

    let addr = base_val + immed_5 * 4;
    // TODO: determine behavior based on CP15 r1 bit_U (22)
    cpu.memory.write::<u32>(addr, cpu.regs[bf!(data.rd) as usize]);

    cpu::InstrStatus::InBlock
}

pub fn str_2(cpu: &mut Cpu, data: thumb::str_2::InstrDesc) -> cpu::InstrStatus {
    let arminst: u32 = 0b111001111000_0000_0000_00000000_0000
                                      | ((bf!(data.rn) as u32) << 16)
                                           | ((bf!(data.rd) as u32) << 12)
                                                         | ((bf!(data.rm) as u32) << 0);
    cpu::instructions_arm::str(cpu, arm::str::InstrDesc::new(arminst))
}

pub fn str_3(cpu: &mut Cpu, data: thumb::str_3::InstrDesc) -> cpu::InstrStatus {
    let arminst: u32 = 0b1110010110001101_0000_00_00000000_00
                                          | ((bf!(data.rd) as u32) << 12)
                                                  | ((bf!(data.immed_8) as u32) << 2);
    cpu::instructions_arm::str(cpu, arm::str::InstrDesc::new(arminst))
}

pub fn strb_1(cpu: &mut Cpu, data: thumb::strb_1::InstrDesc) -> cpu::InstrStatus {
    let arminst: u32 = 0b111001011100_0000_0000_0000000_00000
                                      | ((bf!(data.rn) as u32) << 16)
                                           | ((bf!(data.rd) as u32) << 12)
                                                        | ((bf!(data.immed_5) as u32) << 0);
    cpu::instructions_arm::strb(cpu, arm::strb::InstrDesc::new(arminst))
}

pub fn strb_2(cpu: &mut Cpu, data: thumb::strb_2::InstrDesc) -> cpu::InstrStatus {
    let arminst: u32 = 0b111001111100_0000_0000_00000000_0000
                                      | ((bf!(data.rn) as u32) << 16)
                                           | ((bf!(data.rd) as u32) << 12)
                                                         | ((bf!(data.rm) as u32) << 0);
    cpu::instructions_arm::strb(cpu, arm::strb::InstrDesc::new(arminst))
}

pub fn strh_1(cpu: &mut Cpu, data: thumb::strh_1::InstrDesc) -> cpu::InstrStatus {
    let arminst: u32 = 0b111000011100_0000_0000_00_00_1011_000_0
                                      | ((bf!(data.rn) as u32) << 16)
                                           | ((bf!(data.rd) as u32) << 12)
                                                   | ((bf!(data.immed_5) as u32 >> 3) << 8)
                                                           | ((bf!(data.immed_5) as u32 & 0b111) << 1);
    cpu::instructions_arm::strh(cpu, arm::strh::InstrDesc::new(arminst))
}

pub fn strh_2(cpu: &mut Cpu, data: thumb::strh_2::InstrDesc) -> cpu::InstrStatus {
    let arminst: u32 = 0b111000011000_0000_0000_00001011_0000
                                      | ((bf!(data.rn) as u32) << 16)
                                           | ((bf!(data.rd) as u32) << 12)
                                                         | ((bf!(data.rm) as u32) << 0);
    cpu::instructions_arm::strh(cpu, arm::strh::InstrDesc::new(arminst))
}
