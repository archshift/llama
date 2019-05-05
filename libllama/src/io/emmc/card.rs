use std::fs::File;
use std::io::{self, Seek, Read, Write};

use io::emmc::TransferType;
use utils::bytes;
use utils::cache::TinyCache;
use fs;

#[derive(Clone, Copy)]
pub enum CardType {
    Mmc,
    Sd,
    Sdmmc
}

#[derive(Clone, Copy)]
pub enum CardState {
    _Idle = 0,
    Ready = 1,
    Ident = 2,
    Stby = 3,
    Tran = 4,
    _Data = 5,
    _Rcv = 6,
    _Prg = 7,
    _Dis = 8,
    _Btst = 9,
    _Slp = 10
}

bf!(CardStatusReg[u32] {
    app_cmd: 5:5,
    ready_for_data: 8:8,
    current_state: 9:12,
    erase_reset: 13:13,
    illegal_cmd: 22:22,
    cmd_crc_err: 23:23,
    erase_seq_err: 28:28,
    address_err: 30:30
});

bf!(CardIdentReg[u128] {});
bf!(CardSpecificData[u128] {});

#[derive(Clone, Copy, Debug)]
pub enum TransferLoc {
    Storage,
    RegScr,
    RegSsr
}

#[derive(Debug)]
pub struct ActiveTransfer {
    loc: TransferLoc,
    pub ty: TransferType,
    pub blocks_left: u16,
    pub fifo_pos: u16,
    seek_pos: u64
}

const CACHE_LINE_SIZE: usize = 512;

pub struct Card {
    pub ty: CardType,
    pub csr: CardStatusReg::Bf,
    pub cid: CardIdentReg::Bf,
    pub csd: CardSpecificData::Bf,
    pub rca: u16,

    storage: File,
    cache: TinyCache<[u8; CACHE_LINE_SIZE], File>,
    transfer: Option<ActiveTransfer>,
}

impl Card {
    pub fn new(ty: CardType, storage: File, cid: CardIdentReg::Bf) -> Card {
        let fill_cacheline = |f: &mut File, pos: u32| {
            let mut out = [0u8; CACHE_LINE_SIZE];
            f.seek(io::SeekFrom::Start(CACHE_LINE_SIZE as u64 * pos as u64)).unwrap();
            f.read(&mut out).unwrap();
            out
        };
        let wb_cacheline = |f: &mut File, pos: u32, data: &[u8; 512]| {
            f.seek(io::SeekFrom::Start(CACHE_LINE_SIZE as u64 * pos as u64)).unwrap();
            f.write(data).unwrap();
        };

        Card {
            ty: ty,
            csr: CardStatusReg::new(0),
            cid: cid,
            csd: CardSpecificData::new(0),
            rca: 1,
            storage: storage,
            cache: TinyCache::new(fill_cacheline, wb_cacheline),
            transfer: None
        }
    }

    pub fn make_transfer(&mut self, loc: TransferLoc, ttype: TransferType, num_blocks: u16) {
        let transfer = ActiveTransfer {
            loc: loc,
            ty: ttype,
            blocks_left: num_blocks,
            fifo_pos: 0,
            seek_pos: 0
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
        self.csr.current_state.set(state as u32);
    }

    pub fn reset(&mut self, _spi: bool) {
        // TODO: stubbed
    }
}

impl io::Read for Card {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        let xfer = self.transfer.as_mut()
            .ok_or(io::Error::new(io::ErrorKind::NotConnected, "No active transfer found"))?;
        let to_advance = match xfer.loc {
            TransferLoc::Storage => {
                let pos = xfer.seek_pos as u32;
                let read = self.cache.get_or(pos / CACHE_LINE_SIZE as u32, &mut self.storage);

                let read_start = (pos as usize) % CACHE_LINE_SIZE;
                let read_end = (read_start + buf.len()).min(CACHE_LINE_SIZE);
                let read_amount = read_end - read_start;

                buf[..read_amount].copy_from_slice(&read[read_start..read_end]);

                Ok(read_amount)
            },
            TransferLoc::RegScr => {
                warn!("STUBBED: Read from SD card SCR register");
                for b in buf.iter_mut() { *b = 0; }
                Ok(buf.len().checked_sub(xfer.seek_pos as usize).unwrap())
            }
            TransferLoc::RegSsr => {
                warn!("STUBBED: Read from SD card SSR register");
                for b in buf.iter_mut() { *b = 0; }
                Ok(buf.len().checked_sub(xfer.seek_pos as usize).unwrap())
            }
        };
        if let Ok(to_advance) = to_advance {
            xfer.seek_pos += to_advance as u64;
        }
        to_advance
    }
}

impl io::Write for Card {
    fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
        let xfer = self.transfer.as_mut()
            .ok_or(io::Error::new(io::ErrorKind::NotConnected, "No active transfer found"))?;
        let to_advance = match xfer.loc {
            TransferLoc::Storage => {
                let pos = xfer.seek_pos as u32;

                let write_start = (pos as usize) % CACHE_LINE_SIZE;
                let write_end = (write_start + buf.len()).min(CACHE_LINE_SIZE);
                let write_amount = write_end - write_start;

                let updater = |_: u32, line: &mut [u8; CACHE_LINE_SIZE]| {
                    line[write_start..write_end].copy_from_slice(&buf[..write_amount]);
                };

                self.cache.update_or(pos / CACHE_LINE_SIZE as u32, updater, &mut self.storage);
                Ok(write_amount)
            }
            TransferLoc::RegScr => {
                Err(io::Error::new(io::ErrorKind::PermissionDenied, "Cannot write to SCR register"))
            }
            TransferLoc::RegSsr => {
                Err(io::Error::new(io::ErrorKind::PermissionDenied, "Cannot write to SSR register"))
            }
        };
        if let Ok(to_advance) = to_advance {
            xfer.seek_pos += to_advance as u64;
        }
        to_advance
    }

    fn flush(&mut self) -> Result<(), io::Error> {
        let xfer = self.transfer.as_mut()
            .ok_or(io::Error::new(io::ErrorKind::NotConnected, "No active transfer found"))?;
        match xfer.loc {
            TransferLoc::Storage => Ok(self.cache.invalidate(&mut self.storage)),
            TransferLoc::RegScr => Ok(()),
            TransferLoc::RegSsr => Ok(()),
        }
    }
}

impl io::Seek for Card {
    fn seek(&mut self, seek_from: io::SeekFrom) -> Result<u64, io::Error> {
        let xfer = self.transfer.as_mut()
            .ok_or(io::Error::new(io::ErrorKind::NotConnected, "No active transfer found"))?;
        let new_pos = match seek_from {
            io::SeekFrom::Current(v) => ((xfer.seek_pos as i64) + v as i64) as u64,
            io::SeekFrom::End(_v) => unimplemented!(),
            io::SeekFrom::Start(v) => v as u64,
        };
        xfer.seek_pos = new_pos;
        Ok(new_pos)
    }
}

impl Drop for Card {
    fn drop(&mut self) {
        self.cache.invalidate(&mut self.storage);
    }
}

pub fn nand_cid() -> CardIdentReg::Bf {
    let mut file = fs::open_file(fs::LlamaFile::NandCid).unwrap();
    let mut bytes = [0u8; 16];
    file.read_exact(&mut bytes).unwrap();
    CardIdentReg::new(bytes::to_u128(&bytes))
}

pub fn sd_cid() -> CardIdentReg::Bf {
    CardIdentReg::new(0)
}
