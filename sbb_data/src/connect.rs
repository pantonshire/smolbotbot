use std::fmt::{self, Display, Formatter};
use std::error::Error;
use std::env;
use diesel::prelude::*;
use dotenv::dotenv;

const DATABASE_URL_VAR: &'static str = "DATABASE_URL";

pub fn connect_env() -> Result<PgConnection, EnvConnectionError> {
    dotenv().ok();
    let url = env::var(DATABASE_URL_VAR)
        .map_err(|_| EnvConnectionError::UrlEnvNotFound)?;
    PgConnection::establish(&url)
        .map_err(|err| EnvConnectionError::DieselError(err))
}

#[derive(PartialEq, Debug)]
pub enum EnvConnectionError {
    UrlEnvNotFound,
    DieselError(diesel::ConnectionError),
}

impl Display for EnvConnectionError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            EnvConnectionError::UrlEnvNotFound       => write!(f, "DATABASE_URL environment variable not set"),
            EnvConnectionError::DieselError(ref err) => err.fmt(f),
        }
    }
}

impl Error for EnvConnectionError {}
