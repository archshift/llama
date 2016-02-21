#[macro_use]
extern crate log;
extern crate env_logger;

use std::{env, sync};
use std::io::{Read, Write};

#[macro_use]
mod utils;

mod cpu;
mod mem;

fn from_hex(string: String) -> Result<u32, std::num::ParseIntError> {
    let slice = if string.starts_with("0x") {
        &string[2..]
    } else {
        &string[..]
    };
    u32::from_str_radix(slice, 16)
}

fn main() {
    env_logger::init().unwrap();

    let filename = env::args().nth(1).unwrap();
    let load_offset = from_hex(env::args().nth(2).unwrap()).unwrap();
    let entrypoint = from_hex(env::args().nth(3).unwrap()).unwrap();

    let mut file = std::fs::File::open(filename).unwrap();
    let file_data = file.metadata().unwrap();
    let file_size = file_data.len();

    let mut filebuf = Vec::<u8>::with_capacity(file_size as usize);

    let size = file.read_to_end(&mut filebuf).unwrap();
    info!("Reading {} bytes from input file", size);
    assert!(size == file_size as usize);

    let mut mem = mem::MemController::new();
    mem.add_region(sync::Arc::new(sync::RwLock::new(mem::ItcmRegion::new()))); // ITCM
    mem.add_region(sync::Arc::new(sync::RwLock::new(mem::RamRegion::new(0x08000000, 0x00100000)))); // A9 Internal
    mem.add_region(sync::Arc::new(sync::RwLock::new(mem::RamRegion::new(0x10000000, 0x08000000)))); // IO
    mem.add_region(sync::Arc::new(sync::RwLock::new(mem::RamRegion::new(0x18000000, 0x00600000)))); // VRAM
    mem.add_region(sync::Arc::new(sync::RwLock::new(mem::RamRegion::new(0x1FF00000, 0x00080000)))); // DSP
    mem.add_region(sync::Arc::new(sync::RwLock::new(mem::RamRegion::new(0x1FF80000, 0x00080000)))); // AXI WRAM
    mem.add_region(sync::Arc::new(sync::RwLock::new(mem::RamRegion::new(0x20000000, 0x08000000)))); // FCRAM
    mem.add_region(sync::Arc::new(sync::RwLock::new(mem::RamRegion::new(0xFFF00000, 0x00004000)))); // DTCM
    mem.add_region(sync::Arc::new(sync::RwLock::new(mem::RamRegion::new(0xFFFF0000, 0x00010000)))); // Bootrom

    for i in 0..size {
        mem.write::<u8>(load_offset + i as u32, filebuf[i]);
    }

    // {
    //     let mut memslice = mem.borrow_mut::<u8>(load_offset, size);
    //     {
    //         use std::borrow::Borrow;
    //         let copied_size = memslice.write(filebuf.borrow()).unwrap();
    //         assert!(copied_size == size);
    //     }
    // }

    let mut cpu = cpu::Cpu::new(mem);

    cpu.reset(entrypoint);
    cpu.run();
}
