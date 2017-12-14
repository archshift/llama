use std::io::{Seek, SeekFrom};

use extprim::u128::u128 as u128_t;

use io::emmc::{self, EmmcDevice, Status0, Status1, TransferType};
use io::emmc::card::{CardState, TransferLoc};

pub fn go_idle_state(dev: &mut EmmcDevice) {
    for card in dev._internal_state.cards.iter_mut() {
        card.reset(false);
    }
    warn!("STUBBED: SDMMC CMD0 GO_IDLE_STATE!");
}

pub fn send_op_cond(dev: &mut EmmcDevice) -> u32 {
    let ocr = emmc::get_params_u32(dev);
    emmc::get_active_card(dev).set_state(CardState::Ready);
    warn!("STUBBED: SDMMC CMD1 SEND_OP_COND!");
    return ocr | (1 << 31);
}

pub fn all_send_cid(dev: &mut EmmcDevice) -> u128_t {
    let cid = emmc::get_active_card(dev).cid;
    emmc::get_active_card(dev).set_state(CardState::Ident);
    return cid.raw();
}

pub fn set_relative_addr(dev: &mut EmmcDevice) {
    let reladdr = emmc::get_params_u16(dev)[1];
    emmc::get_active_card(dev).rca = reladdr;
    emmc::get_active_card(dev).set_state(CardState::Stby);
}

pub fn get_relative_addr(dev: &mut EmmcDevice) -> u16 {
    let rca = emmc::get_active_card(dev).rca + 1;
    emmc::get_active_card(dev).rca = rca;
    emmc::get_active_card(dev).set_state(CardState::Stby);
    rca
}

pub fn switch(dev: &mut EmmcDevice) {
    warn!("STUBBED: SDMMC CMD6 SWITCH!");
}

pub fn select_deselect_card(dev: &mut EmmcDevice) {
    emmc::get_active_card(dev).set_state(CardState::Tran);
    warn!("STUBBED: SDMMC CMD7 SELECT_DESELECT_CARD!");
}

pub fn send_if_cond(dev: &mut EmmcDevice) -> u32 {
    let out = emmc::get_params_u32(dev);
    warn!("STUBBED: SDMMC CMD8 SEND_IF_COND!");
    out
}

pub fn send_csd(dev: &mut EmmcDevice) -> u128_t {
    let csd = emmc::get_active_card(dev).csd;
    emmc::get_active_card(dev).set_state(CardState::Ident);
    return csd.raw();
}

pub fn stop_transmission(dev: &mut EmmcDevice) {
    emmc::get_active_card(dev).kill_transfer();
    emmc::clear_status(dev, Status1::RxReady);
    emmc::clear_status(dev, Status1::TxRq);
    warn!("STUBBED: SDMMC CMD12 STOP_TRANSMISSION!");
}

pub fn set_blocklen(dev: &mut EmmcDevice) {
    warn!("STUBBED: SDMMC CMD16 SET_BLOCKLEN!");
}

pub fn prepare_multi_transfer(dev: &mut EmmcDevice, ttype: TransferType) {
    let file_offset = emmc::get_params_u32(&*dev);

    let block_count = if emmc::use_32bit(dev) {
        let ctl = match ttype {
            TransferType::Read => bf!((dev.data32_ctl.get()) @ emmc::RegData32Ctl::rx32rdy as 1),
            TransferType::Write => dev.data32_ctl.get() // TODO: Why is this?
        };
        dev.data32_ctl.set_unchecked(ctl);
        dev.data32_blk_cnt.get()
    } else {
        match ttype {
            TransferType::Read => emmc::trigger_status(dev, Status1::RxReady),
            TransferType::Write => emmc::trigger_status(dev, Status1::TxRq)
        }
        dev.data16_blk_cnt.get()
    };

    let card = &mut emmc::get_active_card(dev);
    card.make_transfer(TransferLoc::Storage, ttype, block_count);
    card.seek(SeekFrom::Start(file_offset as u64)).unwrap();
    trace!("Seeking SDMMC pointer to offset 0x{:08X}!", file_offset);
}

pub fn app_cmd(dev: &mut EmmcDevice) {
    bf!((emmc::get_active_card(dev).csr).app_cmd = 1);
}

pub fn set_bus_width(dev: &mut EmmcDevice) {
    warn!("STUBBED: SDMMC ACMD6 SET_BUS_WIDTH!");
}

pub fn app_send_op_cond(dev: &mut EmmcDevice) -> u32 {
    let voltages = emmc::get_params_u32(dev) & 0xFFF;
    emmc::get_active_card(dev).set_state(CardState::Ready);
    warn!("STUBBED: SDMMC ACMD41 SD_SEND_OP_COND!");
    return voltages | (1 << 31);
}

pub fn set_clr_card_detect(dev: &mut EmmcDevice) {
    warn!("STUBBED: SDMMC ACMD42 SET_CLR_CARD_DETECT!");
}

pub fn get_scr(dev: &mut EmmcDevice) {
    warn!("STUBBED: SDMMC ACMD51 GET_SCR!");
    let num_blocks = dev.data16_blk_cnt.get();
    emmc::trigger_status(dev, Status1::RxReady);
    emmc::get_active_card(dev).make_transfer(TransferLoc::RegScr, TransferType::Read, num_blocks);
}