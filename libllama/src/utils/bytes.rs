use std::mem;
use std::slice;

use extprim::u128::u128 as u128_t;

pub fn from_u128(mut num: u128_t) -> [u8; 16] {
    let mut data = [0u8; 0x10];
    for b in data.iter_mut().rev() {
        *b = num.low64() as u8;
        num >>= 8;
    }
    data
}

pub fn to_u128(data: &[u8]) -> u128_t {
    assert!(data.len() <= 16);
    let mut new = u128_t::new(0);
    for b in data.iter() {
        new <<= 8;
        new |= u128_t::new(*b as u64);
    }
    new
}

pub unsafe fn from_val<'a, T: Copy>(data: &'a T) -> &'a [u8] {
    slice::from_raw_parts(data as *const T as *const u8, mem::size_of::<T>())
}

pub unsafe fn from_mut_val<'a, T: Copy>(data: &'a mut T) -> &'a mut [u8] {
    slice::from_raw_parts_mut(data as *mut T as *mut u8, mem::size_of::<T>())
}

pub unsafe fn to_val<T: Copy>(data: &[u8]) -> T {
    let mut out: T = mem::zeroed();
    from_mut_val(&mut out).copy_from_slice(data);
    out
}

pub fn Tpos<T: Copy>(start: usize) -> ::std::ops::Range<usize> {
    start .. start + mem::size_of::<T>()
}