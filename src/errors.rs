pub mod database {
    use std::io;
    use std::num::{ParseFloatError, ParseIntError};
    use thiserror::Error;

    #[derive(Debug, Error)]
    pub enum LAMDAError {
        #[error("IO error: {0}")]
        Io(#[from] io::Error),

        #[error("Parse error: {0}")]
        ParseError(String),

        #[error("Integer parse error: {0}")]
        ParseInt(#[from] ParseIntError),

        #[error("Float parse error: {0}")]
        ParseFloat(#[from] ParseFloatError),
    }
}

pub mod radio {
    use thiserror::Error;

    #[derive(Debug, Error, PartialEq)]
    pub enum BeamError {
        #[error("Can only specify one of {{major, minor, pa}} and {{area}}.")]
        ExclusiveParameterConflict,

        #[error("Area unit should be equivalent to steradian (solid angle).")]
        InvalidAreaUnit,

        #[error("Missing parameter.")]
        MissingParameter,

        #[error("Minor axis greater than major axis")]
        MinorGreaterThanMajor,
    }
}
