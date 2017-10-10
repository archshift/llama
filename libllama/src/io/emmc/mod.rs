mod card;
mod cmds;
mod mode_sd;

use std::fmt;
use std::io::{Read, Write};
use std::mem;

use io::emmc::card::Card;
use cpu::irq;

bfdesc!(RegCmd: u16, {
    command_index: 0 => 5,
    command_type: 6 => 7,
    response_type: 8 => 10,
    has_data: 11 => 11,
    is_reading: 12 => 12,
    has_multi_block: 13 => 13
});

bfdesc!(RegData16Ctl: u16, {
    use_32bit: 1 => 1
});

bfdesc!(RegData32Ctl: u16, {
    tx32rq_enable: 12 => 12,
    rx32rdy_enable: 11 => 11,
    clear_fifo32: 10 => 10,
    tx32rq: 9 => 9,
    rx32rdy: 8 => 8,
    use_32bit: 1 => 1
});

#[derive(Clone, Copy)]
enum Status {
    Lo(Status0),
    Hi(Status1)
}

#[derive(Clone, Copy)]
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

impl Into<Status> for Status0 {
    fn into(self) -> Status {
        Status::Lo(self)
    }
}

#[derive(Clone, Copy)]
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
    IllegalCmd  = (1 << 15),
}

impl Into<Status> for Status1 {
    fn into(self) -> Status {
        Status::Hi(self)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransferType {
    Read,
    Write
}

pub struct EmmcDeviceState {
    irq_reqs: irq::IrqRequests,
    irq_statuses: [u16; 2],
    cards: [Card; 2],
}

impl EmmcDeviceState {
    pub fn new(irq_reqs: irq::IrqRequests) -> EmmcDeviceState {
        EmmcDeviceState {
            irq_reqs: irq_reqs,
            irq_statuses: [0 | (Status0::SigState as u16), 0],
            cards: [
                Card::new(card::CardType::Sd, card::sd_storage(), card::sd_cid()),
                Card::new(card::CardType::Mmc, card::nand_storage(), card::nand_cid())
            ]
        }
    }
}

impl fmt::Debug for EmmcDeviceState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "EmmcDeviceState {{ }}")
    }
}

fn get_active_card<'a>(dev: &'a mut EmmcDevice) -> &'a mut Card {
    &mut dev._internal_state.cards[(dev.port_select.get() & 1) as usize]
}

fn get_params_u16(dev: &EmmcDevice) -> [u16; 2] {
    [dev.param0.get(), dev.param1.get()]
}
fn get_params_u32(dev: &EmmcDevice) -> u32 {
    dev.param0.get() as u32 | ((dev.param1.get() as u32) << 16)
}

fn push_resp_u32(dev: &mut EmmcDevice, data: u32) {
    dev.response6.set_unchecked(dev.response4.get()); dev.response7.set_unchecked(dev.response5.get());
    dev.response4.set_unchecked(dev.response2.get()); dev.response5.set_unchecked(dev.response3.get());
    dev.response2.set_unchecked(dev.response0.get()); dev.response3.set_unchecked(dev.response1.get());
    dev.response1.set_unchecked((data >> 16) as u16);
    dev.response0.set_unchecked(data as u16)
}

fn set_resp_u16(dev: &mut EmmcDevice, data: &[u16]) {
    let mut resps = [ &mut dev.response0, &mut dev.response1, &mut dev.response2, &mut dev.response3,
                      &mut dev.response4, &mut dev.response5, &mut dev.response6, &mut dev.response7 ];
    for (r, d) in resps.iter_mut().zip(data.iter()) {
        r.set_unchecked(*d);
    }
}

fn set_resp_u8(dev: &mut EmmcDevice, data: &[u8]) {
    let mut resps = [ &mut dev.response0, &mut dev.response1, &mut dev.response2, &mut dev.response3,
                      &mut dev.response4, &mut dev.response5, &mut dev.response6, &mut dev.response7 ];
    let mut data_it = data.iter();
    for r in resps.iter_mut() {
        if let (Some(b0), Some(b1)) = (data_it.next(), data_it.next()) {
            r.set_unchecked(((*b1 as u16) << 8) | *b0 as u16);
        }
    }
}

