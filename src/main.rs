mod decoder;
mod ram;

use std::io::{Read, Write};

struct Cpu {
    regs: [u32; 16],
}

impl Cpu {
    fn new() -> Cpu {
        Cpu {
            regs: [0; 16],
        }
    }
}

fn main() {
    let cpu = Cpu::new();
    let mut ram = ram::Ram::new();

    let mut file = std::fs::File::open("/Users/gui/MEGA/Games/3DS Secrets/9.? NATIVE_FIRM/firm_2_08006800.bin").unwrap();
    let mut filebuf = Vec::<u8>::new();

    let size = file.read_to_end(&mut filebuf).unwrap();
    println!("Reading {} bytes from input file", size);
    {
        let mut memslice = ram.borrow(0x08006800, size as u32);
        let copied_size = filebuf.write(memslice).unwrap();
        assert!(copied_size == size);
    }

    let mut i = 0;
    while i < 0x100 {
        let instr: u32 = unsafe {
            let ptr = &filebuf[i] as *const u8;
            let ptr32 = ptr as *const u32;
            *ptr32
        };
        println!("{:#X}: {:#X} {:?}", 0x08006800 + i, instr, decoder::decode_instruction(instr));
        i += 4;
    }
}
