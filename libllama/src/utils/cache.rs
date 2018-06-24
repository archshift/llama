use std::mem;
use std::ops::{Deref, DerefMut};

#[derive(Copy, Clone)]
union Zeroable<T: Copy> {
    _unused: u8,
    val: T
}

impl<T: Copy> Deref for Zeroable<T> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &T {
        unsafe { &self.val }
    }
}

impl<T: Copy> DerefMut for Zeroable<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut self.val }
    }
}

const CACHE_SIZE_BITS: usize = 6;
const CACHE_SIZE: usize = 1 << CACHE_SIZE_BITS;

pub struct TinyCache<T: Copy> {
    map_keys: [u32; CACHE_SIZE],
    map_vals: [Zeroable<T>; CACHE_SIZE],
}

impl<T: Copy> TinyCache<T> {
    pub fn new() -> Self {
        TinyCache {
            map_keys: [!0; CACHE_SIZE],
            map_vals: [unsafe { mem::zeroed() }; CACHE_SIZE],
        }
    }

    pub fn invalidate(&mut self) {
        self.map_keys = [!0; CACHE_SIZE];
    }

    pub fn key_to_index(key: u32) -> usize {
        let hash = (key.wrapping_mul(2654435761)) as usize;
        hash >> (32 - CACHE_SIZE_BITS) // shifted by 32 - log2(# buckets)
    }

    pub fn get_or<F>(&mut self, key: u32, orelse: F) -> &T
        where F: Fn(u32) -> T {
        let idx = Self::key_to_index(key);
        if self.map_keys[idx] != key {
            self.map_keys[idx] = key;
            *self.map_vals[idx] = orelse(key);
        }
        &*self.map_vals[idx]
    }

    pub fn update_in<F>(&mut self, key: u32, updater: F)
        where F: Fn(u32, &mut T) {
        let idx = Self::key_to_index(key);
        if self.map_keys[idx] == key {
            updater(key, &mut self.map_vals[idx]);
        }
    }
}