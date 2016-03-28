use std;
use std::cmp;
use std::ptr;
use std::sync;

const KB_SIZE: usize = 1024;
type MemoryNode = sync::RwLock<[u8; KB_SIZE]>;

#[derive(Clone)]
pub struct MemoryBlock {
    nodes: sync::Arc<Vec<MemoryNode>>
}

impl MemoryBlock {
    pub fn new(kbs: usize) -> MemoryBlock {
        let mut inner: Vec<MemoryNode> = Vec::new();
        for i in 0..kbs {
            inner.push(sync::RwLock::new([0; KB_SIZE]))
        }

        MemoryBlock {
            nodes: sync::Arc::new(inner)
        }
    }

    pub fn get_kbs(&self) -> usize {
        self.nodes.len()
    }

    pub fn get_bytes(&self) -> u32 {
        (self.get_kbs() * KB_SIZE) as u32
    }

    pub unsafe fn read_to_ptr(&self, offset: usize, buf: *mut u8, buf_size: usize) {
        let mut buf_remaining = buf_size;
        let mut node_index = offset / KB_SIZE;
        let mut node_pos = offset % KB_SIZE;

        while buf_remaining > 0 {
            assert!(node_index < self.nodes.len());
            let copy_amount = cmp::min(KB_SIZE - node_pos, buf_remaining);
            {
                let node_ptr = self.nodes[node_index].read().unwrap().as_ptr();
                let buf_pos = buf_size - buf_remaining;
                ptr::copy_nonoverlapping(node_ptr.offset(node_pos as isize), buf.offset(buf_pos as isize), copy_amount);
            }
            buf_remaining -= copy_amount;
            node_index += 1;
            node_pos = 0;
        }
    }

    pub unsafe fn write_from_ptr(&self, offset: usize, buf: *const u8, buf_size: usize) {
        let mut buf_remaining = buf_size;
        let mut node_index = offset / KB_SIZE;
        let mut node_pos = offset % KB_SIZE;

        while buf_remaining > 0 {
            assert!(node_index < self.nodes.len());
            let copy_amount = cmp::min(KB_SIZE - node_pos, buf_remaining);
            {
                let node_ptr = self.nodes[node_index].write().unwrap().as_mut_ptr();
                let buf_pos = buf_size - buf_remaining;
                ptr::copy_nonoverlapping(buf.offset(buf_pos as isize), node_ptr.offset(node_pos as isize), copy_amount);
            }
            buf_remaining -= copy_amount;
            node_index += 1;
            node_pos = 0;
        }
    }
}

pub struct MemController {
    regions: Vec<(u32, MemoryBlock)>,
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

    fn match_address(&self, address: u32) -> Option<(u32, MemoryBlock)> {
        let index = match self.search_region(address) {
            Ok(a) => a,
            Err(a) => a - 1,
        };
        if index > self.regions.len() {
            return None;
        }

        let (block_addr, ref block) = self.regions[index];
        if (address >= block_addr) && (address < block_addr + block.get_bytes()) {
            return Some((block_addr, block.clone()));
        }
        None
    }

    pub fn map_region(&mut self, address: u32, region: MemoryBlock) {
        let insert_index = self.search_region(address).unwrap_err();
        self.regions.insert(insert_index, (address, region));
    }

    pub fn read<T: Copy>(&self, addr: u32) -> T {
        let (block_addr, block) = self.match_address(addr)
            .unwrap_or_else(|| panic!("Could not match address 0x{:X}", addr));
        unsafe {
            let t: T = std::mem::uninitialized();
            block.read_to_ptr((addr - block_addr) as usize, std::mem::transmute(&t), std::mem::size_of::<T>());
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

    pub fn write_buf(&self, addr: u32, buf: &[u8]) {
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
        let block = MemoryBlock::new(1);
        assert_eq!(block.get_bytes(), 0x400);

        // Write data
        let bytes = [0xFFu8, 0x53u8, 0x28u8, 0xC6u8];
        unsafe {
            block.write_from_ptr(0x2C8, bytes.as_ptr(), bytes.len());
        }

        // Compare memory with data
        let block_mem = block.nodes[0].read().unwrap();
        assert_eq!(block_mem[0x2C8..0x2CC], bytes[..]);
    }

    #[test]
    fn read_intra_block() {
        let block = MemoryBlock::new(1);
        assert_eq!(block.get_bytes(), 0x400);

        // Write data directly to memory
        {
            let mut block_mem = block.nodes[0].write().unwrap();
            (&mut block_mem[0x2C8..0x2CC]).write_all(&[0xFFu8, 0x53u8, 0x28u8, 0xC6u8]);
        }

        // Read memory to buffer
        let mut buf = [0u8; 4];
        unsafe {
            block.read_to_ptr(0x2C8, buf.as_mut_ptr(), buf.len());
        }

        // Compare memory and buffer
        let block_mem = block.nodes[0].read().unwrap();
        assert_eq!(block_mem[0x2C8..0x2CC], buf[..]);
    }

    #[test]
    fn write_inter_block() {
        let block = MemoryBlock::new(2);
        assert_eq!(block.get_bytes(), 0x800);

        // Write data
        let bytes = [0xFFu8, 0x53u8, 0x28u8, 0xC6u8];
        unsafe {
            block.write_from_ptr(0x3FE, bytes.as_ptr(), bytes.len());
        }

        // Compare memory with data
        let block_mem0 = block.nodes[0].read().unwrap();
        let block_mem1 = block.nodes[1].read().unwrap();
        assert_eq!(block_mem0[0x3FE..0x400], bytes[0..2]);
        assert_eq!(block_mem1[0x0..0x2], bytes[2..4]);
    }

    #[test]
    fn read_inter_block() {
        let block = MemoryBlock::new(2);
        assert_eq!(block.get_bytes(), 0x800);

        // Write data directly to memory
        {
            let mut block_mem0 = block.nodes[0].write().unwrap();
            let mut block_mem1 = block.nodes[1].write().unwrap();
            (&mut block_mem0[0x3FE..0x400]).write_all(&[0xFFu8, 0x53u8]);
            (&mut block_mem1[0x0..0x2]).write_all(&[0x28u8, 0xC6u8]);
        }

        // Read memory to buffer
        let mut buf = [0u8; 4];
        unsafe {
            block.read_to_ptr(0x3FE, buf.as_mut_ptr(), buf.len());
        }

        // Compare memory and buffer
        let block_mem0 = block.nodes[0].read().unwrap();
        let block_mem1 = block.nodes[1].read().unwrap();
        assert_eq!(block_mem0[0x3FE..0x400], buf[0..2]);
        assert_eq!(block_mem1[0x0..0x2], buf[2..4]);
    }
}