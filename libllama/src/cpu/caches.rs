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