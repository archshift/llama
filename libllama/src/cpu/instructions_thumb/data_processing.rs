use cpu;
use cpu::Cpu;
use cpu::interpreter_arm as arm;
use cpu::interpreter_thumb as thumb;

enum ProcessInstrBitOp {
    And,
    AndNot,
    Or,
    Xor,
}

fn instr_bitwise(cpu: &mut Cpu, data: thumb::And, op: ProcessInstrBitOp) -> cpu::InstrStatus {
    let base_val = cpu.regs[bf!(data.rd) as usize];
    let rm = cpu.regs[bf!(data.rm) as usize];

    let val = match op {
        ProcessInstrBitOp::And => base_val & rm,
        ProcessInstrBitOp::AndNot => base_val & !rm,
        ProcessInstrBitOp::Or => base_val | rm,
        ProcessInstrBitOp::Xor => base_val ^ rm,
    };

    bf!((cpu.cpsr).n_bit = bit!(val, 31));
    bf!((cpu.cpsr).z_bit = (val == 0) as u32);
    cpu.regs[bf!(data.rd) as usize] = val;

    cpu::InstrStatus::InBlock
}

pub fn adc(cpu: &mut Cpu, data: thumb::Adc) -> cpu::InstrStatus {
    let arminst: u32 = 0b111000001011_0000_0000_00000000_0000
                                      | ((bf!(data.rd) as u32) << 16)
                                           | ((bf!(data.rd) as u32) << 12)
                                                         | ((bf!(data.rm) as u32) << 0);
    cpu::instructions_arm::adc(cpu, arm::Adc::new(arminst))
}

pub fn add_1(cpu: &mut Cpu, data: thumb::Add1) -> cpu::InstrStatus {
    let arminst: u32 = 0b111000101001_0000_0000_000000000_000
                                      | ((bf!(data.rn) as u32) << 16)
                                           | ((bf!(data.rd) as u32) << 12)
                                                          | ((bf!(data.immed_3) as u32) << 0);
    cpu::instructions_arm::add(cpu, arm::Add::new(arminst))
}

pub fn add_2(cpu: &mut Cpu, data: thumb::Add2) -> cpu::InstrStatus {
    let arminst: u32 = 0b111000101001_0000_0000_0000_00000000
                                      | ((bf!(data.rd) as u32) << 16)
                                           | ((bf!(data.rd) as u32) << 12)
                                                     | ((bf!(data.immed_8) as u32) << 0);
    cpu::instructions_arm::add(cpu, arm::Add::new(arminst))
}

pub fn add_3(cpu: &mut Cpu, data: thumb::Add3) -> cpu::InstrStatus {
    let arminst: u32 = 0b111000001001_0000_0000_00000000_0000
                                      | ((bf!(data.rn) as u32) << 16)
                                           | ((bf!(data.rd) as u32) << 12)
                                                         | ((bf!(data.rm) as u32) << 0);
    cpu::instructions_arm::add(cpu, arm::Add::new(arminst))
}

pub fn add_4(cpu: &mut Cpu, data: thumb::Add4) -> cpu::InstrStatus {
    let arminst: u32 = 0b111000001000_0_000_0_000_00000000_0_000
                                      | ((bf!(data.h1) as u32) << 19)
                                        | ((bf!(data.rd) as u32) << 16)
                                            | ((bf!(data.h1) as u32) << 15)
                                              | ((bf!(data.rd) as u32) << 12)
                                                           | ((bf!(data.h2) as u32) << 3)
                                                             | ((bf!(data.rm) as u32) << 0);
    cpu::instructions_arm::add(cpu, arm::Add::new(arminst))
}

pub fn add_5(cpu: &mut Cpu, data: thumb::Add5) -> cpu::InstrStatus {
    let arminst: u32 = 0b1110001010001111_0000_1111_00000000
                                          | ((bf!(data.rd) as u32) << 12)
                                                    | ((bf!(data.immed_8) as u32) << 0);
    cpu::instructions_arm::add(cpu, arm::Add::new(arminst))
}

pub fn add_6(cpu: &mut Cpu, data: thumb::Add6) -> cpu::InstrStatus {
    let arminst: u32 = 0b1110001010001101_0000_1111_00000000
                                          | ((bf!(data.rd) as u32) << 12)
                                                    | ((bf!(data.immed_8) as u32) << 0);
    cpu::instructions_arm::add(cpu, arm::Add::new(arminst))
}

pub fn add_7(cpu: &mut Cpu, data: thumb::Add7) -> cpu::InstrStatus {
    let arminst: u32 = 0b1110001010001101110111110_0000000
                                                   | ((bf!(data.immed_7) as u32) << 0);
    cpu::instructions_arm::add(cpu, arm::Add::new(arminst))
}

pub fn and(cpu: &mut Cpu, data: thumb::And) -> cpu::InstrStatus {
    instr_bitwise(cpu, data, ProcessInstrBitOp::And)
}

