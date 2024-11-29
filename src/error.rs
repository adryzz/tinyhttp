pub enum Error {
    /// TCP error. Can't return any error page
    Tcp(embassy_net::tcp::Error),
    /// HTTP range request out of range. HTTP 416 Range Not Satisfiable
    OutOfRange,
    
}

impl From<embassy_net::tcp::Error> for Error {
    fn from(value: embassy_net::tcp::Error) -> Self {
        Error::Tcp(value)
    }
}