#[cfg(feature = "http_basic_auth")]
#[doc(hidden)]
pub use base64;

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
                        writer
                        .static_page_or_empty(
                            config.http_404,
                            $crate::status::StatusCode::NOT_FOUND
                        ).await
                    }
                }
            }

        routerfn
    }
    };
}

#[macro_export]
#[cfg(feature = "http_basic_auth")]
macro_rules! basic_auth {
    ($username:expr, $password:expr, $reader:expr) => {{
        if let Some(head) = $reader
            .request
            .try_find_header(&$crate::headers::RequestHeader::Authorization)
        {
            use base64::Engine;
            // TODO: make sure this doesn't panic
            if let Some(base64) = head.strip_prefix("Basic ") {

                let mut buf = [0u8; 64];
                if let Ok(len) = ::base64::prelude::BASE64_STANDARD.decode_slice(base64, &mut buf) {
                    let buf = &buf[..len];

                    if let Some(colon) = buf.iter().position(|a| *a == b':') {
                        let name = &buf[..colon];
                        let pass = &buf[colon + 1..];

                        if name == $username.as_bytes() && pass == $password.as_bytes() {
                            $crate::log!(
                                debug,
                                "Authentication succeeded for path {}",
                                $reader.request.path()
                            );
                            true
                        } else {
                            $crate::log!(
                                debug,
                                "Authentication failed for path {}",
                                $reader.request.path()
                            );
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        }
    }};
}

#[cfg(feature = "global_http_basic_auth")]
macro_rules! global_basic_auth {
    ($config:expr, $reader:expr) => {
        if let Some((user, pass)) = $config.basic_auth {
            if basic_auth!(user, pass, $reader) {
                true
            } else {
                false
            }
        } else {
            true
        }
    };
}

#[cfg(not(feature = "global_http_basic_auth"))]
macro_rules! global_basic_auth {
    ($config:expr, $reader:expr) => {
        true
    };
}

pub(crate) use global_basic_auth;
