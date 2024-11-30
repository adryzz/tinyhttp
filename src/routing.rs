use crate::error::Error;
use crate::reader::RequestReader;
use crate::status::StatusCode;
use crate::writer::{HttpResponse, ResponseWriter};

#[macro_export]
macro_rules! router {
    (
        $(
            $route:literal => $func:ident,
        )+
    ) => {{
        |reader, writer| async { Err(::tinyhttp::error::Error::RouterNotFound) }
    }};
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
