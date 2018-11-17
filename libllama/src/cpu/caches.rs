use utils::cache::TinyCache;
use mem;

pub struct MemCache(TinyCache<[u32; 8], mem::MemController>);
impl MemCache {
    fn new() -> Self {
        MemCache(TinyCache::new(Self::src_fn, Self::sink_fn))
    }

    #[inline]
    fn decompose_addr(addr: u32) -> (u32, u32) {
        (addr & !31, addr & 31)
    }

    #[inline]
    fn src_fn(fb_mem: &mut mem::MemController, k: u32) -> [u32; 8] {
        fb_mem.read::<[u32; 8]>(k)
    }

    #[inline]
    fn sink_fn(fb_mem: &mut mem::MemController, k: u32, val: &[u32; 8]) {
        fb_mem.write::<[u32; 8]>(k, *val)
    }

    #[inline]
    fn piece_of_line<T: Copy>(line: &[u32; 8], offs: usize) -> T {
        unsafe { *(((line.as_ptr() as usize) + offs) as *const T) }
    }

    #[inline]
    fn put_in_line<T: Copy>(line: &mut [u32; 8], offs: usize, val: T) {
        unsafe { *(((line.as_mut_ptr() as usize) + offs) as *mut T) = val; }
    }

    #[inline]
    fn read<T: Copy>(&mut self, addr: u32, fallback_mem: &mut mem::MemController) -> T {
        let (line_base, line_rem) = Self::decompose_addr(addr);
        let buf = self.0.get_or(line_base, fallback_mem);
        Self::piece_of_line(&buf, line_rem as usize)
    }

    #[inline]
    fn write<T: Copy>(&mut self, addr: u32, val: T, fallback_mem: &mut mem::MemController) {
        let (line_base, line_rem) = Self::decompose_addr(addr);
        let updater_fn = |_, line: &mut [u32; 8]| Self::put_in_line(line, line_rem as usize, val);
        self.0.update_or(line_base, updater_fn, fallback_mem);
    }

    pub fn invalidate(&mut self, fallback_mem: &mut mem::MemController) {
        self.0.invalidate(fallback_mem);
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

        if mask == 0 { panic!("Attempted to read memory from {:#08X} in nonexistent MpuRegion!", addr) };
        mask
    }

    fn icache_enabled(&self) -> bool {
        self.enabled & self.icache_enabled
    }

    fn dcache_enabled(&self) -> bool {
        self.enabled & self.dcache_enabled
    }

    pub fn imem_read<T: Copy>(&mut self, addr: u32) -> T {
        assert!( (addr as usize) % std::mem::size_of::<T>() == 0 );

        if self.icache_enabled() && (self.region_use_icache & self.region_mask(addr)) != 0 {
            self.icache.read(addr, &mut self.memory)
        } else {
            self.memory.read(addr)
        }
    }

    pub fn icache_invalidate(&mut self) {
        self.icache.invalidate(&mut self.memory);
    }

    pub fn dmem_read<T: Copy>(&mut self, addr: u32) -> T {
        assert!( (addr as usize) % std::mem::size_of::<T>() == 0 );

        if self.dcache_enabled() && (self.region_use_dcache & self.region_mask(addr)) != 0 {
            self.dcache.read(addr, &mut self.memory)
        } else {
            self.memory.read(addr)
        }
    }

    pub fn dmem_write<T: Copy>(&mut self, addr: u32, val: T) {
        assert!( (addr as usize) % std::mem::size_of::<T>() == 0 );

        if self.dcache_enabled() && (self.region_use_dcache & self.region_mask(addr)) != 0 {
            self.dcache.write(addr, val, &mut self.memory);
        } else {
            self.memory.write(addr, val);
        }
    }

    pub fn dcache_invalidate(&mut self) {
        self.dcache.invalidate(&mut self.memory);
    }
}
