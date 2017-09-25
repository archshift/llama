use std;
use std::cell;
use std::cmp;
use std::ptr;
use std::sync::Arc;

use parking_lot::{Mutex, RwLock};

use io;

const KB_SIZE: usize = 1024;
pub type SharedMemoryNode = RwLock<[u8; KB_SIZE]>;

trait MemoryBlock {
    fn get_bytes(&self) -> u32;
    unsafe fn read_to_ptr(&self, offset: usize, buf: *mut u8, buf_size: usize);
    unsafe fn write_from_ptr(&self, offset: usize, buf: *const u8, buf_size: usize);
}

pub struct UniqueMemoryBlock(cell::UnsafeCell<Vec<u8>>);
impl UniqueMemoryBlock {
    pub fn new(kbs: usize) -> UniqueMemoryBlock {
        UniqueMemoryBlock(cell::UnsafeCell::new(vec![0u8; kbs*KB_SIZE]))
    }
}
impl MemoryBlock for UniqueMemoryBlock {
    fn get_bytes(&self) -> u32 {
        unsafe { (*self.0.get()).len() as u32 }
    }

    unsafe fn read_to_ptr(&self, offset: usize, buf: *mut u8, buf_size: usize) {
        let vec = &*self.0.get();
        assert!(offset + buf_size <= vec.len());
        ptr::copy_nonoverlapping(vec.as_ptr().offset(offset as isize), buf, buf_size);
    }

    unsafe fn write_from_ptr(&self, offset: usize, buf: *const u8, buf_size: usize) {
        let vec = &mut *self.0.get();
        assert!(offset + buf_size <= vec.len());
        ptr::copy_nonoverlapping(buf, vec.as_mut_ptr().offset(offset as isize), buf_size);
    }
}

#[derive(Clone)]
pub struct SharedMemoryBlock(Arc<Vec<SharedMemoryNode>>);
impl SharedMemoryBlock {
    pub fn new(kbs: usize) -> SharedMemoryBlock {
        let mut inner: Vec<SharedMemoryNode> = Vec::new();
        for _ in 0..kbs {
            inner.push(RwLock::new([0; KB_SIZE]))
        }

        SharedMemoryBlock(Arc::new(inner))
    }
}
impl MemoryBlock for SharedMemoryBlock {
    fn get_bytes(&self) -> u32 {
        let nodes = &self.0;
        (nodes.len() * KB_SIZE) as u32
    }

    unsafe fn read_to_ptr(&self, offset: usize, buf: *mut u8, buf_size: usize) {
        let nodes = &self.0;
        let mut buf_remaining = buf_size;
        let mut node_index = offset / KB_SIZE;
        let mut node_pos = offset % KB_SIZE;

        while buf_remaining > 0 {
            assert!(node_index < nodes.len());
            let copy_amount = cmp::min(KB_SIZE - node_pos, buf_remaining);
            {
                let node_ptr = nodes[node_index].read().as_ptr();
                let buf_pos = buf_size - buf_remaining;
                ptr::copy_nonoverlapping(node_ptr.offset(node_pos as isize), buf.offset(buf_pos as isize), copy_amount);
            }
            buf_remaining -= copy_amount;
            node_index += 1;
            node_pos = 0;
        }
    }

    unsafe fn write_from_ptr(&self, offset: usize, buf: *const u8, buf_size: usize) {
        let nodes = &self.0;
        let mut buf_remaining = buf_size;
        let mut node_index = offset / KB_SIZE;
        let mut node_pos = offset % KB_SIZE;

        while buf_remaining > 0 {
            assert!(node_index < nodes.len());
            let copy_amount = cmp::min(KB_SIZE - node_pos, buf_remaining);
            {
                let node_ptr = nodes[node_index].write().as_mut_ptr();
                let buf_pos = buf_size - buf_remaining;
                ptr::copy_nonoverlapping(buf.offset(buf_pos as isize), node_ptr.offset(node_pos as isize), copy_amount);
            }
            buf_remaining -= copy_amount;
            node_index += 1;
            node_pos = 0;
        }
    }
}

