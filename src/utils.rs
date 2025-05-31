use core::str;


#[cfg(any(
    feature = "max_headers_32",
    feature = "max_headers_48",
    feature = "max_headers_64"
))]
use heapless::FnvIndexMap;

use numtoa::NumToA;


pub struct USizeStrBuf {
    buf: [u8; 20],
}

impl USizeStrBuf {
    pub fn new() -> Self {
        Self { buf: [0u8; 20] }
    }

    pub fn stringify(&mut self, val: usize) -> &str {
        let utf8 = val.numtoa(10, &mut self.buf);
        // This never panics
        str::from_utf8(utf8).unwrap()
    }
}
