use std::env;
use std::io::{Read, Write};

#[macro_use]
mod utils;

mod cpu;
mod ram;

fn from_hex(string: String) -> Result<u32, std::num::ParseIntError> {
    let slice = if string.starts_with("0x") {
        &string[2..]
    } else {
        &string[..]
    };
    u32::from_str_radix(slice, 16)
}

fn main() {
    let filename = env::args().nth(1).unwrap();
    let load_offset = from_hex(env::args().nth(2).unwrap()).unwrap();
    let entrypoint = from_hex(env::args().nth(3).unwrap()).unwrap();

    let mut cpu = cpu::Cpu::new();
    let mut ram = ram::Ram::new();

    let mut file = std::fs::File::open(filename).unwrap();
    let file_data = file.metadata().unwrap();
    let file_size = file_data.len();

    let mut filebuf = Vec::<u8>::with_capacity(file_size as usize);

    let size = file.read_to_end(&mut filebuf).unwrap();
    println!("Reading {} bytes from input file", size);
    assert!(size == file_size as usize);

    {
        let mut memslice = ram.borrow_mut::<u8>(load_offset, size);
        {
            use std::borrow::Borrow;
            let copied_size = memslice.write(filebuf.borrow()).unwrap();
            assert!(copied_size == size);
        }
    }

    cpu.reset(entrypoint);
    cpu.run(&mut ram);
}
