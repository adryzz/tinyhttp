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
        |reader: &mut ::tinyhttp::reader::HttpReader<'_, '_, '_>,
         writer: &mut ::tinyhttp::writer::HttpWriter<'_, '_, _>| async {
            match reader.request.path() {
                $(
                    $route => $func(reader, writer).await,
                    )+
                _ => Err(::tinyhttp::error::Error::RouterNotFound)
            }
        }
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
