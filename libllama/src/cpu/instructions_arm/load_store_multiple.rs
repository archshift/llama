use cpu;
use cpu::Cpu;
use cpu::interpreter_arm as arm;

fn decode_addressing_mode(instr_data: u32, cpu: &mut Cpu) -> (u32, u32) {
    let instr_data = arm::Ldm1::new(instr_data);

    let register_list = instr_data.register_list.get();
    let num_registers = register_list.count_ones();

    let p_bit = instr_data.p_bit.get() == 1;
    let u_bit = instr_data.u_bit.get() == 1;
    let rn_val = cpu.regs[instr_data.rn.get() as usize];

    let (addr, wb) = match (p_bit, u_bit) {
        (false, true)  => (rn_val, rn_val + num_registers * 4), // Increment after
        (true, true)   => (rn_val + 4, rn_val + num_registers * 4), // Increment before
        (false, false) => (rn_val - num_registers * 4 + 4, rn_val - num_registers * 4), // Decrement after
        (true, false)  => (rn_val - num_registers * 4, rn_val - num_registers * 4) // Decrement before
    };

    if instr_data.w_bit.get() == 0 {
        (addr, addr)
    } else {
        (addr, wb)
    }
}

pub fn ldm_1(cpu: &mut Cpu, data: arm::Ldm1::Bf) -> cpu::InstrStatus {
    if !cpu::cond_passed(data.cond.get(), &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    let (mut addr, writeback) = decode_addressing_mode(data.val, cpu);
    let register_list = data.register_list.get();

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

pub fn ldm_2(cpu: &mut Cpu, data: arm::Ldm2::Bf) -> cpu::InstrStatus {
    if !cpu::cond_passed(data.cond.get(), &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    let (mut addr, _) = decode_addressing_mode(data.val, cpu);
    let register_list = data.register_list.get();

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

pub fn ldm_3(cpu: &mut Cpu, data: arm::Ldm3::Bf) -> cpu::InstrStatus {
    if !cpu::cond_passed(data.cond.get(), &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    let (mut addr, writeback) = decode_addressing_mode(data.val, cpu);
    let register_list = data.register_list.get();

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

pub fn stm_1(cpu: &mut Cpu, data: arm::Stm1::Bf) -> cpu::InstrStatus {
    if !cpu::cond_passed(data.cond.get(), &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    let (mut addr, writeback) = decode_addressing_mode(data.val, cpu);
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

pub fn stm_2(cpu: &mut Cpu, data: arm::Stm2::Bf) -> cpu::InstrStatus {
    if !cpu::cond_passed(data.cond.get(), &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    let (mut addr, _) = decode_addressing_mode(data.val, cpu);
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
