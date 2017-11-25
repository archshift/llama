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

    fn get_inst_or<F>(&mut self, addr: u32, orelse: F) -> T
        where F: Fn(u32) -> T {

        let map_pos = self.mappings.iter().filter_map(|x|*x)
            .position(|(map_addr, _)| map_addr == addr);

        if let Some(pos) = map_pos {
            self.update_plru64(pos);
            self.mappings[pos].unwrap().1
        } else {
            let instr = orelse(addr);
            let lru = self.lru_index;
            self.mappings[lru] = Some((addr, instr));
            self.update_plru64(lru);
            instr
        }
    }
}


use mem;
use cpu::decoder_arm::ArmInstruction;
use cpu::decoder_thumb::ThumbInstruction;

pub struct ICacheThumb(TinyCache<ThumbInstruction>);
impl ICacheThumb {
    pub fn new() -> Self {
        ICacheThumb(TinyCache::new())
    }

    pub fn get_inst(&mut self, addr: u32, mem: &mem::MemController) -> ThumbInstruction {
        self.0.get_inst_or(addr, |addr| ThumbInstruction::decode(mem.read::<u16>(addr)))
    }
}

pub struct ICacheArm(TinyCache<ArmInstruction>);
impl ICacheArm {
    pub fn new() -> Self {
        ICacheArm(TinyCache::new())
    }

    pub fn get_inst(&mut self, addr: u32, mem: &mem::MemController) -> ArmInstruction {
        self.0.get_inst_or(addr, |addr| ArmInstruction::decode(mem.read::<u32>(addr)))
    }
}