pub fn asr_1(cpu: &mut Cpu, data: thumb::Asr1) -> cpu::InstrStatus {
    let arminst: u32 = 0b1110000110110000_0000_00000_100_0000
                                          | ((bf!(data.rd) as u32) << 12)
                                               | ((bf!(data.immed_5) as u32) << 7)
                                                         | ((bf!(data.rm) as u32) << 0);
    cpu::instructions_arm::mov(cpu, arm::Mov::new(arminst))
}

pub fn asr_2(cpu: &mut Cpu, data: thumb::Asr2) -> cpu::InstrStatus {
    let arminst: u32 = 0b1110000110110000_0000_0000_0101_0000
                                          | ((bf!(data.rd) as u32) << 12)
                                               | ((bf!(data.rs) as u32) << 8)
                                                         | ((bf!(data.rd) as u32) << 0);
    cpu::instructions_arm::mov(cpu, arm::Mov::new(arminst))
}

pub fn bic(cpu: &mut Cpu, data: thumb::Bic) -> cpu::InstrStatus {
    instr_bitwise(cpu, thumb::And::new(data.raw()), ProcessInstrBitOp::AndNot)
}

pub fn cmp_1(cpu: &mut Cpu, data: thumb::Cmp1) -> cpu::InstrStatus {
    let base_val = cpu.regs[bf!(data.rn) as usize];
    let immed = bf!(data.immed_8) as u32;

    let val = base_val.wrapping_sub(immed);
    let carry_bit = !base_val.checked_sub(immed).is_none();
    let overflow_bit = (base_val as i32).checked_sub(immed as i32).is_none();

    bf!((cpu.cpsr).n_bit = bit!(val, 31));
    bf!((cpu.cpsr).z_bit = (val == 0) as u32);
    bf!((cpu.cpsr).c_bit = carry_bit as u32);
    bf!((cpu.cpsr).v_bit = overflow_bit as u32);

    cpu::InstrStatus::InBlock
}

pub fn cmp_2(cpu: &mut Cpu, data: thumb::Cmp2) -> cpu::InstrStatus {
    let arminst: u32 = 0b111000010101_0000_0000_00000000_0000
                                      | ((bf!(data.rn) as u32) << 16)
                                                         | ((bf!(data.rm) as u32) << 0);
    cpu::instructions_arm::cmp(cpu, arm::Cmp::new(arminst))
}

