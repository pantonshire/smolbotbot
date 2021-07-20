use std::error;
use std::fmt;

/// Contains information about why a tweet was not successfully processed by sbb_scribe.
#[derive(Debug)]
pub(crate) enum NotScribed {
    /// Case when the robot tweet was malformed in some way.
    InvalidTweet(InvalidTweet),
    /// Case when a serious unexpected error occurred.
    ScribeFailure(ScribeFailure),
}

impl fmt::Display for NotScribed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidTweet(err) => err.fmt(f),
            Self::ScribeFailure(err) => err.fmt(f),
        }
    }
}

impl error::Error for NotScribed {}

impl From<goldcrest::error::RequestError> for NotScribed {
    fn from(err: goldcrest::error::RequestError) -> Self {
        Self::ScribeFailure(err.into())
    }
}

impl From<sqlx::Error> for NotScribed {
    fn from(err: sqlx::Error) -> Self {
        Self::ScribeFailure(err.into())
    }
}

impl From<tokio::task::JoinError> for NotScribed {
    fn from(err: tokio::task::JoinError) -> Self {
        Self::ScribeFailure(err.into())
    }
}

#[derive(Debug)]
pub(crate) enum InvalidTweet {
    ParseUnsuccessful,
    MissingMedia,
    DuplicateTweetId(u64),
    DuplicateRobot(i32, String),
}

impl fmt::Display for InvalidTweet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ParseUnsuccessful => write!(f, "could not parse robot data from tweet"),
            Self::MissingMedia => write!(f, "tweet does not contain media"),
            Self::DuplicateTweetId(id) => write!(f, "tweet id {} already exists in database", id),
            Self::DuplicateRobot(number, ident) => write!(f, "robot ({}, {}) already exists", number, ident),
        }
    }
}

impl error::Error for InvalidTweet {}

impl From<InvalidTweet> for NotScribed {
    fn from(err: InvalidTweet) -> Self {
        Self::InvalidTweet(err)
    }
}

#[derive(Debug)]
pub(crate) enum ScribeFailure {
    TwitterError(Box<goldcrest::error::RequestError>),
    DbError(Box<sqlx::Error>),
    JoinError(Box<tokio::task::JoinError>),
}

impl fmt::Display for ScribeFailure {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::TwitterError(err) => err.fmt(f),
            Self::DbError(err) => err.fmt(f),
            Self::JoinError(err) => err.fmt(f),
        }
    }
}

impl error::Error for ScribeFailure {}

impl From<ScribeFailure> for NotScribed {
    fn from(err: ScribeFailure) -> Self {
        Self::ScribeFailure(err)
    }
}

impl From<goldcrest::error::RequestError> for ScribeFailure {
    fn from(err: goldcrest::error::RequestError) -> Self {
        Self::TwitterError(Box::new(err))
    }
}

impl From<sqlx::Error> for ScribeFailure {
    fn from(err: sqlx::Error) -> Self {
        Self::DbError(Box::new(err))
    }
}

impl From<tokio::task::JoinError> for ScribeFailure {
    fn from(err: tokio::task::JoinError) -> Self {
        Self::JoinError(Box::new(err))
    }
}
