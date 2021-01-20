use std::error::Error;
use std::fmt::{Display, Formatter, Result};

#[derive(Clone, Debug)]
pub enum ScribeError {
    TweetGetFailure,
}

impl Display for ScribeError {
    fn fmt(&self, fmt: &mut Formatter) -> Result {
        write!(fmt, "Scribe error: {}", match self {
            ScribeError::TweetGetFailure => "Tweet get failure",
        })
    }
}

impl Error for ScribeError {}
