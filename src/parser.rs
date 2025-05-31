//! HTTP parser adapted from the winnow example https://github.com/winnow-rs/winnow/blob/main/examples/http/parser.rs

use winnow::combinator::seq;
use winnow::error::{ContextError, ErrMode};
use winnow::prelude::*;
use winnow::{ascii::line_ending, token::take_while};

use crate::error::Error;
use crate::headers::HeaderName;
use crate::request::HttpMethod;
use crate::request::HttpRequest;
use crate::request::HttpVersion;

#[cfg(any(feature = "max_headers_16", feature = "max_headers_24"))]
use heapless::LinearMap as Map;

#[cfg(any(
    feature = "max_headers_32",
    feature = "max_headers_48",
    feature = "max_headers_64"
))]
use heapless::FnvIndexMap as Map;

pub(crate) type Stream<'i> = &'i [u8];

struct RequestLine<'a> {
    method: HttpMethod,
    path: &'a str,
    version: HttpVersion,
}

pub fn parse_request<'s>(mut buf: &'s [u8]) -> core::result::Result<HttpRequest<'s>, Error> {
    let input: &mut Stream<'s> = &mut buf;

    request(input).map_err(|_| Error::BadRequest)?
}

pub fn request<'s>(input: &mut Stream<'s>) -> ModalResult<Result<HttpRequest<'s>, Error>> {
    let req = request_line(input)?;

    let mut headers = Map::new();

    let headers_iter = core::iter::from_fn(|| match header.parse_next(input) {
        Ok(o) => Some(Ok(o)),
        Err(ErrMode::Backtrack(_)) => None,
        Err(e) => Some(Err(e)),
    });

    for n in headers_iter {
        let n = n?;
       match headers.insert(n.0, n.1) {
            Err(_) => return Ok(Err(Error::EntityTooLarge)),
            _ => {}
       }
    }

    let _ = line_ending.parse_next(input)?;

    Ok(Ok(HttpRequest {
        version: req.version,
        method: req.method,
        path: req.path,
        headers,
    }))
}

fn request_line<'s>(input: &mut Stream<'s>) -> ModalResult<RequestLine<'s>> {
    seq!( RequestLine {
        method: http_method,
        _: take_while(1.., is_space),
        path: http_path,
        _: take_while(1.., is_space),
        version: http_version,
        _: line_ending,
    })
    .parse_next(input)
}

fn http_method<'s>(input: &mut Stream<'s>) -> ModalResult<HttpMethod> {
    let method = take_while(1.., is_token).parse_next(input)?;

    match method {
        b"GET" => Ok(HttpMethod::Get),
        b"HEAD" => Ok(HttpMethod::Head),
        b"POST" => Ok(HttpMethod::Post),
        b"OPTIONS" => Ok(HttpMethod::Options),
        b"TRACE" => Ok(HttpMethod::Trace),
        b"DELETE" => Ok(HttpMethod::Delete),
        b"PUT" => Ok(HttpMethod::Put),
        b"PATCH" => Ok(HttpMethod::Patch),
        b"CONNECT" => Ok(HttpMethod::Connect),
        _ => Err(ErrMode::Cut(ContextError::from_input(input))),
    }
}

fn http_path<'s>(input: &mut Stream<'s>) -> ModalResult<&'s str> {
    let buf = take_while(1.., is_not_space).parse_next(input)?;
    str::from_utf8(buf).map_err(|_| ErrMode::Cut(ContextError::from_input(input)))
}

fn http_version<'s>(input: &mut Stream<'s>) -> ModalResult<HttpVersion> {
    let _ = "HTTP/".parse_next(input)?;
    let version = take_while(1.., is_version).parse_next(input)?;

    match version {
        b"1.0" => Ok(HttpVersion::Http10),
        b"1.1" => Ok(HttpVersion::Http11),
        _ => Err(ErrMode::Cut(ContextError::from_input(input))),
    }
}

fn header<'s>(input: &mut Stream<'s>) -> ModalResult<(HeaderName<'s>, &'s str)> {
    seq!((
        header_name,
        _: ':',
        header_value,
    ))
    .parse_next(input)
}

fn header_name<'s>(input: &mut Stream<'s>) -> ModalResult<HeaderName<'s>> {
    let data = take_while(1.., is_token).parse_next(input)?;
    let str = str::from_utf8(data).map_err(|_| ErrMode::Cut(ContextError::from_input(input)))?;
    Ok(HeaderName::from_str(str))
}

fn header_value<'s>(input: &mut Stream<'s>) -> ModalResult<&'s str> {
    let _ = take_while(1.., is_horizontal_space).parse_next(input)?;
    let data = take_while(1.., till_line_ending).parse_next(input)?;
    let _ = line_ending.parse_next(input)?;

    str::from_utf8(data).map_err(|_| ErrMode::Cut(ContextError::from_input(input)))
}

#[rustfmt::skip]
#[allow(clippy::match_same_arms)]
#[allow(clippy::match_like_matches_macro)]
fn is_token(c: u8) -> bool {
  match c {
    128..=255 => false,
    0..=31    => false,
    b'('      => false,
    b')'      => false,
    b'<'      => false,
    b'>'      => false,
    b'@'      => false,
    b','      => false,
    b';'      => false,
    b':'      => false,
    b'\\'     => false,
    b'"'      => false,
    b'/'      => false,
    b'['      => false,
    b']'      => false,
    b'?'      => false,
    b'='      => false,
    b'{'      => false,
    b'}'      => false,
    b' '      => false,
    _         => true,
  }
}

fn is_version(c: u8) -> bool {
    c.is_ascii_digit() || c == b'.'
}

fn till_line_ending(c: u8) -> bool {
    c != b'\r' && c != b'\n'
}

fn is_space(c: u8) -> bool {
    c == b' '
}

fn is_not_space(c: u8) -> bool {
    c != b' '
}

fn is_horizontal_space(c: u8) -> bool {
    c == b' ' || c == b'\t'
}
