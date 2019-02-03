use cpu;
use cpu::{Cpu, Version, v5};
use cpu::interpreter_arm as arm;
use cpu::interpreter_thumb as thumb;

enum ProcessInstrBitOp {
    And,
    AndNot,
    Or,
    Xor,
}

fn instr_bitwise<V: Version>(cpu: &mut Cpu<V>, data: thumb::And::Bf, op: ProcessInstrBitOp) -> cpu::InstrStatus {
    let base_val = cpu.regs[data.rd.get() as usize];
    let rm = cpu.regs[data.rm.get() as usize];

    let val = match op {
        ProcessInstrBitOp::And => base_val & rm,
        ProcessInstrBitOp::AndNot => base_val & !rm,
        ProcessInstrBitOp::Or => base_val | rm,
        ProcessInstrBitOp::Xor => base_val ^ rm,
    };

    cpu.cpsr.n_bit.set(bit!(val, 31));
    cpu.cpsr.z_bit.set((val == 0) as u32);
    cpu.regs[data.rd.get() as usize] = val;

    cpu::InstrStatus::InBlock
}

pub fn adc<V: Version>(cpu: &mut Cpu<V>, data: thumb::Adc::Bf) -> cpu::InstrStatus {
    assert!(V::is::<cpu::v5>());
    let arminst: u32 = 0b111000001011_0000_0000_00000000_0000
                                      | ((data.rd.get() as u32) << 16)
                                           | ((data.rd.get() as u32) << 12)
                                                         | ((data.rm.get() as u32) << 0);
    cpu::instructions_arm::adc(cpu, arm::Adc::new(arminst))
}

pub fn add_1<V: Version>(cpu: &mut Cpu<V>, data: thumb::Add1::Bf) -> cpu::InstrStatus {
    assert!(V::is::<cpu::v5>());
    let arminst: u32 = 0b111000101001_0000_0000_000000000_000
                                      | ((data.rn.get() as u32) << 16)
                                           | ((data.rd.get() as u32) << 12)
                                                          | ((data.immed_3.get() as u32) << 0);
    cpu::instructions_arm::add(cpu, arm::Add::new(arminst))
}

pub fn add_2<V: Version>(cpu: &mut Cpu<V>, data: thumb::Add2::Bf) -> cpu::InstrStatus {
    assert!(V::is::<cpu::v5>());
    let arminst: u32 = 0b111000101001_0000_0000_0000_00000000
                                      | ((data.rd.get() as u32) << 16)
                                           | ((data.rd.get() as u32) << 12)
                                                     | ((data.immed_8.get() as u32) << 0);
    cpu::instructions_arm::add(cpu, arm::Add::new(arminst))
}

pub fn add_3<V: Version>(cpu: &mut Cpu<V>, data: thumb::Add3::Bf) -> cpu::InstrStatus {
    assert!(V::is::<cpu::v5>());
    let arminst: u32 = 0b111000001001_0000_0000_00000000_0000
                                      | ((data.rn.get() as u32) << 16)
                                           | ((data.rd.get() as u32) << 12)
                                                         | ((data.rm.get() as u32) << 0);
    cpu::instructions_arm::add(cpu, arm::Add::new(arminst))
}

pub fn add_4<V: Version>(cpu: &mut Cpu<V>, data: thumb::Add4::Bf) -> cpu::InstrStatus {
    assert!(V::is::<cpu::v5>());
    let arminst: u32 = 0b111000001000_0_000_0_000_00000000_0_000
                                      | ((data.h1.get() as u32) << 19)
                                        | ((data.rd.get() as u32) << 16)
                                            | ((data.h1.get() as u32) << 15)
                                              | ((data.rd.get() as u32) << 12)
                                                           | ((data.h2.get() as u32) << 3)
                                                             | ((data.rm.get() as u32) << 0);
    cpu::instructions_arm::add(cpu, arm::Add::new(arminst))
}

