use std::error::Error;

pub(super) type ScribeResult<T> = std::result::Result<T, ScribeError>;

#[derive(Debug)]
pub(super) enum ScribeError {
    TweetGetFailure,
    TweetAlreadyExists,
    RobotAlreadyExists,
    DbError(diesel::result::Error),
    JoinError(tokio::task::JoinError),
}

impl std::fmt::Display for ScribeError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ScribeError::TweetGetFailure    => write!(fmt, "Failed to get tweet"),
            ScribeError::TweetAlreadyExists => write!(fmt, "Tweet already exists in database"),
            ScribeError::RobotAlreadyExists => write!(fmt, "Robot already exists in database"),
            ScribeError::DbError(err)       => err.fmt(fmt),
            ScribeError::JoinError(err)     => err.fmt(fmt),
        }
    }
}

impl Error for ScribeError {}

impl From<diesel::result::Error> for ScribeError {
    fn from(err: diesel::result::Error) -> Self {
        ScribeError::DbError(err)
    }
}

impl From<tokio::task::JoinError> for ScribeError {
    fn from(err: tokio::task::JoinError) -> Self {
        ScribeError::JoinError(err)
    }
}
