use core::marker::PhantomData;

use embassy_net::tcp::TcpReader;

use crate::{error::Error, request::HttpRequest};

/// Used to read HTTP requests.
///
/// Uses typestate to make it impossible to misuse.
pub struct HttpReader<'a, 'b, 'c> {
    socket: &'a mut TcpReader<'b>,
    pub request: &'c HttpRequest<'c>,
}

impl<'a, 'b, 'c> HttpReader<'a, 'b, 'c> {
    pub(crate) async fn try_new(socket: &'a mut TcpReader<'b>) -> Result<Self, Error> {
        todo!()
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

// HTTP request
// splits socket, creates reader and writer
// reads headers and shit into HttpRequest
// stack handles routing
// gives reader and request to handler
// handler does stuff with request/reads body if available
// write status code and headers
// write body if needed
// returns
// reader/writer are dropped, socket is now unsplit automatically
// handles keepalive/closes connection
