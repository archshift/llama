use cpu;
use cpu::Cpu;

pub fn ldr_1(cpu: &mut Cpu, data: cpu::ThumbInstrLoadStore_1) -> u32 {
    let base_val = cpu.regs[bf!(data.rn) as usize];
    let immed_5 = bf!(data.immed_5) as u32;

    let addr = base_val + immed_5 * 4;
    // TODO: determine behavior based on CP15 r1 bit_U (22)
    cpu.regs[bf!(data.rd) as usize] = cpu.memory.read::<u32>(addr);

    2
}

pub fn ldr_3(cpu: &mut Cpu, data: cpu::ThumbInstrLoadStore_3) -> u32 {
    let immed_8 = bf!(data.immed_8) as u32;
    let addr = (cpu.regs[15] & 0xFFFFFFFC) + immed_8 * 4;
    cpu.regs[bf!(data.rd) as usize] = cpu.memory.read::<u32>(addr);

    2
}

pub fn ldrh_1(cpu: &mut Cpu, data: cpu::ThumbInstrLoadStore_1) -> u32 {
    let base_val = cpu.regs[bf!(data.rn) as usize];
    let immed_5 = bf!(data.immed_5) as u32;

    let addr = base_val + immed_5 * 2;
    // TODO: determine behavior based on CP15 r1 bit_U (22)
    cpu.regs[bf!(data.rd) as usize] = cpu.memory.read::<u16>(addr) as u32;

    2
}

pub fn pop(cpu: &mut Cpu, data: cpu::ThumbInstrPOP) -> u32 {
    let register_list = bf!(data.register_list);
    let r_bit = bf!(data.r_bit);

    let mut addr = cpu.regs[13];

    for i in 0..7 {
        if bit!(register_list, i) == 1 {
            cpu.regs[i] = cpu.memory.read::<u32>(addr);
            addr += 4;
        }
    }

    let ret = if r_bit == 1 {
        let val = cpu.memory.read::<u32>(addr);
        addr += 4;

        bf!((cpu.cpsr).thumb_bit = bit!(val, 0));
        cpu.branch(val & 0xFFFFFFFE);

        0
    } else {
        2
    };

    cpu.regs[13] = addr;
    ret
}

pub fn push(cpu: &mut Cpu, data: cpu::ThumbInstrPUSH) -> u32 {
    let register_list = bf!(data.register_list);
    let r_bit = bf!(data.r_bit);

    let num_registers = register_list.count_ones() + r_bit as u32;
    cpu.regs[13] -= num_registers * 4;
    let mut addr = cpu.regs[13];

    for i in 0..7 {
        if bit!(register_list, i) == 1 {
            cpu.memory.write::<u32>(addr, cpu.regs[i]);
            addr += 4;
        }
    }

    if r_bit == 1 {
        cpu.memory.write::<u32>(addr, cpu.regs[14]);
    }

    2
}

pub fn str_1(cpu: &mut Cpu, data: cpu::ThumbInstrLoadStore_1) -> u32 {
    let base_val = cpu.regs[bf!(data.rn) as usize];
    let immed_5 = bf!(data.immed_5) as u32;

    let addr = base_val + immed_5 * 4;
    // TODO: determine behavior based on CP15 r1 bit_U (22)
    cpu.memory.write::<u32>(addr, cpu.regs[bf!(data.rd) as usize]);

    2
}

pub fn strh_1(cpu: &mut Cpu, data: cpu::ThumbInstrLoadStore_1) -> u32 {
    let base_val = cpu.regs[bf!(data.rn) as usize];
    let immed_5 = bf!(data.immed_5) as u32;

    let addr = base_val + immed_5 * 2;
    // TODO: determine behavior based on CP15 r1 bit_U (22)
    cpu.memory.write::<u16>(addr, cpu.regs[bf!(data.rd) as usize] as u16);

    2
}
