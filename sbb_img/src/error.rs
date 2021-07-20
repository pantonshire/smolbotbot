use std::fmt;
use std::error;
use std::io;

use reqwest::StatusCode;

#[derive(Debug)]
pub(crate) struct ImgError {
    pub(crate) group_id: i32,
    pub(crate) cause: ImgErrorCause,
}

#[derive(Debug)]
pub(crate) enum ImgErrorCause {
    RequestError(Box<reqwest::Error>),
    ImageError(Box<image::ImageError>),
    IoError(Box<io::Error>),
    DbError(Box<sqlx::Error>),
    SemaphoreError(Box<tokio::sync::AcquireError>),
    TaskPanicked(Box<tokio::task::JoinError>),
    InvalidUrl(Box<url::ParseError>),
    HttpError(StatusCode),
    InvalidPath,
    NoRowsUpdated,
}

impl ImgError {
    pub(crate) const fn new(group_id: i32, cause: ImgErrorCause) -> Self {
        ImgError {
            group_id,
            cause,
        }
    }
}

impl fmt::Display for ImgError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error at group {}: {}", self.group_id, self.cause)
    }
}

impl fmt::Display for ImgErrorCause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RequestError(err) => err.fmt(f),
            Self::ImageError(err) => err.fmt(f),
            Self::IoError(err) => err.fmt(f),
            Self::DbError(err) => err.fmt(f),
            Self::SemaphoreError(err) => err.fmt(f),
            Self::TaskPanicked(err) => err.fmt(f),
            Self::InvalidUrl(err) => err.fmt(f),
            Self::HttpError(status) => status.fmt(f),
            Self::InvalidPath => write!(f, "path is not valid utf8"),
            Self::NoRowsUpdated => write!(f, "no rows affected by update"),
        }
    }
}

impl error::Error for ImgError {}

impl From<reqwest::Error> for ImgErrorCause {
    fn from(err: reqwest::Error) -> Self {
        Self::RequestError(Box::new(err))
    }
}

impl From<image::ImageError> for ImgErrorCause {
    fn from(err: image::ImageError) -> Self {
        Self::ImageError(Box::new(err))
    }
}

impl From<io::Error> for ImgErrorCause {
    fn from(err: io::Error) -> Self {
        Self::IoError(Box::new(err))
    }
}

impl From<sqlx::Error> for ImgErrorCause {
    fn from(err: sqlx::Error) -> Self {
        Self::DbError(Box::new(err))
    }
}

impl From<tokio::sync::AcquireError> for ImgErrorCause {
    fn from(err: tokio::sync::AcquireError) -> Self {
        Self::SemaphoreError(Box::new(err))
    }
}

impl From<tokio::task::JoinError> for ImgErrorCause {
    fn from(err: tokio::task::JoinError) -> Self {
        Self::TaskPanicked(Box::new(err))
    }
}

impl From<url::ParseError> for ImgErrorCause {
    fn from(err: url::ParseError) -> Self {
        Self::InvalidUrl(Box::new(err))
    }
}
