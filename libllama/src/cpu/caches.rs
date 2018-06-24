use utils::cache::TinyCache;
use mem;

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


pub struct Mpu {
    pub enabled: bool,
    pub icache_enabled: bool,
    pub dcache_enabled: bool,

    pub region_enabled: u8,
    pub region_use_icache: u8,
    pub region_use_dcache: u8,
    pub region_base_sigbits: [u32; 8],
    pub region_size_exp: [u32; 8],

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

            region_enabled: 0,
            region_use_icache: 0,
            region_use_dcache: 0,
            region_base_sigbits: [0; 8],
            region_size_exp: [0; 8],

            memory: memory,
            icache: MemCache::new(),
            dcache: MemCache::new(),
        }
    }

    fn region_mask(&self, addr: u32) -> u8 {
        let mut mask = 0;
        for i in (0..8).rev() {
            let bit = 1 << i;
            if (self.region_enabled & bit != 0) && ((addr >> self.region_size_exp[i]) == self.region_base_sigbits[i]) {
                mask |= bit;
                break;
            }
        }

        mask != 0 || panic!("Attempted to read memory from {:#08X} in nonexistent MpuRegion!", addr);
        mask
    }

    fn icache_enabled(&self) -> bool {
        self.enabled & self.icache_enabled
    }

    fn dcache_enabled(&self) -> bool {
        self.enabled & self.dcache_enabled
    }

    pub fn imem_read<T: Copy>(&mut self, addr: u32) -> T {
        if self.icache_enabled() && (self.region_use_icache & self.region_mask(addr)) != 0 {
            self.icache.read(addr, &self.memory)
        } else {
            self.memory.read(addr)
        }
    }

    pub fn dmem_read<T: Copy>(&mut self, addr: u32) -> T {
        if self.dcache_enabled() && (self.region_use_dcache & self.region_mask(addr)) != 0 {
            self.dcache.read(addr, &self.memory)
        } else {
            self.memory.read(addr)
        }
    }

    pub fn dmem_write<T: Copy>(&mut self, addr: u32, val: T) {
        self.memory.write(addr, val);
        if self.dcache_enabled() && (self.region_use_dcache & self.region_mask(addr)) != 0 {
            self.dcache.write(addr, val);
        }
    }
}