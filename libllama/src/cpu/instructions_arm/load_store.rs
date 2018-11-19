use cpu::{self, Cpu, Version};
use cpu::interpreter_arm as arm;

use bitutils::sign_extend32;

mod addressing {
    use cpu::{Cpu, Version};
    use cpu::interpreter_arm as arm;

    pub struct LsAddr(pub u32);
    pub struct WbAddr(pub u32);

    fn normal_immed_offset(mode_data: u32) -> u32 {
        bits!(mode_data, 0:11)
    }

    fn normal_shifted_offset<V: Version>(cpu: &Cpu<V>, mode_data: u32, carry_bit: u32) -> u32 {
        let rm = bits!(mode_data, 0:3);
        let shift = bits!(mode_data, 5:6);
        let shift_imm = bits!(mode_data, 7:11);

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
        let immed_lo = bits!(mode_data, 0:3);
        let immed_hi = bits!(mode_data, 8:11);
        immed_lo | (immed_hi << 4)
    }

    fn misc_reg_offset<V: Version>(cpu: &Cpu<V>, mode_data: u32) -> u32 {
        let rm = bits!(mode_data, 0:3);
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

    pub fn decode_normal<V: Version>(instr_data: u32, cpu: &Cpu<V>) -> (LsAddr, WbAddr) {
        let instr_data = arm::Ldr::new(instr_data);
        let carry_bit = cpu.cpsr.c_bit.get() as u32;

        let offset = if instr_data.i_bit.get() == 1 {
            normal_shifted_offset(cpu, instr_data.val, carry_bit)
        } else {
            normal_immed_offset(instr_data.val)
        };

        make_addresses(cpu.regs[instr_data.rn.get() as usize], offset,
                       instr_data.u_bit.get() == 1,
                       instr_data.p_bit.get() == 1,
                       instr_data.w_bit.get() == 1)
    }

    pub fn decode_misc<V: Version>(instr_data: u32, cpu: &Cpu<V>) -> (LsAddr, WbAddr) {
        let instr_data = arm::Ldrh::new(instr_data);

        let offset = if instr_data.i_bit.get() == 1 {
            misc_immed_offset(instr_data.val)
        } else {
            misc_reg_offset(cpu, instr_data.val)
        };

        make_addresses(cpu.regs[instr_data.rn.get() as usize], offset,
                       instr_data.u_bit.get() == 1,
                       instr_data.p_bit.get() == 1,
                       instr_data.w_bit.get() == 1)
    }
}

enum MiscLsType {
    Doubleword,
    Halfword,
    SignedByte,
    SignedHalfword
}

fn instr_load<V: Version>(cpu: &mut Cpu<V>, data: arm::Ldr::Bf, byte: bool) -> cpu::InstrStatus {
    if !cpu::cond_passed(data.cond.get(), &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    let rd = data.rd.get();
    let (addr, wb) = addressing::decode_normal(data.val, cpu);

    let val = if byte {
        cpu.mpu.dmem_read::<u8>(addr.0) as u32
    } else {
        // TODO: determine behavior based on CP15 r1 bit_U (22)
        assert!( V::is::<cpu::v5>() || addr.0 % 4 == 0 );

        cpu.mpu.dmem_read::<u32>(addr.0 & !0b11)
            .rotate_right(8 * bits!(addr.0, 0:1))
    };

    // Writeback
    cpu.regs[data.rn.get() as usize] = wb.0;

    if rd == 15 {
        cpu.cpsr.thumb_bit.set(bit!(val, 0));
        cpu.branch(val & 0xFFFFFFFE);
        return cpu::InstrStatus::Branched;
    } else {
        cpu.regs[rd as usize] = val;
    }

    cpu::InstrStatus::InBlock
}

fn instr_load_misc<V: Version>(cpu: &mut Cpu<V>, data: arm::Ldrh::Bf, ty: MiscLsType) -> cpu::InstrStatus {
    if !cpu::cond_passed(data.cond.get(), &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    let rd = data.rd.get() as usize;
    let (addr, wb) = addressing::decode_misc(data.val, cpu);
    // Writeback
    cpu.regs[data.rn.get() as usize] = wb.0;

    // TODO: determine behavior based on CP15 r1 bit_U (22)
    let val = match ty {
        MiscLsType::Doubleword => {
            assert!((rd % 2 == 0) && (rd != 14) && (addr.0 % 4 == 0));
            let val = cpu.mpu.dmem_read::<u64>(addr.0);
            cpu.regs[rd] = val as u32;
            cpu.regs[rd+1] = (val >> 32) as u32;
            return cpu::InstrStatus::InBlock
        }
        MiscLsType::Halfword => {
            assert!( V::is::<cpu::v5>() || addr.0 % 2 == 0 );
            
            cpu.mpu.dmem_read::<u16>(addr.0) as u32
        }
        MiscLsType::SignedByte => sign_extend32(cpu.mpu.dmem_read::<u8>(addr.0) as u32, 8) as u32,
        MiscLsType::SignedHalfword => {
            assert!( V::is::<cpu::v5>() || addr.0 % 2 == 0 );

            sign_extend32(cpu.mpu.dmem_read::<u16>(addr.0) as u32, 16) as u32
        }
    };

    cpu.regs[data.rd.get() as usize] = val;

    cpu::InstrStatus::InBlock
}

fn instr_store<V: Version>(cpu: &mut Cpu<V>, data: arm::Str::Bf, byte: bool) -> cpu::InstrStatus {
    if !cpu::cond_passed(data.cond.get(), &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    let (addr, wb) = addressing::decode_normal(data.val, cpu);
    let val = cpu.regs[data.rd.get() as usize];

    // Writeback
    cpu.regs[data.rn.get() as usize] = wb.0;

    if byte {
        cpu.mpu.dmem_write::<u8>(addr.0, val as u8);
    } else {
        // TODO: determine behavior based on CP15 r1 bit_U (22)
        assert!( V::is::<cpu::v5>() || addr.0 % 4 == 0 );

        cpu.mpu.dmem_write::<u32>(addr.0 & !0b11, val);
    };

    cpu::InstrStatus::InBlock
}

fn instr_store_misc<V: Version>(cpu: &mut Cpu<V>, data: arm::Strh::Bf, ty: MiscLsType) -> cpu::InstrStatus {
    if !cpu::cond_passed(data.cond.get(), &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    let (addr, wb) = addressing::decode_misc(data.val, cpu);
    let rd = data.rd.get() as usize;

    // Writeback
    cpu.regs[data.rn.get() as usize] = wb.0;
    // TODO: determine behavior based on CP15 r1 bit_U (22)
    match ty {
        MiscLsType::Doubleword => {
            assert!((rd % 2 == 0) && (rd != 14) && (addr.0 % 4 == 0));
            let val = (cpu.regs[rd] as u64) | ((cpu.regs[rd+1] as u64) << 32);
            cpu.mpu.dmem_write::<u64>(addr.0, val)
        }
        MiscLsType::Halfword => {
            assert!( V::is::<cpu::v5>() || addr.0 % 2 == 0 );

            cpu.mpu.dmem_write::<u16>(addr.0, cpu.regs[rd] as u16)
        }
        _ => panic!("Invalid miscellaneous store type!")
    }

    cpu::InstrStatus::InBlock
}

pub fn ldr<V: Version>(cpu: &mut Cpu<V>, data: arm::Ldr::Bf) -> cpu::InstrStatus {
    instr_load(cpu, data, false)
}

pub fn ldrb<V: Version>(cpu: &mut Cpu<V>, data: arm::Ldrb::Bf) -> cpu::InstrStatus {
    instr_load(cpu, arm::Ldr::new(data.val), true)
}

pub fn ldrd<V: Version>(cpu: &mut Cpu<V>, data: arm::Ldrd::Bf) -> cpu::InstrStatus {
    assert!(V::is::<cpu::v5>());
    instr_load_misc(cpu, arm::Ldrh::new(data.val), MiscLsType::Doubleword)
}

pub fn ldrh<V: Version>(cpu: &mut Cpu<V>, data: arm::Ldrh::Bf) -> cpu::InstrStatus {
    instr_load_misc(cpu, data, MiscLsType::Halfword)
}

pub fn ldrsb<V: Version>(cpu: &mut Cpu<V>, data: arm::Ldrsb::Bf) -> cpu::InstrStatus {
    instr_load_misc(cpu, arm::Ldrh::new(data.val), MiscLsType::SignedByte)
}

pub fn ldrsh<V: Version>(cpu: &mut Cpu<V>, data: arm::Ldrsh::Bf) -> cpu::InstrStatus {
    instr_load_misc(cpu, arm::Ldrh::new(data.val), MiscLsType::SignedHalfword)
}

pub fn str<V: Version>(cpu: &mut Cpu<V>, data: arm::Str::Bf) -> cpu::InstrStatus {
    instr_store(cpu, data, false)
}

pub fn strb<V: Version>(cpu: &mut Cpu<V>, data: arm::Strb::Bf) -> cpu::InstrStatus {
    instr_store(cpu, arm::Str::new(data.val), true)
}

pub fn strd<V: Version>(cpu: &mut Cpu<V>, data: arm::Strd::Bf) -> cpu::InstrStatus {
    assert!(V::is::<cpu::v5>());
    instr_store_misc(cpu, arm::Strh::new(data.val), MiscLsType::Doubleword)
}

pub fn strh<V: Version>(cpu: &mut Cpu<V>, data: arm::Strh::Bf) -> cpu::InstrStatus {
    instr_store_misc(cpu, data, MiscLsType::Halfword)
}

pub fn swp<V: Version>(cpu: &mut Cpu<V>, data: arm::Swp::Bf) -> cpu::InstrStatus {
    assert!(V::is::<cpu::v5>());
    if !cpu::cond_passed(data.cond.get(), &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    // TODO: determine behavior based on CP15 r1 bit_U (22)
    let addr = cpu.regs[data.rn.get() as usize];
    let new_val = cpu.regs[data.rm.get() as usize];

    let tmp = cpu.mpu.dmem_read::<u32>(addr);
    cpu.mpu.dmem_write::<u32>(addr, new_val);
    cpu.regs[data.rd.get() as usize] = tmp;

    cpu::InstrStatus::InBlock
}

pub fn swpb<V: Version>(cpu: &mut Cpu<V>, data: arm::Swpb::Bf) -> cpu::InstrStatus {
    assert!(V::is::<cpu::v5>());
    if !cpu::cond_passed(data.cond.get(), &cpu.cpsr) {
        return cpu::InstrStatus::InBlock;
    }

    // TODO: determine behavior based on CP15 r1 bit_U (22)
    let addr = cpu.regs[data.rn.get() as usize];
    let new_val = cpu.regs[data.rm.get() as usize];

    let tmp = cpu.mpu.dmem_read::<u8>(addr);
    cpu.mpu.dmem_write::<u8>(addr, new_val as u8);
    cpu.regs[data.rd.get() as usize] = tmp as u32;

    cpu::InstrStatus::InBlock
}
