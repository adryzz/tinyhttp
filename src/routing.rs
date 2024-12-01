#[macro_export]
macro_rules! router {
    (
        $(
            $route:literal => $func:ident,
        )+
    ) => {
        {
        async fn routerfn<'a, 'b, 'c>(reader: ::tinyhttp::reader::RequestReader<'a, 'b, 'c>,
         writer: ::tinyhttp::writer::ResponseWriter<'a, 'b>) -> Result<::tinyhttp::writer::HttpResponse, ::tinyhttp::error::Error> {
            match reader.request.path() {
                $(
                    $route => {
                        $func(reader, writer).await
                    },
                    )+
                _ => {
                        writer
                        .start(StatusCode::NOT_FOUND)
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