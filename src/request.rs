use crate::headers::RequestHeader;

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

#[cfg(not(any(
    feature = "max_headers_16",
    feature = "max_headers_24",
    feature = "max_headers_32",
    feature = "max_headers_48",
    feature = "max_headers_64"
)))]
compile_error!("You must select the header limit with the corresponding feature flag!");

/// Max number of headers parsed
#[cfg(feature = "max_headers_16")]
pub(crate) const MAX_HEADER_COUNT: usize = 16;

#[cfg(feature = "max_headers_24")]
pub(crate) const MAX_HEADER_COUNT: usize = 24;

#[cfg(feature = "max_headers_32")]
pub(crate) const MAX_HEADER_COUNT: usize = 32;

#[cfg(feature = "max_headers_48")]
pub(crate) const MAX_HEADER_COUNT: usize = 48;

#[cfg(feature = "max_headers_64")]
pub(crate) const MAX_HEADER_COUNT: usize = 64;

#[derive(Debug, Clone)]
/// Represents a HTTP request made by a client.
pub struct HttpRequest<'a> {
    /// The HTTP version used by the client.
    /// 
    /// See [`HttpRequest::version()`].
    pub(crate) version: HttpVersion,

    /// The HTTP method used by the client.
    /// 
    /// See [`HttpRequest::method()`].
    pub(crate) method: HttpMethod,

    /// The HTTP path used by the client.
    /// 
    /// See [`HttpRequest::path()`].
    pub(crate) path: &'a str,

    /// The length of the HTTP request body, in bytes, if present.
    /// Equivalent to the `Content-Length` header.
    /// 
    /// See [`HttpRequest::body_len()`].
    pub(crate) body_len: Option<usize>,

    /// The part of the HTTP body contained in the RX buffer.
    /// Not relayed to the user if the body isn't fully contained in the RX buffer.
    /// 
    /// See [`HttpRequest::body_inline()`].
    pub(crate) body_inline: Option<&'a [u8]>,

    #[cfg(any(feature = "max_headers_16", feature = "max_headers_24"))]
    pub(crate) headers: heapless::LinearMap<RequestHeader<'a>, &'a str, MAX_HEADER_COUNT>,

    #[cfg(any(
        feature = "max_headers_32",
        feature = "max_headers_48",
        feature = "max_headers_64"
    ))]
    pub(crate) headers: heapless::FnvIndexMap<RequestHeader<'a>, &'a str, MAX_HEADER_COUNT>,
}

impl<'a> HttpRequest<'a> {
    /// Gets the length of the body sent by the client in bytes.
    pub fn body_len(&self) -> Option<usize> {
        self.body_len
    }

    /// Gets the HTTP body, if it's entirely contained in the RX buffer.
    /// If the body isn't entirely contained in the RX buffer, returns [`None`].
    /// 
    /// See [`crate::reader::HttpReader::body()`] and [`crate::reader::HttpBodyReader`]
    pub fn body_inline(&'a self) -> Option<&'a [u8]> {
        if let Some(l) = self.body_len {
            if let Some(b) = self.body_inline {
                if l == b.len() {
                    return self.body_inline;
                }
            }
        }

        None
    }

    /// Gets the HTTP version used by the client.
    pub fn version(&self) -> HttpVersion {
        self.version
    }

    /// Gets the resource path asked by the client.
    pub fn path(&'a self) -> &'a str {
        self.path
    }

    /// Gets the HTTP method used by the client.
    pub fn method(&'a self) -> HttpMethod {
        self.method
    }

    /// Gets the value of a request header, if one exists.
    ///
    /// When the `max_headers_16`/`max_headers_24` features are enabled, the search is `O(N)`
    ///
    /// When the `max_headers_32`/`max_headers_48`/`max_headers_64` features are enabled, the search is `O(1)`
    pub fn try_find_header(&'a self, header: &RequestHeader<'_>) -> Option<&'a str> {
        self.headers.get(header).map(|v| &**v)
    }
}