pub fn cmp_3(cpu: &mut Cpu, data: thumb::Cmp3) -> cpu::InstrStatus {
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

pub fn eor(cpu: &mut Cpu, data: thumb::Eor) -> cpu::InstrStatus {
    instr_bitwise(cpu, thumb::And::new(data.raw()), ProcessInstrBitOp::Xor)
}

pub fn lsl_1(cpu: &mut Cpu, data: thumb::Lsl1) -> cpu::InstrStatus {
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

pub fn lsl_2(cpu: &mut Cpu, data: thumb::Lsl2) -> cpu::InstrStatus {
    let arminst: u32 = 0b1110000110110000_0000_0000_0001_0000
                                          | ((bf!(data.rd) as u32) << 12)
                                               | ((bf!(data.rs) as u32) << 8)
                                                         | ((bf!(data.rd) as u32) << 0);
    cpu::instructions_arm::mov(cpu, arm::Mov::new(arminst))
}

pub fn lsr_1(cpu: &mut Cpu, data: thumb::Lsr1) -> cpu::InstrStatus {
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

pub fn lsr_2(cpu: &mut Cpu, data: thumb::Lsr2) -> cpu::InstrStatus {
    let arminst: u32 = 0b1110000110110000_0000_0000_0011_0000
                                          | ((bf!(data.rd) as u32) << 12)
                                               | ((bf!(data.rs) as u32) << 8)
                                                         | ((bf!(data.rd) as u32) << 0);
    cpu::instructions_arm::mov(cpu, arm::Mov::new(arminst))
}

pub fn mov_1(cpu: &mut Cpu, data: thumb::Mov1) -> cpu::InstrStatus {
    let val = bf!(data.immed_8) as u32;

    bf!((cpu.cpsr).n_bit = bit!(val, 31));
    bf!((cpu.cpsr).z_bit = (val == 0) as u32);
    cpu.regs[bf!(data.rd) as usize] = val;

    cpu::InstrStatus::InBlock
}

pub fn mov_2(cpu: &mut Cpu, data: thumb::Mov2) -> cpu::InstrStatus {
    let val = cpu.regs[bf!(data.rn) as usize];

    bf!((cpu.cpsr).n_bit = bit!(val, 31));
    bf!((cpu.cpsr).z_bit = (val == 0) as u32);
    bf!((cpu.cpsr).c_bit = 0);
    bf!((cpu.cpsr).v_bit = 0);
    cpu.regs[bf!(data.rd) as usize] = val;

    cpu::InstrStatus::InBlock
}

pub fn mov_3(cpu: &mut Cpu, data: thumb::Mov3) -> cpu::InstrStatus {
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

pub fn mul(cpu: &mut Cpu, data: thumb::Mul) -> cpu::InstrStatus {
    let rm = cpu.regs[bf!(data.rm) as usize] as u64;
    let rd = cpu.regs[bf!(data.rd) as usize] as u64;

    let val = (rm * rd) as u32;
    cpu.regs[bf!(data.rd) as usize] = val;

    bf!((cpu.cpsr).n_bit = bit!(val, 31));
    bf!((cpu.cpsr).z_bit = (val == 0) as u32);

    cpu::InstrStatus::InBlock
}

pub fn mvn(cpu: &mut Cpu, data: thumb::Mvn) -> cpu::InstrStatus {
    let arminst: u32 = 0b111000011111_0000_0000_00000000_0000
                                           | ((bf!(data.rd) as u32) << 12)
                                                         | ((bf!(data.rm) as u32) << 0);
    cpu::instructions_arm::mvn(cpu, arm::Mvn::new(arminst))
}

pub fn neg(cpu: &mut Cpu, data: thumb::Neg) -> cpu::InstrStatus {
    let arminst: u32 = 0b111000100111_0000_0000_000000000000
                                      | ((bf!(data.rm) as u32) << 16)
                                           | ((bf!(data.rd) as u32) << 12);
    cpu::instructions_arm::rsb(cpu, arm::Rsb::new(arminst))
}

pub fn orr(cpu: &mut Cpu, data: thumb::Orr) -> cpu::InstrStatus {
    instr_bitwise(cpu, thumb::And::new(data.raw()), ProcessInstrBitOp::Or)
}

pub fn ror(cpu: &mut Cpu, data: thumb::Ror) -> cpu::InstrStatus {
    let arminst: u32 = 0b1110000110110000_0000_0000_0111_0000
                                          | ((bf!(data.rd) as u32) << 12)
                                               | ((bf!(data.rs) as u32) << 8)
                                                         | ((bf!(data.rd) as u32) << 0);
    cpu::instructions_arm::mov(cpu, arm::Mov::new(arminst))
}

pub fn sbc(cpu: &mut Cpu, data: thumb::Sbc) -> cpu::InstrStatus {
    let arminst: u32 = 0b111000001101_0000_0000_00000000_0000
                                      | ((bf!(data.rd) as u32) << 16)
                                           | ((bf!(data.rd) as u32) << 12)
                                                         | ((bf!(data.rm) as u32) << 0);
    cpu::instructions_arm::sbc(cpu, arm::Sbc::new(arminst))
}

pub fn sub_1(cpu: &mut Cpu, data: thumb::Sub1) -> cpu::InstrStatus {
    let arminst: u32 = 0b111000100101_0000_0000_000000000_000
                                      | ((bf!(data.rn) as u32) << 16)
                                           | ((bf!(data.rd) as u32) << 12)
                                                          | ((bf!(data.immed_3) as u32) << 0);
    cpu::instructions_arm::sub(cpu, arm::Sub::new(arminst))
}

pub fn sub_2(cpu: &mut Cpu, data: thumb::Sub2) -> cpu::InstrStatus {
    let arminst: u32 = 0b111000100101_0000_0000_0000_00000000
                                      | ((bf!(data.rd) as u32) << 16)
                                           | ((bf!(data.rd) as u32) << 12)
                                                     | ((bf!(data.immed_8) as u32) << 0);
    cpu::instructions_arm::sub(cpu, arm::Sub::new(arminst))
}

pub fn sub_3(cpu: &mut Cpu, data: thumb::Sub3) -> cpu::InstrStatus {
    let arminst: u32 = 0b111000000101_0000_0000_00000000_0000
                                      | ((bf!(data.rn) as u32) << 16)
                                           | ((bf!(data.rd) as u32) << 12)
                                                         | ((bf!(data.rm) as u32) << 0);
    cpu::instructions_arm::sub(cpu, arm::Sub::new(arminst))
}

pub fn sub_4(cpu: &mut Cpu, data: thumb::Sub4) -> cpu::InstrStatus {
    let arminst: u32 = 0b1110001001001101110111110_0000000
                                                   | ((bf!(data.immed_7) as u32) << 0);
    cpu::instructions_arm::sub(cpu, arm::Sub::new(arminst))
}

pub fn tst(cpu: &mut Cpu, data: thumb::Tst) -> cpu::InstrStatus {
    let base_val = cpu.regs[bf!(data.rn) as usize];
    let val = base_val & cpu.regs[bf!(data.rm) as usize];

    bf!((cpu.cpsr).n_bit = bit!(val, 31));
    bf!((cpu.cpsr).z_bit = (val == 0) as u32);

    cpu::InstrStatus::InBlock
}