use std::error;
use std::ffi::OsString;
use std::fmt;

#[derive(Debug)]
pub(crate) struct InvalidVarError {
    pub(crate) val: OsString,
    pub(crate) reason: InvalidVarReason,
}

#[derive(Debug)]
pub(crate) enum InvalidVarReason {
    InvalidUtf8,
    ParseError,
}

impl InvalidVarError {
    pub(crate) const fn invalid_utf8(val: OsString) -> Self {
        Self {
            val,
            reason: InvalidVarReason::InvalidUtf8,
        }
    }

    pub(crate) const fn parse_error(val: OsString) -> Self {
        Self {
            val,
            reason: InvalidVarReason::ParseError,
        }
    }
}

impl fmt::Display for InvalidVarError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.reason {
            InvalidVarReason::InvalidUtf8 => write!(f, "invalid utf8: {:?}", self.val),
            InvalidVarReason::ParseError => write!(f, "value could not be parsed: {:?}", self.val),
        }
    }
}

impl error::Error for InvalidVarError {}

#[derive(Debug)]
pub(crate) struct MissingVarError(pub &'static str);

impl fmt::Display for MissingVarError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} not set", self.0)
    }
}

impl error::Error for MissingVarError {}
