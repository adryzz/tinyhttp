#[macro_export]
macro_rules! router {
    (
        $(
            $route:literal => $func:ident,
        )+
    ) => {
        {
        async fn routerfn<'a, 'b, 'c>(config: &'a $crate::config::HttpConfig<'b>, reader: $crate::reader::RequestReader<'a, 'b, 'c>,
         writer: $crate::writer::ResponseWriter<'a, 'b>) -> Result<$crate::writer::HttpResponse, $crate::error::Error> {
            match reader.request.path() {
                $(
                    $route => {
                        $crate::log!(debug, "Routing page '{}' to {}", reader.request.path(), stringify!($func));

                        $func(reader, writer).await
                    },
                    )+
                _ => {
                        $crate::log!(debug, "Routing page '{}' to 404", reader.request.path());

                        // handle 404s

                        if let Some(page) = config.http_404 {
                            writer
                            .static_page(page, $crate::status::StatusCode::NOT_FOUND)
                            .await
                        } else {
                            writer
                            .start($crate::status::StatusCode::NOT_FOUND)
                            .await?
                            .body_empty()
                            .await
                        }
                    }
                }
            }

        routerfn
    }
    };
}
