#[inline]
fn extract_bits(data: u32, lower_bit: u32, upper_bit: u32) -> u32 {
    assert!(lower_bit <= upper_bit && upper_bit <= 31);
    data << (31 - upper_bit) >> (31 - upper_bit) >> lower_bit
}

#[inline]
fn sign_extend(data: u32, size: u32) -> i32 {
    assert!(size > 0 && size <= 32);
    ((data << (32 - size)) as i32) >> (32 - size)
}

#[derive(Debug)]
enum Condition {
    EQ,  // Equal
    NE,  // Not equal
    CS,  // Unsigned higher or same
    CC,  // Unsigned lower
    MI,  // Negative
    PL,  // Positive or zero
    VS,  // Overflow
    VC,  // No overflow
    HI,  // Unsigned higher
    LS,  // Unsigned lower or same
    GE,  // Signed greater than or equal to
    LT,  // Signed less than
    GT,  // Signed greater than
    LE,  // Signed less than or equal to
    AL,  // Unconditional
    MOD, // Condition code 0b1111, modifies meaning of instruction
}

fn decode_condition(opcode: u32) -> Condition {
    match opcode {
        0b0000 => Condition::EQ,
        0b0001 => Condition::NE,
        0b0010 => Condition::CS,
        0b0011 => Condition::CC,
        0b0100 => Condition::MI,
        0b0101 => Condition::PL,
        0b0110 => Condition::VS,
        0b0111 => Condition::VC,
        0b1000 => Condition::HI,
        0b1001 => Condition::LS,
        0b1010 => Condition::GE,
        0b1011 => Condition::LT,
        0b1100 => Condition::GT,
        0b1101 => Condition::LE,
        0b1110 => Condition::AL,
        0b1111 => Condition::MOD,
        _ => panic!("Invalid condition code {:#b}!", opcode),
    }
}

#[derive(Debug)]
enum Shifter {
    LSL(bool, u32, u32),
    LSR(bool, u32, u32),
    ASR(bool, u32, u32),
    ROR(bool, u32, u32),

    IMMEDIATE(u32, u32),
}

fn decode_shifter(instruction_encoding: u32) -> Shifter {
    if extract_bits(instruction_encoding, 25, 25) == 1 {
        let immed_8 = extract_bits(instruction_encoding, 0, 7);
        let rotate_imm = extract_bits(instruction_encoding, 8, 11);
        return Shifter::IMMEDIATE(immed_8, rotate_imm);
    }

    let is_immediate = extract_bits(instruction_encoding, 4, 4) == 0;
    let opcode = extract_bits(instruction_encoding, 5, 6);
    let register = extract_bits(instruction_encoding, 0, 3);

    let operand = if is_immediate {
        extract_bits(instruction_encoding, 7, 11)
    } else {
        extract_bits(instruction_encoding, 8, 11)
    };

    match opcode {
        0b00 => Shifter::LSL(is_immediate, register, operand),
        0b01 => Shifter::LSR(is_immediate, register, operand),
        0b10 => Shifter::ASR(is_immediate, register, operand),
        0b11 => Shifter::ROR(is_immediate, register, operand),
        _ => panic!("Invalid shifter type {:#b}!", opcode),
    }
}

#[derive(Debug)]
enum MemAccessMode {
    NORMAL,
    USER_MODE,
    OFFSET,
    PRE_INDEXED,
}

#[derive(Debug)]
enum SingleMemAddress {
    IMMEDIATE { add_offset: bool, byte_access: bool, load: bool, mode: MemAccessMode },
    REGISTER,
    SCALED,
}

fn decode_single_mem_address(instruction_encoding: u32) -> SingleMemAddress {
    let i_bit = extract_bits(instruction_encoding, 25, 25) == 1;
    let p_bit = extract_bits(instruction_encoding, 24, 24) == 1;
    let w_bit = extract_bits(instruction_encoding, 21, 21) == 1;

    panic!("Unknown addressing mode!")
}

#[derive(Debug)]
enum MultiMemAddress {

}

fn decode_multi_mem_address(instruction_encoding: u32) -> MultiMemAddress {
    panic!("Unknown addressing mode!")
}

#[derive(Debug)]
pub enum Instruction {
    ADC { cond: Condition, s_bit: bool, rn: u32, rd: u32, shifter: Shifter },
    ADD { cond: Condition, s_bit: bool, rn: u32, rd: u32, shifter: Shifter },
    AND { cond: Condition, s_bit: bool, rn: u32, rd: u32, shifter: Shifter },
    B_BL { cond: Condition, link_bit: bool, signed_imm_24: u32 },
    BLX { cond: Condition, rm: u32 },
    BX { cond: Condition, rm: u32 },
    CMN { cond: Condition, rn: u32, shifter: Shifter },
    CMP { cond: Condition, rn: u32, shifter: Shifter },
    BIC { cond: Condition, s_bit: bool, rn: u32, rd: u32, shifter: Shifter },
    EOR { cond: Condition, s_bit: bool, rn: u32, rd: u32, shifter: Shifter },
    MOV { cond: Condition, s_bit: bool, rd: u32, shifter: Shifter },
    MVN { cond: Condition, s_bit: bool, rd: u32, shifter: Shifter },
    ORR { cond: Condition, s_bit: bool, rn: u32, rd: u32, shifter: Shifter },
    RSB { cond: Condition, s_bit: bool, rn: u32, rd: u32, shifter: Shifter },
    RSC { cond: Condition, s_bit: bool, rn: u32, rd: u32, shifter: Shifter },
    SBC { cond: Condition, s_bit: bool, rn: u32, rd: u32, shifter: Shifter },
    STR { cond: Condition, rn: u32, rd: u32, address: SingleMemAddress },
    SUB { cond: Condition, s_bit: bool, rn: u32, rd: u32, shifter: Shifter },
    TEQ { cond: Condition, rn: u32, shifter: Shifter },
    TST { cond: Condition, rn: u32, shifter: Shifter },
    UNIMPLEMENTED,
    UNKNOWN,
}

