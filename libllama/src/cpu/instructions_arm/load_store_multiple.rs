use cpu;
use cpu::Cpu;

#[inline(always)]
fn decode_addressing_mode(instr_data: &cpu::ArmInstrLoadStoreMulti, cpu: &mut Cpu) -> (u32, u32) {
    let register_list = bf!(instr_data.register_list);
    let num_registers = register_list.count_ones();

    let p_bit = bf!(instr_data.p_bit) == 1;
    let u_bit = bf!(instr_data.u_bit) == 1;
    let rn_val = cpu.regs[bf!(instr_data.rn) as usize];

    if !p_bit && u_bit {
        // Increment after
        return (rn_val, rn_val + num_registers * 4)
    } else if p_bit && u_bit {
        // Increment before
        return (rn_val + 4, rn_val + num_registers * 4)
    } else if !p_bit && !u_bit {
        // Decrement after
        return (rn_val - num_registers * 4 + 4, rn_val - num_registers * 4)
    } else if p_bit && !u_bit {
        // Decrement before
        return (rn_val - num_registers * 4, rn_val - num_registers * 4)
    } else {
        unreachable!();
    };
}

#[inline(always)]
pub fn ldm(cpu: &mut Cpu, data: cpu::ArmInstrLoadStoreMulti) -> u32 {
    if !cpu::cond_passed(bf!(data.cond), &cpu.cpsr) {
        return 4;
    }

    let (mut addr, writeback) = decode_addressing_mode(&data, cpu);
    let register_list = bf!(data.register_list);

    for i in 0..14 {
        if bit!(register_list, i) == 1 {
            cpu.regs[i] = cpu.memory.read::<u32>(addr);
            addr += 4;
        }
    }

    if bf!(data.w_bit) == 1 {
        cpu.regs[bf!(data.rn) as usize] = writeback;
    }

    if bit!(register_list, 15) == 1 {
        let val = cpu.memory.read::<u32>(addr);
        bf!((cpu.cpsr).thumb_bit = bit!(val, 0));
        cpu.branch(val & 0xFFFFFFFE);
        return 0;
    } else {
        return 4;
    }
}

#[inline(always)]
pub fn stm(cpu: &mut Cpu, data: cpu::ArmInstrLoadStoreMulti) -> u32 {
    if !cpu::cond_passed(bf!(data.cond), &cpu.cpsr) {
        return 4;
    }

    let (mut addr, writeback) = decode_addressing_mode(&data, cpu);
    let register_list = bf!(data.register_list);

    for i in 0..15 {
        if bit!(register_list, i) == 1 {
            cpu.memory.write::<u32>(addr, cpu.regs[i]);
            addr += 4;
        }
    }

    if bf!(data.w_bit) == 1 {
        cpu.regs[bf!(data.rn) as usize] = writeback;
    }

    4
}
