use std::env;
use std::fs::{File, OpenOptions};
use std::io::Read;

use extprim::u128::u128 as u128_t;

use io::emmc::TransferType;
use utils::bytes;

#[derive(Clone, Copy)]
pub enum CardType {
    Mmc,
    Sd,
    Sdmmc
}

#[derive(Clone, Copy)]
pub enum CardState {
    Idle = 0,
    Ready = 1,
    Ident = 2,
    Stby = 3,
    Tran = 4,
    Data = 5,
    Rcv = 6,
    Prg = 7,
    Dis = 8,
    Btst = 9,
    Slp = 10
}

bitfield!(CardStatusReg: u32, {
    app_cmd: 5 => 5,
    ready_for_data: 8 => 8,
    current_state: 9 => 12,
    erase_reset: 13 => 13,
    illegal_cmd: 22 => 22,
    cmd_crc_err: 23 => 23,
    erase_seq_err: 28 => 28,
    address_err: 30 => 30
});

bitfield!(CardIdentReg: u128_t, {});
bitfield!(CardSpecificData: u128_t, {});

#[derive(Debug)]
pub struct ActiveTransfer {
    pub ty: TransferType,
    pub blocks_left: u16,
    pub fifo_pos: u16
}

pub struct Card {
    pub ty: CardType,
    pub csr: CardStatusReg,
    pub cid: CardIdentReg,
    pub csd: CardSpecificData,
    pub rca: u16,

    pub storage: File,
    transfer: Option<ActiveTransfer>,
}

impl Card {
    pub fn new(ty: CardType, storage: File, cid: CardIdentReg) -> Card {
        Card {
            ty: ty,
            csr: CardStatusReg::new(0),
            cid: cid,
            csd: CardSpecificData::new(u128_t::new(0)),
            rca: 1,
            storage: storage,
            transfer: None
        }
    }

    pub fn make_transfer(&mut self, ttype: TransferType, num_blocks: u16) {
        let transfer = ActiveTransfer {
            ty: ttype,
            blocks_left: num_blocks,
            fifo_pos: 0
        };
        trace!("Initializing SDMMC transfer ({}): {:?}", if ttype == TransferType::Read { "read" } else { "write" }, transfer);
        self.transfer = Some(transfer);
    }

    pub fn get_transfer_mut<'a>(&'a mut self) -> Option<&'a mut ActiveTransfer> {
        self.transfer.as_mut()
    }

    pub fn kill_transfer(&mut self) {
        self.transfer = None;
    }

    pub fn set_state(&mut self, state: CardState) {
        bf!((self.csr).current_state = state as u32);
    }

    pub fn reset(&mut self, spi: bool) {
        // TODO: stubbed
    }
}

pub fn nand_storage() -> File {
    let filename = format!("{}/{}", env::var("HOME").unwrap(), "/.config/llama-nand.bin");
    match OpenOptions::new().read(true).write(true)
                            .open(&filename) {
        Ok(file) => file,
        Err(x) => panic!("Failed to open NAND file `{}`; {:?}", filename, x)
    }
}

pub fn nand_cid() -> CardIdentReg {
    let filename = format!("{}/{}", env::var("HOME").unwrap(), "/.config/llama-nand-cid.bin");
    let mut file = match OpenOptions::new().read(true).write(true)
                                           .open(&filename) {
        Ok(file) => file,
        Err(x) => panic!("Failed to open NAND CID file `{}`; {:?}", filename, x)
    };
    let mut bytes = [0u8; 16];
    file.read_exact(&mut bytes).unwrap();
    CardIdentReg::new(bytes::to_u128(&bytes))
}

pub fn sd_storage() -> File {
    let filename = format!("{}/{}", env::var("HOME").unwrap(), "/.config/llama-sd.fat");
    match OpenOptions::new().read(true).write(true)
                            .open(&filename) {
        Ok(file) => file,
        Err(x) => panic!("Failed to open SD card file `{}`; {:?}", filename, x)
    }
}

pub fn sd_cid() -> CardIdentReg {
    CardIdentReg::new(u128_t::new(0))
}