pub fn add_5<V: Version>(cpu: &mut Cpu<V>, data: thumb::Add5::Bf) -> cpu::InstrStatus {
    assert!(V::is::<cpu::v5>());
    let arminst: u32 = 0b1110001010001111_0000_1111_00000000
                                          | ((data.rd.get() as u32) << 12)
                                                    | ((data.immed_8.get() as u32) << 0);
    cpu::instructions_arm::add(cpu, arm::Add::new(arminst))
}

pub fn add_6<V: Version>(cpu: &mut Cpu<V>, data: thumb::Add6::Bf) -> cpu::InstrStatus {
    assert!(V::is::<cpu::v5>());
    let arminst: u32 = 0b1110001010001101_0000_1111_00000000
                                          | ((data.rd.get() as u32) << 12)
                                                    | ((data.immed_8.get() as u32) << 0);
    cpu::instructions_arm::add(cpu, arm::Add::new(arminst))
}

pub fn add_7<V: Version>(cpu: &mut Cpu<V>, data: thumb::Add7::Bf) -> cpu::InstrStatus {
    assert!(V::is::<cpu::v5>());
    let arminst: u32 = 0b1110001010001101110111110_0000000
                                                   | ((data.immed_7.get() as u32) << 0);
    cpu::instructions_arm::add(cpu, arm::Add::new(arminst))
}

pub fn and<V: Version>(cpu: &mut Cpu<V>, data: thumb::And::Bf) -> cpu::InstrStatus {
    assert!(V::is::<cpu::v5>());
    instr_bitwise(cpu, data, ProcessInstrBitOp::And)
}

pub fn asr_1<V: Version>(cpu: &mut Cpu<V>, data: thumb::Asr1::Bf) -> cpu::InstrStatus {
    assert!(V::is::<cpu::v5>());
    let arminst: u32 = 0b1110000110110000_0000_00000_100_0000
                                          | ((data.rd.get() as u32) << 12)
                                               | ((data.immed_5.get() as u32) << 7)
                                                         | ((data.rm.get() as u32) << 0);
    cpu::instructions_arm::mov(cpu, arm::Mov::new(arminst))
}

pub fn asr_2<V: Version>(cpu: &mut Cpu<V>, data: thumb::Asr2::Bf) -> cpu::InstrStatus {
    assert!(V::is::<cpu::v5>());
    let arminst: u32 = 0b1110000110110000_0000_0000_0101_0000
                                          | ((data.rd.get() as u32) << 12)
                                               | ((data.rs.get() as u32) << 8)
                                                         | ((data.rd.get() as u32) << 0);
    cpu::instructions_arm::mov(cpu, arm::Mov::new(arminst))
}

pub fn bic<V: Version>(cpu: &mut Cpu<V>, data: thumb::Bic::Bf) -> cpu::InstrStatus {
    assert!(V::is::<cpu::v5>());
    instr_bitwise(cpu, thumb::And::new(data.val), ProcessInstrBitOp::AndNot)
}

pub fn cmn<V: Version>(cpu: &mut Cpu<V>, data: thumb::Cmn::Bf) -> cpu::InstrStatus {
    assert!(V::is::<cpu::v5>());
    let arminst: u32 = 0b111000010111_0000_0000_00000000_0000
                                      | ((data.rn.get() as u32) << 16)
                                                         | ((data.rm.get() as u32) << 0);
    cpu::instructions_arm::cmn(cpu, arm::Cmn::new(arminst))
}

pub fn cmp_1<V: Version>(cpu: &mut Cpu<V>, data: thumb::Cmp1::Bf) -> cpu::InstrStatus {
    assert!(V::is::<cpu::v5>());
    let base_val = cpu.regs[data.rn.get() as usize];
    let immed = data.immed_8.get() as u32;

    let val = base_val.wrapping_sub(immed);
    let carry_bit = !base_val.checked_sub(immed).is_none();
    let overflow_bit = (base_val as i32).checked_sub(immed as i32).is_none();

    cpu.cpsr.n_bit.set(bit!(val, 31));
    cpu.cpsr.z_bit.set((val == 0) as u32);
    cpu.cpsr.c_bit.set(carry_bit as u32);
    cpu.cpsr.v_bit.set(overflow_bit as u32);

    cpu::InstrStatus::InBlock
}

