use cpu;
use cpu::Cpu;
use cpu::decoder_arm as arm;

#[inline(always)]
fn decode_addressing_mode(instr_data: u32, cpu: &mut Cpu) -> (u32, u32) {
    let instr_data = arm::ldm_1::InstrDesc::new(instr_data);

    let register_list = bf!(instr_data.register_list);
    let num_registers = register_list.count_ones();

    let p_bit = bf!(instr_data.p_bit) == 1;
    let u_bit = bf!(instr_data.u_bit) == 1;
    let rn_val = cpu.regs[bf!(instr_data.rn) as usize];

    match (p_bit, u_bit) {
        (false, true)  => (rn_val, rn_val + num_registers * 4), // Increment after
        (true, true)   => (rn_val + 4, rn_val + num_registers * 4), // Increment before
        (false, false) => (rn_val - num_registers * 4 + 4, rn_val - num_registers * 4), // Decrement after
        (true, false)  => (rn_val - num_registers * 4, rn_val - num_registers * 4) // Decrement before
    }
}

#[inline(always)]
pub fn instr_ldm(cpu: &mut Cpu, data: arm::ldm_1::InstrDesc) -> cpu::InstrStatus {
    if !cpu::cond_passed(bf!(data.cond), &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    let (mut addr, writeback) = decode_addressing_mode(data.raw(), cpu);
    let register_list = bf!(data.register_list);

    for i in 0..15 {
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
        return cpu::InstrStatus::Branched;
    } else {
        return cpu::InstrStatus::InBlock;
    }
}

#[inline(always)]
pub fn instr_stm(cpu: &mut Cpu, data: arm::stm_1::InstrDesc) -> cpu::InstrStatus {
    if !cpu::cond_passed(bf!(data.cond), &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    let (mut addr, writeback) = decode_addressing_mode(data.raw(), cpu);
    let register_list = bf!(data.register_list);

    for i in 0..16 {
        if bit!(register_list, i) == 1 {
            cpu.memory.write::<u32>(addr, cpu.regs[i]);
            addr += 4;
        }
    }

    if bf!(data.w_bit) == 1 {
        cpu.regs[bf!(data.rn) as usize] = writeback;
    }

    cpu::InstrStatus::InBlock
}

#[inline(always)]
pub fn ldm_1(cpu: &mut Cpu, data: arm::ldm_1::InstrDesc) -> cpu::InstrStatus {
    instr_ldm(cpu, data)
}

#[inline(always)]
pub fn ldm_2(cpu: &mut Cpu, data: arm::ldm_2::InstrDesc) -> cpu::InstrStatus {
    instr_ldm(cpu, arm::ldm_1::InstrDesc::new(data.raw()))
}

#[inline(always)]
pub fn ldm_3(cpu: &mut Cpu, data: arm::ldm_3::InstrDesc) -> cpu::InstrStatus {
    instr_ldm(cpu, arm::ldm_1::InstrDesc::new(data.raw()))
}

#[inline(always)]
pub fn stm_1(cpu: &mut Cpu, data: arm::stm_1::InstrDesc) -> cpu::InstrStatus {
    instr_stm(cpu, data)
}

#[inline(always)]
pub fn stm_2(cpu: &mut Cpu, data: arm::stm_2::InstrDesc) -> cpu::InstrStatus {
    instr_stm(cpu, arm::stm_1::InstrDesc::new(data.raw()))
}