#[derive(Debug)]
pub enum ThumbInstruction {
    AND(ThumbInstrBitwise),
    ASR_1(ThumbInstrShift_1),
    BLX_2(ThumbInstrBLX_2),
    BIC(ThumbInstrBitwise),
    BRANCH(ThumbInstrBRANCH),
    EOR(ThumbInstrBitwise),
    LDR_1(ThumbInstrLoadStore_1),
    LDR_2(ThumbInstrLoadStore_2),
    LDR_3(ThumbInstrLoadStore_3),
    LDR_4(ThumbInstrLoadStore_3),
    LDRB_1(ThumbInstrLoadStore_1),
    LDRB_2(ThumbInstrLoadStore_2),
    LDRH_1(ThumbInstrLoadStore_1),
    LDRH_2(ThumbInstrLoadStore_2),
    LSL_1(ThumbInstrShift_1),
    LSR_1(ThumbInstrShift_1),
    MOV_1(ThumbInstrMOV_1),
    MOV_2(ThumbInstrMOV_2),
    ORR(ThumbInstrBitwise),
    PUSH(ThumbInstrPUSH),
    POP(ThumbInstrPOP),
    STR_1(ThumbInstrLoadStore_1),
    STR_2(ThumbInstrLoadStore_2),
    STR_3(ThumbInstrLoadStore_3),
    STRB_1(ThumbInstrLoadStore_1),
    STRB_2(ThumbInstrLoadStore_2),
    STRH_1(ThumbInstrLoadStore_1),
    STRH_2(ThumbInstrLoadStore_2),

    UNKNOWN,
}

bitfield!(ThumbInstrBitwise: u16, {
    rd: 0 => 2,
    rm: 3 => 5
});

bitfield!(ThumbInstrBLX_2: u16, {
    rm: 3 => 5,
    h2: 6 => 6
});

bitfield!(ThumbInstrBRANCH: u16, {
    offset_11: 0 => 10,
    h_bits: 11 => 12
});

bitfield!(ThumbInstrLoadStore_1: u16, {
    rd: 0 => 2,
    rn: 3 => 5,
    immed_5: 6 => 10
});

bitfield!(ThumbInstrLoadStore_2: u16, {
    rd: 0 => 2,
    rn: 3 => 5,
    rm: 6 => 8
});

bitfield!(ThumbInstrLoadStore_3: u16, {
    immed_8: 0 => 7,
    rd: 8 => 10
});

bitfield!(ThumbInstrShift_1: u16, {
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

pub fn decode_thumb_instruction(encoding: u16) -> ThumbInstruction {
    macro_rules! handle {
        ($instr:ident: $data:ident, $mask:expr, $val:expr) => {
            if encoding & $mask == $val {
                return ThumbInstruction::$instr($data::new(encoding));
            }
        };
    }

    //
    // Data Processing instructions
    //

    handle!(AND: ThumbInstrBitwise, 0xFFC0, 0x4000);
    handle!(ASR_1: ThumbInstrShift_1, 0xF800, 0x1000);
    handle!(BIC: ThumbInstrBitwise, 0xFFC0, 0x4380);
    handle!(EOR: ThumbInstrBitwise, 0xFFC0, 0x4040);
    handle!(LSL_1: ThumbInstrShift_1, 0xF800, 0x0000);
    handle!(LSR_1: ThumbInstrShift_1, 0xF800, 0x0800);
    handle!(MOV_1: ThumbInstrMOV_1, 0xF800, 0x2000);
    handle!(MOV_2: ThumbInstrMOV_2, 0xFFC0, 0x1C00);
    handle!(ORR: ThumbInstrBitwise, 0xFFC0, 0x4300);

    //
    // Branch instructions
    //

    handle!(BRANCH: ThumbInstrBRANCH, 0xE000, 0xE000);
    handle!(BLX_2: ThumbInstrBLX_2, 0xFF80, 0x4780);

    //
    // Load/store instructions
    //

    handle!(LDR_1: ThumbInstrLoadStore_1, 0xF800, 0x6800);
    handle!(LDR_2: ThumbInstrLoadStore_2, 0xFE00, 0x5800);
    handle!(LDR_3: ThumbInstrLoadStore_3, 0xF800, 0x4800);
    handle!(LDR_4: ThumbInstrLoadStore_3, 0xF800, 0x9800);
    handle!(LDRB_1: ThumbInstrLoadStore_1, 0xF800, 0x7800);
    handle!(LDRB_2: ThumbInstrLoadStore_2, 0xFE00, 0x5c00);
    handle!(LDRH_1: ThumbInstrLoadStore_1, 0xF800, 0x8800);
    handle!(LDRH_2: ThumbInstrLoadStore_2, 0xFE00, 0x5a00);

    handle!(STR_1: ThumbInstrLoadStore_1, 0xF800, 0x6000);
    handle!(STR_2: ThumbInstrLoadStore_2, 0xFE00, 0x5000);
    handle!(STR_3: ThumbInstrLoadStore_3, 0xF800, 0x9000);
    handle!(STRB_1: ThumbInstrLoadStore_1, 0xF800, 0x7000);
    handle!(STRB_2: ThumbInstrLoadStore_2, 0xFE00, 0x5400);
    handle!(STRH_1: ThumbInstrLoadStore_1, 0xF800, 0x8000);
    handle!(STRH_2: ThumbInstrLoadStore_2, 0xFE00, 0x5200);

    handle!(POP: ThumbInstrPOP, 0xFE00, 0xBC00);
    handle!(PUSH: ThumbInstrPUSH, 0xFE00, 0xB400);


    ThumbInstruction::UNKNOWN
}
