use std::collections::HashMap;
use std::{fmt, fs, io, error, result};
use serde::Deserialize;

const DEFAULT_PATH: &'static str = "services.yaml";

#[derive(Deserialize, Clone, Debug)]
pub struct Services {
    pub goldcrest: Option<GoldcrestService>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct GoldcrestService {
    pub scheme: String,
    pub host: String,
    pub port: u32,
    pub authentication: HashMap<String, GoldcrestAuthentication>,
    pub request_timeout_seconds: Option<i64>,
    pub wait_timeout_seconds: Option<i64>,
}

impl GoldcrestService {
    pub const DEFAULT_AUTH_KEY: &'static str = "default";
}

#[derive(Deserialize, Clone, Debug)]
pub struct GoldcrestAuthentication {
    pub consumer_key: String,
    pub consumer_secret: String,
    pub access_token: String,
    pub token_secret: String,
}

/// Attempts to load the YAML services specification at the given path.
/// If the given path is None, the default path `services.yaml` is used.
pub fn load(path: Option<&str>) -> Result<Services> {
    load_with_default_path(path, DEFAULT_PATH)
}

pub fn load_with_default_path(path: Option<&str>, default_path: &str) -> Result<Services> {
    let contents = fs::read_to_string(path.unwrap_or(default_path))?;
    let services = serde_yaml::from_str(&contents)?;
    Ok(services)
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    FileIO(Box<io::Error>),
    Deserialization(Box<serde_yaml::Error>),
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::FileIO(err)          => err.fmt(fmt),
            Error::Deserialization(err) => err.fmt(fmt),
        }
    }
}

impl error::Error for Error {}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::FileIO(Box::new(err))
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(err: serde_yaml::Error) -> Self {
        Error::Deserialization(Box::new(err))
    }
}
