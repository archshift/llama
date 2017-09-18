use std::mem;

use extprim::u128::u128 as u128_t;

use io::emmc::{self, EmmcDevice, TransferType};
use io::emmc::card::{CardStatusReg, CardType};
use io::emmc::cmds;
use utils::bytes;

enum CmdHandler {
    R1(fn(&mut EmmcDevice) -> ()),
    R2(fn(&mut EmmcDevice) -> u128_t),
    R3(fn(&mut EmmcDevice) -> u32),
    R6(fn(&mut EmmcDevice) -> u16),
    R7(fn(&mut EmmcDevice) -> u32)
}

static CMDs: [(usize, CmdHandler, CardType); 13] = [
    (0, CmdHandler::R1(cmds::go_idle_state), CardType::Sdmmc),
    (1, CmdHandler::R3(cmds::send_op_cond), CardType::Mmc),
    (2, CmdHandler::R2(cmds::all_send_cid), CardType::Sdmmc),
    (3, CmdHandler::R6(cmds::get_relative_addr), CardType::Sd),
    (3, CmdHandler::R1(cmds::set_relative_addr), CardType::Mmc),
    (7, CmdHandler::R1(cmds::select_deselect_card), CardType::Sdmmc),
    (8, CmdHandler::R7(cmds::send_if_cond), CardType::Sd),
    (9, CmdHandler::R2(cmds::send_csd), CardType::Sdmmc),
    // (10, CmdHandler::R1(cmds::send_cid)),
    // (12, CmdHandler::R1b(cmds::stop_transmission)),
    (13, CmdHandler::R1(cmds::get_status), CardType::Sdmmc),
    (16, CmdHandler::R1(cmds::set_blocklen), CardType::Sdmmc),
    // (17, CmdHandler::R1(cmds::read_single_block)),
    (18, CmdHandler::R1(|dev: &mut EmmcDevice| cmds::prepare_multi_transfer(dev, TransferType::Read)), CardType::Sdmmc),
    // (23, CmdHandler::R1(cmds::set_block_count)),
    // (24, CmdHandler::R1(cmds::write_block)),
    (25, CmdHandler::R1(|dev: &mut EmmcDevice| cmds::prepare_multi_transfer(dev, TransferType::Write)), CardType::Sdmmc),
    (55, CmdHandler::R1(cmds::app_cmd), CardType::Sd),
    // (58, CmdHandler::R3(cmds::read_ocr))
];

static ACMDs: [(usize, CmdHandler, CardType); 4] = [
    (6, CmdHandler::R1(cmds::set_bus_width), CardType::Sd),
    (41, CmdHandler::R3(cmds::app_send_op_cond), CardType::Sd),
    (42, CmdHandler::R1(cmds::set_clr_card_detect), CardType::Sd),
    (51, CmdHandler::R1(cmds::get_scr), CardType::Sd),
];

#[inline]
fn handle_any_cmd(dev: &mut EmmcDevice, cmdlist: &[(usize, CmdHandler, CardType)], cmd_index: u16) {
    let mut found_wrong_type = false;

    for &(i, ref handler, ty) in cmdlist.iter() {
        if i != cmd_index as usize { continue }

        let card_ty = emmc::get_active_card(dev).ty;
        match (card_ty, ty) {
            (CardType::Sd, CardType::Mmc) | (CardType::Mmc, CardType::Sd) => {
                found_wrong_type = true;
                continue
            }
            (CardType::Sdmmc, _) => panic!("Found card with illegal joint type `CardType::Sdmmc`"),
            _ => {}
        }

        match handler {
            &CmdHandler::R1(f) => {
                f(dev);
                let csr = emmc::get_active_card(dev).csr;
                emmc::push_resp_u32(dev, csr.raw());
            }
            &CmdHandler::R2(f) => {
                let data = f(dev);
                emmc::set_resp_u8(dev, &bytes::from_u128(data));
            }
            &CmdHandler::R3(f) | &CmdHandler::R7(f) => {
                let data = f(dev);
                emmc::push_resp_u32(dev, data);
            }
            &CmdHandler::R6(f) => {
                let data = f(dev);
                let csr = emmc::get_active_card(dev).csr.raw();
                let data32 = (data as u32) << 16
                             | (((csr >> 22) & 0b11) << 14)
                             | (((csr >> 19) & 0b1) << 13)
                             | (csr & 0b1111111111111);
                emmc::push_resp_u32(dev, data32);
            }
        }
        return
    }

    if found_wrong_type {
        let mut csr = emmc::get_active_card(dev).csr;
        bf!(csr.illegal_cmd = 1);
        emmc::push_resp_u32(dev, csr.raw());

        warn!("Tried to run illegal SDMMC (APP_?')CMD{}", cmd_index);
        dev.irq_status0.bitadd_unchecked(emmc::Status0::CmdResponseEnd as u16);
    } else {
        panic!("UNIMPLEMENTED: SDMMC (APP_?')CMD{}", cmd_index)
    }
}

pub fn handle_cmd(dev: &mut EmmcDevice, cmd_index: u16) {
    handle_any_cmd(dev, &CMDs, cmd_index);
}

pub fn handle_acmd(dev: &mut EmmcDevice, cmd_index: u16) {
    handle_any_cmd(dev, &ACMDs, cmd_index);
}