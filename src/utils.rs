use core::str;

use numtoa::NumToA;

pub struct USizeStrBuf {
    buf: [u8; 20],
}

impl USizeStrBuf {
    pub fn new() -> Self {
        Self { buf: [0u8; 20] }
    }

    pub fn stringify<'a>(&'a mut self, val: usize) -> &'a str {
        let utf8 = val.numtoa(10, &mut self.buf);
        // This never panics
        str::from_utf8(&utf8).unwrap()
    }
}