fn use_32bit(dev: &EmmcDevice) -> bool {
    bf!((dev.data16_ctl.get()) @ RegData16Ctl::use_32bit) == 1
    && bf!((dev.data32_ctl.get()) @ RegData32Ctl::use_32bit) == 1
}

fn trigger_status<S: Into<Status>>(dev: &mut EmmcDevice, status: S) {
    let irq_add = match status.into() {
        Status::Lo(s0) => {
            let s0 = s0 as u16;
            dev._internal_state.irq_statuses[0] |= s0;
            s0 & !dev.irq_mask0.get() & 0b00000011_00011101 != 0
        }
        Status::Hi(s1) => {
            let s1 = s1 as u16;
            dev._internal_state.irq_statuses[1] |= s1;
            s1 & !dev.irq_mask1.get() & 0b10001011_01111111 != 0
        }
    };
    if irq_add {
        trace!("SDMMC: Triggering IRQs 0: {:08X}, 1: {:08X}",
                dev._internal_state.irq_statuses[0] & dev.irq_mask0.get(),
                dev._internal_state.irq_statuses[1] & dev.irq_mask1.get());
        dev._internal_state.irq_reqs.add(irq::IrqType::Sdio1);
    }
}

fn clear_status<S: Into<Status>>(dev: &mut EmmcDevice, status: S) {
    match status.into() {
        Status::Lo(s0) => {
            let s0 = s0 as u16;
            dev._internal_state.irq_statuses[0] &= !s0;
        }
        Status::Hi(s1) => {
            let s1 = s1 as u16;
            dev._internal_state.irq_statuses[1] &= !s1;
        }
    };
}

fn reg_cmd_onupdate(dev: &mut EmmcDevice) {
    let index = bf!((dev.cmd.get()) @ RegCmd::command_index);

    let csr = get_active_card(dev).csr;
    if bf!((dev.cmd.get()) @ RegCmd::command_type) == 1 || bf!(csr.app_cmd) == 1 {
        bf!((get_active_card(dev).csr).app_cmd = 0);
        trace!("Running SDMMC ACMD{}", index);
        mode_sd::handle_acmd(dev, index);
    } else {
        trace!("Running SDMMC CMD{}", index);
        mode_sd::handle_cmd(dev, index)
    }

    trigger_status(dev, Status0::CmdResponseEnd);
    clear_status(dev, Status1::CmdBusy);
}

fn reg_irqstat_read(dev: &mut EmmcDevice, stat_index: usize) {
    match stat_index {
        0 => dev.irq_status0.set_unchecked(dev._internal_state.irq_statuses[0]),
        1 => dev.irq_status1.set_unchecked(dev._internal_state.irq_statuses[1]),
        _ => unreachable!()
    }
}

fn reg_irqstat_onupdate(dev: &mut EmmcDevice, stat_index: usize) {
    match stat_index {
        0 => dev._internal_state.irq_statuses[0] &= dev.irq_status0.get() | !0b00000011_00011101,
        1 => dev._internal_state.irq_statuses[1] &= dev.irq_status1.get() | !0b10001011_01111111,
        _ => unreachable!()
    }
}

