pub enum Error {
    Tcp(embassy_net::tcp::Error)
}

impl From<embassy_net::tcp::Error> for Error {
    fn from(value: embassy_net::tcp::Error) -> Self {
        Error::Tcp(value)
    }
}