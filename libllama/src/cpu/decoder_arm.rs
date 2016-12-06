#[derive(Debug)]
pub enum ArmInstruction {
    ADC(ArmInstrDProc),
    ADD(ArmInstrDProc),
    AND(ArmInstrDProc),
    B_BL(ArmInstrBBL),
    BLX(ArmInstrBranchExchange),
    BX(ArmInstrBranchExchange),
    CMN(ArmInstrDProc),
    CMP(ArmInstrDProc),
    BIC(ArmInstrDProc),
    EOR(ArmInstrDProc),
    LDM(ArmInstrLoadStoreMulti),
    LDR(ArmInstrLoadStore),
    LDRB(ArmInstrLoadStore),
    MCR(ArmInstrMoveCoproc),
    MRC(ArmInstrMoveCoproc),
    MRS(ArmInstrMoveStatusReg),
    MSR(ArmInstrMoveStatusReg),
    MOV(ArmInstrDProc),
    MVN(ArmInstrDProc),
    ORR(ArmInstrDProc),
    RSB(ArmInstrDProc),
    RSC(ArmInstrDProc),
    SBC(ArmInstrDProc),
    STM(ArmInstrLoadStoreMulti),
    STR(ArmInstrLoadStore),
    STRB(ArmInstrLoadStore),
    SUB(ArmInstrDProc),
    TEQ(ArmInstrDProc),
    TST(ArmInstrDProc),

    MOD_BLX(ArmInstrModBLX),

    UNKNOWN,
}

bitfield!(ArmInstrDProc: u32, {
    shifter_operand: 0 => 11,
    rd: 12 => 15,
    rn: 16 => 19,
    s_bit: 20 => 20,
    opcode: 21 => 24,
    i_bit: 25 => 25,
    cond: 28 => 31
});

bitfield!(ArmInstrBBL: u32, {
    signed_imm_24: 0 => 23,
    link_bit: 24 => 24,
    cond: 28 => 31
});

bitfield!(ArmInstrBranchExchange: u32, {
    rm: 0 => 3,
    cond: 28 => 31
});

bitfield!(ArmInstrLoadStore: u32, {
    addressing_mode_specific: 0 => 11,
    rd: 12 => 15,
    rn: 16 => 19,
    l_bit: 20 => 20,
    w_bit: 21 => 21,
    b_bit: 22 => 22,
    u_bit: 23 => 23,
    p_bit: 24 => 24,
    i_bit: 25 => 25,
    cond: 28 => 31
});

bitfield!(ArmInstrLoadStoreMulti: u32, {
    register_list: 0 => 15,
    rn: 16 => 19,
    l_bit: 20 => 20,
    w_bit: 21 => 21,
    s_bit: 22 => 22,
    u_bit: 23 => 23,
    p_bit: 24 => 24,
    cond: 28 => 31
});

bitfield!(ArmInstrMoveCoproc: u32, {
    crm: 0 => 3,
    opcode_2: 5 => 7,
    cp_num: 8 => 11,
    rd: 12 => 15,
    crn: 16 => 19,
    opcode_1: 21 => 23,
    cond: 28 => 31
});

bitfield!(ArmInstrMoveStatusReg: u32, {
    shifter_operand: 0 => 11,
    rd: 12 => 15,
    field_mask: 16 => 19,
    r_bit: 22 => 22,
    i_bit: 25 => 25,
    cond: 28 => 31
});

bitfield!(ArmInstrModBLX: u32, {
    signed_imm_24: 0 => 23,
    h_bit: 24 => 24
});


