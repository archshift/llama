use cpu;
use cpu::Cpu;
use cpu::decoder_arm as arm;
use cpu::decoder_thumb as thumb;

enum ProcessInstrBitOp {
    AND,
    AND_NOT,
    OR,
    XOR,
}

#[inline(always)]
fn instr_bitwise(cpu: &mut Cpu, data: thumb::and::InstrDesc, op: ProcessInstrBitOp) -> cpu::InstrStatus {
    let base_val = cpu.regs[bf!(data.rd) as usize];
    let rm = cpu.regs[bf!(data.rm) as usize];

    let val = match op {
        ProcessInstrBitOp::AND => base_val & rm,
        ProcessInstrBitOp::AND_NOT => base_val & !rm,
        ProcessInstrBitOp::OR => base_val | rm,
        ProcessInstrBitOp::XOR => base_val ^ rm,
    };

    bf!((cpu.cpsr).n_bit = bit!(val, 31));
    bf!((cpu.cpsr).z_bit = (val == 0) as u32);
    cpu.regs[bf!(data.rd) as usize] = val;

    cpu::InstrStatus::InBlock
}

#[inline(always)]
pub fn add_1(cpu: &mut Cpu, data: thumb::add_1::InstrDesc) -> cpu::InstrStatus {
    let arminst: u32 = 0b111000101001_0000_0000_000000000_000
                                      | ((bf!(data.rn) as u32) << 16)
                                           | ((bf!(data.rd) as u32) << 12)
                                                          | ((bf!(data.immed_3) as u32) << 0);
    cpu::instructions_arm::add(cpu, arm::add::InstrDesc::new(arminst))
}

#[inline(always)]
pub fn add_2(cpu: &mut Cpu, data: thumb::add_2::InstrDesc) -> cpu::InstrStatus {
    let arminst: u32 = 0b111000101001_0000_0000_0000_00000000
                                      | ((bf!(data.rd) as u32) << 16)
                                           | ((bf!(data.rd) as u32) << 12)
                                                     | ((bf!(data.immed_8) as u32) << 0);
    cpu::instructions_arm::add(cpu, arm::add::InstrDesc::new(arminst))
}

#[inline(always)]
pub fn add_3(cpu: &mut Cpu, data: thumb::add_3::InstrDesc) -> cpu::InstrStatus {
    let arminst: u32 = 0b111000001001_0000_0000_00000000_0000
                                      | ((bf!(data.rn) as u32) << 16)
                                           | ((bf!(data.rd) as u32) << 12)
                                                         | ((bf!(data.rm) as u32) << 0);
    cpu::instructions_arm::add(cpu, arm::add::InstrDesc::new(arminst))
}

#[inline(always)]
pub fn add_4(cpu: &mut Cpu, data: thumb::add_4::InstrDesc) -> cpu::InstrStatus {
    let arminst: u32 = 0b111000001000_0_000_0_000_00000000_0_000
                                      | ((bf!(data.h1) as u32) << 19)
                                        | ((bf!(data.rd) as u32) << 16)
                                            | ((bf!(data.h1) as u32) << 15)
                                              | ((bf!(data.rd) as u32) << 12)
                                                           | ((bf!(data.h2) as u32) << 3)
                                                             | ((bf!(data.rm) as u32) << 0);
    cpu::instructions_arm::add(cpu, arm::add::InstrDesc::new(arminst))
}

#[inline(always)]
pub fn add_5(cpu: &mut Cpu, data: thumb::add_5::InstrDesc) -> cpu::InstrStatus {
    let arminst: u32 = 0b1110001010001111_0000_1111_00000000
                                          | ((bf!(data.rd) as u32) << 12)
                                                    | ((bf!(data.immed_8) as u32) << 0);
    cpu::instructions_arm::add(cpu, arm::add::InstrDesc::new(arminst))
}

