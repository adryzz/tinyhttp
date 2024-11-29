use core::marker::PhantomData;

use crate::error::Error;
use crate::reader::RequestReader;
use crate::request::HttpRequest;
use crate::status::StatusCode;
use crate::writer::{HttpResponse, ResponseWriter};

type RequestHandler = fn(RequestReader, ResponseWriter) -> Result<HttpResponse, Error>;

pub struct Router<'a, T, const ROUTES: usize> where T : StaticDispatchRouter<'a, 'a, 'a> {
    pub routes: [(&'static str, T); ROUTES],
    pub marker: &'a PhantomData<T>
}

impl<'a, T, const ROUTES: usize> Router<'a, T, ROUTES> where T : StaticDispatchRouter<'a, 'a, 'a> {
    /// Checks if the request can be handled by any of the request handlers, and if so, returns it.
    ///
    /// Returns None when no request handler can handle the request (HTTP 404)
    pub fn handler(&self, request: &HttpRequest) -> Option<RequestHandler> {
        // TODO: write router matching
        None
    }
}

pub trait StaticDispatchRouter<'a, 'b, 'c> {
    async fn run(&self, reader: RequestReader<'a, 'b, 'c>, writer: ResponseWriter<'a, 'b>) -> Result<HttpResponse, Error>;
}

#[macro_export]
macro_rules! router {
    (
        $(
            $route:literal => $func:ident,
        )+
    ) => {
        {
            #[derive(Debug, Copy, Clone)]
            #[allow(non_camel_case_types)]
            /// Placeholder enum allowing for async static dispatch
            /// 
            /// Implementation details
            /// 
            /// Contains one variant per function
            enum Func {
                $(
                    $func,
                    )+
            }
    
            impl<'a, 'b, 'c> ::tinyhttp::routing::StaticDispatchRouter<'a, 'b, 'c> for Func {
                async fn run(&self,
                    reader: ::tinyhttp::reader::RequestReader<'a, 'b, 'c>,
                    writer: ::tinyhttp::writer::ResponseWriter<'a, 'b>,
                ) -> Result<::tinyhttp::writer::HttpResponse, ::tinyhttp::error::Error> {
                    match self {
                        $(
                            Func::$func => $func(reader, writer).await,
                            )+
                    }
                }
            }
            
        
        ::tinyhttp::routing::Router {
            routes: [
                $(
                    ($route, Func::$func),
                    )+
            ],
            marker: &::core::marker::PhantomData
        }
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
