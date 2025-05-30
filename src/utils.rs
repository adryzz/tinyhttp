use core::str;
use core::str::FromStr;

#[cfg(any(feature = "max_headers_16", feature = "max_headers_24"))]
use heapless::LinearMap;

#[cfg(any(
    feature = "max_headers_32",
    feature = "max_headers_48",
    feature = "max_headers_64"
))]
use heapless::FnvIndexMap;

use numtoa::NumToA;

use crate::{
    error::Error,
    headers,
    request::{self, HttpRequest, HttpVersion},
};

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

    #[cfg(any(feature = "max_headers_16", feature = "max_headers_24"))]
    let mut headers = LinearMap::new();

    #[cfg(any(
        feature = "max_headers_32",
        feature = "max_headers_48",
        feature = "max_headers_64"
    ))]
    let mut headers = FnvIndexMap::new();

    let mut next_section = &buf[idx + 1..];

    // if there's no headers or body, early return
    // TODO: see if 5 is the correct number
    if next_section.len() < 5 {
        return Ok(HttpRequest {
            version,
            method,
            path,
            headers,
            body_len: None,
            body_inline: None
        })
    }

    // TODO: check if there's a \r\n (that would indicate the headers have an ending)
    // if not, return with a 413 Entity Too Large.

    let mut last_idx = idx+1;

    for _ in 0..request::MAX_HEADER_COUNT {
        let idx = next_section
            .iter()
            .position(|a| *a == b'\n')
            .ok_or(Error::BadRequest)?;

        last_idx = idx + 1;

        // the ending is \r\n\r\n, so after the \n there's a \r
        if next_section[idx + 1] == b'\r' {
            continue;
        }

        let header_bytes = &next_section[..idx];

        let colon_idx = header_bytes
            .iter()
            .position(|a| *a == b':')
            .ok_or(Error::BadRequest)?;

        let name = str::from_utf8(&header_bytes[..colon_idx]).map_err(|_| Error::BadRequest)?;
        let value = str::from_utf8(&header_bytes[colon_idx + 2..header_bytes.len() - 1])
            .map_err(|_| Error::BadRequest)?;
        let header = headers::RequestHeader::from_str(name);

        next_section = &next_section[idx + 1..];

        headers
            .insert(header, value)
            .map_err(|_| Error::EntityTooLarge)?;
    }

    // find content-length header
    let body_len = match headers.get(&headers::RequestHeader::ContentLength) {
        Some(head) => usize::from_str(head).ok(),
        None => None,
    };


    // TODO: if the body is inline (fully in the buffer), parse it
    // otherwise let the user read it in chunks

    let last = &next_section[last_idx+2..];

    let body_inline = if last.is_empty() {
        None
    } else {
        if let Some(len) = body_len {
            if len < last.len() {
                return Err(Error::BadRequest);
            }
        }

        Some(last)
    };

    Ok(HttpRequest {
        version,
        method,
        path,
        headers,
        body_len,
        body_inline
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
