use cpu;
use cpu::Cpu;
use cpu::decoder_arm as arm;

mod addressing {
    use cpu::Cpu;
    use cpu::decoder_arm as arm;

    pub struct LsAddr(pub u32);
    pub struct WbAddr(pub u32);

    fn normal_immed_offset(mode_data: u32) -> u32 {
        bits!(mode_data, 0 => 11)
    }

    fn normal_shifted_offset(cpu: &Cpu, mode_data: u32, carry_bit: u32) -> u32 {
        let rm = bits!(mode_data, 0 => 3);
        let shift = bits!(mode_data, 5 => 6);
        let shift_imm = bits!(mode_data, 7 => 11);

        let pre_shift = cpu.regs[rm as usize];
        match shift {
            0b00 => pre_shift << shift_imm,
            0b01 => {
                if shift_imm == 0 { 0 }
                else { pre_shift >> shift_imm }
            },
            0b10 => {
                if shift_imm == 0 {
                    if bit!(pre_shift, 31) == 1 { 0xFFFFFFFF }
                    else { 0 }
                } else {
                    ((pre_shift as i32) >> shift_imm) as u32
                }
            },
            0b11 => {
                if shift_imm == 0 {
                    (carry_bit << 31) | (pre_shift >> 1)
                } else {
                    pre_shift.rotate_right(shift_imm)
                }
            }
            _ => unreachable!()
        }
    }

    fn make_addresses(base_addr: u32, offset: u32, u_bit: bool, p_bit: bool, w_bit: bool) -> (LsAddr, WbAddr) {
        let mod_addr = if u_bit {
            base_addr.wrapping_add(offset)
        } else {
            base_addr.wrapping_sub(offset)
        };

        match (p_bit, w_bit) {
            (true, false)  => (LsAddr(mod_addr), WbAddr(base_addr)), // Pre-indexed, writeback disabled
            (true, true)   => (LsAddr(mod_addr), WbAddr(mod_addr)), // Pre-indexed, writeback enabled
            (false, false) => (LsAddr(base_addr), WbAddr(mod_addr)), // Post-indexed, writeback enabled
            (false, true)  => panic!("Invalid writeback mode!") // UNPREDICTABLE
        }
    }

    pub fn decode_normal(instr_data: u32, cpu: &Cpu) -> (LsAddr, WbAddr) {
        let instr_data = arm::ldr::InstrDesc::new(instr_data);
        let carry_bit = bf!((cpu.cpsr).c_bit) as u32;

        let offset = if bf!(instr_data.i_bit) == 1 {
            normal_shifted_offset(cpu, instr_data.raw(), carry_bit)
        } else {
            normal_immed_offset(instr_data.raw())
        };

        make_addresses(cpu.regs[bf!(instr_data.rn) as usize], offset,
                       bf!(instr_data.u_bit) == 1,
                       bf!(instr_data.p_bit) == 1,
                       bf!(instr_data.w_bit) == 1)
    }
}

#[inline(always)]
fn instr_load(cpu: &mut Cpu, data: arm::ldr::InstrDesc, byte: bool) -> cpu::InstrStatus {
    if !cpu::cond_passed(bf!(data.cond), &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    let rd = bf!(data.rd);
    let (addr, wb) = addressing::decode_normal(data.raw(), cpu);

    // TODO: determine behavior based on CP15 r1 bit_U (22)
    let val = if byte {
        cpu.memory.read::<u8>(addr.0) as u32
    } else {
        cpu.memory.read::<u32>(addr.0.rotate_right(8 * bits!(addr.0, 0 => 1)))
    };

    // Writeback
    cpu.regs[bf!(data.rn) as usize] = wb.0;

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

    let (addr, wb) = addressing::decode_normal(data.raw(), cpu);
    let val = cpu.regs[bf!(data.rd) as usize];

    // Writeback
    cpu.regs[bf!(data.rn) as usize] = wb.0;

    if byte {
        cpu.memory.write::<u8>(addr.0, val as u8);
    } else {
        cpu.memory.write::<u32>(addr.0, val);
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
