#![no_std]
#![feature(async_fn_traits)]
#![feature(try_find)]
pub mod config;
pub mod error;
mod headers;
pub mod reader;
mod request;
pub mod routing;
pub mod status;
mod utils;
pub mod writer;

#[doc(hidden)]
pub mod logging;

use core::ops::AsyncFn;

use config::HttpConfig;
use embassy_net::tcp::TcpSocket;
use error::Error;
use reader::{HttpReader, RequestReader};
use status::StatusCode;
use writer::{HttpResponse, ResponseWriter};

#[cfg(not(any(feature = "ipv4", feature = "ipv6")))]
compile_error!("You must select at least one of the following features: 'ipv4', 'ipv6'");

/// HTTP server without any routes associated with it
pub struct HttpServer<'a> {
    network_stack: embassy_net::Stack<'a>,
    config: &'a HttpConfig<'a>,
}

pub struct RoutableHttpServer<'a, F>
where
    F: for<'c, 'd, 'e> AsyncFn(
        &'c HttpConfig<'d>,
        RequestReader<'c, 'd, 'e>,
        ResponseWriter<'c, 'd>,
    ) -> Result<HttpResponse, Error>,
{
    network_stack: embassy_net::Stack<'a>,
    config: &'a HttpConfig<'a>,
    router: F,
}

impl<'a> HttpServer<'a> {
    pub fn new(network_stack: embassy_net::Stack<'static>, config: &'a HttpConfig) -> Self {
        Self {
            network_stack,
            config,
        }
    }

    /// Adds routing information to this [HttpServer]
    ///
    /// Use the [router!] macro to specify your routes
    pub fn route<F>(self, f: F) -> RoutableHttpServer<'a, F>
    where
        F: for<'c, 'd, 'e> AsyncFn(
            &'c HttpConfig<'d>,
            RequestReader<'c, 'd, 'e>,
            ResponseWriter<'c, 'd>,
        ) -> Result<HttpResponse, Error>,
    {
        RoutableHttpServer {
            network_stack: self.network_stack,
            config: self.config,
            router: f,
        }
    }
}

impl<'a, F>
    RoutableHttpServer<'a, F>
where
    F: for<'c, 'd, 'e> AsyncFn(
        &'c HttpConfig<'d>,
        RequestReader<'c, 'd, 'e>,
        ResponseWriter<'c, 'd>,
    ) -> Result<HttpResponse, Error>,
{

    /// Runs the HTTP server.
    /// Recommended buffer sizes:
    /// 
    /// `tx_buf` >=`1024`
    /// 
    /// `rx_buf` >=`1024`
    /// 
    /// `http_buf` >=`2048`
    pub async fn run(&mut self, tx_buf: &mut [u8], rx_buf: &mut [u8], http_buf: &mut [u8]) {

        loop {
            let mut socket = TcpSocket::new(self.network_stack, rx_buf, tx_buf);

            // set the timeout to the configured value, or if none, set it to the default, and then handle closing the socket separately
            socket.set_timeout(Some(embassy_time::Duration::from_secs(
                self.config.keepalive.unwrap_or(5) as u64,
            )));

            if (socket.accept(self.config.port).await).is_err() {
                log!(error, "Error while accepting socket");

                continue;
            }


            loop {
                let (mut reader, mut writer) = socket.split();
                // wait for HTTP request
                let reader = match HttpReader::try_new(&mut reader, http_buf).await {
                    Ok(r) => r,
                    Err(Error::Tcp(_)) => {
                        log!(error, "TCP error while parsing HTTP request.");

                        socket.abort();
                        _ = socket.flush().await;
                        break;
                    }
                    Err(Error::EOF) => {
                        socket.close();
                        _ = socket.flush().await;
                        break;
                    }
                    _ => {
                        log!(debug, "Error while parsing HTTP request, sending HTTP 400.");

                        // send 400
                        let writer = ResponseWriter::new_http_11(&mut writer);

                        let _ = writer
                            .static_page_or_empty(self.config.http_400, StatusCode::BAD_REQUEST)
                            .await;

                        socket.close();
                        _ = socket.flush().await;
                        break;
                    }
                };
                // create writer so the handler can write out an HTTP response
                let writer = ResponseWriter::new(&mut writer, &reader);

                // if global http basic auth is enabled, check for authentication
                // if not, this is always true at compile time
                let result = if routing::global_basic_auth!(self.config, reader) {
                    // if a handler exists for this request, use it, otherwise send a 404
                    self.router.async_call((self.config, reader, writer)).await
                } else {
                    log!(
                        debug,
                        "Asking for authentication to access page {}",
                        reader.request.path()
                    );
                    writer::static_or_empty_page!(
                        writer,
                        self.config.http_401,
                        StatusCode::UNAUTHORIZED,
                        ("WWW-Authenticate", "Basic")
                    )
                };

                // flush and map the error
                let result = match result {
                    Ok(r) => socket.flush().await.map(|_| r).map_err(|e| e.into()),
                    Err(e) => Err(e),
                };

                match result {
                    Ok(_) => {
                        // TODO: handle connection keepalive if enabled
                    }
                    Err(Error::Tcp(_)) => {
                        log!(error, "TCP error while sending HTTP response.");

                        socket.abort();
                        _ = socket.flush().await;
                        break;
                    }
                    _ => {
                        log!(error, "Error while handling HTTP request.");

                        // TODO: instead of sending RST, see if we can send other HTTP error codes

                        socket.abort();
                        _ = socket.flush().await;
                        break;
                    }
                }
            }
        }
    }
}