#[derive(Clone)]
pub struct IoMemoryBlock(Arc<(usize, Mutex<io::IoRegion>)>);
impl IoMemoryBlock {
    pub fn new(variant: io::IoRegion, kbs: usize) -> IoMemoryBlock {
        IoMemoryBlock(Arc::new((kbs, Mutex::new(variant))))
    }
}
impl MemoryBlock for IoMemoryBlock {
    fn get_bytes(&self) -> u32 {
        ((self.0).0 * KB_SIZE) as u32
    }

    unsafe fn read_to_ptr(&self, offset: usize, buf: *mut u8, buf_size: usize) {
        let (_, ref region) = *self.0;
        match *region.lock() {
            io::IoRegion::Arm9(ref mut x) => x.read_reg(offset, buf, buf_size),
            io::IoRegion::Shared(ref mut x) => x.read_reg(offset, buf, buf_size),
            _ => unimplemented!(),
        }
    }

    unsafe fn write_from_ptr(&self, offset: usize, buf: *const u8, buf_size: usize) {
        let (_, ref region) = *self.0;
        match *region.lock() {
            io::IoRegion::Arm9(ref mut x) => x.write_reg(offset, buf, buf_size),
            io::IoRegion::Shared(ref mut x) => x.write_reg(offset, buf, buf_size),
            _ => unimplemented!(),
        }
    }
}

pub enum AddressBlock {
    UniqueRam(UniqueMemoryBlock),
    SharedRam(SharedMemoryBlock),
    Io(IoMemoryBlock),
}
impl MemoryBlock for AddressBlock {
    fn get_bytes(&self) -> u32 {
        match *self {
            AddressBlock::UniqueRam(ref inner) => inner.get_bytes(),
            AddressBlock::SharedRam(ref inner) => inner.get_bytes(),
            AddressBlock::Io(ref inner) => inner.get_bytes(),
        }
    }

    unsafe fn read_to_ptr(&self, offset: usize, buf: *mut u8, buf_size: usize) {
        match *self {
            AddressBlock::UniqueRam(ref inner) => inner.read_to_ptr(offset, buf, buf_size),
            AddressBlock::SharedRam(ref inner) => inner.read_to_ptr(offset, buf, buf_size),
            AddressBlock::Io(ref inner) => inner.read_to_ptr(offset, buf, buf_size)
        }
    }

    unsafe fn write_from_ptr(&self, offset: usize, buf: *const u8, buf_size: usize) {
        match *self {
            AddressBlock::UniqueRam(ref inner) => inner.write_from_ptr(offset, buf, buf_size),
            AddressBlock::SharedRam(ref inner) => inner.write_from_ptr(offset, buf, buf_size),
            AddressBlock::Io(ref inner) => inner.write_from_ptr(offset, buf, buf_size)
        }
    }
}

pub struct MemController {
    regions: Vec<(u32, AddressBlock)>,
}

impl MemController {
    pub fn new() -> MemController {
        MemController {
            regions: Vec::new(),
        }
    }

    fn search_region(&self, address: u32) -> Result<usize, usize> {
        self.regions.binary_search_by(|&(addr, _)| addr.cmp(&address))
    }

