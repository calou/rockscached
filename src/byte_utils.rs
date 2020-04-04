use std::str::FromStr;

pub fn bytes_to_u64(bytes: &[u8]) -> u64 {
    let x = String::from_utf8_lossy(bytes);
    u64::from_str(x.as_ref()).unwrap()
}

pub fn u64_to_bytes<'a>(u: u64) -> Vec<u8> {
    u.to_string().into_bytes()
}