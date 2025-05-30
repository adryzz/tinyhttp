#[cfg(feature = "default_error_pages")]
/// The default HTTP 400 Bad Request page
pub const DEFAULT_400: StaticPage = StaticPage::html(include_str!("../static/400.html"));
#[cfg(feature = "default_error_pages")]
/// The default HTTP 401 Unauthorized page
pub const DEFAULT_401: StaticPage = StaticPage::html(include_str!("../static/401.html"));
#[cfg(feature = "default_error_pages")]
/// The default HTTP 404 Not Found page
pub const DEFAULT_404: StaticPage = StaticPage::html(include_str!("../static/404.html"));
#[cfg(feature = "default_error_pages")]
/// The default HTTP 500 Internal Server Error page
pub const DEFAULT_500: StaticPage = StaticPage::html(include_str!("../static/500.html"));

#[derive(Debug, Clone, Copy)]
pub struct HttpConfig<'a> {
    /// Port to TCP listen on
    ///
    /// Default: 80
    pub port: u16,
    /// Number of seconds each TCP socket is allowed to be kept alive for without any data transfer.
    ///
    /// If None, the connection is closed immediately.
    ///
    /// Default: 5 seconds.
    pub keepalive: Option<u16>,

    /// Username/Password combo for global basic authentication.
    ///
    /// Default: None
    #[cfg(feature = "global_http_basic_auth")]
    pub basic_auth: Option<(&'a str, &'a str)>,

    /// A static page to load when sending a 400 Bad Request error code.
    ///
    /// Default: None
    pub http_400: Option<StaticPage<'a>>,

    /// A static page to load when sending a 401 Unauthorized error code.
    ///
    /// Default: None
    pub http_401: Option<StaticPage<'a>>,

    /// A static page to load when sending a 404 Not Found error code.
    ///
    /// Default: None
    pub http_404: Option<StaticPage<'a>>,

    /// A static page to load when sending a 500 Internal Server Error error code.
    ///
    /// Default: None
    pub http_500: Option<StaticPage<'a>>,
}

impl Default for HttpConfig<'static> {
    #[cfg(not(feature = "default_error_pages"))]
    fn default() -> Self {
        Self {
            port: 80,
            keepalive: Some(5),
            #[cfg(feature = "global_http_basic_auth")]
            basic_auth: None,
            http_400: None,
            http_401: None,
            http_404: None,
            http_500: None,
        }
    }

    #[cfg(feature = "default_error_pages")]
    fn default() -> Self {
        Self {
            port: 80,
            keepalive: Some(5),
            #[cfg(feature = "global_http_basic_auth")]
            basic_auth: None,
            http_400: Some(DEFAULT_400),
            http_401: Some(DEFAULT_401),
            http_404: Some(DEFAULT_404),
            http_500: Some(DEFAULT_500),
        }
    }
}

/// Represents a static page, loaded from flash.
/// Currently only supports uncompressed pages.
#[derive(Debug, Clone, Copy)]
pub struct StaticPage<'a> {
    pub(crate) content_type: &'a str,
    pub(crate) body: &'a str,
}

impl<'a> StaticPage<'a> {
    pub const fn html(body: &'a str) -> Self {
        StaticPage {
            content_type: "text/html; charset=UTF-8",
            body,
        }
    }

    pub const fn text(body: &'a str) -> Self {
        StaticPage {
            content_type: "text/plain; charset=UTF-8",
            body,
        }
    }

    pub const fn json(body: &'a str) -> Self {
        StaticPage {
            content_type: "text/json; charset=UTF-8",
            body,
        }
    }
}
