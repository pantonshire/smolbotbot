use std::borrow::Cow;
use std::error;
use std::fmt;

use chrono::{Utc, DateTime};
use goldcrest::data::{Tweet, Media};
use goldcrest::data::tweet::TweetTextOptions;
use sqlx::Connection;
use sqlx::postgres::PgConnection;

use crate::model;
use crate::parse::{self, Robot};
use crate::plural::Plural;

#[derive(Clone, Debug)]
struct RobotTweetData<'a> {
    tweet_id: i64,
    tweet_time: DateTime<Utc>,
    image_url: &'a str,
    body: &'a str,
    alt: Option<&'a str>,
    cw: Option<&'a str>,
}

/// Parses and stores a collection of tweets in series, skipping any tweets that are not valid
/// small robots.
pub(crate) async fn scribe_tweets(
    db_conn: &mut PgConnection,
    tweets: Vec<Tweet>,
    verbose: bool
) -> Result<Vec<i32>, ScribeFailure>
{
    let mut group_ids = Vec::new();

    for tweet in tweets {
        let tweet_id = tweet.id;

        match scribe_tweet(db_conn, tweet).await {
            Ok(robot_ids) => group_ids.extend(robot_ids.iter()),

            Err(NotScribed::InvalidTweet(err)) => if verbose {
                eprintln!("skip tweet {}: {}", tweet_id, err);
            },

            Err(NotScribed::ScribeFailure(err)) => return Err(err)
        }
    }

    Ok(group_ids)
}

/// Parses the given tweet, adds it to the database and returns the id of the new robot group.
pub(crate) async fn scribe_tweet(
    db_conn: &mut PgConnection,
    tweet: Tweet
) -> Result<Plural<i32>, NotScribed>
{
    const TEXT_OPTIONS: TweetTextOptions = TweetTextOptions::all()
        .media(false)
        .urls(false);

    let tweet = tweet.original();
    let tweet_text = tweet.text(TEXT_OPTIONS);

    let group = match parse::parse_group(&tweet_text) {
        Some(group) if !group.robots.is_empty() => group,
        _ => return Err(InvalidTweet::ParseUnsuccessful.into()),
    };

    let body = group.body.trim();

    let media = {
        let media = tweet.media
            .iter()
            .find(|media| is_valid_robot_media(media));

        match media {
            Some(media) => media,
            None => return Err(InvalidTweet::MissingMedia.into()),
        }
    };

    let media_url = media.media_url.as_str();

    let alt = {
        let alt = media.alt.trim();
        if alt.is_empty() {
            None
        } else {
            Some(alt)
        }
    };

    let tweet_data = RobotTweetData {
        tweet_id: tweet.id as i64,
        tweet_time: tweet.created_at,
        image_url: media_url,
        body: body,
        alt: alt,
        cw: group.cw,
    };

    match group.robots.as_slice() {
        [] => Err(InvalidTweet::NoRobots.into()),

        [robot] => store_robot(db_conn, robot, &tweet_data)
            .await
            .map(Plural::One),

        robots => {
            let mut robot_ids = Vec::with_capacity(robots.len());
            let mut tx = db_conn.begin().await?;
            for robot in robots {
                robot_ids.push(store_robot(&mut tx, robot, &tweet_data).await?);
            }
            tx.commit().await?;
            Ok(Plural::Many(robot_ids))
        },
    }
}

async fn store_robot(
    db_conn: &mut PgConnection,
    robot: &Robot<'_>,
    tweet_data: &RobotTweetData<'_>,
) -> Result<i32, NotScribed>
{
    let ident = robot.name.identifier();

    sqlx::query_as::<_, model::Id>(
        "INSERT INTO robots \
            (robot_number, prefix, suffix, plural, ident, tweet_id, tweet_time, \
             image_url, body, alt, content_warning) \
        VALUES \
            ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) \
        ON CONFLICT (robot_number, ident) DO NOTHING \
        RETURNING id"
    )
    .bind(robot.number)
    .bind(robot.name.prefix.as_ref())
    .bind(robot.name.suffix.as_ref())
    .bind(robot.name.plural.as_ref().map(Cow::as_ref))
    .bind(ident.as_str())
    .bind(tweet_data.tweet_id)
    .bind(tweet_data.tweet_time)
    .bind(tweet_data.image_url)
    .bind(tweet_data.body)
    .bind(tweet_data.alt)
    .bind(tweet_data.cw)
    .fetch_optional(db_conn)
    .await
    .map_err(NotScribed::from)
    .and_then(|row| row
        .ok_or(InvalidTweet::DuplicateRobot(robot.number, ident).into()))
    .map(|row| row.id)
}

fn is_valid_robot_media(media: &Media) -> bool {
    match media.media_type.as_str() {
        "photo" | "animated_gif" | "video" => true,
        _ => false,
    }
}

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
    DuplicateRobot(i32, String),
    NoRobots,
}

impl fmt::Display for InvalidTweet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ParseUnsuccessful => write!(f, "could not parse robot data from tweet"),
            Self::MissingMedia => write!(f, "tweet does not contain media"),
            Self::DuplicateRobot(number, ident) => write!(f, "robot ({}, {}) already exists", number, ident),
            Self::NoRobots => write!(f, "no robots in tweet"),
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