pub fn cmp_2<V: Version>(cpu: &mut Cpu<V>, data: thumb::Cmp2::Bf) -> cpu::InstrStatus {
    assert!(V::is::<cpu::v5>());
    let arminst: u32 = 0b111000010101_0000_0000_00000000_0000
                                      | ((data.rn.get() as u32) << 16)
                                                         | ((data.rm.get() as u32) << 0);
    cpu::instructions_arm::cmp(cpu, arm::Cmp::new(arminst))
}

pub fn cmp_3<V: Version>(cpu: &mut Cpu<V>, data: thumb::Cmp3::Bf) -> cpu::InstrStatus {
    assert!(V::is::<cpu::v5>());
    let rn = data.rn.get() | (data.h1.get() << 3);
    let rm = data.rm.get() | (data.h2.get() << 3);
    let base_val = cpu.regs[rn as usize];
    let other = cpu.regs[rm as usize];

    let val = base_val - other;
    let carry_bit = !base_val.checked_sub(other).is_none();
    let overflow_bit = (base_val as i32).checked_sub(other as i32).is_none();

    cpu.cpsr.n_bit.set(bit!(val, 31));
    cpu.cpsr.z_bit.set((val == 0) as u32);
    cpu.cpsr.c_bit.set(carry_bit as u32);
    cpu.cpsr.v_bit.set(overflow_bit as u32);

    cpu::InstrStatus::InBlock
}

pub fn eor<V: Version>(cpu: &mut Cpu<V>, data: thumb::Eor::Bf) -> cpu::InstrStatus {
    assert!(V::is::<cpu::v5>());
    instr_bitwise(cpu, thumb::And::new(data.val), ProcessInstrBitOp::Xor)
}

pub fn lsl_1<V: Version>(cpu: &mut Cpu<V>, data: thumb::Lsl1::Bf) -> cpu::InstrStatus {
    assert!(V::is::<cpu::v5>());
    let base_val = cpu.regs[data.rm.get() as usize];
    let amount = data.immed_5.get() as u32;
    let val = base_val << amount;

    if amount > 0 {
        cpu.cpsr.c_bit.set(bit!(base_val, 32 - (amount as usize)));
    }
    cpu.cpsr.n_bit.set(bit!(val, 31));
    cpu.cpsr.z_bit.set((val == 0) as u32);
    cpu.regs[data.rd.get() as usize] = val;

    cpu::InstrStatus::InBlock
}

pub fn lsl_2<V: Version>(cpu: &mut Cpu<V>, data: thumb::Lsl2::Bf) -> cpu::InstrStatus {
    assert!(V::is::<cpu::v5>());
    let arminst: u32 = 0b1110000110110000_0000_0000_0001_0000
                                          | ((data.rd.get() as u32) << 12)
                                               | ((data.rs.get() as u32) << 8)
                                                         | ((data.rd.get() as u32) << 0);
    cpu::instructions_arm::mov(cpu, arm::Mov::new(arminst))
}

pub fn lsr_1<V: Version>(cpu: &mut Cpu<V>, data: thumb::Lsr1::Bf) -> cpu::InstrStatus {
    assert!(V::is::<cpu::v5>());
    let base_val = cpu.regs[data.rm.get() as usize];
    let amount = data.immed_5.get() as u32;

    let val = if amount == 0 {
        // LSR 32
        cpu.cpsr.c_bit.set(bit!(base_val, 31));
        0
    } else {
        cpu.cpsr.c_bit.set(bit!(base_val, (amount as usize) - 1));
        base_val >> amount
    };
    cpu.cpsr.n_bit.set(bit!(val, 31));
    cpu.cpsr.z_bit.set((val == 0) as u32);
    cpu.regs[data.rd.get() as usize] = val;

    cpu::InstrStatus::InBlock
}

