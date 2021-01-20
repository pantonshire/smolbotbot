mod error;

use std::env;
use std::cmp::min;
use std::iter::FromIterator;
use std::sync::Arc;
use chrono::{prelude::*, Duration};
use diesel::Connection;
use diesel::result::{Error::DatabaseError, DatabaseErrorKind};

use goldcrest::request::TweetOptions;

use sbb_parse::twitter::{parse_tweet, new_robot};
use sbb_data::Create;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tweet_ids = include_str!("../tweet_ids");
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

    let auth = goldcrest::Authentication::new(&consumer_key, &consumer_secret, &access_token, &token_secret);

    let mut client = goldcrest::ClientBuilder::new();
    client
        .authenticate(auth)
        .port(7400)
        .request_timeout(Duration::seconds(30))
        .wait_timeout(Duration::minutes(16));

    let client = Arc::new(client.connect().await?);

    scribe_ids(client, &db_conn, tweet_ids, true).await?;

    Ok(())
}

//TODO: buffered variant which calls scribe_ids multiple times on slices of the ids

async fn scribe_ids(client: Arc<goldcrest::Client>, db_conn: &diesel::PgConnection, mut tweet_ids: Vec<u64>, show_status: bool) -> Result<(Vec<u64>, Vec<u64>, Vec<u64>), Box<dyn std::error::Error>> {
    tweet_ids.sort();
    tweet_ids.dedup();
    let n_tweet_ids = tweet_ids.len();

    let mut join_handles = Vec::new();
    {
        const BATCH_SIZE: usize = 100;
        let mut assigned: usize = 0;
        while assigned < n_tweet_ids {
            let max_id = min(assigned + BATCH_SIZE, n_tweet_ids);
            let ids = (&tweet_ids[assigned..max_id]).to_vec();
            let client = client.clone();
            join_handles.push(tokio::spawn(async move {
                client
                    .get_tweets(ids, TweetOptions::default())
                    .await
                    .map_err(|_| error::ScribeError::TweetGetFailure)
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
        let parse_res = parse_tweet(&tweet, |group, robots| {
            db_conn.transaction::<(), diesel::result::Error, _>(|| {
                let group_id = group.create(&db_conn)?.id;
                for ref robot in robots {
                    new_robot(robot, group_id, |robot| {
                        robot.create(&db_conn)
                    })?;
                }
                Ok(())
            })
        });

        match parse_res {
            None => &mut unparsed_ids,
            Some(Err(DatabaseError(DatabaseErrorKind::UniqueViolation, _))) => &mut existing_ids,
            Some(Err(err)) => return Err(Box::new(err)),
            Some(Ok(())) => &mut parsed_ids,
        }.push(tweet.id);

        n_tweets_done += 1;
        if show_status && (n_tweets_done == n_tweets || last_shown.is_none() || Utc::now() - last_shown.unwrap() > Duration::milliseconds(250)) {
            println!("{}{}Progress: {} / {}", termion::cursor::Up(1), termion::clear::CurrentLine, n_tweets_done, n_tweets);
            last_shown = Some(Utc::now());
        }
    }

    Ok((parsed_ids, unparsed_ids, existing_ids))
}
