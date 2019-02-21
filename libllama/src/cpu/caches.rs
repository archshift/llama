use utils::cache::TinyCache;
use mem;

use cpu::{Version, v5};

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


pub trait Ops {
    fn set_enabled(&mut self, enabled: bool);
    fn imem_read<T: Copy>(&mut self, addr: u32) -> T;
    fn dmem_read<T: Copy>(&mut self, addr: u32) -> T;
    fn dmem_write<T: Copy>(&mut self, addr: u32, val: T);
    fn icache_set_enabled(&mut self, enabled: bool);
    fn icache_invalidate(&mut self);
    fn dcache_set_enabled(&mut self, enabled: bool);
    fn dcache_invalidate(&mut self);
    fn main_mem(&self) -> &mem::MemController;
    fn main_mem_mut(&mut self) -> &mut mem::MemController;
}


pub struct Mpu {
    enabled: bool,
    icache_enabled: bool,
    dcache_enabled: bool,

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

        if mask == 0 { panic!("Attempted to read memory from 0x{:08X} in nonexistent MpuRegion!", addr) };
        mask
    }

    fn icache_enabled(&self) -> bool {
        self.enabled & self.icache_enabled
    }

    fn dcache_enabled(&self) -> bool {
        self.enabled & self.dcache_enabled
    }
}

impl Ops for Mpu {
    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled
    }

    fn imem_read<T: Copy>(&mut self, addr: u32) -> T {
        assert!( (addr as usize) % std::mem::size_of::<T>() == 0 );

        if self.icache_enabled() && (self.region_use_icache & self.region_mask(addr)) != 0 {
            self.icache.read(addr, &mut self.memory)
        } else {
            self.memory.read(addr)
        }
    }

    fn icache_set_enabled(&mut self, enabled: bool) {
        self.icache_enabled = enabled;
        if !enabled { self.icache_invalidate() }
    }
    fn icache_invalidate(&mut self) {
        self.icache.invalidate(&mut self.memory);
    }

    fn dmem_read<T: Copy>(&mut self, addr: u32) -> T {
        assert!( (addr as usize) % std::mem::size_of::<T>() == 0 );

        if self.dcache_enabled() && (self.region_use_dcache & self.region_mask(addr)) != 0 {
            self.dcache.read(addr, &mut self.memory)
        } else {
            self.memory.read(addr)
        }
    }

    fn dmem_write<T: Copy>(&mut self, addr: u32, val: T) {
        assert!( (addr as usize) % std::mem::size_of::<T>() == 0 );

        if self.dcache_enabled() && (self.region_use_dcache & self.region_mask(addr)) != 0 {
            self.dcache.write(addr, val, &mut self.memory);
        } else {
            self.memory.write(addr, val);
        }
    }

    fn dcache_set_enabled(&mut self, enabled: bool) {
        self.dcache_enabled = enabled;
        if !enabled { self.dcache_invalidate() }
    }
    fn dcache_invalidate(&mut self) {
        self.dcache.invalidate(&mut self.memory);
    }

    fn main_mem(&self) -> &mem::MemController {
        &self.memory
    }
    fn main_mem_mut(&mut self) -> &mut mem::MemController {
        &mut self.memory
    }
}



pub struct Mmu {
    enabled: bool,
    icache_enabled: bool,
    dcache_enabled: bool,

    pub(crate) pagesel: usize,
    pub(crate) page_tables: [u32; 2],
    pub(crate) backcompat_walk: bool,

    pub memory: mem::MemController,
    pub icache: MemCache,
    pub dcache: MemCache
}

impl Mmu {
    pub fn new(memory: mem::MemController) -> Self {
        Mmu {
            enabled: false,
            icache_enabled: false,
            dcache_enabled: false,

            pagesel: 0,
            page_tables: [0u32; 2],
            backcompat_walk: true,

            memory: memory,
            icache: MemCache::new(),
            dcache: MemCache::new(),
        }
    }

