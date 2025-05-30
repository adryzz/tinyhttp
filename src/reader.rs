use embassy_futures::select::select;
use embassy_net::tcp::TcpReader;
use embassy_time::Timer;

use crate::{error::Error, request::HttpRequest, utils};

/// Used to read HTTP requests.
///
/// Uses typestate to make it impossible to misuse.
pub struct HttpReader<'a, 'b, 'c> {
    socket: &'a mut TcpReader<'b>,
    pub request: HttpRequest<'c>,
}

impl<'a, 'b, 'c> HttpReader<'a, 'b, 'c> {
    pub(crate) async fn try_new(
        socket: &'a mut TcpReader<'b>,
        buf: &'c mut [u8],
    ) -> Result<Self, Error> {
        // read from the buffer until either the first newline is found, we run out of data,
        // or if we can't find it in the buffer and there's more to read, send a bad request error.

        let mut total = 0usize;
        let mut body_inline = false;

        loop {
            if buf.len() == total {
                // our buffer is full
                break;
            }
            let rbuf = &mut buf[total..];

            // bad idea
            let count = match select(socket.read(rbuf), Timer::after_millis(3)).await {
                embassy_futures::select::Either::First(c) => c?,
                embassy_futures::select::Either::Second(_) => break,
            };
            if count == 0 {
                // we ran out of data
                // if the request has a body, it should be within our buffer
                body_inline = true;
                break;
            }
            total += count;
        }

        if total == 0 {
            return Err(Error::EOF);
        }

        // replace with actual minimum
        if total < 15 {
            return Err(Error::BadRequest);
        }
        let buf = &buf[0..total];

        let request = utils::parse(buf)?;
        Ok(Self { socket, request })
    }

    pub fn body(self) -> Option<HttpBodyReader<'a, 'b>> {
        if let Some(len) = self.request.body_len() {
            Some(HttpBodyReader::new(self.socket, len))
        } else {
            None
        }
    }
}

pub type RequestReader<'a, 'b, 'c> = HttpReader<'a, 'b, 'c>;

/// Used to read HTTP response bodies.
///
/// Uses typestate to make it impossible to misuse.
pub struct HttpBodyReader<'a, 'b> {
    socket: &'a mut TcpReader<'b>,
    len: usize,
    read: usize,
}

impl<'a, 'b> HttpBodyReader<'a, 'b> {
    fn new(socket: &'a mut TcpReader<'b>, len: usize) -> Self {
        Self {
            socket,
            len,
            read: 0,
        }
    }

    pub async fn try_read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        if self.read == self.len {
            return Ok(0);
        }
        let read = self.socket.read(buf).await?;
        if read == 0 {
            return Err(Error::EOF);
        }
        self.read += read;

        Ok(read)
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn read(&self) -> usize {
        self.read
    }
}

/*
impl<'a, 'b, 'c> Drop for HttpReader<'a, 'b, 'c> {
    fn drop(&mut self) {
        // TODO: if a HTTP body does exist, read (and discard) it from the TCP buffer
    }
}

impl<'a, 'b> Drop for HttpBodyReader<'a, 'b> {
    fn drop(&mut self) {
        // TODO: if the HTTP body hasn't been read to completion, read (and discard)
        // the rest of it from the TCP buffer.
    }
}
*/