fn reg_fifo_mod(dev: &mut EmmcDevice, transfer_type: TransferType, is_32bit: bool) {
    let fifo_size = if use_32bit(dev) {
        dev.data32_blk_len.get()
    } else {
        dev.data16_blk_len.get()
    };

    let should_stop = {
        let transfer = match get_active_card(dev).get_transfer_mut() {
            Some(t) => t,
            None => return
        };
        assert_eq!(transfer.ty, transfer_type);

        trace!("{} SD FIFO! blocks left: {}, fifo pos: {}",
               match transfer_type { TransferType::Read => "Reading from", TransferType::Write => "Writing to"},
               transfer.blocks_left, transfer.fifo_pos);

        transfer.fifo_pos += if is_32bit { 4 } else { 2 };

        if transfer.fifo_pos >= fifo_size {
            transfer.blocks_left -= 1;
            transfer.fifo_pos = 0;
        }
        transfer.blocks_left == 0
    };

    let mut buf16 = [0u8; 2];
    let mut buf32 = [0u8; 4];
    match (transfer_type, is_32bit) {
        (TransferType::Read, false) => {
            get_active_card(dev).storage.read_exact(&mut buf16).unwrap();
            dev.data16_fifo.set_unchecked(unsafe { mem::transmute(buf16) });

            // Setting these flags: hack to keep the client reading even after acknowledging
            trigger_status(dev, Status1::RxReady);
        }
        (TransferType::Write, false) => {
            buf16 = unsafe { mem::transmute(dev.data16_fifo.get()) };
            get_active_card(dev).storage.write_all(&buf16).unwrap();

            trigger_status(dev, Status1::TxRq);
        }
        (TransferType::Read, true) => {
            get_active_card(dev).storage.read_exact(&mut buf32).unwrap();
            dev.data32_fifo.set_unchecked(unsafe { mem::transmute(buf32) });

            let new_ctl = bf!((dev.data32_ctl.get()) @ RegData32Ctl::rx32rdy as 1);
            dev.data32_ctl.set_unchecked(new_ctl);
        }
        (TransferType::Write, true) => {
            buf32 = unsafe { mem::transmute(dev.data32_fifo.get()) };
            get_active_card(dev).storage.write_all(&buf32).unwrap();

            // Don't set flags. TODO: Why is this?
        }
    };

    if should_stop {
        trigger_status(dev, Status0::DataEnd);
        mode_sd::handle_cmd(dev, 12); // STOP_TRANSMISSION
    }
}

iodevice!(EmmcDevice, {
    internal_state: EmmcDeviceState;
    regs: {
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
        0x01C => irq_status0: u16 {
            read_effect = |dev: &mut EmmcDevice| reg_irqstat_read(dev, 0);
            write_effect = |dev: &mut EmmcDevice| reg_irqstat_onupdate(dev, 0);
        }
        0x01E => irq_status1: u16 {
            read_effect = |dev: &mut EmmcDevice| reg_irqstat_read(dev, 1);
            write_effect = |dev: &mut EmmcDevice| reg_irqstat_onupdate(dev, 1);
        }
        0x020 => irq_mask0: u16 {
            write_bits = 0b00000011_00011101;
            write_effect = |dev: &mut EmmcDevice| trace!("Masking irq_status0 bits {:04X}", dev.irq_mask0.get());
        }
        0x022 => irq_mask1: u16 {
            write_bits = 0b10001011_01111111;
            write_effect = |dev: &mut EmmcDevice| trace!("Masking irq_status1 bits {:04X}", dev.irq_mask1.get());
        }
        0x024 => clk_ctl: u16 { }
        0x026 => data16_blk_len: u16 { }
        0x028 => card_option: u16 { }
        0x02C => err_status0: u16 { }
        0x02E => err_status1: u16 { }
        0x030 => data16_fifo: u16 {
            read_effect = |dev: &mut EmmcDevice| reg_fifo_mod(dev, TransferType::Read, false);
            write_effect = |dev: &mut EmmcDevice| reg_fifo_mod(dev, TransferType::Write, false);
        }
        0x036 => unkirq_stat: u16 { }
        0x038 => unkirq_mask: u16 { }
        0x0D8 => data16_ctl: u16 {
            default = 0b00010000_00010000;
            write_bits = 0b00000000_00100010;
        }
        0x0E0 => software_reset: u16 { write_bits = 0b1; }
        0x0F6 => protected: u16 { }
        0x0F8 => nand_conn_stat: u16 { default = 0b00000000_00000100; }
        0x0FA => nand_conn_mask: u16 { }
        0x0FC => unknown2: u16 { }
        0x0FE => unknown3: u16 { }
        0x100 => data32_ctl: u16 {
            write_bits = 0b00011111_00000010;
        }
        0x104 => data32_blk_len: u16 { }
        0x108 => data32_blk_cnt: u16 { }
        0x10C => data32_fifo: u32 {
            read_effect = |dev: &mut EmmcDevice| reg_fifo_mod(dev, TransferType::Read, true);
            write_effect = |dev: &mut EmmcDevice| reg_fifo_mod(dev, TransferType::Write, true);
        }
    }
});