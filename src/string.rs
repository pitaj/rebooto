use std::fmt;

#[derive(Debug)]
pub struct FromUtf16Error;

impl fmt::Display for FromUtf16Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt("invalid utf-16: lone surrogate found", f)
    }
}

impl std::error::Error for FromUtf16Error {}

pub fn from_utf16le(v: &[u8]) -> Result<String, FromUtf16Error> {
    if v.len() % 2 != 0 {
        return Err(FromUtf16Error);
    }
    match (cfg!(target_endian = "little"), unsafe {
        v.align_to::<u16>()
    }) {
        (true, ([], v, [])) => String::from_utf16(v).map_err(|_| FromUtf16Error),
        _ => char::decode_utf16(
            v.chunks_exact(2)
                .map(|chunk| u16::from_le_bytes(chunk.try_into().unwrap())),
        )
        .collect::<Result<_, _>>()
        .map_err(|_| FromUtf16Error),
    }
}

pub fn encode_utf16le(s: &str) -> Vec<u8> {
    s.encode_utf16().flat_map(|c| c.to_le_bytes()).collect()
}