macro_rules! add_decoding {
    ($data:expr, [$($low:expr => $high:expr, $val:expr);*], $code:block) => {
        if $(extract_bits($data, $low, $high) == $val)&&* {
            $code
        }
    };
}

macro_rules! add_decoding_data_processing_opcode1 {
    ($inst_enum:ident, $encoding:expr, $opcode:expr, $condition: expr) => {
        add_decoding!($encoding, [26 => 27, 0b00; 21 => 24, $opcode], {
            return Instruction::$inst_enum {
                cond: $condition,
                s_bit: extract_bits($encoding, 20, 20) == 1,
                rd: extract_bits($encoding, 12, 15),
                shifter: decode_shifter($encoding)
            };
        });
    };
}

macro_rules! add_decoding_data_processing_opcode2 {
    ($inst_enum:ident, $encoding:expr, $opcode:expr, $condition: expr) => {
        add_decoding!($encoding, [26 => 27, 0b00; 21 => 24, $opcode; 20 => 20, 0b1], {
            return Instruction::$inst_enum {
                cond: $condition,
                rn: extract_bits($encoding, 16, 19),
                shifter: decode_shifter($encoding)
            };
        });
    };
}

macro_rules! add_decoding_data_processing_opcode3 {
    ($inst_enum:ident, $encoding:expr, $opcode:expr, $condition: expr) => {
        add_decoding!($encoding, [26 => 27, 0b00; 21 => 24, $opcode], {
            return Instruction::$inst_enum {
                cond: $condition,
                s_bit: extract_bits($encoding, 20, 20) == 1,
                rn: extract_bits($encoding, 16, 19),
                rd: extract_bits($encoding, 12, 15),
                shifter: decode_shifter($encoding)
            };
        });
    };
}

pub fn decode_instruction(encoding: u32) -> Instruction {
    let condition = decode_condition(extract_bits(encoding, 28, 31));
    match condition {
        Condition::MOD => return Instruction::UNIMPLEMENTED,// panic!("Condition code 0b1111 not supported!"),
        _ => (),
    };

    add_decoding_data_processing_opcode3!(AND, encoding, 0b0000, condition);
    add_decoding_data_processing_opcode3!(EOR, encoding, 0b0001, condition);
    add_decoding_data_processing_opcode3!(SUB, encoding, 0b0010, condition);
    add_decoding_data_processing_opcode3!(RSB, encoding, 0b0011, condition);
    add_decoding_data_processing_opcode3!(ADD, encoding, 0b0100, condition);
    add_decoding_data_processing_opcode3!(ADC, encoding, 0b0101, condition);
    add_decoding_data_processing_opcode3!(SBC, encoding, 0b0110, condition);
    add_decoding_data_processing_opcode3!(RSC, encoding, 0b0111, condition);
    add_decoding_data_processing_opcode2!(TST, encoding, 0b1000, condition);
    add_decoding_data_processing_opcode2!(TEQ, encoding, 0b1001, condition);
    add_decoding_data_processing_opcode2!(CMP, encoding, 0b1010, condition);
    add_decoding_data_processing_opcode2!(CMN, encoding, 0b1011, condition);
    add_decoding_data_processing_opcode3!(ORR, encoding, 0b1100, condition);
    add_decoding_data_processing_opcode1!(MOV, encoding, 0b1101, condition);
    add_decoding_data_processing_opcode3!(BIC, encoding, 0b1110, condition);
    add_decoding_data_processing_opcode1!(MVN, encoding, 0b1111, condition);

    add_decoding!(encoding, [25 => 27, 0b101], {
        return Instruction::B_BL {
            cond: condition,
            link_bit: extract_bits(encoding, 24, 24) == 1,
            signed_imm_24: extract_bits(encoding, 0, 23),
        };
    });

    add_decoding!(encoding, [20 => 27, 0b00010010; 4 => 7, 0b0011], {
        return Instruction::BLX {
            cond: condition,
            rm: extract_bits(encoding, 0, 3),
        };
    });

    add_decoding!(encoding, [20 => 27, 0b00010010; 4 => 7, 0b0001], {
        return Instruction::BX {
            cond: condition,
            rm: extract_bits(encoding, 0, 3),
        };
    });

    add_decoding!(encoding, [26 => 27, 0b01; 22 => 22, 0b0; 20 => 20, 0b0], {
        return Instruction::STR {
            cond: condition,
            rn: extract_bits(encoding, 16, 19),
            rd: extract_bits(encoding, 12, 15),
            address: decode_single_mem_address(encoding),
        };
    });

    Instruction::UNKNOWN
}