    fn _icache_enabled(&self) -> bool {
        self.enabled & self.icache_enabled
    }

    fn _dcache_enabled(&self) -> bool {
        self.enabled & self.dcache_enabled
    }

    fn select_page_table(&self, vaddr: u32) -> u32 {
        if self.pagesel == 0 || bits!(vaddr, (32-self.pagesel) : 31) == 0 {
            self.page_tables[0]
        } else {
            self.page_tables[1]
        }
    }

    fn walk_l2(&self, table: u32, vaddr: u32) -> u32 {
        if self.backcompat_walk {
            const DESC_TYPE_FAULT: u32 = 0;
            const DESC_TYPE_LARGE_PAGE: u32 = 1;
            const DESC_TYPE_SMALL_PAGE: u32 = 2;
            const DESC_TYPE_EXT_PAGE: u32 = 3;
            bf!(Descriptor[u32] {
                ty: 0:1,
                large_page_base: 16:31,
                small_page_base: 12:31,
                ext_page_base: 12:31,
            });

            let desc_addr = table + 4 * bits!(vaddr, 12:19);
            let desc = Descriptor::new(self.memory.read(desc_addr));

            match desc.ty.get() {
                DESC_TYPE_FAULT => unimplemented!(),
                DESC_TYPE_LARGE_PAGE => {
                    let base = desc.large_page_base.get() << 16;
                    let offs = bits!(vaddr, 0:15);
                    base | offs
                }
                DESC_TYPE_SMALL_PAGE | DESC_TYPE_EXT_PAGE => {
                    let base = desc.large_page_base.get() << 12;
                    let offs = bits!(vaddr, 0:11);
                    base | offs
                }
                _ => unreachable!()
            }
        } else {
            bf!(Descriptor[u32] {
                is_large_page: 0:0,
                is_small_page: 1:1,

                large_page_base: 16:31,
                small_page_base: 12:31,
            });

            let desc_addr = table + 4 * bits!(vaddr, 12:19);
            let desc = Descriptor::new(self.memory.read(desc_addr));
            
            match (desc.is_large_page.get(), desc.is_small_page.get()) {
                (0, 0) => unimplemented!(),
                (1, 0) => {
                    let base = desc.large_page_base.get() << 16;
                    let offs = bits!(vaddr, 0:15);
                    base | offs
                }
                (_xn, 1) => {
                    let base = desc.small_page_base.get() << 12;
                    let offs = bits!(vaddr, 0:11);
                    base | offs
                }
                _ => unreachable!()
            }
        }
    }

    fn walk_l1(&self, table: u32, vaddr: u32) -> u32 {
        const DESC_TYPE_FAULT: u32 = 0;
        const DESC_TYPE_L2: u32 = 1;
        const DESC_TYPE_SECTION: u32 = 2;
        const DESC_TYPE_RESERVED: u32 = 3;
        bf!(Descriptor[u32] {
            ty: 0:1,
            _domain: 5:8,
            l2_base: 10:31,
            is_supersection: 18:18,
            section_base: 20:31,
            supersection_base: 24:31
        });

        let desc_addr = table + 4 * bits!(vaddr, 20:31);
        let desc = Descriptor::new(self.memory.read(desc_addr));

        match desc.ty.get() {
            DESC_TYPE_FAULT => unimplemented!(),
            DESC_TYPE_L2 => {
                let l2_table = desc.l2_base.get() << 10;
                self.walk_l2(l2_table, vaddr)
            }
            DESC_TYPE_SECTION => {
                if desc.is_supersection.get() == 1 {
                    let base = desc.supersection_base.get() << 24;
                    let offs = bits!(vaddr, 0:23);
                    base | offs
                } else {
                    let base = desc.section_base.get() << 20;
                    let offs = bits!(vaddr, 0:19);
                    base | offs
                }
            }
            DESC_TYPE_RESERVED | _ => unreachable!()
        }
    }

