#[derive(Debug)]
pub enum ThumbInstruction {
    B_2(ThumbInstrB_2),
    BL_BLX(ThumbInstrBL_BLX),
    LDR_1(ThumbInstrLDR_1),
    LDR_2(ThumbInstrLDR_2),
    LDR_3(ThumbInstrLDR_3),
    LDR_4(ThumbInstrLDR_4),
    LSL_1(ThumbInstrLSL_1),
    MOV_1(ThumbInstrMOV_1),
    MOV_2(ThumbInstrMOV_2),
    PUSH(ThumbInstrPUSH),
    POP(ThumbInstrPOP),
    STR_1(ThumbInstrSTR_1),

    UNKNOWN,
}

bitfield!(ThumbInstrB_2: u16, {
    offset_11: 0 => 10
});

bitfield!(ThumbInstrBL_BLX: u16, {
    offset_11: 0 => 10,
    h_bits: 11 => 12
});

bitfield!(ThumbInstrLDR_1: u16, {
    rd: 0 => 2,
    rn: 3 => 5,
    immed_5: 6 => 10
});

bitfield!(ThumbInstrLDR_2: u16, {
    rd: 0 => 2,
    rn: 3 => 5,
    rm: 6 => 8
});

bitfield!(ThumbInstrLDR_3: u16, {
    immed_8: 0 => 7,
    rd: 8 => 10
});

bitfield!(ThumbInstrLDR_4: u16, {
    immed_8: 0 => 7,
    rd: 8 => 10
});

bitfield!(ThumbInstrLSL_1: u16, {
    rd: 0 => 2,
    rm: 3 => 5,
    immed_5: 6 => 10
});

bitfield!(ThumbInstrMOV_1: u16, {
    immed_8: 0 => 7,
    rd: 8 => 10
});

bitfield!(ThumbInstrMOV_2: u16, {
    rd: 0 => 2,
    rn: 3 => 5
});

bitfield!(ThumbInstrPUSH: u16, {
    register_list: 0 => 7,
    r_bit: 8 => 8
});

bitfield!(ThumbInstrPOP: u16, {
    register_list: 0 => 7,
    r_bit: 8 => 8
});

bitfield!(ThumbInstrSTR_1: u16, {
    rd: 0 => 2,
    rn: 3 => 5,
    immed_5: 6 => 10
});

pub fn decode_thumb_instruction(encoding: u16) -> ThumbInstruction {
    //
    // Data Processing instructions
    //

    if encoding & 0xF800 == 0x2000 {
        return ThumbInstruction::MOV_1(ThumbInstrMOV_1::new(encoding));
    }

    if encoding & 0xFFC0 == 0x1C00 {
        return ThumbInstruction::MOV_2(ThumbInstrMOV_2::new(encoding));
    }

    if encoding & 0xF800 == 0x0000 {
        return ThumbInstruction::LSL_1(ThumbInstrLSL_1::new(encoding));
    }

    //
    // Branch instructions
    //

    if encoding & 0xF800 == 0xE000 {
        return ThumbInstruction::B_2(ThumbInstrB_2::new(encoding));
    }

    if encoding & 0xE000 == 0xE000 {
        return ThumbInstruction::BL_BLX(ThumbInstrBL_BLX::new(encoding));
    }

    //
    // Load/store instructions
    //

    if encoding & 0xF800 == 0x6800 {
        return ThumbInstruction::LDR_1(ThumbInstrLDR_1::new(encoding));
    }

    if encoding & 0xFE00 == 0x5800 {
        return ThumbInstruction::LDR_2(ThumbInstrLDR_2::new(encoding));
    }

    if encoding & 0xF800 == 0x4800 {
        return ThumbInstruction::LDR_3(ThumbInstrLDR_3::new(encoding));
    }

    if encoding & 0xF800 == 0x9800 {
        return ThumbInstruction::LDR_4(ThumbInstrLDR_4::new(encoding));
    }

    if encoding & 0xF800 == 0x6000 {
        return ThumbInstruction::STR_1(ThumbInstrSTR_1::new(encoding));
    }

    if encoding & 0xFE00 == 0xBC00 {
        return ThumbInstruction::POP(ThumbInstrPOP::new(encoding));
    }

    if encoding & 0xFE00 == 0xB400 {
        return ThumbInstruction::PUSH(ThumbInstrPUSH::new(encoding));
    }


    ThumbInstruction::UNKNOWN
}
