use std::str::FromStr;

pub fn bytes_to_u64(bytes: &[u8]) -> u64 {
    let x = String::from_utf8_lossy(bytes);
    u64::from_str(x.as_ref()).unwrap()
}

pub fn bytes_to_u32(bytes: &[u8]) -> u32 {
    let x = String::from_utf8_lossy(bytes);
    u32::from_str(x.as_ref()).unwrap()
}