use crate::headers::HeaderName;
use cfg_if::cfg_if;

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

cfg_if! {
if #[cfg(feature = "max_headers_16")] {
pub(crate) const MAX_HEADER_COUNT: usize = 16;
} else if #[cfg(feature = "max_headers_24")] {
pub(crate) const MAX_HEADER_COUNT: usize = 24;
} else if #[cfg(feature = "max_headers_32")] {
pub(crate) const MAX_HEADER_COUNT: usize = 32;
} else if #[cfg(feature = "max_headers_48")] {
pub(crate) const MAX_HEADER_COUNT: usize = 48;
} else {
pub(crate) const MAX_HEADER_COUNT: usize = 64;
}
}

cfg_if! {
    if #[cfg(any(feature = "max_headers_32", feature = "max_headers_48", feature = "max_headers_64"))] {
        pub(crate) type HeaderMap<'a> = heapless::FnvIndexMap<HeaderName<'a>, &'a str, MAX_HEADER_COUNT>;
    } else {
        pub(crate) type HeaderMap<'a> = heapless::LinearMap<HeaderName<'a>, &'a str, MAX_HEADER_COUNT>;
    }
}

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

    /// The HTTP request headers.
    ///
    /// See [`HttpRequest::try_find_header()`].
    pub(crate) headers: HeaderMap<'a>,
}

impl<'a> HttpRequest<'a> {
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
    pub fn try_find_header(&'a self, header: &HeaderName<'_>) -> Option<&'a str> {
        self.headers.get(header).map(|v| &**v)
    }
}
