use std;
use std::sync;

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
    fn take(&self, addr: u32, size: u32) -> &[u8];
    fn take_mut(&mut self, addr: u32, size: u32) -> &mut [u8];

    fn get_base_addr(&self) -> u32;
    fn get_size(&self) -> u32;

    fn check_addr(&self, addr: u32) -> bool {
        self.check_bounds(addr, 0)
    }
    fn check_bounds(&self, addr: u32, size: u32) -> bool {
        (addr >= self.get_base_addr()) && (addr + size < self.get_base_addr() + self.get_size())
    }
}

pub struct RamRegion {
    base_addr: u32,
    data: Box<[u8]>,
}

impl RamRegion {
    pub fn new(base_addr: u32, size: u32) -> RamRegion {
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

    fn take(&self, addr: u32, size: u32) -> &[u8] {
        assert!(self.check_bounds(addr, size));
        let index = (addr - self.get_base_addr()) as usize;
        &self.data[index .. index + size as usize]
    }

    fn take_mut(&mut self, addr: u32, size: u32) -> &mut [u8] {
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

pub struct ItcmRegion {
    base_addr: u32,
    region_size: u32,
    repeat_size: u32,
    data: Box<[u8]>,
}

impl ItcmRegion {
    pub fn new() -> ItcmRegion {
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

    fn take(&self, addr: u32, size: u32) -> &[u8] {
        assert!(self.check_bounds(addr, size));
        let index = self.to_index(addr);
        &self.data[index .. index + size as usize]
    }

    fn take_mut(&mut self, addr: u32, size: u32) -> &mut [u8] {
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

pub struct MemController {
    regions: Vec<sync::Arc<sync::RwLock<MemoryRegion>>>,
}

impl MemController {
    pub fn new() -> MemController {
        MemController {
            regions: Vec::new(),
        }
    }

    pub fn add_region(&mut self, region: sync::Arc<sync::RwLock<MemoryRegion>>) {
        self.regions.push(region);
    }

    fn get_region_index(&self, addr: u32, size: u32) -> usize {
        for (index, region) in self.regions.iter().enumerate() {
            if region.read().unwrap().check_bounds(addr, size) {
                return index;
            }
        }
        panic!();
    }

    pub fn read<T: Copy>(&self, addr: u32) -> T {
        let size = std::mem::size_of::<T>() as u32;
        let region = self.regions[self.get_region_index(addr, size)].read().unwrap();
        unsafe {
            *(region.take(addr, size).as_ptr() as *const T)
        }
    }

    pub fn write<T: Copy>(&mut self, addr: u32, data: T) {
        let size = std::mem::size_of::<T>() as u32;
        let mut region = self.regions[self.get_region_index(addr, size)].write().unwrap();
        unsafe {
            *(region.take_mut(addr, size).as_mut_ptr() as *mut T) = data;
        };
    }

    // TODO: Replace with Writer/Reader

    // pub fn take<T: Copy>(&self, addr: u32, qty: usize) -> &[T] {
    //     let size = (std::mem::size_of::<T>() * qty) as u32;
    //     let index = self.get_region_index(addr, size);
    //     let slice = self.regions[index].take(addr, size);
    //     unsafe {
    //         slice::from_raw_parts(slice.as_ptr() as *const T, qty)
    //     }
    // }

    // pub fn take_mut<T: Copy>(&mut self, addr: u32, qty: usize) -> &mut [T] {
    //     let size = (std::mem::size_of::<T>() * qty) as u32;
    //     let index = self.get_region_index(addr, size);
    //     let slice = self.regions[index].take_mut(addr, size);
    //     unsafe {
    //         slice::from_raw_parts_mut(slice.as_mut_ptr() as *mut T, qty)
    //     }
    // }
}
