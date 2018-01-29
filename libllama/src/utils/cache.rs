use std::mem;

pub struct TinyCache<T: Copy> {
    plru_set: u64,
    lru_index: usize,
    map_keys: [u64; 64],
    map_vals: [T; 64],
}

impl<T: Copy> TinyCache<T> {
    pub fn new() -> Self {
        TinyCache {
            plru_set: 0,
            lru_index: 0,
            map_keys: [!0; 64],
            map_vals: [unsafe { mem::zeroed() }; 64]
        }
    }

    pub fn invalidate(&mut self) {
        self.plru_set = 0;
        self.lru_index = 0;
        self.map_keys = [!0; 64];
    }

    pub fn update_plru64(&mut self, accessed: usize) {
        let mut bitset = self.plru_set;
        bitset |= (1 << accessed) as u64;
        bitset *= (bitset != !0) as u64;
        self.plru_set = bitset;
        self.lru_index = (!bitset).trailing_zeros() as usize;
    }

    pub fn get_or<F>(&mut self, key: u32, orelse: F) -> &T
        where F: Fn(u32) -> T {
        let key = key as u64;
        let map_pos = self.map_keys.iter().position(|map_key| *map_key == key);

        let pos = if let Some(pos) = map_pos {
            self.update_plru64(pos);
            pos
        } else {
            let instr = orelse(key as u32);
            let lru = self.lru_index;
            self.map_keys[lru] = key;
            self.map_vals[lru] = instr;
            self.update_plru64(lru);
            lru
        };
        &self.map_vals[pos]
    }

    pub fn update_in<F>(&mut self, key: u32, updater: F)
        where F: Fn(u32, &mut T) {
        let key = key as u64;
        let map_item = self.map_keys.iter().position(|map_key| *map_key == key);
        if let Some(pos) = map_item {
            updater(key as u32, &mut self.map_vals[pos]);
        }
    }
}