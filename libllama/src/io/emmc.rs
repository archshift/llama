bfdesc!(RegCmd: u16, {
    command_index: 0 => 5,
    command_type: 6 => 7,
    response_type: 8 => 10,
    has_data: 11 => 11,
    is_reading: 12 => 12,
    has_multi_block: 13 => 13
});

enum Status0 {
    CmdResponseEnd = (1 << 0),
    DataEnd     = (1 << 2),
    CardRemove  = (1 << 3),
    CardInsert  = (1 << 4),
    SigState    = (1 << 5),
    WRProtect   = (1 << 7),
    CardRemoveA = (1 << 8),
    CardInsertA = (1 << 9),
    SigStateA   = (1 << 10),
}

enum Status1 {
    CmdIndexErr = (1 << 0),
    CrcFail     = (1 << 1),
    StopBitErr  = (1 << 2),
    DataTimeout = (1 << 3),
    RxOverflow  = (1 << 4),
    TxUnderrun  = (1 << 5),
    CmdTimeout  = (1 << 6),
    RxReady     = (1 << 8),
    TxRq        = (1 << 9),
    IllFunc     = (1 << 13),
    CmdBusy     = (1 << 14),
    IllAccess   = (1 << 15),
}

iodevice!(EmmcDevice, {
    0x000 => cmd: u16 {
        write_effect = |dev: &mut EmmcDevice| {
            let cmd = bf!((dev.cmd.get()) @ RegCmd::command_index);
            dev.irq_status0.set_unchecked(0);
            match cmd {
                1 => {
                    dev.response0.set_unchecked(0x0000);
                    dev.response1.set_unchecked(0x8000);
                    dev.irq_status0.set_unchecked(Status0::CmdResponseEnd as u16);
                    warn!("STUBBED: SDMMC CMD1 SEND_OP_COND!");
                }
                c @ _ => error!("UNIMPLEMENTED: SDMMC CMD{}!", c),
            }
        };
    }
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
    0x0FC => unknown0: u16 { }
    0x0FE => unknown1: u16 { }
    0x100 => data32_ctl: u16 { }
    0x104 => data32_blk_len: u16 { }
    0x108 => data32_blk_cnt: u16 { }
    0x10C => data32_fifo: u16 { }
});