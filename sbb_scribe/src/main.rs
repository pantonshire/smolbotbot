mod error;

use std::env;
use std::sync::Arc;
use chrono::{prelude::*, Duration};
use diesel::prelude::*;
use diesel::{Connection, select, QueryDsl};
use diesel::result::{Error::DatabaseError, DatabaseErrorKind};

use goldcrest::request::TweetOptions;

use sbb_parse::twitter::{parse_tweet, new_robot};
use sbb_data::Create;
use diesel::expression::exists::exists;
use goldcrest::data::Tweet;

use error::{ScribeError, ScribeResult};

//TODO: option to scribe from user timeline rather than the text file

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tweet_ids = include_str!("../tweet_ids_2");
    let tweet_ids = tweet_ids
        .split_whitespace()
        .map(|s| s.parse::<u64>().unwrap())
        .collect::<Vec<u64>>();

    let db_conn = sbb_data::connect_env()?;

    dotenv::dotenv().ok();

    //TODO: error messages, read from CLI flags first
    let consumer_key = env::var("TWITTER_CONSUMER_KEY").unwrap();
    let consumer_secret = env::var("TWITTER_CONSUMER_SECRET").unwrap();
    let access_token = env::var("TWITTER_ACCESS_TOKEN").unwrap();
    let token_secret = env::var("TWITTER_TOKEN_SECRET").unwrap();

    let auth = goldcrest::Authentication::new(consumer_key, consumer_secret, access_token, token_secret);

    let mut client = goldcrest::ClientBuilder::new();
    client
        .authenticate(auth)
        .port(7400)
        .request_timeout(Duration::seconds(30))
        .wait_timeout(Duration::minutes(16));

    let client = Arc::new(client.connect().await?);

    scribe_ids(client, &db_conn, &tweet_ids, true).await?;

    Ok(())
}

async fn scribe_ids_batched(client: Arc<goldcrest::Client>, db_conn: &PgConnection, tweet_ids: &[u64], batch_size: usize, show_status: bool) -> ScribeResult<(Vec<u64>, Vec<u64>, Vec<u64>)> {
    let mut parsed = Vec::new();
    let mut unparsed = Vec::new();
    let mut existing = Vec::new();
    let n = tweet_ids.len();
    let mut i: usize = 0;
    let mut batch_no = 0;
    while i < n {
        if show_status {
            println!("Batch {}", batch_no + 1);
        }
        let next_i = (i + batch_size).min(n);
        let (batch_parsed, batch_unparsed, batch_existing) = scribe_ids(client.clone(), db_conn, &tweet_ids[i..next_i], show_status).await?;
        parsed.extend(batch_parsed.into_iter());
        unparsed.extend(batch_unparsed.into_iter());
        existing.extend(batch_existing.into_iter());
        i = next_i;
        batch_no += 1;
    }
    Ok((parsed, unparsed, existing))
}

async fn scribe_ids(client: Arc<goldcrest::Client>, db_conn: &PgConnection, tweet_ids: &[u64], show_status: bool) -> ScribeResult<(Vec<u64>, Vec<u64>, Vec<u64>)> {
    let mut tweet_ids = tweet_ids.to_vec();
    tweet_ids.sort();
    tweet_ids.dedup();
    let n_tweet_ids = tweet_ids.len();

    let mut join_handles = Vec::new();
    {
        const BATCH_SIZE: usize = 100;
        let mut assigned: usize = 0;
        while assigned < n_tweet_ids {
            let max_id = (assigned + BATCH_SIZE).min(n_tweet_ids);
            let ids = (&tweet_ids[assigned..max_id]).to_vec();
            let client = client.clone();
            join_handles.push(tokio::spawn(async move {
                client
                    .get_tweets(ids, TweetOptions::default())
                    .await
                    .map_err(|_| ScribeError::TweetGetFailure)
            }));
            assigned = max_id;
        }
    }

    let mut tweets = Vec::new();
    for join_handle in join_handles {
        tweets.extend(join_handle.await??.into_iter());
    }

    let mut parsed_ids = Vec::new();
    let mut unparsed_ids = Vec::new();
    let mut existing_ids = Vec::new();

    let n_tweets = tweets.len();
    let mut n_tweets_done = 0;
    let mut last_shown = None;

    if show_status {
        println!();
    }

    for tweet in tweets {
        let tweet = tweet.original();
        let res = scribe(db_conn, &tweet);
        match res {
            Ok(Some(_group_id)) => &mut parsed_ids,
            Ok(None) => &mut unparsed_ids,
            Err(err) => match err {
                ScribeError::DbError(DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => &mut existing_ids,
                ScribeError::TweetAlreadyExists => &mut existing_ids,
                ScribeError::RobotAlreadyExists => &mut existing_ids,
                err => return Err(err),
            },
        }.push(tweet.id);

        n_tweets_done += 1;
        if show_status && (n_tweets_done == n_tweets || last_shown.is_none() || Utc::now() - last_shown.unwrap() > Duration::milliseconds(250)) {
            println!("{}{}Progress: {} / {}", termion::cursor::Up(1), termion::clear::CurrentLine, n_tweets_done, n_tweets);
            last_shown = Some(Utc::now());
        }
    }

    Ok((parsed_ids, unparsed_ids, existing_ids))
}

fn scribe(db_conn: &PgConnection, tweet: &Tweet) -> ScribeResult<Option<i32>> {
    db_conn.transaction::<Option<i32>, ScribeError, _>(|| {
        if !tweet_unique(db_conn, tweet)? {
            return Err(ScribeError::TweetAlreadyExists);
        }
        let parse_res = parse_tweet::<_, ScribeResult<i32>>(tweet, |group, robots| {
            let group_id = group.create(db_conn)?.id;
            for ref robot in robots {
                if !robot_unique(db_conn, robot)? {
                    return Err(ScribeError::RobotAlreadyExists);
                }
                new_robot(robot, group_id, |robot| {
                    robot.create(db_conn)
                })?;
            }
            Ok(group_id)
        });
        match parse_res {
            Some(Ok(val)) => Ok(Some(val)),
            Some(Err(err)) => Err(err),
            None => Ok(None),
        }
    })
}

fn tweet_unique(db_conn: &PgConnection, tweet: &Tweet) -> ScribeResult<bool> {
    use sbb_data::schema::robot_groups::dsl::*;
    select(exists(robot_groups.filter(tweet_id.eq(tweet.id as i64))))
        .get_result::<bool>(db_conn)
        .map_err(|err| err.into())
        .map(|x| !x)
}

fn robot_unique(db_conn: &PgConnection, robot: &sbb_parse::Robot) -> ScribeResult<bool> {
    use sbb_data::schema::robots::dsl::*;
    select(exists(robots.filter(prefix.eq(robot.name.prefix).and(robot_number.eq(robot.number)))))
        .get_result::<bool>(db_conn)
        .map_err(|err| err.into())
        .map(|x| !x)
}
