use core::str;

use numtoa::NumToA;
use winnow::BStr;

use crate::{
    error::Error,
    request::{self, HttpRequest, HttpVersion},
};

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

pub fn parse<'a>(buf: &'a [u8]) -> Result<HttpRequest<'a>, Error> {
    let idx = buf
        .iter()
        .position(|a| *a == b'\n')
        .ok_or(Error::BadRequest)?;

    let line = &buf[..idx];

    let space = line
        .iter()
        .take(8)
        .position(|a| *a == b' ')
        .ok_or(Error::BadRequest)?;
    let method = find_method(&line[..space]).ok_or(Error::BadRequest)?;

    let next_section = &line[space + 1..];

    let space = next_section
        .iter()
        .position(|a| *a == b' ')
        .ok_or(Error::BadRequest)?;

    let path: &str = str::from_utf8(&next_section[..space]).map_err(|_| Error::BadRequest)?;

    let version_str = &next_section[space + 1..space + 9];

    let version = match version_str {
        b"HTTP/1.0" => HttpVersion::Http10,
        b"HTTP/1.1" => HttpVersion::Http11,
        _ => return Err(Error::BadRequest),
    };

    Ok(HttpRequest {
        version,
        method,
        path: path,
        body_len: None,
    })
}

pub fn find_method(buf: &[u8]) -> Option<request::HttpMethod> {
    match buf {
        b"GET" => Some(request::HttpMethod::Get),
        b"HEAD" => Some(request::HttpMethod::Head),
        b"POST" => Some(request::HttpMethod::Post),
        b"OPTIONS" => Some(request::HttpMethod::Options),
        b"TRACE" => Some(request::HttpMethod::Trace),
        b"DELETE" => Some(request::HttpMethod::Delete),
        b"PUT" => Some(request::HttpMethod::Put),
        b"PATCH" => Some(request::HttpMethod::Patch),
        b"CONNECT" => Some(request::HttpMethod::Connect),
        _ => None,
    }
}
