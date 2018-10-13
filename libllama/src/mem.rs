use std;
// use std::cell;
use std::cmp;
use std::ptr;
use std::sync::Arc;

use parking_lot::RwLock;

use io;

const KB_SIZE: usize = 1024;
pub type SharedMemoryNode = RwLock<[u8; KB_SIZE]>;

pub(crate) trait MemoryBlock {
    fn get_bytes(&self) -> u32;
    unsafe fn read_to_ptr(&self, offset: usize, buf: *mut u8, buf_size: usize);
    unsafe fn write_from_ptr(&mut self, offset: usize, buf: *const u8, buf_size: usize);
}

pub struct UniqueMemoryBlock(Vec<u8>);
impl UniqueMemoryBlock {
    pub fn new(kbs: usize) -> UniqueMemoryBlock {
        UniqueMemoryBlock(vec![0u8; kbs*KB_SIZE])
    }
}
impl MemoryBlock for UniqueMemoryBlock {
    fn get_bytes(&self) -> u32 {
        self.0.len() as u32
    }

    unsafe fn read_to_ptr(&self, offset: usize, buf: *mut u8, buf_size: usize) {
        let vec = &self.0;
        assert!(offset + buf_size <= vec.len());
        ptr::copy_nonoverlapping(vec.as_ptr().offset(offset as isize), buf, buf_size);
    }

    unsafe fn write_from_ptr(&mut self, offset: usize, buf: *const u8, buf_size: usize) {
        let vec = &mut self.0;
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

    unsafe fn write_from_ptr(&mut self, offset: usize, buf: *const u8, buf_size: usize) {
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

pub enum AddressBlock {
    UniqueRam(UniqueMemoryBlock),
    SharedRam(SharedMemoryBlock),
    Io9(io::IoRegsArm9),
    IoShared(io::IoRegsShared),
}
impl MemoryBlock for AddressBlock {
    fn get_bytes(&self) -> u32 {
        match *self {
            AddressBlock::UniqueRam(ref inner) => inner.get_bytes(),
            AddressBlock::SharedRam(ref inner) => inner.get_bytes(),
            AddressBlock::Io9(ref inner) => inner.get_bytes(),
            AddressBlock::IoShared(ref inner) => inner.get_bytes(),
        }
    }

    unsafe fn read_to_ptr(&self, offset: usize, buf: *mut u8, buf_size: usize) {
        match *self {
            AddressBlock::UniqueRam(ref inner) => inner.read_to_ptr(offset, buf, buf_size),
            AddressBlock::SharedRam(ref inner) => inner.read_to_ptr(offset, buf, buf_size),
            AddressBlock::Io9(ref inner) => inner.read_to_ptr(offset, buf, buf_size),
            AddressBlock::IoShared(ref inner) => inner.read_to_ptr(offset, buf, buf_size)
        }
    }

    unsafe fn write_from_ptr(&mut self, offset: usize, buf: *const u8, buf_size: usize) {
        match *self {
            AddressBlock::UniqueRam(ref mut inner) => inner.write_from_ptr(offset, buf, buf_size),
            AddressBlock::SharedRam(ref mut inner) => inner.write_from_ptr(offset, buf, buf_size),
            AddressBlock::Io9(ref mut inner) => inner.write_from_ptr(offset, buf, buf_size),
            AddressBlock::IoShared(ref mut inner) => inner.write_from_ptr(offset, buf, buf_size)
        }
    }
}

pub struct AddressBlockHandle(u32);

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

    fn match_address<'a>(&'a self, address: u32) -> Option<usize> {
        let index = match self.search_region(address) {
            Ok(a) => a,
            Err(a) => a - 1,
        };
        if index > self.regions.len() {
            return None;
        }

        let (block_addr, ref block) = self.regions[index];
        if address - block_addr < block.get_bytes() {
            return Some(index);
        }
        None
    }

    pub fn map_region(&mut self, address: u32, region: AddressBlock) -> AddressBlockHandle {
        let insert_index = self.search_region(address).unwrap_err();
        self.regions.insert(insert_index, (address, region));
        AddressBlockHandle(address)
    }

    pub(crate) fn region(&self, handle: &AddressBlockHandle) -> &AddressBlock {
        let index = self.search_region(handle.0)
            .expect("Attempted to find region from non-existant handle!");
        &self.regions[index].1
    }

    pub(crate) fn _region_mut(&mut self, handle: &AddressBlockHandle) -> &mut AddressBlock {
        let index = self.search_region(handle.0)
            .expect("Attempted to find region from non-existant handle!");
        &mut self.regions[index].1
    }

    pub fn read<T: Copy>(&self, addr: u32) -> T {
        let block_index = self.match_address(addr)
            .unwrap_or_else(|| panic!("Could not match address 0x{:X}", addr));
        let (block_addr, ref block) = self.regions[block_index];

        unsafe {
            let mut t: T = std::mem::uninitialized();
            block.read_to_ptr((addr - block_addr) as usize, std::mem::transmute(&mut t), std::mem::size_of::<T>());
            t
        }
    }

    #[inline]
    fn try_read_buf(&self, addr: u32, buf: &mut [u8], debug: bool) -> Result<(), String> {
        let block_index = self.match_address(addr)
            .ok_or(format!("Could not match address 0x{:X}", addr))?;
        let (block_addr, ref block) = self.regions[block_index];
    
        match (debug, block) {
            | (true, AddressBlock::Io9(_))
            | (true, AddressBlock::IoShared(_)) => return Err(format!("Cannot issue debug read for IO address 0x{:X}", addr)),
            (_, block) => unsafe {
                block.read_to_ptr((addr - block_addr) as usize, buf.as_mut_ptr(), buf.len());
            }
        }
        Ok(())
    }

    pub fn read_buf(&self, addr: u32, buf: &mut [u8]) {
        self.try_read_buf(addr, buf, false).unwrap();
    }

    pub fn debug_read_buf(&self, addr: u32, buf: &mut [u8]) -> Result<(), String> {
        self.try_read_buf(addr, buf, true)
    }

    pub fn write<T: Copy>(&mut self, addr: u32, data: T) {
        let block_index = self.match_address(addr)
            .unwrap_or_else(|| panic!("Could not match address 0x{:X}", addr));
        let (block_addr, ref mut block) = self.regions[block_index];

        unsafe {
            block.write_from_ptr((addr - block_addr) as usize, std::mem::transmute(&data), std::mem::size_of::<T>());
        }
    }

    pub fn write_buf(&mut self, addr: u32, buf: &[u8]) {
        let block_index = self.match_address(addr)
            .unwrap_or_else(|| panic!("Could not match address 0x{:X}", addr));
        let (block_addr, ref mut block) = self.regions[block_index];

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