pub fn decode_arm_instruction(encoding: u32) -> ArmInstruction {
    macro_rules! constrain {
        ($data:expr, $([$low:expr => $high:expr, $val:expr, $boolean:expr]),*) => {{
            $((bits!($data, $low => $high) == $val) == $boolean)&&*
        }};
    }

    //
    // Special (0b1111) instructions
    //

    if bits!(encoding, 28 => 31) == 0b1111 {
        if constrain!(encoding, [25 => 27, 0b101, true]) {
            return ArmInstruction::MOD_BLX(ArmInstrModBLX::new(encoding));
        }

        return ArmInstruction::UNKNOWN;
    }

    //
    // Data Processing instructions
    //

    macro_rules! constrain_data_proc {
        ($encoding:expr, $opcode:expr, $condition:expr) => {{
            !constrain!($encoding, [25 => 25, 0b0, true], [7 => 7, 0b1, true], [4 => 4, 0b1, true])
        }};
    }

    macro_rules! constrain_move_proc {
        ($encoding:expr, $opcode:expr, $condition:expr) => {{
            constrain!($encoding, [26 => 27, 0b00, true], [21 => 24, $opcode, true]) &&
                constrain_data_proc!($encoding, $opcode, $condition)
        }};
    }

    macro_rules! constrain_compare_proc {
        ($encoding:expr, $opcode:expr, $condition:expr) => {{
            constrain!($encoding, [26 => 27, 0b00, true], [21 => 24, $opcode, true], [20 => 20, 0b1, true]) &&
                constrain_data_proc!($encoding, $opcode, $condition)
        }};
    }

    macro_rules! constrain_compute_proc {
        ($encoding:expr, $opcode:expr, $condition:expr) => {{
            constrain!($encoding, [26 => 27, 0b00, true], [21 => 24, $opcode, true]) &&
                constrain_data_proc!($encoding, $opcode, $condition)
        }};
    }

    if constrain_compute_proc!(encoding, 0b0000, condition) {
        return ArmInstruction::AND(ArmInstrDProc::new(encoding));
    }

    if constrain_compute_proc!(encoding, 0b0001, condition) {
        return ArmInstruction::EOR(ArmInstrDProc::new(encoding));
    }

    if constrain_compute_proc!(encoding, 0b0010, condition) {
        return ArmInstruction::SUB(ArmInstrDProc::new(encoding));
    }

    if constrain_compute_proc!(encoding, 0b0011, condition) {
        return ArmInstruction::RSB(ArmInstrDProc::new(encoding));
    }

    if constrain_compute_proc!(encoding, 0b0100, condition) {
        return ArmInstruction::ADD(ArmInstrDProc::new(encoding));
    }

    if constrain_compute_proc!(encoding, 0b0101, condition) {
        return ArmInstruction::ADC(ArmInstrDProc::new(encoding));
    }

    if constrain_compute_proc!(encoding, 0b0110, condition) {
        return ArmInstruction::SBC(ArmInstrDProc::new(encoding));
    }

    if constrain_compute_proc!(encoding, 0b0111, condition) {
        return ArmInstruction::RSC(ArmInstrDProc::new(encoding));
    }

    if constrain_compare_proc!(encoding, 0b1000, condition) {
        return ArmInstruction::TST(ArmInstrDProc::new(encoding));
    }

    if constrain_compare_proc!(encoding, 0b1001, condition) {
        return ArmInstruction::TEQ(ArmInstrDProc::new(encoding));
    }

    if constrain_compare_proc!(encoding, 0b1010, condition) {
        return ArmInstruction::CMP(ArmInstrDProc::new(encoding));
    }

    if constrain_compare_proc!(encoding, 0b1011, condition) {
        return ArmInstruction::CMN(ArmInstrDProc::new(encoding));
    }

    if constrain_compute_proc!(encoding, 0b1100, condition) {
        return ArmInstruction::ORR(ArmInstrDProc::new(encoding));
    }

    if constrain_move_proc!(encoding, 0b1101, condition) {
        return ArmInstruction::MOV(ArmInstrDProc::new(encoding));
    }

    if constrain_compute_proc!(encoding, 0b1110, condition) {
        return ArmInstruction::BIC(ArmInstrDProc::new(encoding));
    }

    if constrain_move_proc!(encoding, 0b1111, condition) {
        return ArmInstruction::MVN(ArmInstrDProc::new(encoding));
    }

    //
    // Branch instructions
    //

    if constrain!(encoding, [25 => 27, 0b101, true]) {
        return ArmInstruction::B_BL(ArmInstrBBL::new(encoding));
    }

    if constrain!(encoding, [20 => 27, 0b00010010, true], [4 => 7, 0b0011, true]) {
        return ArmInstruction::BLX(ArmInstrBranchExchange::new(encoding));
    }

    if constrain!(encoding, [20 => 27, 0b00010010, true], [4 => 7, 0b0001, true]) {
        return ArmInstruction::BX(ArmInstrBranchExchange::new(encoding));
    }

    //
    // Load/store instructions
    //

    if constrain!(encoding, [26 => 27, 0b01, true], [22 => 22, 0b0, true], [20 => 20, 0b1, true]) {
        return ArmInstruction::LDR(ArmInstrLoadStore::new(encoding));
    }

    if constrain!(encoding, [26 => 27, 0b01, true], [22 => 22, 0b1, true], [20 => 20, 0b1, true]) {
        return ArmInstruction::LDRB(ArmInstrLoadStore::new(encoding));
    }

    if constrain!(encoding, [26 => 27, 0b01, true], [22 => 22, 0b0, true], [20 => 20, 0b0, true]) {
        return ArmInstruction::STR(ArmInstrLoadStore::new(encoding));
    }

    if constrain!(encoding, [26 => 27, 0b01, true], [22 => 22, 0b1, true], [20 => 20, 0b0, true]) {
        return ArmInstruction::STRB(ArmInstrLoadStore::new(encoding));
    }

    //
    // Load/store multiple instructions
    //

    if constrain!(encoding, [25 => 27, 0b100, true], [22 => 22, 0b0, true], [20 => 20, 0b0, true]) ||
        constrain!(encoding, [25 => 27, 0b100, true], [20 => 22, 0b100, true]) {
        return ArmInstruction::STM(ArmInstrLoadStoreMulti::new(encoding));
    }

    if constrain!(encoding, [25 => 27, 0b100, true], [22 => 22, 0b0, true], [20 => 20, 0b1, true]) ||
        constrain!(encoding, [25 => 27, 0b100, true], [20 => 22, 0b101, true], [15 => 15, 0b0, true]) ||
        constrain!(encoding, [25 => 25, 0b100, true], [22 => 22, 0b1, true], [20 => 20, 0b1, true], [15 => 15, 0b1, true]) {
        return ArmInstruction::LDM(ArmInstrLoadStoreMulti::new(encoding));
    }

    //
    // Coprocessor instructions
    //

    if constrain!(encoding, [24 => 27, 0b1110, true], [20 => 20, 0b1, true], [4 => 4, 0b1, true]) {
        return ArmInstruction::MRC(ArmInstrMoveCoproc::new(encoding));
    }

    if constrain!(encoding, [24 => 27, 0b1110, true], [20 => 20, 0b0, true], [4 => 4, 0b1, true]) {
        return ArmInstruction::MCR(ArmInstrMoveCoproc::new(encoding));
    }

    //
    // Status register instructions
    //

    if constrain!(encoding, [23 => 27, 0b00010, true], [20 => 21, 0b00, true]) {
        return ArmInstruction::MRS(ArmInstrMoveStatusReg::new(encoding));
    }

    if constrain!(encoding, [23 => 27, 0b00110, true], [20 => 21, 0b10, true]) ||
        constrain!(encoding, [23 => 27, 0b00010, true], [20 => 21, 0b10, true], [4 => 7, 0b0000, true]) {
        return ArmInstruction::MSR(ArmInstrMoveStatusReg::new(encoding));
    }

    ArmInstruction::UNKNOWN
}
