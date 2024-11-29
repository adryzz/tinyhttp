use crate::error::Error;
use crate::reader::RequestReader;
use crate::request::HttpRequest;
use crate::status::StatusCode;
use crate::writer::{HttpResponse, ResponseWriter};

type RequestHandler = fn(RequestReader, ResponseWriter) -> Result<HttpResponse, Error>;

pub struct Router<const ROUTES: usize> {
    pub routes: [(&'static str, RequestHandler); ROUTES],
}

impl<const ROUTES: usize> Router<ROUTES> {
    /// Checks if the request can be handled by any of the request handlers, and if so, returns it.
    ///
    /// Returns None when no request handler can handle the request (HTTP 404)
    pub fn handler(&self, request: &HttpRequest) -> Option<RequestHandler> {
        // TODO: write router matching
        None
    }
}

#[macro_export]
macro_rules! router {
    (
        $(
            $route:literal => $func:expr,
        )+
    ) => {
        ::tinyhttp::routing::Router {
            routes: [
                $(
                    ($route, $func),
                    )+
            ]
        }
    }
}

pub(crate) async fn empty_404<'a, 'b, 'c>(
    _reader: RequestReader<'a, 'b, 'c>,
    writer: ResponseWriter<'a, 'b>,
) -> Result<HttpResponse, Error> {
    writer
        .start(StatusCode::NOT_FOUND)
        .await?
        .body_empty()
        .await
}
