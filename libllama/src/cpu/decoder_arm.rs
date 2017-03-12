define_insts!(ArmInstruction: u32, {
    with [ {0b1111}.4; {}.28 ] // Unconditional instructions
    {
        mod_blx: [ {0b1111101}.7; h_bit.1; signed_imm_24.24 ]
    }

    with [ {}.4; {0b000}.3; {}.20; {0}.1; {}.4 ] // Data processing immediate shift
      or [ {}.4; {0b000}.3; {}.17; {0}.1; {}.2; {1}.1; {}.4 ] // Data processing register shift
      or [ {}.4; {0b001}.3; {}.25 ] // Data processing immediate
    {
        and: [ cond.4; {0b00}.2; i_bit.1; {0b0000}.4; s_bit.1; rn.4; rd.4; shifter_operand.12 ],
        eor: [ cond.4; {0b00}.2; i_bit.1; {0b0001}.4; s_bit.1; rn.4; rd.4; shifter_operand.12 ],
        sub: [ cond.4; {0b00}.2; i_bit.1; {0b0010}.4; s_bit.1; rn.4; rd.4; shifter_operand.12 ],
        rsb: [ cond.4; {0b00}.2; i_bit.1; {0b0011}.4; s_bit.1; rn.4; rd.4; shifter_operand.12 ],
        add: [ cond.4; {0b00}.2; i_bit.1; {0b0100}.4; s_bit.1; rn.4; rd.4; shifter_operand.12 ],
        adc: [ cond.4; {0b00}.2; i_bit.1; {0b0101}.4; s_bit.1; rn.4; rd.4; shifter_operand.12 ],
        sbc: [ cond.4; {0b00}.2; i_bit.1; {0b0110}.4; s_bit.1; rn.4; rd.4; shifter_operand.12 ],
        rsc: [ cond.4; {0b00}.2; i_bit.1; {0b0111}.4; s_bit.1; rn.4; rd.4; shifter_operand.12 ],
        tst: [ cond.4; {0b00}.2; i_bit.1; {0b1000}.4; {0b1}.1; rn.4; {0b0000}.4; shifter_operand.12 ],
        teq: [ cond.4; {0b00}.2; i_bit.1; {0b1001}.4; {0b1}.1; rn.4; {0b0000}.4; shifter_operand.12 ],
        cmp: [ cond.4; {0b00}.2; i_bit.1; {0b1010}.4; {0b1}.1; rn.4; {0b0000}.4; shifter_operand.12 ],
        cmn: [ cond.4; {0b00}.2; i_bit.1; {0b1011}.4; {0b1}.1; rn.4; {0b0000}.4; shifter_operand.12 ],
        orr: [ cond.4; {0b00}.2; i_bit.1; {0b1100}.4; s_bit.1; rn.4; rd.4; shifter_operand.12 ],
        mov: [ cond.4; {0b00}.2; i_bit.1; {0b1101}.4; s_bit.1; {0b0000}.4; rd.4; shifter_operand.12 ],
        bic: [ cond.4; {0b00}.2; i_bit.1; {0b1110}.4; s_bit.1; rn.4; rd.4; shifter_operand.12 ],
        mvn: [ cond.4; {0b00}.2; i_bit.1; {0b1111}.4; s_bit.1; {0b0000}.4; rd.4; shifter_operand.12 ]
    }

    with [ {}.4; {0b00010}.5; {}.2; {0}.1; {}.15; {0}.1; {}.4 ] // Misc instructions 1
      or [ {}.4; {0b00010}.5; {}.2; {0}.1; {}.12; {0}.1; {}.2; {1}.1; {}.4 ] // Misc instructions 2
    {
        blx_2: [ cond.4; {0b000100101111111111110011}.24; rm.4 ],
        bx: [ cond.4; {0b000100101111111111110001}.24; rm.4 ],
        clz: [ cond.4; {0b000101101111}.12; rd.4; {0b1111}.4; {0b0001}.4; rm.4 ],
        mrs: [ cond.4; {0b00010}.5; r_bit.1; {0b00}.2; {0b1111}.4; rd.4; {0b000000000000}.12 ],
        msr_2: [ cond.4; {0b00010}.5; r_bit.1; {0b10}.2; field_mask.4; {0b111100000000}.12; rm.4 ]
    }

    with [ {}.4; {0b000}.3; {}.17; {1}.1; {}.2; {1}.1; {}.4 ] // Multiplies, extra loads/stores
    {
        ldrd: [ cond.4; {0b000}.3; p_bit.1; u_bit.1; i_bit.1; w_bit.1; {0}.1; rn.4; rd.4; addr_mode_hi.4; {0b1101}.4; addr_mode_lo.4 ],
        ldrh: [ cond.4; {0b000}.3; p_bit.1; u_bit.1; i_bit.1; w_bit.1; {1}.1; rn.4; rd.4; addr_mode_hi.4; {0b1011}.4; addr_mode_lo.4 ],
        ldrsb: [ cond.4; {0b000}.3; p_bit.1; u_bit.1; i_bit.1; w_bit.1; {1}.1; rn.4; rd.4; addr_mode_hi.4; {0b1101}.4; addr_mode_lo.4 ],
        ldrsh: [ cond.4; {0b000}.3; p_bit.1; u_bit.1; i_bit.1; w_bit.1; {1}.1; rn.4; rd.4; addr_mode_hi.4; {0b1111}.4; addr_mode_lo.4 ],
        mla: [ cond.4; {0b0000001}.7; s_bit.1; rd.4; rn.4; rs.4; {0b1001}.4; rm.4 ],
        mul: [ cond.4; {0b0000000}.7; s_bit.1; rd.4; {0b0000}.4; rs.4; {0b1001}.4; rm.4 ],
        smlal: [ cond.4; {0b0000111}.7; s_bit.1; rd_hi.4; rd_lo.4; rs.4; {0b1001}.4; rm.4 ],
        smull: [ cond.4; {0b0000110}.7; s_bit.1; rd_hi.4; rd_lo.4; rs.4; {0b1001}.4; rm.4 ],
        strd: [ cond.4; {0b000}.3; p_bit.1; u_bit.1; i_bit.1; w_bit.1; {0}.1; rn.4; rd.4; addr_mode_hi.4; {0b1111}.4; addr_mode_lo.4 ],
        strh: [ cond.4; {0b000}.3; p_bit.1; u_bit.1; i_bit.1; w_bit.1; {0}.1; rn.4; rd.4; addr_mode_hi.4; {0b1011}.4; addr_mode_lo.4 ],
        swp: [ cond.4; {0b00010000}.8; rn.4; rd.4; {0b0000}.4; {0b1001}.4; rm.4 ],
        swpb: [ cond.4; {0b00010100}.8; rn.4; rd.4; {0b0000}.4; {0b1001}.4; rm.4 ],
        umlal: [ cond.4; {0b0000101}.7; s_bit.1; rd_hi.4; rd_lo.4; rs.4; {0b1001}.4; rm.4 ],
        umull: [ cond.4; {0b0000100}.7; s_bit.1; rd_hi.4; rd_lo.4; rs.4; {0b1001}.4; rm.4 ]
    }

    with [ {}.4; {0b010}.3; {}.25 ] // Load/store immediate offset
      or [ {}.4; {0b011}.3; {}.20; {0}.1; {}.4 ] // Load/store register offset
    {
        ldr: [ cond.4; {0b01}.2; i_bit.1; p_bit.1; u_bit.1; {0}.1; w_bit.1; {1}.1; rn.4; rd.4; addr_mode.12 ],
        ldrb: [ cond.4; {0b01}.2; i_bit.1; p_bit.1; u_bit.1; {1}.1; w_bit.1; {1}.1; rn.4; rd.4; addr_mode.12 ],
        str: [ cond.4; {0b01}.2; i_bit.1; p_bit.1; u_bit.1; {0}.1; w_bit.1; {0}.1; rn.4; rd.4; addr_mode.12 ],
        strb: [ cond.4; {0b01}.2; i_bit.1; p_bit.1; u_bit.1; {1}.1; w_bit.1; {0}.1; rn.4; rd.4; addr_mode.12 ]
    }

    with [ {}.4; {0b100}.3; {}.25 ] // Load/store multiple
    {
        ldm_1: [ cond.4; {0b100}.3; p_bit.1; u_bit.1; {0}.1; w_bit.1; {1}.1; rn.4; register_list.16 ],
        ldm_2: [ cond.4; {0b100}.3; p_bit.1; u_bit.1; {0b101}.3; rn.4; {0}.1; register_list.15 ],
        ldm_3: [ cond.4; {0b100}.3; p_bit.1; u_bit.1; {1}.1; w_bit.1; {1}.1; rn.4; {1}.1; register_list.15 ],
        stm_1: [ cond.4; {0b100}.3; p_bit.1; u_bit.1; {0}.1; w_bit.1; {0}.1; rn.4; register_list.16 ],
        stm_2: [ cond.4; {0b100}.3; p_bit.1; u_bit.1; {0b100}.3; rn.4; register_list.16 ]
    }

    with [ {}.32 ] // Other
    {
        bbl: [ cond.4; {0b101}.3; link_bit.1; signed_imm_24.24 ],
        mcr: [ cond.4; {0b1110}.4; opcode_1.3; {0}.1; crn.4; rd.4; cp_num.4; opcode_2.3; {1}.1; crm.4 ],
        mrc: [ cond.4; {0b1110}.4; opcode_1.3; {1}.1; crn.4; rd.4; cp_num.4; opcode_2.3; {1}.1; crm.4 ],
        msr_1: [ cond.4; {0b00110}.5; r_bit.1; {0b10}.2; field_mask.4; {0b1111}.4; shifter_operand.12 ],
        swi: [ cond.4; {0b1111}.4; swi_index.24 ]
    }
});
