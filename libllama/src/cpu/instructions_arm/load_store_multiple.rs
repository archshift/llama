use cpu;
use cpu::{Cpu, Version};
use cpu::interpreter_arm as arm;

fn addressing_mode_inner(p_bit: bool, u_bit: bool, w_bit: bool, rn_val: u32, num_registers: u32) -> (u32, u32) {
    let (addr, wb) = match (p_bit, u_bit) {
        (false, true)  => (rn_val, rn_val + num_registers * 4), // Increment after
        (true, true)   => (rn_val + 4, rn_val + num_registers * 4), // Increment before
        (false, false) => (rn_val - num_registers * 4 + 4, rn_val - num_registers * 4), // Decrement after
        (true, false)  => (rn_val - num_registers * 4, rn_val - num_registers * 4) // Decrement before
    };

    if !w_bit {
        (addr, addr)
    } else {
        (addr, wb)
    }
}

fn decode_addressing_mode<V: Version>(instr_data: u32, cpu: &mut Cpu<V>) -> (u32, u32) {
    let instr_data = arm::Ldm1::new(instr_data);

    let register_list = instr_data.register_list.get();
    let num_registers = register_list.count_ones();

    let p_bit = instr_data.p_bit.get() == 1;
    let u_bit = instr_data.u_bit.get() == 1;
    let w_bit = instr_data.w_bit.get() == 1;
    let rn_val = cpu.regs[instr_data.rn.get() as usize];

    addressing_mode_inner(p_bit, u_bit, w_bit, rn_val, num_registers)
}

pub fn ldm_1<V: Version>(cpu: &mut Cpu<V>, data: arm::Ldm1::Bf) -> cpu::InstrStatus {
    if !cpu::cond_passed(data.cond.get(), &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    let (mut addr, writeback) = decode_addressing_mode(data.val, cpu);
    let register_list = data.register_list.get();

    // TODO: determine behavior based on CP15 r1 bit_U (22)
    assert!( V::is::<cpu::v5>() || addr % 4 == 0 );

    for i in 0..15 {
        if bit!(register_list, i) == 1 {
            cpu.regs[data.rn.get() as usize] = writeback;
            cpu.regs[i] = cpu.mpu.dmem_read::<u32>(addr);
            addr += 4;
        }
    }

    if bit!(register_list, 15) == 1 {
        cpu.regs[data.rn.get() as usize] = writeback;
        let val = cpu.mpu.dmem_read::<u32>(addr);
        cpu.cpsr.thumb_bit.set(bit!(val, 0));
        cpu.branch(val & 0xFFFFFFFE);
        return cpu::InstrStatus::Branched;
    } else {
        return cpu::InstrStatus::InBlock;
    }
}

pub fn ldm_2<V: Version>(cpu: &mut Cpu<V>, data: arm::Ldm2::Bf) -> cpu::InstrStatus {
    if !cpu::cond_passed(data.cond.get(), &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    let (mut addr, _) = decode_addressing_mode(data.val, cpu);
    let register_list = data.register_list.get();

    // TODO: determine behavior based on CP15 r1 bit_U (22)
    assert!( V::is::<cpu::v5>() || addr % 4 == 0 );

    let current_mode = cpu::Mode::from_num(cpu.cpsr.mode.get());
    cpu.regs.swap(cpu::Mode::Usr);
    for i in 0..15 {
        if bit!(register_list, i) == 1 {
            cpu.regs[i] = cpu.mpu.dmem_read::<u32>(addr);
            addr += 4;
        }
    }
    cpu.regs.swap(current_mode);

    return cpu::InstrStatus::InBlock;
}

pub fn ldm_3<V: Version>(cpu: &mut Cpu<V>, data: arm::Ldm3::Bf) -> cpu::InstrStatus {
    if !cpu::cond_passed(data.cond.get(), &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    let (mut addr, writeback) = decode_addressing_mode(data.val, cpu);
    let register_list = data.register_list.get();

    // TODO: determine behavior based on CP15 r1 bit_U (22)
    assert!( V::is::<cpu::v5>() || addr % 4 == 0 );

    for i in 0..15 {
        if bit!(register_list, i) == 1 {
            cpu.regs[data.rn.get() as usize] = writeback;
            cpu.regs[i] = cpu.mpu.dmem_read::<u32>(addr);
            addr += 4;
        }
    }

    cpu.spsr_make_current();
    let dest = cpu.mpu.dmem_read::<u32>(addr);
    cpu.branch(dest & 0xFFFFFFFE);
    cpu::InstrStatus::Branched
}

pub fn srs<V: Version>(cpu: &mut Cpu<V>, data: arm::Srs::Bf) -> cpu::InstrStatus {
    assert!( V::is::<cpu::v6>() );

    let src_lr = cpu.regs[14];
    let src_spsr = cpu.get_current_spsr().val;

    let current_mode = cpu::Mode::from_num(cpu.cpsr.mode.get());
    let dst_mode = cpu::Mode::from_num(data.mode.get());
    cpu.regs.swap(dst_mode);

    let p_bit = data.p_bit.get() == 1;
    let u_bit = data.u_bit.get() == 1;
    let w_bit = data.w_bit.get() == 1;
    let rn_val = cpu.regs[13];
    let (addr, writeback) = addressing_mode_inner(p_bit, u_bit, w_bit, rn_val, 2);

    // TODO: determine behavior based on CP15 r1 bit_U (22)
    assert!( addr % 4 == 0 );

    cpu.mpu.dmem_write::<u32>(addr, src_lr);
    cpu.mpu.dmem_write::<u32>(addr+4, src_spsr);

    if data.w_bit.get() == 1 {
        cpu.regs[13] = writeback;
    }

    cpu.regs.swap(current_mode);
    cpu::InstrStatus::InBlock
}

pub fn stm_1<V: Version>(cpu: &mut Cpu<V>, data: arm::Stm1::Bf) -> cpu::InstrStatus {
    if !cpu::cond_passed(data.cond.get(), &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    let (mut addr, writeback) = decode_addressing_mode(data.val, cpu);

    // TODO: determine behavior based on CP15 r1 bit_U (22)
    assert!( V::is::<cpu::v5>() || addr % 4 == 0 );

    let register_list = data.register_list.get();

    for i in 0..16 {
        if bit!(register_list, i) == 1 {
            cpu.mpu.dmem_write::<u32>(addr, cpu.regs[i]);
            addr += 4;
        }
    }

    if data.w_bit.get() == 1 {
        cpu.regs[data.rn.get() as usize] = writeback;
    }

    cpu::InstrStatus::InBlock
}

pub fn stm_2<V: Version>(cpu: &mut Cpu<V>, data: arm::Stm2::Bf) -> cpu::InstrStatus {
    if !cpu::cond_passed(data.cond.get(), &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    let (mut addr, _) = decode_addressing_mode(data.val, cpu);

    // TODO: determine behavior based on CP15 r1 bit_U (22)
    assert!( V::is::<cpu::v5>() || addr % 4 == 0 );

    let register_list = data.register_list.get();

    let current_mode = cpu::Mode::from_num(cpu.cpsr.mode.get());
    cpu.regs.swap(cpu::Mode::Usr);
    for i in 0..16 {
        if bit!(register_list, i) == 1 {
            cpu.mpu.dmem_write::<u32>(addr, cpu.regs[i]);
            addr += 4;
        }
    }
    cpu.regs.swap(current_mode);

    return cpu::InstrStatus::InBlock;
}
