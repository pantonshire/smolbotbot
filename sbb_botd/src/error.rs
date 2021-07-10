use std::error::Error;
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub(super) enum BotdError {
    GoldcrestError(Box<goldcrest::error::RequestError>),
    DbError(Box<sqlx::Error>),
}

impl Display for BotdError {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match self {
            BotdError::GoldcrestError(err) => err.fmt(fmt),
            BotdError::DbError(err)        => err.fmt(fmt),
        }
    }
}

impl Error for BotdError {}

impl From<goldcrest::error::RequestError> for BotdError {
    fn from(err: goldcrest::error::RequestError) -> Self {
        BotdError::GoldcrestError(Box::new(err))
    }
}

impl From<sqlx::Error> for BotdError {
    fn from(err: sqlx::Error) -> Self {
        BotdError::DbError(Box::new(err))
    }
}