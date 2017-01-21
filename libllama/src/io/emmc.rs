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
    0x004 => param0: u16 { }
    0x006 => param1: u16 { }
    0x008 => stop: u16 { }
    0x00A => data16_blk_cnt: u16 { }
    0x00C => response0: u16 { write_bits = 0; }
    0x00E => response1: u16 { write_bits = 0; }
    0x010 => response2: u16 { write_bits = 0; }
    0x012 => response3: u16 { write_bits = 0; }
    0x014 => response4: u16 { write_bits = 0; }
    0x016 => response5: u16 { write_bits = 0; }
    0x018 => response6: u16 { write_bits = 0; }
    0x01A => response7: u16 { write_bits = 0; }
    0x01C => irq_status0: u16 { }
    0x01E => irq_status1: u16 { }
    0x020 => irq_mask0: u16 { }
    0x022 => irq_mask1: u16 { }
    0x024 => clk_ctl: u16 { }
    0x026 => data16_blk_len: u16 { }
    0x028 => card_option: u16 { }
    0x02C => err_status0: u16 { }
    0x02E => err_status1: u16 { }
    0x030 => data16_fifo: u16 { }
    0x0D8 => data16_ctl: u16 { }
    0x0E0 => software_reset: u16 { write_bits = 0b1; }
    0x0F6 => protected: u16 { }
    0x100 => data32_ctl: u16 { }
    0x104 => data32_blk_len: u16 { }
    0x108 => data32_blk_cnt: u16 { }
    0x10C => data32_fifo: u16 { }
});