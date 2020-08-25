use std::convert::From;
use std::fmt;
use std::io;
use std::num;

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    ParseIntError(num::ParseIntError),
    InputReadByteError,
    InputNotFoundEscapeError,
    UnknownWindowSize,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;
        match self {
            IoError(err) => write!(f, "{}", err),
            ParseIntError(err) => write!(f, "{}", err),
            UnknownWindowSize => write!(f, "Could not detect terminal window size"),
            // TODO: いらいないかも
            InputReadByteError => write!(f, "input read byte error"),
            InputNotFoundEscapeError => write!(f, "input not found escape error"),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IoError(err)
    }
}

impl From<num::ParseIntError> for Error {
    fn from(err: num::ParseIntError) -> Error {
        Error::ParseIntError(err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
