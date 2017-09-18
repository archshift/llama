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