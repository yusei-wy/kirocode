use std::{fmt, io};

// Deriving Debug is necesary to use .expect() method
#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    // TODO: Temporary. Remove later.
    UnexpectedError,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;
        match self {
            IoError(err) => write!(f, "{}", err),
            UnexpectedError => write!(f, "{}", "unexpected error"),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IoError(err)
    }
}

// Aliaces result
pub type Result<T> = std::result::Result<T, Error>;
