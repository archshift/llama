use std::num::ParseIntError;

pub fn from_hex(string: &str) -> Result<u32, ParseIntError> {
    let slice = if string.starts_with("0x") {
        &string[2..]
    } else {
        &string[..]
    };
    u32::from_str_radix(slice, 16)
}