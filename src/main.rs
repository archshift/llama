#[macro_use]
extern crate log;
extern crate env_logger;

use std::env;
use std::io::Read;

#[macro_use]
mod utils;

mod cpu;
mod dbgcore;
mod io;
mod mem;
mod hwcore;

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

    let mem = hwcore::map_memory_regions();
    mem.write_buf(load_offset, filebuf.as_slice());

    let mut hwcore = hwcore::HwCore::new(entrypoint, mem);
    hwcore.start();
    let mut debugger = dbgcore::DbgCore::new(hwcore);
    let hw = debugger.pause();
}
