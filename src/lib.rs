#![no_std]

pub mod config;
pub mod error;
mod headers;
pub mod reader;
mod request;
pub mod routing;
pub mod status;
mod utils;
pub mod writer;

use config::HttpConfig;
use embassy_net::tcp::TcpSocket;
use error::Error;
use reader::HttpReader;
use routing::Router;
use status::StatusCode;
use writer::ResponseWriter;

#[cfg(not(any(feature = "ipv4", feature = "ipv6")))]
compile_error!("You must select at least one of the following features: 'ipv4', 'ipv6'");

pub struct HttpServer<'a, const TX: usize, const RX: usize, const ROUTES: usize> {
    network_stack: embassy_net::Stack<'a>,
    config: &'a HttpConfig<'a>,
    router: &'a Router<ROUTES>,
}

impl<'a, const TX: usize, const RX: usize, const ROUTES: usize> HttpServer<'a, TX, RX, ROUTES> {
    pub fn new(
        network_stack: embassy_net::Stack<'static>,
        config: &'a HttpConfig,
        router: &'a Router<ROUTES>,
    ) -> Self {
        // TODO: add router
        Self {
            network_stack,
            config,
            router,
        }
    }

    pub async fn run(&mut self) {
        let mut tx_buf = [0u8; TX];
        let mut rx_buf = [0u8; RX];

        loop {
            let mut socket = TcpSocket::new(self.network_stack, &mut rx_buf, &mut tx_buf);

            // set the timeout to the configured value, or if none, set it to the default, and then handle closing the socket separately
            socket.set_timeout(Some(embassy_time::Duration::from_secs(
                self.config.keepalive.unwrap_or(5) as u64,
            )));

            if let Err(_) = socket.accept(self.config.port).await {
                #[cfg(feature = "defmt")]
                defmt::debug!("Error while accepting socket");

                continue;
            }

            loop {
                let (mut reader, mut writer) = socket.split();
                // wait for HTTP request
                let reader = match HttpReader::try_new(&mut reader).await {
                    Ok(r) => r,
                    Err(Error::Tcp(e)) => {
                        #[cfg(feature = "defmt")]
                        defmt::debug!("TCP error while parsing HTTP request.");

                        socket.abort();
                        _ = socket.flush().await;
                        break;
                    }
                    _ => {
                        #[cfg(feature = "defmt")]
                        defmt::debug!("Error while parsing HTTP request.");

                        socket.abort();
                        _ = socket.flush().await;
                        break;
                    }
                };
                // create writer so the handler can write out an HTTP response
                let writer = ResponseWriter::new(&mut writer, reader.request.version());
                // if a handler exists for this request, use it, otherwise send a 404
                let result = if let Some(handler) = self.router.handler(reader.request) {
                    handler(reader, writer)
                } else {
                    // if there's a static 404 page, use it.
                    // otherwise just send an empty 404
                    if let Some(page) = self.config.http_404 {
                        writer.static_page(page, StatusCode::NOT_FOUND).await
                    } else {
                        routing::empty_404(reader, writer).await
                    }
                };
                // TODO: flush TCP socket

                match result {
                    Ok(_) => {
                        // TODO: handle connection keepalive if enabled
                    }
                    Err(Error::Tcp(e)) => {
                        #[cfg(feature = "defmt")]
                        defmt::debug!("TCP error while sending HTTP response.");

                        socket.abort();
                        _ = socket.flush().await;
                        break;
                    }
                    _ => {
                        #[cfg(feature = "defmt")]
                        defmt::debug!("Error while handling HTTP request.");

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
