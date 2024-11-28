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
/*
impl<'a> core::str::FromStr for RequestHeader<'a> {
    type Err = core::convert::Infallible;

    fn from_str(s: & str) -> Result<Self, Self::Err> {
        let case = UniCase::ascii(s);


        Ok(match case {
            HOST => Self::Host,
            ACCEPT => Self::Accept,
            ACCEPT_ENCODING => Self::AcceptEncoding,
            AUTHORIZATION => Self::Authorization,
            CONNECTION => Self::Connection,
            CONTENT_ENCODING => Self::ContentEncoding,
            CONTENT_LENGTH => Self::ContentLength,
            CONTENT_TYPE => Self::ContentType,
            COOKIE => Self::Cookie,
            DATE => Self::Date,
            RANGE => Self::Range,
            _ => Self::Other(s)
        })
    }
} */
