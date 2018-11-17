use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use ldr;
use mem;
use utils::bytes;

error_chain! {
    foreign_links {
        Io(::std::io::Error);
    }

    errors {
    }
}

struct FirmSection {
    data_offs: usize,
    size: usize,
    dst_addr: u32,
}

impl FirmSection {
    fn from_header(header: &[u8; 0x30]) -> Option<Self> {
        let offs: u32 = unsafe { bytes::val_at_offs(header, 0x0) };
        let dst: u32  = unsafe { bytes::val_at_offs(header, 0x4) };
        let size: u32 = unsafe { bytes::val_at_offs(header, 0x8) };
        if size == 0 {
            None
        } else {
            Some(FirmSection {
                data_offs: offs as usize,
                size: size as usize,
                dst_addr: dst
            })
        }
    }
}

pub struct FirmLoader {
    filename: PathBuf,
    entry9: u32,
    entry11: u32,
    sections: Vec<FirmSection>
}

impl FirmLoader {
    pub fn from_file(filename: &Path) -> Result<Self> {
        let mut file = File::open(filename)?;
        let mut header = [0u8; 0x200];
        file.read_exact(&mut header)?;

        assert!(&header[..4] == b"FIRM");

        let entry11: u32 = unsafe { bytes::val_at_offs(&header, 0x8) };
        let entry9: u32  = unsafe { bytes::val_at_offs(&header, 0xC) };
        let mut sections = Vec::new();

        for i in 0..4 {
            let section: [u8; 0x30] = unsafe { bytes::val_at_offs(&header, 0x40 + 0x30*i) };
            if let Some(section) = FirmSection::from_header(&section) {
                sections.push(section);
            }
        }

        Ok(FirmLoader {
            filename: filename.to_owned(),
            entry9,
            entry11,
            sections
        })
    }
}

impl ldr::Loader for FirmLoader {
    fn entrypoint9(&self) -> u32 {
        self.entry9
    }
    fn load9(&self, controller: &mut mem::MemController) {
        let mut file = File::open(&self.filename).unwrap();

        for section in &self.sections {
            file.seek(SeekFrom::Start(section.data_offs as u64)).unwrap();

            let mut vaddr = section.dst_addr;
            let mut read_amount = 0;
            let mut read_buf = [0u8; 1024];
            loop {
                let read_buf_size = 1024.min(section.size - read_amount);
                let read_buf = &mut read_buf[..read_buf_size];
                if read_buf_size == 0 { break }
                file.read_exact(read_buf).unwrap();
                controller.write_buf(vaddr, read_buf);
                
                read_amount += read_buf_size;
                vaddr += read_buf_size as u32;
            }
        }
    }

    fn entrypoint11(&self) -> u32 {
        self.entry11
    }
    fn load11(&self, _controller: &mut mem::MemController) {
        // Unnecessary because FIRMs on the 3DS can only load into ARM9-accessible memory 
    }
}
