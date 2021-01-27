use std::error::Error;
use std::fmt::{self, Display, Formatter};

pub type ConnectionResult<T> = Result<T, ConnectionError>;

#[derive(Debug)]
pub enum ConnectionError {
    InvalidUri,
    Tonic(Box<tonic::transport::Error>),
}

impl Display for ConnectionError {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match self {
            ConnectionError::InvalidUri => write!(fmt, "invalid URI"),
            ConnectionError::Tonic(err) => err.fmt(fmt),
        }
    }
}

impl Error for ConnectionError {}

impl From<tonic::transport::Error> for ConnectionError {
    fn from(err: tonic::transport::Error) -> Self {
        ConnectionError::Tonic(Box::new(err))
    }
}

pub type RequestResult<T> = Result<T, RequestError>;

#[derive(Debug)]
pub enum RequestError {
    Tonic(Box<tonic::Status>),
    Deserialization(Box<DeserializationError>),
}

impl Display for RequestError {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match self {
            RequestError::Tonic(err)           => err.fmt(fmt),
            RequestError::Deserialization(err) => err.fmt(fmt),
        }
    }
}

impl Error for RequestError {}

impl From<tonic::Status> for RequestError {
    fn from(err: tonic::Status) -> Self {
        RequestError::Tonic(Box::new(err))
    }
}

impl From<DeserializationError> for RequestError {
    fn from(err: DeserializationError) -> Self {
        RequestError::Deserialization(Box::new(err))
    }
}

pub(crate) type DeserializationResult<T> = Result<T, DeserializationError>;

#[derive(Debug)]
pub enum DeserializationError {
    FieldMissing,
    FieldOutOfRange,
}

impl Display for DeserializationError {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "error deserializing NLPewee response: {}", match self {
            DeserializationError::FieldMissing    => "missing field",
            DeserializationError::FieldOutOfRange => "field out of range",
        })
    }
}

impl Error for DeserializationError {}

pub(crate) trait Exists<T> {
    fn exists(self) -> DeserializationResult<T>;
}

impl<T> Exists<T> for Option<T> {
    fn exists(self) -> DeserializationResult<T> {
        self.ok_or(DeserializationError::FieldMissing)
    }
}
