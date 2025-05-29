#[macro_export]
macro_rules! router {
    (
        $(
            $route:literal => $func:ident,
        )+
    ) => {
        {
        async fn routerfn<'a, 'b, 'c>(reader: $crate::reader::RequestReader<'a, 'b, 'c>,
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

                        writer
                        .start($crate::status::StatusCode::NOT_FOUND)
                        .await?
                        .body_empty()
                        .await
                    }
                }
            }

        routerfn
    }
    };
}
