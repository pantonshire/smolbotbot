use std::collections::HashSet;

use anyhow::Context;
use clap::Clap;
use goldcrest::{TweetOptions, TimelineOptions, UserIdentifier};
use sqlx::postgres::{PgPool, PgConnection};

use crate::scribe::{self, ScribeFailure};
use crate::model;

#[derive(Clap)]
pub(crate) struct Opts {
    /// The maximum number of Tweets per timeline page, up to 200.
    #[clap(short = 'l', long, default_value = "200")]
    page_length: u32,
    
    /// The maximum number of timeline pages to retrieve.
    #[clap(short = 'n', long, default_value = "1")]
    pages: usize,

    /// Display additional information.
    #[clap(short, long)]
    verbose: bool,

    /// The handle of the user whose timeline should be read.
    #[clap(default_value = "smolrobots")]
    user: String,
}

pub(crate) async fn run(
    db_pool: &PgPool,
    au_client: &goldcrest::Client,
    opts: Opts
) -> anyhow::Result<()>
{
    let user = UserIdentifier::Handle(opts.user
        .strip_prefix('@')
        .map(str::to_owned)
        .unwrap_or(opts.user));

    let mut db_conn = db_pool.acquire()
        .await
        .context("failed to connect to database")?;

    let robot_ids = scribe_timeline(au_client, &mut db_conn, user, opts.page_length, opts.pages, opts.verbose)
        .await
        .context("failed getting robots from user timeline")?;

    for robot_id in robot_ids {
        println!("{}", robot_id);
    }

    Ok(())
}

async fn scribe_timeline(
    au_client: &goldcrest::Client,
    db_conn: &mut PgConnection,
    user: goldcrest::UserIdentifier,
    page_length: u32,
    pages: usize,
    verbose: bool
) -> Result<Vec<i32>, ScribeFailure>
{
    let mut group_ids = Vec::new();
    let mut max_id = None;

    for _ in 0..pages {
        let tweet_options = TweetOptions::default();

        let timeline_options = TimelineOptions::default().count(page_length);
        let timeline_options = match max_id {
            None => timeline_options,
            Some(id) => timeline_options.max_id(id),
        };

        let tweets = {
            let mut tweets = au_client
                .user_timeline(user.clone(), timeline_options, tweet_options, true, true)
                .await?;

            let all_ids = tweets
                .iter()
                .map(|tweet| scribe::tweet_original(tweet).id as i64)
                .collect::<Vec<_>>();

            // Get the ids of the tweets already in the database; there is no need to parse these
            // tweets again. Filtering them out now also avoids the robots.id sequence from being
            // unneccessarily incremented ON CONFLICT
            let existing_ids = sqlx::query_as::<_, model::TweetId>(
                "SELECT tweet_id FROM UNNEST($1) as tweet_ids(tweet_id) \
                WHERE EXISTS (SELECT 1 FROM robots WHERE robots.tweet_id = tweet_ids.tweet_id)"
            )
            .bind(all_ids)
            .fetch_all(&mut *db_conn)
            .await?
            .into_iter()
            .map(|row| row.tweet_id as u64)
            .collect::<HashSet<_>>();

            tweets.retain(|tweet| tweet.id > 0
                && !existing_ids.contains(&scribe::tweet_original(tweet).id));
            
            tweets
        };

        if tweets.is_empty() {
            if verbose {
                eprintln!("empty timeline page reached, stopping");
            }
            break;
        }

        max_id = Some(tweets
            .iter()
            .map(|tweet| tweet.id)
            .min()
            //Subtract 1 because, at the time of writing, max_id is inclusive
            .unwrap() - 1);

        group_ids.extend(
            scribe::scribe_tweets(&mut *db_conn, &tweets, verbose)
                .await?
                .into_iter());
    }

    Ok(group_ids)
}