pub fn lsr_2<V: Version>(cpu: &mut Cpu<V>, data: thumb::Lsr2::Bf) -> cpu::InstrStatus {
    assert!(V::is::<cpu::v5>());
    let arminst: u32 = 0b1110000110110000_0000_0000_0011_0000
                                          | ((data.rd.get() as u32) << 12)
                                               | ((data.rs.get() as u32) << 8)
                                                         | ((data.rd.get() as u32) << 0);
    cpu::instructions_arm::mov(cpu, arm::Mov::new(arminst))
}

pub fn mov_1<V: Version>(cpu: &mut Cpu<V>, data: thumb::Mov1::Bf) -> cpu::InstrStatus {
    assert!(V::is::<cpu::v5>());
    let val = data.immed_8.get() as u32;

    cpu.cpsr.n_bit.set(bit!(val, 31));
    cpu.cpsr.z_bit.set((val == 0) as u32);
    cpu.regs[data.rd.get() as usize] = val;

    cpu::InstrStatus::InBlock
}

pub fn mov_2<V: Version>(cpu: &mut Cpu<V>, data: thumb::Mov2::Bf) -> cpu::InstrStatus {
    assert!(V::is::<cpu::v5>());
    let val = cpu.regs[data.rn.get() as usize];

    cpu.cpsr.n_bit.set(bit!(val, 31));
    cpu.cpsr.z_bit.set((val == 0) as u32);
    cpu.cpsr.c_bit.set(0);
    cpu.cpsr.v_bit.set(0);
    cpu.regs[data.rd.get() as usize] = val;

    cpu::InstrStatus::InBlock
}

pub fn mov_3<V: Version>(cpu: &mut Cpu<V>, data: thumb::Mov3::Bf) -> cpu::InstrStatus {
    assert!(V::is::<cpu::v5>());
    let rd = data.rd.get() | (data.h1.get() << 3);
    let rm = data.rm.get() | (data.h2.get() << 3);
    let mut base_val = cpu.regs[rm as usize];

    if rd == 15 {
        // Can't figure out why the CPU does this but it's tested :/
        if V::is::<v5>() {
            base_val &= 0xFFFFFFFE;
        }
        cpu.branch(base_val);
        return cpu::InstrStatus::Branched
    }

    cpu.regs[rd as usize] = base_val;
    cpu::InstrStatus::InBlock
}

pub fn mul<V: Version>(cpu: &mut Cpu<V>, data: thumb::Mul::Bf) -> cpu::InstrStatus {
    assert!(V::is::<cpu::v5>());
    let rm = cpu.regs[data.rm.get() as usize] as u64;
    let rd = cpu.regs[data.rd.get() as usize] as u64;

    let val = (rm * rd) as u32;
    cpu.regs[data.rd.get() as usize] = val;

    cpu.cpsr.n_bit.set(bit!(val, 31));
    cpu.cpsr.z_bit.set((val == 0) as u32);

    cpu::InstrStatus::InBlock
}

pub fn mvn<V: Version>(cpu: &mut Cpu<V>, data: thumb::Mvn::Bf) -> cpu::InstrStatus {
    assert!(V::is::<cpu::v5>());
    let arminst: u32 = 0b111000011111_0000_0000_00000000_0000
                                           | ((data.rd.get() as u32) << 12)
                                                         | ((data.rm.get() as u32) << 0);
    cpu::instructions_arm::mvn(cpu, arm::Mvn::new(arminst))
}

pub fn neg<V: Version>(cpu: &mut Cpu<V>, data: thumb::Neg::Bf) -> cpu::InstrStatus {
    assert!(V::is::<cpu::v5>());
    let arminst: u32 = 0b111000100111_0000_0000_000000000000
                                      | ((data.rm.get() as u32) << 16)
                                           | ((data.rd.get() as u32) << 12);
    cpu::instructions_arm::rsb(cpu, arm::Rsb::new(arminst))
}

