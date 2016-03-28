use cpu;
use cpu::Cpu;

pub fn ldr_1(cpu: &mut Cpu, data: cpu::ThumbInstrLoadStore_1) -> u32 {
    use cpu::ThumbInstrLoadStore_1 as ThumbInstr;

    let base_val = cpu.regs[data.get(ThumbInstr::rn()) as usize];
    let immed_5 = data.get(ThumbInstr::immed_5()) as u32;

    let addr = base_val + immed_5 * 4;
    // TODO: determine behavior based on CP15 r1 bit_U (22)
    cpu.regs[data.get(ThumbInstr::rd()) as usize] = cpu.memory.read::<u32>(addr);

    2
}

pub fn ldr_3(cpu: &mut Cpu, data: cpu::ThumbInstrLoadStore_3) -> u32 {
    use cpu::ThumbInstrLoadStore_3 as ThumbInstr;

    let immed_8 = data.get(ThumbInstr::immed_8()) as u32;
    let addr = (cpu.regs[15] & 0xFFFFFFFC) + immed_8 * 4;
    cpu.regs[data.get(ThumbInstr::rd()) as usize] = cpu.memory.read::<u32>(addr);

    2
}

pub fn pop(cpu: &mut Cpu, data: cpu::ThumbInstrPOP) -> u32 {
    use cpu::ThumbInstrPUSH as ThumbInstr;

    let register_list = data.get(ThumbInstr::register_list());
    let r_bit = data.get(ThumbInstr::r_bit());

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

        cpu.cpsr.set(cpu::Psr::thumb_bit(), bit!(val, 0));
        cpu.branch(val & 0xFFFFFFFC);

        0
    } else {
        2
    };

    cpu.regs[13] = addr;
    ret
}

pub fn push(cpu: &mut Cpu, data: cpu::ThumbInstrPUSH) -> u32 {
    use cpu::ThumbInstrPUSH as ThumbInstr;

    let register_list = data.get(ThumbInstr::register_list());
    let r_bit = data.get(ThumbInstr::r_bit());

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
    use cpu::ThumbInstrLoadStore_1 as ThumbInstr;

    let base_val = cpu.regs[data.get(ThumbInstr::rn()) as usize];
    let immed_5 = data.get(ThumbInstr::immed_5()) as u32;

    let addr = base_val + immed_5 * 4;
    // TODO: determine behavior based on CP15 r1 bit_U (22)
    cpu.memory.write::<u32>(addr, cpu.regs[data.get(ThumbInstr::rd()) as usize]);

    2
}
