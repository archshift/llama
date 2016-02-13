use cpu;
use cpu::{Cpu, ArmInstruction, ThumbInstruction};

// #[inline]
// fn sign_extend(data: u32, size: u32) -> i32 {
//     assert!(size > 0 && size <= 32);
//     ((data << (32 - size)) as i32) >> (32 - size)
// }

// #[derive(Debug)]
// enum MemAccessMode {
//     NORMAL,
//     USER_MODE,
//     OFFSET,
//     PRE_INDEXED,
// }

// #[derive(Debug)]
// enum SingleMemAddress {
//     IMMEDIATE { add_offset: bool, byte_access: bool, load: bool, mode: MemAccessMode },
//     REGISTER,
//     SCALED,
// }

// create_bitfield!(AddressDataSingleImm: u32, {

// });

// create_bitfield!(AddressDataSingleReg: u32, {
    
// });

// create_bitfield!(AddressDataSingleScaled: u32, {
    
// });

// fn decode_single_mem_address(instruction_encoding: u32) -> SingleMemAddress {
//     let i_bit = extract_bits!(instruction_encoding, 25 => 25) == 1;
//     let p_bit = extract_bits!(instruction_encoding, 24 => 24) == 1;
//     let w_bit = extract_bits!(instruction_encoding, 21 => 21) == 1;

//     panic!("Unknown addressing mode!")
// }

// #[derive(Debug)]
// enum MultiMemAddress {

// }

// fn decode_multi_mem_address(instruction_encoding: u32) -> MultiMemAddress {
//     panic!("Unknown addressing mode!")
// }

create_bitfield!(ShifterDataComputeImm: u32, {
    rd: 0 => 3,
    opcode: 5 => 6,
    imm: 7 => 11
});

create_bitfield!(ShifterDataComputeReg: u32, {
    rd: 0 => 3,
    opcode: 5 => 6,
    rn: 7 => 11
});

#[inline(always)]
fn get_shifter_val(instr_data: &cpu::InstrDataDProc::Type, cpu: &Cpu) -> (u32, bool) {
    let shifter_bits = instr_data.get::<cpu::InstrDataDProc::shifter_operand>();
    let c_bit = cpu.cpsr.get::<cpu::Psr::c_bit>() == 1;

    if instr_data.get::<cpu::InstrDataDProc::i_bit>() == 1 {
        let immed_8 = extract_bits!(shifter_bits, 0 => 7);
        let rotate_imm = extract_bits!(shifter_bits, 8 => 11);

        let res = immed_8.rotate_right(rotate_imm * 2);
        if rotate_imm == 0 {
            return (res, c_bit);
        } else {
            return (res, extract_bits!(res, 31 => 31) == 1);
        }
    }

    let pre_shift: u32 = cpu.regs[extract_bits!(shifter_bits, 0 => 3) as usize];
    let amount = if extract_bits!(shifter_bits, 4 => 4) == 0 {
        extract_bits!(shifter_bits, 7 => 11) as u8
    } else {
        cpu.regs[extract_bits!(shifter_bits, 8 => 11) as usize] as u8
    };

    match extract_bits!(shifter_bits, 4 => 6) {
        0b000 | 0b001 => { // LSL
            if amount == 0 {
                return (pre_shift, c_bit)
            } else if amount < 32 {
                let res = pre_shift << amount;
                return (res, extract_bits!(pre_shift, (32 - amount) as usize => (32 - amount) as usize) == 1)
            } else if amount == 32 {
                return (0, extract_bits!(pre_shift, 0 => 0) == 1)
            } else {
                return (0, false)
            }
        },
        0b010 | 0b011 => { // LSR
            if amount == 0 {
                return (pre_shift, c_bit)
            } else if amount < 32 {
                let res = pre_shift >> amount;
                return (res, extract_bits!(pre_shift, (amount - 1) as usize => (amount - 1) as usize) == 1)
            } else if amount == 32 {
                return (0, extract_bits!(pre_shift, 31 => 31) == 1)
            } else {
                return (0, false)
            }
        },
        0b100 => { // ASR immedate
            if amount == 0 {
                if extract_bits!(pre_shift, 31 => 31) == 0 {
                    return (0, false)
                } else {
                    return (0xFFFFFFFF, true)
                }
            } else {
                let res = ((pre_shift as i32) >> amount) as u32;
                return (res, extract_bits!(pre_shift, (amount - 1) as usize => (amount - 1) as usize) == 1)
            }
        },
        0b101 => { // ASR register
            if amount == 0 {
                return (pre_shift, c_bit)
            } else if amount < 32 {
                let res = ((pre_shift as i32) >> amount) as u32;
                return (res, extract_bits!(pre_shift, (amount - 1) as usize => (amount - 1) as usize) == 1)
            } else {
                if extract_bits!(pre_shift, 31 => 31) == 0 {
                    return (0, false)
                } else {
                    return (0xFFFFFFFF, true)
                }
            }
        },
        0b110 => { // ROR immediate
            if amount == 0 {
                let res = ((c_bit as u32) << 31) | (pre_shift >> 1);
                return (res, extract_bits!(pre_shift, 0 => 0) == 1)
            } else {
                let res = pre_shift.rotate_right(amount as u32);
                return (res, extract_bits!(pre_shift, (amount - 1) as usize => (amount - 1) as usize) == 1)
            }
        },
        0b111 => { // ROR register
            if amount == 0 {
                return (pre_shift, c_bit)
            } else if amount & 0xF == 0 {
                return (pre_shift, extract_bits!(pre_shift, 31 => 31) == 1)
            } else {
                let amount = amount & 0xF;
                let res = pre_shift.rotate_right(amount as u32);
                return (res, extract_bits!(pre_shift, (amount - 1) as usize => (amount - 1) as usize) == 1)
            }
        }
        _ => {
            panic!("Unhandled shifter operation!");
        }
    }
}

