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

#[derive(Debug, Default)]
struct EmmcDeviceState {
    expect_appcmd: bool,
}

fn handle_cmd(dev: &mut EmmcDevice, cmd_index: u16) {
    match cmd_index {
        0 => {
            warn!("STUBBED: SDMMC CMD0 GO_IDLE_STATE!");
        }
        1 => {
            dev.response0.set_unchecked(0x0000);
            dev.response1.set_unchecked(0x8000);
            dev.irq_status0.set_unchecked(Status0::CmdResponseEnd as u16);
            warn!("STUBBED: SDMMC CMD1 SEND_OP_COND!");
        }
        2 => {
            dev.irq_status0.set_unchecked(Status0::CmdResponseEnd as u16);
            warn!("STUBBED: SDMMC CMD2 ALL_SEND_CID!");
        }
        3 => {
            dev.irq_status0.set_unchecked(Status0::CmdResponseEnd as u16);
            warn!("STUBBED: SDMMC CMD3 SEND/SET_RELATIVE_ADDR!");
        }
        6 => {
            dev.irq_status0.set_unchecked(Status0::CmdResponseEnd as u16);
            warn!("STUBBED: SDMMC CMD6 SWITCH!");
        }
        7 => {
            dev.irq_status0.set_unchecked(Status0::CmdResponseEnd as u16);
            warn!("STUBBED: SDMMC CMD7 SELECT_DESELECT_CARD!");
        }
        8 => {
            dev.irq_status0.set_unchecked(Status0::CmdResponseEnd as u16);
            warn!("STUBBED: SDMMC CMD8 SET_IF_COND!");
        }
        9 => {
            dev.irq_status0.set_unchecked(Status0::CmdResponseEnd as u16);
            warn!("STUBBED: SDMMC CMD9 SEND_CSD!");
        }
        13 => {
            dev.irq_status0.set_unchecked(Status0::CmdResponseEnd as u16);
            warn!("STUBBED: SDMMC CMD13 GET_STATUS!");
        }
        16 => {
            dev.irq_status0.set_unchecked(Status0::CmdResponseEnd as u16);
            warn!("STUBBED: SDMMC CMD16 SET_BLOCKLEN!");
        }
        55 => {
            dev._internal_state.expect_appcmd = true;
            dev.irq_status0.set_unchecked(Status0::CmdResponseEnd as u16);
            warn!("STUBBED: SDMMC CMD55 APP_CMD!");
        }
        c => panic!("UNIMPLEMENTED: SDMMC CMD{}; device.cmd=0x{:X}!", c, dev.cmd.get()),
    }
}

fn handle_acmd(dev: &mut EmmcDevice, acmd_index: u16) {
    match acmd_index {
        6 => {
            dev.irq_status0.set_unchecked(Status0::CmdResponseEnd as u16);
            warn!("STUBBED: SDMMC ACMD6 SET_BUS_WIDTH!");
        }
        41 => {
            dev.irq_status0.set_unchecked(Status0::CmdResponseEnd as u16);
            warn!("STUBBED: SDMMC ACMD41 SD_SEND_OP_COND!");
        }
        c => panic!("UNIMPLEMENTED: SDMMC ACMD{}; device.cmd=0x{:X}!", c, dev.cmd.get()),
    }
}

fn reg_cmd_onupdate(dev: &mut EmmcDevice) {
    let index = bf!((dev.cmd.get()) @ RegCmd::command_index);

    dev.irq_status0.set_unchecked(0);

    if dev._internal_state.expect_appcmd {
        assert_eq!(bf!((dev.cmd.get()) @ RegCmd::command_type), 1);
        handle_acmd(dev, index);
        dev._internal_state.expect_appcmd = false;
    } else {
        handle_cmd(dev, index);
    }
}

iodevice!(EmmcDevice, {
    internal_state: EmmcDeviceState;
    regs:
    0x000 => cmd: u16 {
        write_effect = reg_cmd_onupdate;
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