pub fn orr<V: Version>(cpu: &mut Cpu<V>, data: thumb::Orr::Bf) -> cpu::InstrStatus {
    assert!(V::is::<cpu::v5>());
    instr_bitwise(cpu, thumb::And::new(data.val), ProcessInstrBitOp::Or)
}

pub fn ror<V: Version>(cpu: &mut Cpu<V>, data: thumb::Ror::Bf) -> cpu::InstrStatus {
    assert!(V::is::<cpu::v5>());
    let arminst: u32 = 0b1110000110110000_0000_0000_0111_0000
                                          | ((data.rd.get() as u32) << 12)
                                               | ((data.rs.get() as u32) << 8)
                                                         | ((data.rd.get() as u32) << 0);
    cpu::instructions_arm::mov(cpu, arm::Mov::new(arminst))
}

pub fn sbc<V: Version>(cpu: &mut Cpu<V>, data: thumb::Sbc::Bf) -> cpu::InstrStatus {
    assert!(V::is::<cpu::v5>());
    let arminst: u32 = 0b111000001101_0000_0000_00000000_0000
                                      | ((data.rd.get() as u32) << 16)
                                           | ((data.rd.get() as u32) << 12)
                                                         | ((data.rm.get() as u32) << 0);
    cpu::instructions_arm::sbc(cpu, arm::Sbc::new(arminst))
}

pub fn sub_1<V: Version>(cpu: &mut Cpu<V>, data: thumb::Sub1::Bf) -> cpu::InstrStatus {
    assert!(V::is::<cpu::v5>());
    let arminst: u32 = 0b111000100101_0000_0000_000000000_000
                                      | ((data.rn.get() as u32) << 16)
                                           | ((data.rd.get() as u32) << 12)
                                                          | ((data.immed_3.get() as u32) << 0);
    cpu::instructions_arm::sub(cpu, arm::Sub::new(arminst))
}

pub fn sub_2<V: Version>(cpu: &mut Cpu<V>, data: thumb::Sub2::Bf) -> cpu::InstrStatus {
    assert!(V::is::<cpu::v5>());
    let arminst: u32 = 0b111000100101_0000_0000_0000_00000000
                                      | ((data.rd.get() as u32) << 16)
                                           | ((data.rd.get() as u32) << 12)
                                                     | ((data.immed_8.get() as u32) << 0);
    cpu::instructions_arm::sub(cpu, arm::Sub::new(arminst))
}

pub fn sub_3<V: Version>(cpu: &mut Cpu<V>, data: thumb::Sub3::Bf) -> cpu::InstrStatus {
    assert!(V::is::<cpu::v5>());
    let arminst: u32 = 0b111000000101_0000_0000_00000000_0000
                                      | ((data.rn.get() as u32) << 16)
                                           | ((data.rd.get() as u32) << 12)
                                                         | ((data.rm.get() as u32) << 0);
    cpu::instructions_arm::sub(cpu, arm::Sub::new(arminst))
}

pub fn sub_4<V: Version>(cpu: &mut Cpu<V>, data: thumb::Sub4::Bf) -> cpu::InstrStatus {
    assert!(V::is::<cpu::v5>());
    let arminst: u32 = 0b1110001001001101110111110_0000000
                                                   | ((data.immed_7.get() as u32) << 0);
    cpu::instructions_arm::sub(cpu, arm::Sub::new(arminst))
}

pub fn tst<V: Version>(cpu: &mut Cpu<V>, data: thumb::Tst::Bf) -> cpu::InstrStatus {
    assert!(V::is::<cpu::v5>());
    let base_val = cpu.regs[data.rn.get() as usize];
    let val = base_val & cpu.regs[data.rm.get() as usize];

    cpu.cpsr.n_bit.set(bit!(val, 31));
    cpu.cpsr.z_bit.set((val == 0) as u32);

    cpu::InstrStatus::InBlock
}