    fn match_address<'a>(&'a self, address: u32) -> Option<(u32, &'a AddressBlock)> {
        let index = match self.search_region(address) {
            Ok(a) => a,
            Err(a) => a - 1,
        };
        if index > self.regions.len() {
            return None;
        }

        let (block_addr, ref block) = self.regions[index];
        if (address >= block_addr) && (address <= block_addr + block.get_bytes() - 1) {
            return Some((block_addr, block));
        }
        None
    }

    pub fn map_region(&mut self, address: u32, region: AddressBlock) {
        let insert_index = self.search_region(address).unwrap_err();
        self.regions.insert(insert_index, (address, region));
    }

    pub fn read<T: Copy>(&self, addr: u32) -> T {
        let (block_addr, block) = self.match_address(addr)
            .unwrap_or_else(|| panic!("Could not match address 0x{:X}", addr));
        unsafe {
            let mut t: T = std::mem::uninitialized();
            block.read_to_ptr((addr - block_addr) as usize, std::mem::transmute(&mut t), std::mem::size_of::<T>());
            t
        }
    }

    pub fn read_buf(&self, addr: u32, buf: &mut [u8]) {
        let (block_addr, block) = self.match_address(addr)
            .unwrap_or_else(|| panic!("Could not match address 0x{:X}", addr));
        unsafe {
            block.read_to_ptr((addr - block_addr) as usize, buf.as_mut_ptr(), buf.len());
        }
    }

    pub fn write<T: Copy>(&mut self, addr: u32, data: T) {
        let (block_addr, block) = self.match_address(addr)
            .unwrap_or_else(|| panic!("Could not match address 0x{:X}", addr));
        unsafe {
            block.write_from_ptr((addr - block_addr) as usize, std::mem::transmute(&data), std::mem::size_of::<T>());
        }
    }

    pub fn write_buf(&mut self, addr: u32, buf: &[u8]) {
        let (block_addr, block) = self.match_address(addr)
            .unwrap_or_else(|| panic!("Could not match address 0x{:X}", addr));
        unsafe {
            block.write_from_ptr((addr - block_addr) as usize, buf.as_ptr(), buf.len());
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Write;

    #[test]
    fn write_intra_block() {
        let block = SharedMemoryBlock::new(1);
        assert_eq!(block.get_bytes(), 0x400);
        let nodes = &block.0;

        // Write data
        let bytes = [0xFFu8, 0x53u8, 0x28u8, 0xC6u8];
        unsafe {
            block.write_from_ptr(0x2C8, bytes.as_ptr(), bytes.len());
        }

        // Compare memory with data
        let block_mem = nodes[0].read();
        assert_eq!(block_mem[0x2C8..0x2CC], bytes[..]);
    }

    #[test]
    fn read_intra_block() {
        let block = SharedMemoryBlock::new(1);
        assert_eq!(block.get_bytes(), 0x400);
        let nodes = &block.0;

        // Write data directly to memory
        {
            let mut block_mem = nodes[0].write();
            (&mut block_mem[0x2C8..0x2CC]).write_all(&[0xFFu8, 0x53u8, 0x28u8, 0xC6u8]);
        }

        // Read memory to buffer
        let mut buf = [0u8; 4];
        unsafe {
            block.read_to_ptr(0x2C8, buf.as_mut_ptr(), buf.len());
        }

        // Compare memory and buffer
        let block_mem = nodes[0].read();
        assert_eq!(block_mem[0x2C8..0x2CC], buf[..]);
    }

    #[test]
    fn write_inter_block() {
        let block = SharedMemoryBlock::new(2);
        assert_eq!(block.get_bytes(), 0x800);
        let nodes = &block.0;

        // Write data
        let bytes = [0xFFu8, 0x53u8, 0x28u8, 0xC6u8];
        unsafe {
            block.write_from_ptr(0x3FE, bytes.as_ptr(), bytes.len());
        }

        // Compare memory with data
        let block_mem0 = nodes[0].read();
        let block_mem1 = nodes[1].read();
        assert_eq!(block_mem0[0x3FE..0x400], bytes[0..2]);
        assert_eq!(block_mem1[0x0..0x2], bytes[2..4]);
    }

    #[test]
    fn read_inter_block() {
        let block = SharedMemoryBlock::new(2);
        assert_eq!(block.get_bytes(), 0x800);
        let nodes = &block.0;

        // Write data directly to memory
        {
            let mut block_mem0 = nodes[0].write();
            let mut block_mem1 = nodes[1].write();
            (&mut block_mem0[0x3FE..0x400]).write_all(&[0xFFu8, 0x53u8]);
            (&mut block_mem1[0x0..0x2]).write_all(&[0x28u8, 0xC6u8]);
        }

        // Read memory to buffer
        let mut buf = [0u8; 4];
        unsafe {
            block.read_to_ptr(0x3FE, buf.as_mut_ptr(), buf.len());
        }

        // Compare memory and buffer
        let block_mem0 = nodes[0].read();
        let block_mem1 = nodes[1].read();
        assert_eq!(block_mem0[0x3FE..0x400], buf[0..2]);
        assert_eq!(block_mem1[0x0..0x2], buf[2..4]);
    }
}