#[inline(always)]
fn cond_passed(cond_opcode: u32, cpsr: &cpu::Psr::Type) -> bool {
    match cond_opcode {
        0b0000 => return cpsr.get::<cpu::Psr::z_bit>() == 1, // EQ
        0b0001 => return cpsr.get::<cpu::Psr::z_bit>() == 0, // NE
        0b0010 => return cpsr.get::<cpu::Psr::c_bit>() == 1, // CS
        0b0011 => return cpsr.get::<cpu::Psr::c_bit>() == 0, // CC
        0b0100 => return cpsr.get::<cpu::Psr::n_bit>() == 1, // MI
        0b0101 => return cpsr.get::<cpu::Psr::n_bit>() == 0, // PL
        0b0110 => return cpsr.get::<cpu::Psr::v_bit>() == 1, // VS
        0b0111 => return cpsr.get::<cpu::Psr::v_bit>() == 0, // VC
        0b1000 => { // HI
            return (cpsr.get::<cpu::Psr::c_bit>() == 1) && 
                (cpsr.get::<cpu::Psr::z_bit>() == 0)
        },
        0b1001 => { // LS
            return (cpsr.get::<cpu::Psr::c_bit>() == 0) || 
                (cpsr.get::<cpu::Psr::z_bit>() == 1)
        },
        0b1010 => { // GE
            return cpsr.get::<cpu::Psr::n_bit>() ==
                cpsr.get::<cpu::Psr::v_bit>()
        },
        0b1011 => { // LT
            return cpsr.get::<cpu::Psr::n_bit>() !=
                cpsr.get::<cpu::Psr::v_bit>()
        },
        0b1100 => { // GT
            return (cpsr.get::<cpu::Psr::z_bit>() == 0) &&
                (cpsr.get::<cpu::Psr::n_bit>() ==
                 cpsr.get::<cpu::Psr::v_bit>())
        },
        0b1101 => { // LE
            return (cpsr.get::<cpu::Psr::z_bit>() == 1) ||
                (cpsr.get::<cpu::Psr::n_bit>() !=
                cpsr.get::<cpu::Psr::v_bit>())
        },
        0b1110 => return true, // AL
        _ => panic!("Unhandled condition code {:#b}!", cond_opcode),
    }
}

#[inline(always)]
fn process_instr_mov(cpu: &mut Cpu, data: cpu::InstrDataDProc::Type, negate: bool) {
    if !cond_passed(data.get::<cpu::InstrDataDProc::cond>(), &cpu.cpsr) {
        return
    }

    let dst_reg = data.get::<cpu::InstrDataDProc::rd>();
    let s_bit = data.get::<cpu::InstrDataDProc::s_bit>() == 1;
    let (mut src_val, shifter_carry) = get_shifter_val(&data, cpu);
    if negate {
        src_val = !src_val;
    }

    if s_bit {
        if dst_reg == 15 {
            cpu.select_saved_psr();
        } else {
            cpu.cpsr.set::<cpu::Psr::n_bit>(extract_bits!(src_val, 31 => 31));
            cpu.cpsr.set::<cpu::Psr::z_bit>((dst_reg == 0) as u32);
            cpu.cpsr.set::<cpu::Psr::c_bit>(shifter_carry as u32);
        }
    }

    if dst_reg == 15 {
        cpu.branch(src_val);
    } else {
        cpu.regs[dst_reg as usize] = src_val;
    }
}

#[inline(always)]
pub fn interpret_arm(cpu: &mut Cpu, instr: ArmInstruction) {
    match instr {
        ArmInstruction::MOV(data) => process_instr_mov(cpu, data, false),
        ArmInstruction::MVN(data) => process_instr_mov(cpu, data, true),
        _ => {
            println!("Unimplemented instruction! {:#X}: {:?}", cpu.regs[15] - cpu.get_pc_offset(), instr);
        }
    };
    cpu.regs[15] += 4;
}

pub fn interpret_thumb(cpu: &mut Cpu, instr: ThumbInstruction) {

}
