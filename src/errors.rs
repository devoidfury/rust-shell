use std::error;
use std::fmt;
use std::io;


#[derive(Debug)]
pub enum RashError {
	Io(io::Error),
	Eof,
}

impl fmt::Display for RashError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RashError::Io(ref err) => err.fmt(f),
            RashError::Eof => write!(f, "EOF"),
        }
    }
}

impl error::Error for RashError {
    fn description(&self) -> &str {
        match *self {
            RashError::Io(ref err) => err.description(),
            RashError::Eof => "EOF",
        }
    }
}

impl From<io::Error> for RashError {
    fn from(err: io::Error) -> RashError { RashError::Io(err) }
}
