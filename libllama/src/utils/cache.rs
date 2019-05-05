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

type SrcFn<T, C> = fn(&mut C, u32) -> T;
type SinkFn<T, C> = fn(&mut C, u32, &T);

pub struct TinyCache<T: Copy, C> {
    map_keys: [u32; CACHE_SIZE],
    map_vals: [Zeroable<T>; CACHE_SIZE],
    dirty: [bool; CACHE_SIZE],
    src: SrcFn<T, C>,
    sink: SinkFn<T, C>,
}

impl<T: Copy, C> TinyCache<T, C> {
    pub fn new(src: SrcFn<T, C>, sink: SinkFn<T, C>) -> Self {
        TinyCache {
            map_keys: [!0; CACHE_SIZE],
            map_vals: [unsafe { mem::zeroed() }; CACHE_SIZE],
            dirty: [false; CACHE_SIZE],
            src: src,
            sink: sink
        }
    }

    fn flush(&mut self, idx: usize, ctx: &mut C) {
        if self.dirty[idx] {
            (self.sink)(ctx, self.map_keys[idx], &*self.map_vals[idx]);
            self.dirty[idx] = false;
        }
    }

    pub fn invalidate(&mut self, ctx: &mut C) {
        for i in 0..CACHE_SIZE {
            self.flush(i, ctx);
        }
        self.map_keys = [!0; CACHE_SIZE];
    }

    pub fn key_to_index(key: u32) -> usize {
        let hash = (key.wrapping_mul(2654435761)) as usize;
        hash >> (32 - CACHE_SIZE_BITS) // shifted by 32 - log2(# buckets)
    }

    pub fn get_or(&mut self, key: u32, ctx: &mut C) -> &T {
        let idx = Self::key_to_index(key);
        if self.map_keys[idx] != key {
            self.flush(idx, ctx);
            self.map_keys[idx] = key;
            *self.map_vals[idx] = (self.src)(ctx, key);
        }
        &*self.map_vals[idx]
    }

    pub fn update_or<F>(&mut self, key: u32, updater: F, ctx: &mut C)
        where F: Fn(u32, &mut T) {
        let idx = Self::key_to_index(key);
        if self.map_keys[idx] != key {
            self.flush(idx, ctx);
            self.map_keys[idx] = key;
            *self.map_vals[idx] = (self.src)(ctx, key);
        }
        updater(key, &mut self.map_vals[idx]);
        self.dirty[idx] = true;
    }
}
