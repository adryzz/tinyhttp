#![no_std]

use managed::ManagedMap;
mod config;
mod headers;
use headers::RequestHeader;

#[cfg(not(any(feature = "ipv4", feature = "ipv6")))]
compile_error!("You must select at least one of the following features: 'ipv4', 'ipv6'");

/// Specifies the version of HTTP supported by the client.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum HttpVersion {
    /// HTTP/1.0
    /// Only supports the GET, HEAD and POST methods.
    ///
    /// Only supports Content-Type, caching, basic authorization, and status codes.
    Http10 = 1,

    /// HTTP/1.1
    #[default]
    Http11 = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    /// Supported in all HTTP versions
    Get,
    /// Supported in HTTP/1.0
    Head,
    /// Supported in HTTP/1.0
    Post,
    /// Only HTTP/1.1
    Options,
    /// Only HTTP/1.1
    Trace,
    /// Only HTTP/1.1
    Delete,
    /// Only HTTP/1.1
    Put,
    /// Only HTTP/1.1
    Patch,
    /// Only HTTP/1.1
    Connect,
}

/// Max number of headers parsed
const MAX_HEADER_COUNT: usize = 16;

pub struct HttpRequest<'a> {
    version: HttpVersion,
    method: HttpMethod,
    path: &'a str,
    headers: ManagedMap<'a, RequestHeader<'a>, &'a str>,
}