#[inline(always)]
pub fn add_6(cpu: &mut Cpu, data: thumb::add_6::InstrDesc) -> cpu::InstrStatus {
    let arminst: u32 = 0b1110001010001101_0000_1111_00000000
                                          | ((bf!(data.rd) as u32) << 12)
                                                    | ((bf!(data.immed_8) as u32) << 0);
    cpu::instructions_arm::add(cpu, arm::add::InstrDesc::new(arminst))
}

#[inline(always)]
pub fn add_7(cpu: &mut Cpu, data: thumb::add_7::InstrDesc) -> cpu::InstrStatus {
    let arminst: u32 = 0b1110001010001101110111110_0000000
                                                   | ((bf!(data.immed_7) as u32) << 0);
    cpu::instructions_arm::add(cpu, arm::add::InstrDesc::new(arminst))
}

#[inline(always)]
pub fn and(cpu: &mut Cpu, data: thumb::and::InstrDesc) -> cpu::InstrStatus {
    instr_bitwise(cpu, data, ProcessInstrBitOp::AND)
}

#[inline(always)]
pub fn bic(cpu: &mut Cpu, data: thumb::bic::InstrDesc) -> cpu::InstrStatus {
    instr_bitwise(cpu, thumb::and::InstrDesc::new(data.raw()), ProcessInstrBitOp::AND_NOT)
}

#[inline(always)]
pub fn cmp_1(cpu: &mut Cpu, data: thumb::cmp_1::InstrDesc) -> cpu::InstrStatus {
    let base_val = cpu.regs[bf!(data.rn) as usize];
    let immed = bf!(data.immed_8) as u32;

    let val = base_val - immed;
    let carry_bit = !base_val.checked_sub(immed).is_none();
    let overflow_bit = (base_val as i32).checked_sub(immed as i32).is_none();

    bf!((cpu.cpsr).n_bit = bit!(val, 31));
    bf!((cpu.cpsr).z_bit = (val == 0) as u32);
    bf!((cpu.cpsr).c_bit = carry_bit as u32);
    bf!((cpu.cpsr).v_bit = overflow_bit as u32);

    cpu::InstrStatus::InBlock
}

#[inline(always)]
pub fn cmp_2(cpu: &mut Cpu, data: thumb::cmp_2::InstrDesc) -> cpu::InstrStatus {
    let arminst: u32 = 0b111000010101_0000_0000_00000000_0000
                                      | ((bf!(data.rn) as u32) << 16)
                                                         | ((bf!(data.rm) as u32) << 0);
    cpu::instructions_arm::cmp(cpu, arm::cmp::InstrDesc::new(arminst))
}

#[inline(always)]
pub fn cmp_3(cpu: &mut Cpu, data: thumb::cmp_3::InstrDesc) -> cpu::InstrStatus {
    let rn = bf!(data.rn) | (bf!(data.h1) << 3);
    let rm = bf!(data.rm) | (bf!(data.h2) << 3);
    let base_val = cpu.regs[rn as usize];
    let other = cpu.regs[rm as usize];

    let val = base_val - other;
    let carry_bit = !base_val.checked_sub(other).is_none();
    let overflow_bit = (base_val as i32).checked_sub(other as i32).is_none();

    bf!((cpu.cpsr).n_bit = bit!(val, 31));
    bf!((cpu.cpsr).z_bit = (val == 0) as u32);
    bf!((cpu.cpsr).c_bit = carry_bit as u32);
    bf!((cpu.cpsr).v_bit = overflow_bit as u32);

    cpu::InstrStatus::InBlock
}

#[inline(always)]
pub fn eor(cpu: &mut Cpu, data: thumb::eor::InstrDesc) -> cpu::InstrStatus {
    instr_bitwise(cpu, thumb::and::InstrDesc::new(data.raw()), ProcessInstrBitOp::XOR)
}

