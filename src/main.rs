extern crate num;

#[macro_use]
mod utils;

mod cpu;
mod ram;

use std::io::{Read, Write};

fn main() {
    let mut cpu = cpu::Cpu::new();
    let mut ram = ram::Ram::new();

    let mut file = std::fs::File::open("").unwrap();
    let mut filebuf = Vec::<u8>::new();

    let size = file.read_to_end(&mut filebuf).unwrap();
    println!("Reading {} bytes from input file", size);
    {
        let mut memslice = ram.borrow_mut(0x08006800, size as u32);
        {
            use std::borrow::Borrow;
            let copied_size = memslice.write(filebuf.borrow()).unwrap();
            assert!(copied_size == size);
        }
    }

    cpu.reset(0x0801B01C);
    cpu.run(&mut ram);
}
