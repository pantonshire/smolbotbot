use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use clap::Clap;
use goldcrest::TweetOptions;
use sqlx::postgres::PgPool;
use tokio::io::AsyncReadExt;

use crate::model;
use crate::scribe::ScribeFailure;

#[derive(Clap)]
pub(crate) struct Opts {
    /// The maximum number of tweets that can be requested concurrently. If omitted, all tweets
    /// will be requested concurrently.
    #[clap(short, long)]
    batch_size: Option<usize>,

    /// Display additional information.
    #[clap(short, long)]
    verbose: bool,

    /// The file to read the Tweet ids from.
    /// If omitted, they will be read from stdin instead.
    file: Option<PathBuf>,
}

pub(crate) async fn run(
    db_pool: &PgPool,
    au_client: Arc<goldcrest::Client>,
    opts: Opts
) -> anyhow::Result<()>
{
    let input_tweet_ids = {
        let input = match opts.file {
            Some(input_path) =>
                tokio::fs::read_to_string(&input_path)
                    .await
                    .with_context(|| format!("failed to read input file {}", input_path.to_string_lossy()))?,
    
            None => {
                let mut buf = String::new();
                tokio::io::stdin()
                    .read_to_string(&mut buf)
                    .await
                    .context("failed to read from stdin")?;
                buf
            },
        };

        let mut tweet_ids = input
            .split_ascii_whitespace()
            .map(|id| id.parse()
                .with_context(|| format!(r#"invalid tweet id "{}""#, id)))
            .collect::<anyhow::Result<Vec<i64>>>()?;

        tweet_ids.sort_unstable();
        tweet_ids.dedup();
        tweet_ids
    };

    // Only use tweet ids that are not already in the database
    let tweet_ids = sqlx::query_as::<_, model::TweetId>(
        "SELECT tweet_id FROM UNNEST($1) as tweet_ids(tweet_id) \
        WHERE tweet_id NOT IN (SELECT tweet_id FROM robots)"
    )
    .bind(input_tweet_ids)
    .fetch_all(db_pool)
    .await
    .context("failed to check for existing tweet ids")?
    .into_iter()
    .map(|row| row.tweet_id as u64)
    .collect::<Vec<_>>();

    let robot_ids = match opts.batch_size {
        Some(batch_size) => batched_fetch_and_scribe(au_client, db_pool, &tweet_ids, batch_size, opts.verbose).await,
        None => fetch_and_scribe(au_client, db_pool, &tweet_ids, opts.verbose).await,
    }.context("failed to fetch some tweets")?;

    for robot_id in robot_ids {
        println!("{}", robot_id);
    }

    Ok(())
}

/// Wrapper function around fetch_and_scribe to put a limit on the number of tweets that can be in
/// memory at once. Each batch is requested, parsed and stored in series. All of the tweet ids within
/// a given batch will be requested, parsed and stored concurrently.
async fn batched_fetch_and_scribe(
    client: Arc<goldcrest::Client>,
    db_pool: &PgPool,
    tweet_ids: &[u64],
    batch_size: usize,
    verbose: bool
) -> Result<Vec<i32>, ScribeFailure>
{
    let mut group_ids = Vec::new();
    let num_tweets = tweet_ids.len();
    let mut min_tweet_index = 0usize;

    while min_tweet_index < num_tweets {
        let max_tweet_index = (min_tweet_index + batch_size).min(num_tweets);
        let current_batch = &tweet_ids[min_tweet_index..max_tweet_index];

        group_ids.extend(
            fetch_and_scribe(client.clone(), db_pool, current_batch, verbose)
                .await?
                .into_iter());

        min_tweet_index = max_tweet_index;
    }

    Ok(group_ids)
}

/// Splits the given tweet ids into groups of 100, then concurrently requests each group of 100,
/// parses the received tweets and adds them to the database.
async fn fetch_and_scribe(
    client: Arc<goldcrest::Client>,
    db_pool: &PgPool,
    tweet_ids: &[u64],
    verbose: bool
) -> Result<Vec<i32>, ScribeFailure>
{
    const TWEETS_PER_REQUEST: usize = 100;

    let mut tweet_ids = tweet_ids.to_vec();
    tweet_ids.sort_unstable();
    tweet_ids.dedup();
    let n_tweet_ids = tweet_ids.len();

    let mut join_handles = Vec::new();
    
    let mut assigned: usize = 0;
    while assigned < n_tweet_ids {
        let max_id = (assigned + TWEETS_PER_REQUEST).min(n_tweet_ids);
        let ids = (&tweet_ids[assigned..max_id]).to_vec();

        let client = client.clone();
        // Clone the pool because it's just a wrapper around an Arc
        let db_pool = db_pool.clone();

        join_handles.push(tokio::spawn(async move {
            let tweets_res = client
                .get_tweets(ids, TweetOptions::default())
                .await;

            match tweets_res {
                Err(err) => Err(err.into()),

                Ok(tweets) => match db_pool.acquire().await {
                    Err(err) => Err(err.into()),

                    Ok(mut pool_conn) =>
                        crate::scribe::scribe_tweets(&mut pool_conn, tweets, verbose).await,
                },
            }
        }));

        assigned = max_id;
    }

    let mut group_ids = Vec::new();
    for join_handle in join_handles {
        group_ids.extend(join_handle.await??.into_iter());
    }

    Ok(group_ids)
}