#[inline(always)]
pub fn lsl_1(cpu: &mut Cpu, data: thumb::lsl_1::InstrDesc) -> cpu::InstrStatus {
    let base_val = cpu.regs[bf!(data.rm) as usize];
    let amount = bf!(data.immed_5) as u32;
    let val = base_val << amount;

    if amount > 0 {
        bf!((cpu.cpsr).c_bit = bit!(base_val, 32 - (amount as usize)));
    }
    bf!((cpu.cpsr).n_bit = bit!(val, 31));
    bf!((cpu.cpsr).z_bit = (val == 0) as u32);
    cpu.regs[bf!(data.rd) as usize] = val;

    cpu::InstrStatus::InBlock
}

#[inline(always)]
pub fn lsl_2(cpu: &mut Cpu, data: thumb::lsl_2::InstrDesc) -> cpu::InstrStatus {
    let arminst: u32 = 0b1110000110110000_0000_0000_0001_0000
                                          | ((bf!(data.rd) as u32) << 12)
                                               | ((bf!(data.rs) as u32) << 8)
                                                         | ((bf!(data.rd) as u32) << 0);
    cpu::instructions_arm::mov(cpu, arm::mov::InstrDesc::new(arminst))
}

#[inline(always)]
pub fn lsr_1(cpu: &mut Cpu, data: thumb::lsr_1::InstrDesc) -> cpu::InstrStatus {
    let base_val = cpu.regs[bf!(data.rm) as usize];
    let amount = bf!(data.immed_5) as u32;

    let val = if amount == 0 {
        // LSR 32
        bf!((cpu.cpsr).c_bit = bit!(base_val, 31));
        0
    } else {
        bf!((cpu.cpsr).c_bit = bit!(base_val, (amount as usize) - 1));
        base_val >> amount
    };
    bf!((cpu.cpsr).n_bit = bit!(val, 31));
    bf!((cpu.cpsr).z_bit = (val == 0) as u32);
    cpu.regs[bf!(data.rd) as usize] = val;

    cpu::InstrStatus::InBlock
}

#[inline(always)]
pub fn mov_1(cpu: &mut Cpu, data: thumb::mov_1::InstrDesc) -> cpu::InstrStatus {
    let val = bf!(data.immed_8) as u32;

    bf!((cpu.cpsr).n_bit = bit!(val, 31));
    bf!((cpu.cpsr).z_bit = (val == 0) as u32);
    cpu.regs[bf!(data.rd) as usize] = val;

    cpu::InstrStatus::InBlock
}

#[inline(always)]
pub fn mov_2(cpu: &mut Cpu, data: thumb::mov_2::InstrDesc) -> cpu::InstrStatus {
    let val = cpu.regs[bf!(data.rn) as usize];

    bf!((cpu.cpsr).n_bit = bit!(val, 31));
    bf!((cpu.cpsr).z_bit = (val == 0) as u32);
    bf!((cpu.cpsr).c_bit = 0);
    bf!((cpu.cpsr).v_bit = 0);
    cpu.regs[bf!(data.rd) as usize] = val;

    cpu::InstrStatus::InBlock
}

#[inline(always)]
pub fn mov_3(cpu: &mut Cpu, data: thumb::mov_3::InstrDesc) -> cpu::InstrStatus {
    let rd = bf!(data.rd) | (bf!(data.h1) << 3);
    let rm = bf!(data.rm) | (bf!(data.h2) << 3);
    let base_val = cpu.regs[rm as usize];

    if rd == 15 {
        cpu.branch(base_val);
        return cpu::InstrStatus::Branched
    }

    cpu.regs[rd as usize] = base_val;
    cpu::InstrStatus::InBlock
}

#[inline(always)]
pub fn mul(cpu: &mut Cpu, data: thumb::mul::InstrDesc) -> cpu::InstrStatus {
    let rm = cpu.regs[bf!(data.rm) as usize] as u64;
    let rd = cpu.regs[bf!(data.rd) as usize] as u64;

    let val = (rm * rd) as u32;
    cpu.regs[bf!(data.rd) as usize] = val;

    bf!((cpu.cpsr).n_bit = bit!(val, 31));
    bf!((cpu.cpsr).z_bit = (val == 0) as u32);

    cpu::InstrStatus::InBlock
}

