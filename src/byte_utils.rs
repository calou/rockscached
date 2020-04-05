use std::str::FromStr;
use std::num::ParseIntError;

pub fn bytes_to_u64(bytes: &[u8]) -> u64 {
    let result = convert_bytes_to_u64(bytes);
    result.unwrap()
}

pub fn convert_bytes_to_u64(bytes: &[u8]) -> Result<u64, ParseIntError> {
    let x = String::from_utf8_lossy(bytes);
    let result = u64::from_str(x.as_ref());
    result
}

pub fn bytes_to_u32(bytes: &[u8]) -> u32 {
    let x = String::from_utf8_lossy(bytes);
    u32::from_str(x.as_ref()).unwrap()
}

pub fn u64_to_bytes<'a>(u: u64) -> Vec<u8> {
    u.to_string().into_bytes()
}