use core::marker::PhantomData;

use embassy_net::tcp::TcpWriter;

use crate::{
    config::StaticPage, error::Error, reader::RequestReader, request::HttpVersion,
    status::StatusCode, utils,
};

/// Used to write HTTP responses.
///
/// Uses typestate to make it impossible to misuse.
pub struct HttpWriter<'a, 'b, T>
where
    T:,
{
    socket: &'a mut TcpWriter<'b>,
    version: HttpVersion,
    marker: PhantomData<T>,
}

pub type ResponseWriter<'a, 'b> = HttpWriter<'a, 'b, Start>;

/// Http response
///
/// Internally, it's just a marker to make sure that every HTTP handler function has a response.
pub struct HttpResponse {
    _marker: (),
}

pub enum Start {}
pub enum Headers {}

pub trait HttpStage {}
impl HttpStage for Start {}
impl HttpStage for Headers {}

async fn write_bytes<'a, 'b>(socket: &'a mut TcpWriter<'b>, bytes: &[u8]) -> Result<(), Error> {
    #[cfg(feature = "defmt")]
    if bytes.len() > self.socket.send_capacity() {
        defmt::trace!("Write bigger than buffer size!");
    }

    let mut total = 0;
    while total < bytes.len() {
        let written = socket.write(bytes).await?;
        total += written;
    }

    Ok(())
}

impl<'a, 'b> HttpWriter<'a, 'b, Start> {
    pub(crate) fn new(
        socket: &'a mut TcpWriter<'b>,
        reader: &RequestReader,
    ) -> HttpWriter<'a, 'b, Start> {
        HttpWriter {
            socket,
            version: reader.request.version(),
            marker: PhantomData,
        }
    }

    pub async fn start(self, code: StatusCode) -> Result<HttpWriter<'a, 'b, Headers>, Error> {
        match self.version {
            HttpVersion::Http10 => write_bytes(self.socket, b"HTTP/1.0 ").await?,
            HttpVersion::Http11 => write_bytes(self.socket, b"HTTP/1.1 ").await?,
        }
        write_bytes(self.socket, code.as_str().unwrap().as_bytes()).await?;
        write_bytes(self.socket, b"\r\n").await?;

        Ok(HttpWriter {
            socket: self.socket,
            version: self.version,
            marker: PhantomData,
        })
    }

    pub async fn static_page(
        self,
        page: StaticPage<'a>,
        code: StatusCode,
    ) -> Result<HttpResponse, Error> {
        self.start(code).await?.body_static_page(page).await
    }
}

impl<'a, 'b> HttpWriter<'a, 'b, Headers> {
    pub async fn header(self, name: &str, value: &str) -> Result<Self, Error> {
        write_bytes(self.socket, name.as_bytes()).await?;
        write_bytes(self.socket, b": ").await?;
        write_bytes(self.socket, value.as_bytes()).await?;
        write_bytes(self.socket, b"\r\n").await?;

        Ok(self)
    }

    pub async fn body_empty(self) -> Result<HttpResponse, Error> {
        write_bytes(self.socket, b"\r\n").await?;

        Ok(HttpResponse { _marker: () })
    }

    pub async fn body_str(self, body: &str, content_type: &str) -> Result<HttpResponse, Error> {
        self.body_bytes(body.as_bytes(), content_type).await
    }

    pub async fn body_static_page(self, page: StaticPage<'a>) -> Result<HttpResponse, Error> {
        self.body_str(&page.body, &page.content_type).await
    }

    pub async fn body_bytes(
        mut self,
        body: &[u8],
        content_type: &str,
    ) -> Result<HttpResponse, Error> {
        let mut buf = utils::USizeStrBuf::new();
        self = self
            .header("Content-Type", content_type)
            .await?
            .header("Content-Length", buf.stringify(body.len()))
            .await?;

        // send newline to go to body section
        write_bytes(self.socket, b"\r\n").await?;
        write_bytes(self.socket, body).await?;

        Ok(HttpResponse { _marker: () })
    }

    pub async fn body_chunked(
        mut self,
        length: usize,
        content_type: &str,
    ) -> Result<ChunkedHttpWriter<'a, 'b>, Error> {
        let mut buf = utils::USizeStrBuf::new();
        self = self
            .header("Content-Type", content_type)
            .await?
            .header("Content-Length", buf.stringify(length))
            .await?;

        // send newline to go to body section
        write_bytes(self.socket, b"\r\n").await?;

        Ok(ChunkedHttpWriter {
            socket: self.socket,
            total: length,
            written: 0,
        })
    }
}

pub struct ChunkedHttpWriter<'a, 'b> {
    socket: &'a mut TcpWriter<'b>,
    total: usize,
    written: usize,
}

impl<'a, 'b> ChunkedHttpWriter<'a, 'b> {
    pub async fn write_chunk(&mut self, chunk: &[u8]) -> Result<Option<HttpResponse>, Error> {
        if self.written == self.total {
            return Ok(Some(HttpResponse { _marker: () }));
        }
        write_bytes(self.socket, chunk).await?;
        self.written += chunk.len();
        // TODO: implement Drop to send a RST packet here
        // is it needed?

        Ok(None)
    }

    pub async fn write_chunk_str(&mut self, chunk: &str) -> Result<Option<HttpResponse>, Error> {
        self.write_chunk(chunk.as_bytes()).await
    }

    pub fn total(&self) -> usize {
        self.total
    }

    pub fn written(&self) -> usize {
        self.written
    }
}
