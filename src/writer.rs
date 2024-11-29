use core::marker::PhantomData;

use embassy_net::tcp::TcpSocket;

use crate::{config::StaticPage, error::Error, status::StatusCode, utils, HttpVersion};

/// Used to write HTTP responses.
/// 
/// Uses typestate to make it impossible to misuse.
pub struct HttpWriter<'a, 'b, T> where T :  {
    socket: &'a mut TcpSocket<'b>,
    version: HttpVersion,
    marker: PhantomData<T>
}

/// Http response
/// 
/// Internally, it's just a marker to make sure that every HTTP handler function has a response.
pub struct HttpResponse {
    _marker: ()
}

pub enum Start {}
pub enum Headers {}

pub trait HttpStage {}
impl HttpStage for Start {}
impl HttpStage for Headers {}

impl<'a, 'b> HttpWriter<'a, 'b, Start> {
    pub(crate) fn new(socket: &'a mut TcpSocket<'b>, version: HttpVersion) -> HttpWriter<'a, 'b, Start> {
        HttpWriter { socket, version, marker: PhantomData }
    }

    pub async fn start(mut self, code: StatusCode) -> Result<HttpWriter<'a, 'b, Headers>, Error> {
        match self.version {
            HttpVersion::Http10 => self.write_bytes(b"HTTP/1.0 ").await?,
            HttpVersion::Http11 => self.write_bytes(b"HTTP/1.1 ").await?,
        }
        self.write_bytes(code.as_str().unwrap().as_bytes()).await?;
        self.write_bytes(b"\r\n").await?;

        Ok(HttpWriter { socket: self.socket, version: self.version, marker: PhantomData })
    }

    pub async fn static_page(self, page: StaticPage<'a>, code: StatusCode) -> Result<HttpResponse, Error> {
        self.start(code).await?.body_static_page(page).await
    }
}

impl<'a, 'b, T> HttpWriter<'a, 'b, T> where T : HttpStage {
    async fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), Error> {
        #[cfg(feature = "defmt")]
        if bytes.len() > self.socket.send_capacity() {
            defmt::trace!("Write bigger than buffer size!");
        }

        let mut total = 0;
        while total < bytes.len() {
            let written = self.socket.write(bytes).await?;
            total += written;
        }

        Ok(())
    }
    // TODO: streamed writing
    // TODO: buffered writing
}

impl<'a, 'b> HttpWriter<'a, 'b, Headers> {
    pub async fn header(mut self, name: &str, value: &str) -> Result<Self, Error> {
        self.write_bytes(name.as_bytes()).await?;
        self.write_bytes(b": ").await?;
        self.write_bytes(value.as_bytes()).await?;
        self.write_bytes(b"\r\n").await?;

        Ok(self)
    }

    pub async fn body_str(self, body: &str, content_type: &str) -> Result<HttpResponse, Error> {
        self.body_bytes(body.as_bytes(), content_type).await
    }

    pub async fn body_static_page(self, page: StaticPage<'a>) -> Result<HttpResponse, Error> {
        self.body_str(&page.body, &page.content_type).await
    }

    pub async fn body_bytes(mut self, body: &[u8], content_type: &str) -> Result<HttpResponse, Error> {
        let mut buf = utils::USizeStrBuf::new();
        self = self.header("Content-Type", content_type).await?
        .header("Content-Length", buf.stringify(body.len())).await?;

        // send newline to go to body section
        self.write_bytes(b"\r\n").await?;
        self.write_bytes(body).await?;

        Ok(HttpResponse { _marker: () })
    }
}