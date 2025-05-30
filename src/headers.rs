use unicase::UniCase;

const HOST: UniCase<&str> = UniCase::ascii("Host");
const ACCEPT: UniCase<&str> = UniCase::ascii("Accept");
const ACCEPT_ENCODING: UniCase<&str> = UniCase::ascii("Accept-Encoding");
const AUTHORIZATION: UniCase<&str> = UniCase::ascii("Authorization");
const CONNECTION: UniCase<&str> = UniCase::ascii("Connection");
const CONTENT_ENCODING: UniCase<&str> = UniCase::ascii("Content-Encoding");
const CONTENT_LENGTH: UniCase<&str> = UniCase::ascii("Content-Length");
const CONTENT_TYPE: UniCase<&str> = UniCase::ascii("Content-Type");
const COOKIE: UniCase<&str> = UniCase::ascii("Cookie");
const DATE: UniCase<&str> = UniCase::ascii("Date");
const RANGE: UniCase<&str> = UniCase::ascii("Range");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestHeader<'a> {
    Host,
    Accept,
    AcceptEncoding,
    Authorization,
    Connection,
    ContentEncoding,
    ContentLength,
    ContentType,
    Cookie,
    Date,
    Range,
    Other(&'a str),
}

impl<'a> RequestHeader<'a> {
    pub fn from_str(s: &'a str) -> RequestHeader<'a> {
        let case = UniCase::ascii(s);

        // TODO: improve this garbage
        if case == HOST {
            Self::Host
        } else if case == ACCEPT {
            Self::Accept
        } else if case == ACCEPT_ENCODING {
            Self::AcceptEncoding
        } else if case == AUTHORIZATION {
            Self::Authorization
        } else if case == CONNECTION {
            Self::Connection
        } else if case == CONTENT_ENCODING {
            Self::ContentEncoding
        } else if case == CONTENT_LENGTH {
            Self::ContentLength
        } else if case == CONTENT_TYPE {
            Self::ContentType
        } else if case == COOKIE {
            Self::Cookie
        } else if case == DATE {
            Self::Date
        } else if case == RANGE {
            Self::Range
        } else {
            Self::Other(s)
        }
    }
}
