use std::fmt;
use std::io;
use std::num::ParseFloatError;

#[derive(Debug)]
pub enum LAMDAError {
    Io(io::Error),
    ParseError(String),
    ParseInt(std::num::ParseIntError),
    ParseFloat(ParseFloatError),
}

impl From<io::Error> for LAMDAError {
    fn from(e: io::Error) -> Self {
        LAMDAError::Io(e)
    }
}

impl From<std::num::ParseIntError> for LAMDAError {
    fn from(e: std::num::ParseIntError) -> Self {
        LAMDAError::ParseInt(e)
    }
}

impl From<ParseFloatError> for LAMDAError {
    fn from(e: ParseFloatError) -> Self {
        LAMDAError::ParseFloat(e)
    }
}

impl fmt::Display for LAMDAError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LAMDAError::Io(e) => write!(f, "IO error: {}", e),
            LAMDAError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            LAMDAError::ParseInt(e) => write!(f, "Integer parse error: {}", e),
            LAMDAError::ParseFloat(e) => write!(f, "Float parse error: {}", e),
        }
    }
}

impl std::error::Error for LAMDAError {}
