#![no_std]

use config::HttpConfig;
use embassy_net::tcp::TcpSocket;
use managed::ManagedMap;
mod config;
mod headers;
mod status;
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

pub struct HttpServer<'a, const TX: usize, const RX: usize> {
    network_stack: embassy_net::Stack<'a>,
    config: &'a HttpConfig<'a>,

}

impl<'a, const TX: usize, const RX: usize> HttpServer<'a, TX, RX> {
    pub fn new(network_stack: embassy_net::Stack<'static>, config: &'a HttpConfig) -> Self {
        // TODO: add router
        Self { network_stack, config }
    }

    pub async fn run(&mut self) {
        let mut tx_buf = [0u8; TX];
        let mut rx_buf = [0u8; RX];

        loop {
            let mut socket = TcpSocket::new(self.network_stack, &mut rx_buf, &mut tx_buf);

            // set the timeout to the configured value, or if none, set it to the default, and then handle closing the socket separately
            socket.set_timeout(Some(embassy_time::Duration::from_secs(self.config.keepalive.unwrap_or(5) as u64)));

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