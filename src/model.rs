use std::error;
use std::fmt;
use std::str::FromStr;
use std::num::ParseIntError;

use sqlx::{FromRow, Type};
use sqlx::postgres::{PgHasArrayType, PgTypeInfo};

#[derive(Type, Clone, Debug)]
#[sqlx(type_name = "robot_ident")]
pub struct IdentBuf {
    pub number: i32,
    pub name: String,
}

impl IdentBuf {
    pub fn new(number: i32, name: String) -> Self {
        Self {
            number,
            name,
        }
    }
}

impl PgHasArrayType for IdentBuf {
    fn array_type_info() -> PgTypeInfo {
        // PostgreSQL internally names array types by prefixing the type name with an underscore
        PgTypeInfo::with_name("_robot_ident")
    }
}

// #[derive(Type, Copy, Clone, Debug)]
// pub struct Ident<'a> {
//     pub number: i32,
//     pub name: &'a str,
// }

impl fmt::Display for IdentBuf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.number, self.name)
    }
}

impl FromStr for IdentBuf {
    type Err = ParseIdentError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (number, name) = s.split_once('/')
            .ok_or(ParseIdentError::MissingSlash)?;

        let number = number.parse::<i32>()
            .map_err(ParseIdentError::InvalidNumber)?;

        Ok(IdentBuf::new(number, name.to_owned()))
    }
}

#[derive(Debug)]
pub enum ParseIdentError {
    MissingSlash,
    InvalidNumber(ParseIntError),
}

impl fmt::Display for ParseIdentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseIdentError::MissingSlash => write!(f, "missing slash separator"),
            ParseIdentError::InvalidNumber(err) => err.fmt(f),
        }
    }
}

impl error::Error for ParseIdentError {}

#[derive(FromRow)]
pub(crate) struct TweetId {
    pub(crate) tweet_id: i64,
}

#[derive(FromRow)]
pub(crate) struct DailyRobot {
    pub(crate) id: IdentBuf,
    pub(crate) prefix: String,
    pub(crate) suffix: String,
    pub(crate) plural: Option<String>,
    pub(crate) tweet_id: i64,
    pub(crate) content_warning: Option<String>,
}

impl DailyRobot {
    pub(crate) fn full_name(&self) -> String {
        let mut name_buf = String::with_capacity(
            self.prefix.len()
            + self.suffix.len()
            + self.plural.as_deref().map_or(0, |plural| plural.len())
        );

        name_buf.push_str(&self.prefix);
        name_buf.push_str(&self.suffix);

        if let Some(ref plural) = self.plural {
            name_buf.push_str(plural);
        }

        name_buf
    }

    pub(crate) fn tweet_url(&self) -> String {
        format!("https://twitter.com/smolrobots/status/{}", self.tweet_id)
    }
}

#[derive(FromRow, Clone, Debug)]
pub(crate) struct RobotImageUrl {
    pub(crate) id: IdentBuf,
    pub(crate) image_url: String,
}

#[derive(FromRow, Clone, Debug)]
pub(crate) struct RobotImagePath {
    pub(crate) id: IdentBuf,
    pub(crate) image_path: String,
}

#[derive(FromRow, Clone, Debug)]
pub(crate) struct RobotImagePathOpt {
    pub(crate) id: IdentBuf,
    pub(crate) image_path: Option<String>,
}

#[derive(FromRow, Clone, Debug)]
pub(crate) struct RobotCustomAltExport {
    pub(crate) id: IdentBuf,
    pub(crate) custom_alt: String,
}