    fn translate_addr(&self, vaddr: u32) -> u32 {
        if !self.enabled {
            vaddr
        } else {
            let page_table = self.select_page_table(vaddr);
            self.walk_l1(page_table, vaddr)
        }
    }
}

impl Ops for Mmu {
    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled
    }

    fn imem_read<T: Copy>(&mut self, vaddr: u32) -> T {
        assert!( (vaddr as usize) % std::mem::size_of::<T>() == 0 );

        let paddr = self.translate_addr(vaddr);
        self.memory.read(paddr)
    }

    fn icache_invalidate(&mut self) {
        self.icache.invalidate(&mut self.memory);
    }
    fn icache_set_enabled(&mut self, enabled: bool) {
        self.icache_enabled = enabled;
        if !enabled { self.icache_invalidate() }
    }

    fn dmem_read<T: Copy>(&mut self, vaddr: u32) -> T {
        assert!( (vaddr as usize) % std::mem::size_of::<T>() == 0 );

        let paddr = self.translate_addr(vaddr);
        self.memory.read(paddr)
    }

    fn dmem_write<T: Copy>(&mut self, vaddr: u32, val: T) {
        assert!( (vaddr as usize) % std::mem::size_of::<T>() == 0 );

        let paddr = self.translate_addr(vaddr);
        self.memory.write(paddr, val);
    }

    fn dcache_invalidate(&mut self) {
        self.dcache.invalidate(&mut self.memory);
    }
    fn dcache_set_enabled(&mut self, enabled: bool) {
        self.dcache_enabled = enabled;
        if !enabled { self.dcache_invalidate() }
    }

    fn main_mem(&self) -> &mem::MemController {
        &self.memory
    }
    fn main_mem_mut(&mut self) -> &mut mem::MemController {
        &mut self.memory
    }
}






pub enum MemMgr {
    Mpu(Mpu),
    Mmu(Mmu),
}

impl MemMgr {
    pub(crate) fn new<V: Version>(memory: mem::MemController) -> Self {
        if V::is::<v5>() {
            MemMgr::Mpu(Mpu::new(memory))
        } else {
            MemMgr::Mmu(Mmu::new(memory))
        }
    }
}

macro_rules! match_mgr {
    ($self:expr, $(+$mut:ident)* $func:ident ( $($args:expr),* ) ) => {
        match *$self {
            MemMgr::Mpu(ref $($mut)* mgr)    => mgr.$func($($args),*),
            MemMgr::Mmu(ref $($mut)* mgr)    => mgr.$func($($args),*),
        }
    }
}

impl Ops for MemMgr {
    fn set_enabled(&mut self, enabled: bool) {
        match_mgr!(self, +mut set_enabled(enabled))
    }

    fn imem_read<T: Copy>(&mut self, addr: u32) -> T {
        match_mgr!(self, +mut imem_read(addr))
    }
    fn dmem_read<T: Copy>(&mut self, addr: u32) -> T {
        match_mgr!(self, +mut dmem_read(addr))
    }
    fn dmem_write<T: Copy>(&mut self, addr: u32, val: T) {
        match_mgr!(self, +mut dmem_write(addr, val))
    }

    fn icache_set_enabled(&mut self, enabled: bool) {
        match_mgr!(self, +mut icache_set_enabled(enabled))
    }
    fn icache_invalidate(&mut self) {
        match_mgr!(self, +mut icache_invalidate())
    }
    fn dcache_set_enabled(&mut self, enabled: bool) {
        match_mgr!(self, +mut dcache_set_enabled(enabled))
    }
    fn dcache_invalidate(&mut self) {
        match_mgr!(self, +mut dcache_invalidate())
    }

    fn main_mem(&self) -> &mem::MemController {
        match_mgr!(self, main_mem())
    }
    fn main_mem_mut(&mut self) -> &mut mem::MemController {
        match_mgr!(self, +mut main_mem_mut())
    }
}
