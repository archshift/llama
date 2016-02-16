use std;
use std::slice;

fn fast_zeroed_mem(size: usize) -> Box<[u8]> {
    let mut vec = Vec::<u8>::with_capacity(size);
    unsafe {
        std::ptr::write_bytes(vec.as_mut_ptr(), 0, size);
        vec.set_len(size);
    }
    vec.into_boxed_slice()
}

trait MemoryRegion: Send {
    fn read(&self, addr: u32) -> u8;
    fn write(&mut self, addr: u32, data: u8);
    fn borrow(&self, addr: u32, size: u32) -> &[u8];
    fn borrow_mut(&mut self, addr: u32, size: u32) -> &mut [u8];

    fn get_base_addr(&self) -> u32;
    fn get_size(&self) -> u32;

    fn check_addr(&self, addr: u32) -> bool {
        self.check_bounds(addr, 0)
    }
    fn check_bounds(&self, addr: u32, size: u32) -> bool {
        (addr >= self.get_base_addr()) && (addr + size < self.get_base_addr() + self.get_size())
    }
}

struct RamRegion {
    base_addr: u32,
    data: Box<[u8]>,
}

impl RamRegion {
    fn new(base_addr: u32, size: u32) -> RamRegion {
        RamRegion {
            base_addr: base_addr,
            data: fast_zeroed_mem(size as usize),
        }
    }
}

impl MemoryRegion for RamRegion {
    fn read(&self, addr: u32) -> u8 {
        assert!(self.check_addr(addr));
        self.data[(addr - self.get_base_addr()) as usize]
    }

    fn write(&mut self, addr: u32, data: u8) {
        assert!(self.check_addr(addr));
        self.data[(addr - self.get_base_addr()) as usize] = data;
    }

    fn borrow(&self, addr: u32, size: u32) -> &[u8] {
        assert!(self.check_bounds(addr, size));
        let index = (addr - self.get_base_addr()) as usize;
        &self.data[index .. index + size as usize]
    }

    fn borrow_mut(&mut self, addr: u32, size: u32) -> &mut [u8] {
        assert!(self.check_bounds(addr, size));
        let index = (addr - self.get_base_addr()) as usize;
        &mut self.data[index .. index + size as usize]
    }

    fn get_base_addr(&self) -> u32 {
        self.base_addr
    }

    fn get_size(&self) -> u32 {
        self.data.len() as u32
    }
}

// struct IoRegion {

// }

// impl MemoryRegion for IoRegion {

// }

struct ItcmRegion {
    base_addr: u32,
    region_size: u32,
    repeat_size: u32,
    data: Box<[u8]>,
}

impl ItcmRegion {
    fn new() -> ItcmRegion {
        let base_addr:   u32 = 0x00000000;
        let region_size: u32 = 0x08000000;
        let repeat_size: u32 = 0x8000;

        ItcmRegion {
            base_addr: base_addr,
            region_size: region_size,
            repeat_size: repeat_size,
            data: fast_zeroed_mem(repeat_size as usize),
        }
    }

    fn to_index(&self, addr: u32) -> usize {
        ((addr - self.base_addr) % self.repeat_size) as usize
    }
}

impl MemoryRegion for ItcmRegion {
    fn read(&self, addr: u32) -> u8 {
        assert!(self.check_addr(addr));
        self.data[self.to_index(addr)]
    }

    fn write(&mut self, addr: u32, data: u8) {
        assert!(self.check_addr(addr));
        self.data[self.to_index(addr)] = data;
    }

    fn borrow(&self, addr: u32, size: u32) -> &[u8] {
        assert!(self.check_bounds(addr, size));
        let index = self.to_index(addr);
        &self.data[index .. index + size as usize]
    }

    fn borrow_mut(&mut self, addr: u32, size: u32) -> &mut [u8] {
        assert!(self.check_bounds(addr, size));
        let index = self.to_index(addr);
        &mut self.data[index .. index + size as usize]
    }

    fn get_base_addr(&self) -> u32 {
        self.base_addr
    }

    fn get_size(&self) -> u32 {
        self.region_size
    }

    fn check_bounds(&self, addr: u32, size: u32) -> bool {
        (addr >= self.get_base_addr())
        && (addr < self.get_base_addr() + self.get_size())
        && (self.to_index(addr) + (size as usize) < self.data.len())
    }
}

pub struct Ram {
    regions: Vec<Box<MemoryRegion>>,
}

impl Ram {
    pub fn new() -> Ram {
        let mut regions: Vec<Box<MemoryRegion>> = Vec::new();
        regions.push(Box::new(ItcmRegion::new())); // ITCM
        regions.push(Box::new(RamRegion::new(0x08000000, 0x00100000))); // A9 Internal
        regions.push(Box::new(RamRegion::new(0x10000000, 0x08000000))); // IO
        regions.push(Box::new(RamRegion::new(0x18000000, 0x00600000))); // VRAM
        regions.push(Box::new(RamRegion::new(0x1FF00000, 0x00080000))); // DSP
        regions.push(Box::new(RamRegion::new(0x1FF80000, 0x00080000))); // AXI WRAM
        regions.push(Box::new(RamRegion::new(0x20000000, 0x08000000))); // FCRAM
        regions.push(Box::new(RamRegion::new(0xFFF00000, 0x00004000))); // DTCM
        regions.push(Box::new(RamRegion::new(0xFFFF0000, 0x00010000))); // Bootrom

        Ram {
            regions: regions,
        }
    }

    fn get_region_index(&self, addr: u32, size: u32) -> usize {
        for (index, region) in self.regions.iter().enumerate() {
            if region.check_bounds(addr, size) {
                return index;
            }
        }

        panic!("Invalid memory read! addr: {}, size: {}", addr, size);
    }

    pub fn read<T: Copy>(&self, addr: u32) -> T {
        let size = std::mem::size_of::<T>() as u32;
        let slice = self.regions[self.get_region_index(addr, size)].borrow(addr, size);
        unsafe {
            *(slice.as_ptr() as *const T)
        }
    }

    pub fn write<T: Copy>(&mut self, addr: u32, data: T) {
        let size = std::mem::size_of::<T>() as u32;
        let index = self.get_region_index(addr, size);
        let slice = self.regions[index].borrow_mut(addr, size);
        unsafe {
            *(slice.as_mut_ptr() as *mut T) = data;
        };
    }

    pub fn borrow<T: Copy>(&self, addr: u32, qty: usize) -> &[T] {
        let size = (std::mem::size_of::<T>() * qty) as u32;
        let index = self.get_region_index(addr, size);
        let slice = self.regions[index].borrow(addr, size);
        unsafe {
            slice::from_raw_parts(slice.as_ptr() as *const T, qty)
        }
    }

    pub fn borrow_mut<T: Copy>(&mut self, addr: u32, qty: usize) -> &mut [T] {
        let size = (std::mem::size_of::<T>() * qty) as u32;
        let index = self.get_region_index(addr, size);
        let slice = self.regions[index].borrow_mut(addr, size);
        unsafe {
            slice::from_raw_parts_mut(slice.as_mut_ptr() as *mut T, qty)
        }
    }
}
