use std::convert::From;
use std::fmt;
use std::io;

pub enum Error {
    IoError(io::Error),
    InputReadByteError,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;
        match self {
            IoError(err) => write!(f, "{}", err),
            InputReadByteError => write!(f, "input read byte error"),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IoError(err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
