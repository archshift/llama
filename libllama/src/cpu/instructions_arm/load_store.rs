use cpu;
use cpu::Cpu;
use cpu::interpreter_arm as arm;

use bitutils::sign_extend;

mod addressing {
    use cpu::Cpu;
    use cpu::interpreter_arm as arm;

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

    fn misc_immed_offset(mode_data: u32) -> u32 {
        let immed_lo = bits!(mode_data, 0 => 3);
        let immed_hi = bits!(mode_data, 8 => 11);
        immed_lo | (immed_hi << 4)
    }

    fn misc_reg_offset(cpu: &Cpu, mode_data: u32) -> u32 {
        let rm = bits!(mode_data, 0 => 3);
        cpu.regs[rm as usize]
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
        let instr_data = arm::Ldr::new(instr_data);
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

    pub fn decode_misc(instr_data: u32, cpu: &Cpu) -> (LsAddr, WbAddr) {
        let instr_data = arm::Ldrh::new(instr_data);

        let offset = if bf!(instr_data.i_bit) == 1 {
            misc_immed_offset(instr_data.raw())
        } else {
            misc_reg_offset(cpu, instr_data.raw())
        };

        make_addresses(cpu.regs[bf!(instr_data.rn) as usize], offset,
                       bf!(instr_data.u_bit) == 1,
                       bf!(instr_data.p_bit) == 1,
                       bf!(instr_data.w_bit) == 1)
    }
}

enum MiscLsType {
    Doubleword,
    Halfword,
    SignedByte,
    SignedHalfword
}

fn instr_load(cpu: &mut Cpu, data: arm::Ldr, byte: bool) -> cpu::InstrStatus {
    if !cpu::cond_passed(bf!(data.cond), &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    let rd = bf!(data.rd);
    let (addr, wb) = addressing::decode_normal(data.raw(), cpu);

    // TODO: determine behavior based on CP15 r1 bit_U (22)
    let val = if byte {
        cpu.mpu.dmem_read::<u8>(addr.0) as u32
    } else {
        cpu.mpu.dmem_read::<u32>(addr.0 & !0b11)
            .rotate_right(8 * bits!(addr.0, 0 => 1))
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

fn instr_load_misc(cpu: &mut Cpu, data: arm::Ldrh, ty: MiscLsType) -> cpu::InstrStatus {
    if !cpu::cond_passed(bf!(data.cond), &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    let rd = bf!(data.rd) as usize;
    let (addr, wb) = addressing::decode_misc(data.raw(), cpu);
    // Writeback
    cpu.regs[bf!(data.rn) as usize] = wb.0;

    // TODO: determine behavior based on CP15 r1 bit_U (22)
    let val = match ty {
        MiscLsType::Doubleword => {
            assert!((rd % 2 == 0) && (rd != 14) && (addr.0 % 4 == 0));
            let val = cpu.mpu.dmem_read::<u64>(addr.0);
            cpu.regs[rd] = val as u32;
            cpu.regs[rd+1] = (val >> 32) as u32;
            return cpu::InstrStatus::InBlock
        }
        MiscLsType::Halfword => cpu.mpu.dmem_read::<u16>(addr.0) as u32,
        MiscLsType::SignedByte => sign_extend(cpu.mpu.dmem_read::<u8>(addr.0) as u32, 8) as u32,
        MiscLsType::SignedHalfword => sign_extend(cpu.mpu.dmem_read::<u16>(addr.0) as u32, 16) as u32
    };

    cpu.regs[bf!(data.rd) as usize] = val;

    cpu::InstrStatus::InBlock
}

fn instr_store(cpu: &mut Cpu, data: arm::Str, byte: bool) -> cpu::InstrStatus {
    if !cpu::cond_passed(bf!(data.cond), &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    let (addr, wb) = addressing::decode_normal(data.raw(), cpu);
    let val = cpu.regs[bf!(data.rd) as usize];

    // Writeback
    cpu.regs[bf!(data.rn) as usize] = wb.0;

    if byte {
        cpu.mpu.dmem_write::<u8>(addr.0, val as u8);
    } else {
        cpu.mpu.dmem_write::<u32>(addr.0 & !0b11, val);
    };

    cpu::InstrStatus::InBlock
}

fn instr_store_misc(cpu: &mut Cpu, data: arm::Strh, ty: MiscLsType) -> cpu::InstrStatus {
    if !cpu::cond_passed(bf!(data.cond), &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    let (addr, wb) = addressing::decode_misc(data.raw(), cpu);
    let rd = bf!(data.rd) as usize;

    // Writeback
    cpu.regs[bf!(data.rn) as usize] = wb.0;
    // TODO: determine behavior based on CP15 r1 bit_U (22)
    match ty {
        MiscLsType::Doubleword => {
            assert!((rd % 2 == 0) && (rd != 14) && (addr.0 % 4 == 0));
            let val = (cpu.regs[rd] as u64) | ((cpu.regs[rd+1] as u64) << 32);
            cpu.mpu.dmem_write::<u64>(addr.0, val)
        }
        MiscLsType::Halfword => cpu.mpu.dmem_write::<u16>(addr.0, cpu.regs[rd] as u16),
        _ => panic!("Invalid miscellaneous store type!")
    }

    cpu::InstrStatus::InBlock
}

pub fn ldr(cpu: &mut Cpu, data: arm::Ldr) -> cpu::InstrStatus {
    instr_load(cpu, data, false)
}

pub fn ldrb(cpu: &mut Cpu, data: arm::Ldrb) -> cpu::InstrStatus {
    instr_load(cpu, arm::Ldr::new(data.raw()), true)
}

pub fn ldrd(cpu: &mut Cpu, data: arm::Ldrd) -> cpu::InstrStatus {
    instr_load_misc(cpu, arm::Ldrh::new(data.raw()), MiscLsType::Doubleword)
}

pub fn ldrh(cpu: &mut Cpu, data: arm::Ldrh) -> cpu::InstrStatus {
    instr_load_misc(cpu, data, MiscLsType::Halfword)
}

pub fn ldrsb(cpu: &mut Cpu, data: arm::Ldrsb) -> cpu::InstrStatus {
    instr_load_misc(cpu, arm::Ldrh::new(data.raw()), MiscLsType::SignedByte)
}

pub fn ldrsh(cpu: &mut Cpu, data: arm::Ldrsh) -> cpu::InstrStatus {
    instr_load_misc(cpu, arm::Ldrh::new(data.raw()), MiscLsType::SignedHalfword)
}

pub fn str(cpu: &mut Cpu, data: arm::Str) -> cpu::InstrStatus {
    instr_store(cpu, data, false)
}

pub fn strb(cpu: &mut Cpu, data: arm::Strb) -> cpu::InstrStatus {
    instr_store(cpu, arm::Str::new(data.raw()), true)
}

pub fn strd(cpu: &mut Cpu, data: arm::Strd) -> cpu::InstrStatus {
    instr_store_misc(cpu, arm::Strh::new(data.raw()), MiscLsType::Doubleword)
}

pub fn strh(cpu: &mut Cpu, data: arm::Strh) -> cpu::InstrStatus {
    instr_store_misc(cpu, data, MiscLsType::Halfword)
}

pub fn swp(cpu: &mut Cpu, data: arm::Swp) -> cpu::InstrStatus {
    if !cpu::cond_passed(bf!(data.cond), &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    // TODO: determine behavior based on CP15 r1 bit_U (22)
    let addr = cpu.regs[bf!(data.rn) as usize];
    let new_val = cpu.regs[bf!(data.rm) as usize];

    let tmp = cpu.mpu.dmem_read::<u32>(addr);
    cpu.mpu.dmem_write::<u32>(addr, new_val);
    cpu.regs[bf!(data.rd) as usize] = tmp;

    cpu::InstrStatus::InBlock
}

pub fn swpb(cpu: &mut Cpu, data: arm::Swpb) -> cpu::InstrStatus {
    if !cpu::cond_passed(bf!(data.cond), &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    // TODO: determine behavior based on CP15 r1 bit_U (22)
    let addr = cpu.regs[bf!(data.rn) as usize];
    let new_val = cpu.regs[bf!(data.rm) as usize];

    let tmp = cpu.mpu.dmem_read::<u8>(addr);
    cpu.mpu.dmem_write::<u8>(addr, new_val as u8);
    cpu.regs[bf!(data.rd) as usize] = tmp as u32;

    cpu::InstrStatus::InBlock
}