#[inline(always)]
pub fn neg(cpu: &mut Cpu, data: thumb::neg::InstrDesc) -> cpu::InstrStatus {
    let arminst: u32 = 0b111000100111_0000_0000_000000000000
                                      | ((bf!(data.rm) as u32) << 16)
                                           | ((bf!(data.rd) as u32) << 12);
    cpu::instructions_arm::rsb(cpu, arm::rsb::InstrDesc::new(arminst))
}

#[inline(always)]
pub fn orr(cpu: &mut Cpu, data: thumb::orr::InstrDesc) -> cpu::InstrStatus {
    instr_bitwise(cpu, thumb::and::InstrDesc::new(data.raw()), ProcessInstrBitOp::OR)
}

#[inline(always)]
pub fn sbc(cpu: &mut Cpu, data: thumb::sbc::InstrDesc) -> cpu::InstrStatus {
    let arminst: u32 = 0b111000001101_0000_0000_00000000_0000
                                      | ((bf!(data.rd) as u32) << 16)
                                           | ((bf!(data.rd) as u32) << 12)
                                                         | ((bf!(data.rm) as u32) << 0);
    cpu::instructions_arm::sbc(cpu, arm::sbc::InstrDesc::new(arminst))
}

#[inline(always)]
pub fn sub_1(cpu: &mut Cpu, data: thumb::sub_1::InstrDesc) -> cpu::InstrStatus {
    let arminst: u32 = 0b111000100101_0000_0000_000000000_000
                                      | ((bf!(data.rn) as u32) << 16)
                                           | ((bf!(data.rd) as u32) << 12)
                                                          | ((bf!(data.immed_3) as u32) << 0);
    cpu::instructions_arm::sub(cpu, arm::sub::InstrDesc::new(arminst))
}

#[inline(always)]
pub fn sub_2(cpu: &mut Cpu, data: thumb::sub_2::InstrDesc) -> cpu::InstrStatus {
    let arminst: u32 = 0b111000100101_0000_0000_0000_00000000
                                      | ((bf!(data.rd) as u32) << 16)
                                           | ((bf!(data.rd) as u32) << 12)
                                                     | ((bf!(data.immed_8) as u32) << 0);
    cpu::instructions_arm::sub(cpu, arm::sub::InstrDesc::new(arminst))
}

#[inline(always)]
pub fn sub_3(cpu: &mut Cpu, data: thumb::sub_3::InstrDesc) -> cpu::InstrStatus {
    let arminst: u32 = 0b111000000101_0000_0000_00000000_0000
                                      | ((bf!(data.rn) as u32) << 16)
                                           | ((bf!(data.rd) as u32) << 12)
                                                         | ((bf!(data.rm) as u32) << 0);
    cpu::instructions_arm::sub(cpu, arm::sub::InstrDesc::new(arminst))
}

#[inline(always)]
pub fn sub_4(cpu: &mut Cpu, data: thumb::sub_4::InstrDesc) -> cpu::InstrStatus {
    let arminst: u32 = 0b1110001001001101110111110_0000000
                                                   | ((bf!(data.immed_7) as u32) << 0);
    cpu::instructions_arm::sub(cpu, arm::sub::InstrDesc::new(arminst))
}

#[inline(always)]
pub fn tst(cpu: &mut Cpu, data: thumb::tst::InstrDesc) -> cpu::InstrStatus {
    let base_val = cpu.regs[bf!(data.rn) as usize];
    let val = base_val & cpu.regs[bf!(data.rm) as usize];

    bf!((cpu.cpsr).n_bit = bit!(val, 31));
    bf!((cpu.cpsr).z_bit = (val == 0) as u32);

    cpu::InstrStatus::InBlock
}