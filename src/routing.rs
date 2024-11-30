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
        |reader: ::tinyhttp::reader::HttpReader<'_, '_, '_>,
         writer: ::tinyhttp::writer::HttpWriter<'_, '_, _>,
         http_404: Option<::tinyhttp::config::StaticPage<'_>>| async {
            match reader.request.path() {
                $(
                    $route => $func(reader, writer).await,
                    )+
                _ => {
                    if let Some(page) = http_404 {
                        writer.static_page(page, StatusCode::NOT_FOUND).await
                    } else {
                        writer
                        .start(StatusCode::NOT_FOUND)
                        .await?
                        .body_empty()
                        .await
                    }
                }
            }
        }
    }};
}