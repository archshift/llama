bitfield!(RegCmd: u16, {
    command_index: 0 => 5,
    command_type: 6 => 7,
    response_type: 8 => 10,
    has_data: 11 => 11,
    is_reading: 12 => 12,
    has_multi_block: 13 => 13
});

iodevice!(EmmcDevice, {
    0x000 => cmd: u16 { }
    0x002 => port_select: u16 { }
    0x004 => response0: u16 { write_bits = 0; }
    0x006 => response1: u16 { write_bits = 0; }
    0x008 => response2: u16 { write_bits = 0; }
    0x00A => response3: u16 { write_bits = 0; }
    0x00C => response4: u16 { write_bits = 0; }
    0x00E => response5: u16 { write_bits = 0; }
    0x010 => response6: u16 { write_bits = 0; }
    0x012 => response7: u16 { write_bits = 0; }
    // TODO: these registers below
    0x014 => reg_response4: u16 { }
    0x016 => reg_response5: u16 { }
    0x018 => reg_response6: u16 { }
    0x01A => reg_response7: u16 { }
    0x01C => reg_irq_status0: u16 { }
    0x01E => reg_irq_status1: u16 { }
    0x020 => reg_irq_mask0: u16 { }
    0x022 => reg_irq_mask1: u16 { }
    0x024 => reg_clk_ctl: u16 { }
    0x026 => reg_data16_blk_len: u16 { }
    0x028 => reg_card_option: u16 { }
    0x02C => reg_err_status0: u16 { }
    0x02E => reg_err_status1: u16 { }
    0x030 => reg_data16_fifo: u16 { }
    0x0D8 => reg_data16_ctl: u16 { }
    0x0E0 => reg_software_reset: u16 { write_bits = 0b1; }
    0x0F6 => reg_protected: u16 { }
    0x100 => reg_data32_ctl: u16 { }
    0x104 => reg_data32_blk_len: u16 { }
    0x108 => reg_data32_blk_cnt: u16 { }
    0x10C => reg_data32_fifo: u16 { }
});