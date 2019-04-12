use std;
use std::cmp;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::rc::Rc;
use std::cell::RefCell;

use parking_lot::RwLock;

use io;
use utils::bytes;

const KB_SIZE: usize = 1024;
pub type SharedMemoryNode = RwLock<[u8; KB_SIZE]>;

pub(crate) trait MemoryBlock {
    fn get_bytes(&self) -> u32;
    fn read_buf(&self, offset: usize, buf: &mut [u8]);
    fn write_buf(&mut self, offset: usize, buf: &[u8]);
}

#[derive(Clone)]
pub struct UniqueMemoryBlock(Rc<RefCell<Vec<u8>>>);
impl UniqueMemoryBlock {
    pub fn new(kbs: usize) -> UniqueMemoryBlock {
        UniqueMemoryBlock(
            Rc::new(RefCell::new(vec![0u8; kbs*KB_SIZE]))
        )
    }
}
impl MemoryBlock for UniqueMemoryBlock {
    fn get_bytes(&self) -> u32 {
        self.0.borrow().len() as u32
    }

    fn read_buf(&self, offset: usize, buf: &mut [u8]) {
        let vec = self.0.borrow();
        assert!(offset + buf.len() <= vec.len());
        let src = &vec[offset..offset + buf.len()];
        buf.copy_from_slice(src);
    }

    fn write_buf(&mut self, offset: usize, buf: &[u8]) {
        let mut vec = self.0.borrow_mut();
        assert!(offset + buf.len() <= vec.len());
        let dst = &mut vec[offset..offset + buf.len()];
        dst.copy_from_slice(buf);
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

    fn read_buf(&self, offset: usize, mut buf: &mut [u8]) {
        let nodes = &self.0;

        let mut node_index = offset / KB_SIZE;
        let mut node_pos = offset % KB_SIZE;

        while buf.len() > 0 {
            assert!(node_index < nodes.len());
            let copy_amount = cmp::min(KB_SIZE - node_pos, buf.len());
        
            buf = {
                let (buf, buf_rest) = {buf}.split_at_mut(copy_amount);
                let node = nodes[node_index].read();
                let src = &node[node_pos .. node_pos + copy_amount];
                buf.copy_from_slice(src);
                buf_rest
            };

            node_index += 1;
            node_pos = 0;
        }
    }

    fn write_buf(&mut self, offset: usize, mut buf: &[u8]) {
        let nodes = &self.0;

        let mut node_index = offset / KB_SIZE;
        let mut node_pos = offset % KB_SIZE;

        while buf.len() > 0 {
            assert!(node_index < nodes.len());
            let copy_amount = cmp::min(KB_SIZE - node_pos, buf.len());
            
            buf = {
                let (buf, buf_rest) = buf.split_at(copy_amount);
                let mut node = nodes[node_index].write();
                let dst = &mut node[node_pos .. node_pos + copy_amount];
                dst.copy_from_slice(buf);
                buf_rest
            };

            node_index += 1;
            node_pos = 0;
        }
    }
}

pub enum AddressBlock {
    UniqueRam(UniqueMemoryBlock),
    SharedRam(SharedMemoryBlock),
    Io9(io::IoRegsArm9),
    Io11(io::IoRegsArm11),
    IoPriv11(io::IoRegsArm11Priv),
    IoShared(io::IoRegsShared),
}
impl MemoryBlock for AddressBlock {
    fn get_bytes(&self) -> u32 {
        match *self {
            AddressBlock::UniqueRam(ref inner) => inner.get_bytes(),
            AddressBlock::SharedRam(ref inner) => inner.get_bytes(),
            AddressBlock::Io9(ref inner) => inner.get_bytes(),
            AddressBlock::Io11(ref inner) => inner.get_bytes(),
            AddressBlock::IoPriv11(ref inner) => inner.get_bytes(),
            AddressBlock::IoShared(ref inner) => inner.get_bytes(),
        }
    }

    fn read_buf(&self, offset: usize, buf: &mut [u8]) {
        match *self {
            AddressBlock::UniqueRam(ref inner) => inner.read_buf(offset, buf),
            AddressBlock::SharedRam(ref inner) => inner.read_buf(offset, buf),
            AddressBlock::Io9(ref inner) => inner.read_buf(offset, buf),
            AddressBlock::Io11(ref inner) => inner.read_buf(offset, buf),
            AddressBlock::IoPriv11(ref inner) => inner.read_buf(offset, buf),
            AddressBlock::IoShared(ref inner) => inner.read_buf(offset, buf)
        }
    }

