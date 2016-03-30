#[macro_use]
extern crate log;
extern crate env_logger;

use std::{env, sync};
use std::default::Default;
use std::io::{Read, Write};

#[macro_use]
mod utils;

mod cpu;
mod io;
mod mem;
mod system;

fn from_hex(string: String) -> Result<u32, std::num::ParseIntError> {
    let slice = if string.starts_with("0x") {
        &string[2..]
    } else {
        &string[..]
    };
    u32::from_str_radix(slice, 16)
}

fn map_arm9_mem(cont: &mut mem::MemController) {
    let mem_itcm = mem::MemoryBlock::make_ram(0x20);
    for i in 0..0x1000 {
        cont.map_region(i * 0x8000, mem_itcm.clone()); // ITCM
    }
    cont.map_region(0x08000000, mem::MemoryBlock::make_ram(0x400)); // ARM9 RAM
    cont.map_region(0x10000000, mem::MemoryBlock::make_io(io::IoRegion::Arm9(Default::default()), 0x400)); // ARM9 IO
    cont.map_region(0x10100000, mem::MemoryBlock::make_io(io::IoRegion::Arm9(Default::default()), 0x400)); // Shared IO
    cont.map_region(0x18000000, mem::MemoryBlock::make_ram(0x1800)); // VRAM
    cont.map_region(0x1FF00000, mem::MemoryBlock::make_ram(0x200)); // DSP
    cont.map_region(0x1FF80000, mem::MemoryBlock::make_ram(0x200)); // AXI WRAM
    cont.map_region(0x20000000, mem::MemoryBlock::make_ram(0x20000)); // FCRAM
    cont.map_region(0xFFF00000, mem::MemoryBlock::make_ram(0x10)); // DTCM
    cont.map_region(0xFFFF0000, mem::MemoryBlock::make_ram(0x40)); // Bootrom
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
    map_arm9_mem(&mut mem);

    mem.write_buf(load_offset, filebuf.as_slice());

    let mut system = system::System::new(entrypoint, mem);
    system.start();
    system.wait();
}
