#![no_std]

use config::{HttpConfig, StaticPage};
use embassy_net::tcp::TcpSocket;
use error::Error;
use managed::ManagedMap;
mod config;
mod headers;
mod status;
mod utils;
mod writer;
mod reader;
mod request;
pub mod error;
use headers::RequestHeader;
use writer::{HttpResponse, HttpWriter, ResponseWriter};

#[cfg(not(any(feature = "ipv4", feature = "ipv6")))]
compile_error!("You must select at least one of the following features: 'ipv4', 'ipv6'");

pub struct HttpServer<'a, const TX: usize, const RX: usize> {
    network_stack: embassy_net::Stack<'a>,
    config: &'a HttpConfig<'a>,
}

impl<'a, const TX: usize, const RX: usize> HttpServer<'a, TX, RX> {
    pub fn new(network_stack: embassy_net::Stack<'static>, config: &'a HttpConfig) -> Self {
        // TODO: add router
        Self {
            network_stack,
            config,
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
                // TODO:
                // wait for HTTP request
                // route request
                // handle request
                // send response
                // wait on socket for new HTTP request or close it
            }
        }
    }
}

struct RequestReader;

pub struct Router<const N: usize> {
    routes: [(&'static str, fn(RequestReader, ResponseWriter) -> Result<HttpResponse, Error>); N]
}