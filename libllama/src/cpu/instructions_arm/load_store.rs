use cpu;
use cpu::Cpu;
use cpu::decoder_arm as arm;

#[inline(always)]
fn decode_addressing_mode(instr_data: arm::ldr::InstrDesc, cpu: &Cpu) -> u32 {
    let c_bit = bf!((cpu.cpsr).c_bit) == 1;

    let i_bit = bf!(instr_data.i_bit);
    let u_bit = bf!(instr_data.u_bit);
    let base_addr = cpu.regs[bf!(instr_data.rn) as usize];

    let offset = if i_bit == 0 {
        bits!(instr_data.raw(), 0 => 11)
    } else {
        let pre_shift = cpu.regs[bits!(instr_data.raw(), 0 => 3) as usize];

        let offset = if bits!(instr_data.raw(), 4 => 11) == 0 {
            pre_shift
        } else {
            let shift = bits!(instr_data.raw(), 5 => 6);
            let shift_imm = bits!(instr_data.raw(), 7 => 11);

            match shift {
                0b00 => pre_shift << shift_imm,
                0b01 => {
                    let index = if shift_imm == 0 {
                        0
                    } else {
                        pre_shift >> shift_imm
                    }; index
                },
                0b10 => {
                    let index = if shift_imm == 0 {
                        let index = if bit!(pre_shift, 31) == 1 {
                            0xFFFFFFFF
                        } else {
                            0
                        }; index
                    } else {
                        ((pre_shift as i32) >> shift_imm) as u32
                    }; index
                },
                0b11 => {
                    let index = if shift_imm == 0 {
                        ((c_bit as u32) << 31) | (pre_shift >> 1)
                    } else {
                        pre_shift.rotate_right(shift_imm)
                    }; index
                }
                _ => {
                    panic!("Unhandled shifter operation!");
                }
            }
        }; offset
    };

    if u_bit == 1 {
        return base_addr + offset;
    } else {
        return base_addr - offset;
    }
}

#[inline(always)]
fn decode_addr_and_writeback(instr_data: u32, cpu: &Cpu) -> (u32, u32) {
    // For convenience
    let instr_data = arm::ldr::InstrDesc::new(instr_data);

    let base_addr = cpu.regs[bf!(instr_data.rn) as usize];
    let mod_addr = decode_addressing_mode(instr_data, cpu);

    if bf!(instr_data.p_bit) == 1 {
        // Pre-indexed
        if bf!(instr_data.w_bit) == 0 {
            // Writeback disabled
            (mod_addr, base_addr)
        } else {
            (mod_addr, mod_addr)
        }
    } else {
        // Post-indexed
        assert!(bf!(instr_data.w_bit) == 0); // TODO: Implement
        (base_addr, mod_addr)
    }
}

#[inline(always)]
fn instr_load(cpu: &mut Cpu, data: arm::ldr::InstrDesc, byte: bool) -> cpu::InstrStatus {
    if !cpu::cond_passed(bf!(data.cond), &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    let rd = bf!(data.rd);
    let (addr, wb) = decode_addr_and_writeback(data.raw(), cpu);

    // TODO: determine behavior based on CP15 r1 bit_U (22)
    let val = if byte {
        cpu.memory.read::<u8>(addr) as u32
    } else {
        cpu.memory.read::<u32>(addr.rotate_right(8 * bits!(addr, 0 => 1)))
    };

    // Writeback
    cpu.regs[bf!(data.rn) as usize] = wb;

    if rd == 15 {
        bf!((cpu.cpsr).thumb_bit = bit!(val, 0));
        cpu.branch(val & 0xFFFFFFFE);
        return cpu::InstrStatus::Branched;
    } else {
        cpu.regs[rd as usize] = val;
    }

    cpu::InstrStatus::InBlock
}

#[inline(always)]
fn instr_store(cpu: &mut Cpu, data: arm::ldr::InstrDesc, byte: bool) -> cpu::InstrStatus {
    if !cpu::cond_passed(bf!(data.cond), &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    let (addr, wb) = decode_addr_and_writeback(data.raw(), cpu);
    let val = cpu.regs[bf!(data.rd) as usize];

    // Writeback
    cpu.regs[bf!(data.rn) as usize] = wb;

    if byte {
        cpu.memory.write::<u8>(addr, val as u8);
    } else {
        cpu.memory.write::<u32>(addr, val);
    };

    cpu::InstrStatus::InBlock
}

#[inline(always)]
pub fn ldr(cpu: &mut Cpu, data: arm::ldr::InstrDesc) -> cpu::InstrStatus {
    instr_load(cpu, data, false)
}

#[inline(always)]
pub fn ldrb(cpu: &mut Cpu, data: arm::ldrb::InstrDesc) -> cpu::InstrStatus {
    instr_load(cpu, arm::ldr::InstrDesc::new(data.raw()), true)
}

#[inline(always)]
pub fn str(cpu: &mut Cpu, data: arm::str::InstrDesc) -> cpu::InstrStatus {
    instr_store(cpu, arm::ldr::InstrDesc::new(data.raw()), false)
}

#[inline(always)]
pub fn strb(cpu: &mut Cpu, data: arm::strb::InstrDesc) -> cpu::InstrStatus {
    instr_store(cpu, arm::ldr::InstrDesc::new(data.raw()), true)
}
