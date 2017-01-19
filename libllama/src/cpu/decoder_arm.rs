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
    macro_rules! handle {
        ($instr:ident: $data:ident, $mask:expr, $val:expr) => {
            if encoding & $mask == $val {
                return ArmInstruction::$instr($data::new(encoding));
            }
        };
    }

    // Special (0b1111) instructions
    if bits!(encoding, 28 => 31) == 0b1111 {
        handle!(MOD_BLX: ArmInstrModBLX, 0xFE000000, 0xFA000000);
        return ArmInstruction::UNKNOWN;
    }

    // Data Processing instructions
    handle!(AND: ArmInstrDProc, 0x0FE00090, 0x00000090);
    handle!(EOR: ArmInstrDProc, 0x0FE00090, 0x00200090);
    handle!(SUB: ArmInstrDProc, 0x0FE00090, 0x00400090);
    handle!(RSB: ArmInstrDProc, 0x0FE00090, 0x00600090);
    handle!(ADD: ArmInstrDProc, 0x0FE00090, 0x00800090);
    handle!(ADC: ArmInstrDProc, 0x0FE00090, 0x00A00090);
    handle!(SBC: ArmInstrDProc, 0x0FE00090, 0x00C00090);
    handle!(RSC: ArmInstrDProc, 0x0FE00090, 0x00E00090);
    handle!(TST: ArmInstrDProc, 0x0FF00090, 0x01100090);
    handle!(TEQ: ArmInstrDProc, 0x0FF00090, 0x01300090);
    handle!(CMP: ArmInstrDProc, 0x0FF00090, 0x01500090);
    handle!(CMN: ArmInstrDProc, 0x0FF00090, 0x01700090);
    handle!(ORR: ArmInstrDProc, 0x0FE00090, 0x01800090);
    handle!(MOV: ArmInstrDProc, 0x0FE00090, 0x01A00090);
    handle!(BIC: ArmInstrDProc, 0x0FE00090, 0x01C00090);
    handle!(MVN: ArmInstrDProc, 0x0FE00090, 0x01E00090);

    // Branch instructions
    handle!(B_BL: ArmInstrBBL, 0x0E000000, 0x0A000000);
    handle!(BLX: ArmInstrBranchExchange, 0x0FF000F0, 0x01200030);
    handle!(BX: ArmInstrBranchExchange, 0x0FF000F0, 0x01200010);

    // Load/store instructions
    handle!(LDR: ArmInstrLoadStore, 0x0C500000, 0x04100000);
    handle!(LDRB: ArmInstrLoadStore, 0x0C500000, 0x04500000);
    handle!(STR: ArmInstrLoadStore, 0x0C500000, 0x04000000);
    handle!(STRB: ArmInstrLoadStore, 0x0C500000, 0x04400000);

    // Load/store multiple instructions
    handle!(STM: ArmInstrLoadStoreMulti, 0x0E500000, 0x08000000);
    handle!(STM: ArmInstrLoadStoreMulti, 0x0E700000, 0x08400000);
    handle!(LDM: ArmInstrLoadStoreMulti, 0x0E500000, 0x08100000);
    handle!(LDM: ArmInstrLoadStoreMulti, 0x0E708000, 0x08500000);
    handle!(LDM: ArmInstrLoadStoreMulti, 0x02508000, 0x08508000);

    // Coprocessor instructions
    handle!(MRC: ArmInstrMoveCoproc, 0x0F100010, 0x0E100010);
    handle!(MCR: ArmInstrMoveCoproc, 0x0F100010, 0x0E000010);

    // Status register instructions
    handle!(MRS: ArmInstrMoveStatusReg, 0x0FB00000, 0x01000000);
    handle!(MSR: ArmInstrMoveStatusReg, 0x0FB00000, 0x03200000);
    handle!(MSR: ArmInstrMoveStatusReg, 0x0FB000F0, 0x01200000);

    ArmInstruction::UNKNOWN
}
