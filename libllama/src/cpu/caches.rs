pub struct TinyCache<T: Copy> {
    plru_set: u64,
    lru_index: usize,
    mappings: [Option<(u32, T)>; 64]
}

impl<T: Copy> TinyCache<T> {
    fn new() -> Self {
        TinyCache {
            plru_set: 0,
            lru_index: 0,
            mappings: [None; 64]
        }
    }

    fn update_plru64(&mut self, accessed: usize) {
        let mut bitset = self.plru_set;
        bitset |= (1 << accessed) as u64;
        bitset *= (bitset != !0) as u64;
        self.plru_set = bitset;
        self.lru_index = (!bitset).trailing_zeros() as usize;
    }

    fn get_or<F>(&mut self, key: u32, orelse: F) -> &T
        where F: Fn(u32) -> T {

        let map_pos = self.mappings.iter().filter_map(|x|*x)
            .position(|(map_key, _)| map_key == key);

        let pos = if let Some(pos) = map_pos {
            self.update_plru64(pos);
            pos
        } else {
            let instr = orelse(key);
            let lru = self.lru_index;
            self.mappings[lru] = Some((key, instr));
            self.update_plru64(lru);
            lru
        };
        &self.mappings[pos].as_ref().unwrap().1
    }

    fn update_in<F>(&mut self, key: u32, updater: F)
        where F: Fn(u32, &mut T) {
        let map_item = self.mappings.iter().filter_map(|x|*x)
            .position(|(map_key, _)| map_key == key);
        if let Some(pos) = map_item {
            updater(key, &mut self.mappings[pos].as_mut().unwrap().1);
        }
    }

    fn invalidate(&mut self) {
        *self = Self::new();
    }
}


use mem;
use cpu::decoder_arm::ArmInstruction;
use cpu::decoder_thumb::ThumbInstruction;


pub struct MemCache(TinyCache<[u32; 8]>);
impl MemCache {
    fn new() -> Self {
        MemCache(TinyCache::new())
    }

    #[inline]
    fn decompose_addr(addr: u32) -> (u32, u32) {
        (addr & !31, addr & 31)
    }

    #[inline]
    fn read<T: Copy>(&mut self, addr: u32, fallback_mem: &mem::MemController) -> T {
        let (line_base, line_rem) = Self::decompose_addr(addr);
        let fallback_fn = |k| fallback_mem.read::<[u32; 8]>(k);

        let buf = self.0.get_or(line_base, fallback_fn);
        unsafe { *(((buf.as_ptr() as usize) + line_rem as usize) as *const T) }
    }

    #[inline]
    fn write<T: Copy>(&mut self, addr: u32, val: T) {
        let (line_base, line_rem) = Self::decompose_addr(addr);
        self.0.update_in(line_base, |k, line| {
            unsafe { *(((line.as_mut_ptr() as usize) + line_rem as usize) as *mut T) = val; }
        });
    }

    pub fn invalidate(&mut self) {
        self.0.invalidate();
    }
}


#[derive(Default, Copy, Clone)]
pub struct MpuRegion {
    pub base_sigbits: u32,
    pub size_exp: u16,
    pub enabled: bool,
    pub use_icache: bool,
    pub use_dcache: bool,
}

pub struct Mpu {
    pub enabled: bool,
    pub icache_enabled: bool,
    pub dcache_enabled: bool,
    pub regions: [MpuRegion; 8],

    pub memory: mem::MemController,
    pub icache: MemCache,
    pub dcache: MemCache
}

impl Mpu {
    pub fn new(memory: mem::MemController) -> Self {
        Mpu {
            enabled: false,
            icache_enabled: false,
            dcache_enabled: false,
            regions: [Default::default(); 8],
            memory: memory,
            icache: MemCache::new(),
            dcache: MemCache::new(),
        }
    }

    fn addr_region(&self, addr: u32) -> &MpuRegion {
        for region in self.regions.iter().rev() {
            if region.enabled && ((addr >> region.size_exp as u32) == region.base_sigbits) {
                return region
            }
        }
        panic!("Attempted to read memory from {:#08X} in nonexistent MpuRegion!", addr);
    }

    fn icache_enabled(&self) -> bool {
        self.enabled & self.icache_enabled
    }

    fn dcache_enabled(&self) -> bool {
        self.enabled & self.dcache_enabled
    }

    pub fn imem_read<T: Copy>(&mut self, addr: u32) -> T {
        if self.icache_enabled() && self.addr_region(addr).use_icache {
            self.icache.read(addr, &self.memory)
        } else {
            self.memory.read(addr)
        }
    }

    pub fn dmem_read<T: Copy>(&mut self, addr: u32) -> T {
        if self.dcache_enabled() && self.addr_region(addr).use_dcache {
            self.dcache.read(addr, &self.memory)
        } else {
            self.memory.read(addr)
        }
    }

    pub fn dmem_write<T: Copy>(&mut self, addr: u32, val: T) {
        self.memory.write(addr, val);
        if self.dcache_enabled() && self.addr_region(addr).use_dcache {
            self.dcache.write(addr, val);
        }
    }
}