    fn write_buf(&mut self, offset: usize, buf: &[u8]) {
        match *self {
            AddressBlock::UniqueRam(ref mut inner) => inner.write_buf(offset, buf),
            AddressBlock::SharedRam(ref mut inner) => inner.write_buf(offset, buf),
            AddressBlock::Io9(ref mut inner) => inner.write_buf(offset, buf),
            AddressBlock::Io11(ref mut inner) => inner.write_buf(offset, buf),
            AddressBlock::IoPriv11(ref mut inner) => inner.write_buf(offset, buf),
            AddressBlock::IoShared(ref mut inner) => inner.write_buf(offset, buf)
        }
    }
}

pub struct AddressBlockHandle(u32);

pub struct MemController {
    regions: BTreeMap<u32, AddressBlock>,
}

impl MemController {
    pub fn new() -> MemController {
        MemController {
            regions: BTreeMap::new(),
        }
    }

    fn match_address(&self, address: u32) -> Option<(u32, &AddressBlock)> {
        let matched = self.regions.range(..=address).next_back()?;
        let (block_addr, block) = matched;
        if address - block_addr < block.get_bytes() {
            return Some((*block_addr, block));
        }
        None
    }

    fn match_address_mut(&mut self, address: u32) -> Option<(u32, &mut AddressBlock)> {
        let matched = self.regions.range_mut(..=address).next_back()?;
        let (block_addr, block) = matched;
        if address - block_addr < block.get_bytes() {
            return Some((*block_addr, block));
        }
        None
    }

    pub fn map_region(&mut self, address: u32, region: AddressBlock) -> AddressBlockHandle {
        self.regions.insert(address, region);
        AddressBlockHandle(address)
    }

    pub(crate) fn region(&self, handle: &AddressBlockHandle) -> &AddressBlock {
        self.regions.get(&handle.0)
            .expect("Attempted to find region from non-existant handle!")
    }

    pub(crate) fn _region_mut(&mut self, handle: &AddressBlockHandle) -> &mut AddressBlock {
        self.regions.get_mut(&handle.0)
            .expect("Attempted to find region from non-existant handle!")
    }

    pub fn read<T: Copy>(&self, addr: u32) -> T {
        let (block_addr, ref block) = self.match_address(addr)
            .unwrap_or_else(|| panic!("Could not match address 0x{:X}", addr));

        unsafe {
            let mut t: T = std::mem::zeroed();
            block.read_buf((addr - block_addr) as usize, bytes::from_mut_val(&mut t));
            t
        }
    }

    #[inline]
    fn try_read_buf(&self, addr: u32, buf: &mut [u8], debug: bool) -> Result<(), String> {
        let (block_addr, ref block) = self.match_address(addr)
            .ok_or(format!("Could not match address 0x{:X}", addr))?;
    
        match (debug, block) {
            | (true, AddressBlock::Io9(_))
            | (true, AddressBlock::IoShared(_)) => return Err(format!("Cannot issue debug read for IO address 0x{:X}", addr)),
            (_, block) => block.read_buf((addr - block_addr) as usize, buf),
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
        let (block_addr, ref mut block) = self.match_address_mut(addr)
            .unwrap_or_else(|| panic!("Could not match address 0x{:X}", addr));

        block.write_buf((addr - block_addr) as usize, unsafe { bytes::from_val(&data) });
    }

    pub fn write_buf(&mut self, addr: u32, buf: &[u8]) {
        let (block_addr, ref mut block) = self.match_address_mut(addr)
            .unwrap_or_else(|| panic!("Could not match address 0x{:X}", addr));

        block.write_buf((addr - block_addr) as usize, buf);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn write_intra_block() {
        let mut block = SharedMemoryBlock::new(1);
        assert_eq!(block.get_bytes(), 0x400);

        // Write data
        let bytes = [0xFFu8, 0x53u8, 0x28u8, 0xC6u8];
        block.write_buf(0x2C8, &bytes);

        let nodes = &block.0;
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
            block_mem[0x2C8..0x2CC].copy_from_slice(&[0xFFu8, 0x53u8, 0x28u8, 0xC6u8]);
        }

        // Read memory to buffer
        let mut buf = [0u8; 4];
        block.read_buf(0x2C8, &mut buf);

        // Compare memory and buffer
        let block_mem = nodes[0].read();
        assert_eq!(block_mem[0x2C8..0x2CC], buf[..]);
    }

    #[test]
    fn write_inter_block() {
        let mut block = SharedMemoryBlock::new(2);
        assert_eq!(block.get_bytes(), 0x800);

        // Write data
        let bytes = [0xFFu8, 0x53u8, 0x28u8, 0xC6u8];
        block.write_buf(0x3FE, &bytes);

        let nodes = &block.0;
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
            block_mem0[0x3FE..0x400].copy_from_slice(&[0xFFu8, 0x53u8]);
            block_mem1[0x0..0x2].copy_from_slice(&[0x28u8, 0xC6u8]);
        }

        // Read memory to buffer
        let mut buf = [0u8; 4];
        block.read_buf(0x3FE, &mut buf);

        // Compare memory and buffer
        let block_mem0 = nodes[0].read();
        let block_mem1 = nodes[1].read();
        assert_eq!(block_mem0[0x3FE..0x400], buf[0..2]);
        assert_eq!(block_mem1[0x0..0x2], buf[2..4]);
    }
}
