mod error;

use std::env;
use std::sync::Arc;
use std::borrow::Cow;
use std::io::{self, Read};

use anyhow::anyhow;
use chrono::Duration;
use clap::{Clap, crate_version, crate_authors, crate_description};
use sqlx::Connection;
use sqlx::postgres::{PgConnection, PgPool};

use goldcrest::data::{Tweet, tweet::TweetTextOptions, Media};
use goldcrest::request::{TweetOptions, TimelineOptions};

use services::GoldcrestService;

use error::{NotScribed, InvalidTweet, ScribeFailure};

#[derive(Clap)]
#[clap(version = crate_version!(), author = crate_authors!(), about = crate_description!())]
struct Opts {
    /// The path to the services YAML. If omitted, "services.yaml" will be used
    #[clap(long)]
    services: Option<String>,
    /// The services YAML key corresponding to the Goldcrest Twitter authentication data to use.
    /// If omitted, the key "default" will be used
    #[clap(long)]
    goldcrest_auth: Option<String>,
    /// Show additional information while running
    #[clap(short, long)]
    verbose: bool,
    #[clap(subcommand)]
    subcommand: Subcommand,
}

#[derive(Clap)]
enum Subcommand {
    /// Read tweet ids from stdin
    Ids(IdsCommand),
    /// Traverse a user's timeline
    Timeline(TimelineCommand),
}

#[derive(Clap)]
struct IdsCommand {
    /// The maximum number of tweets that can be requested concurrently. If omitted, all tweets
    /// will be requested concurrently.
    #[clap(short, long)]
    batch_size: Option<usize>,
}

#[derive(Clap)]
struct TimelineCommand {
    /// The handle of the user whose timeline should be read
    #[clap(short, long, default_value = "smolrobots")]
    user: String,
    /// The maximum number of Tweets per timeline page, up to 200
    #[clap(short = 'l', long, default_value = "200")]
    page_length: u32,
    /// The maximum number of timeline pages to retrieve
    #[clap(short = 'n', long, default_value = "1")]
    pages: usize,
}

const DB_URL_VAR: &str = "DATABASE_URL";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(feature = "dotenv")] {
        dotenv::dotenv().ok();
    }

    let opts = Opts::parse();

    let sc = services::load(opts.services.as_deref())?;

    let mut sc_goldcrest = sc.goldcrest
        .expect("No Goldcrest config found");

    let auth_key = opts
        .goldcrest_auth
        .as_deref()
        .unwrap_or(GoldcrestService::DEFAULT_AUTH_KEY);

    let auth = sc_goldcrest.authentication.remove(auth_key)
        .expect("Authentication data not found");

    let auth = goldcrest::Authentication::new(
        auth.consumer_key,
        auth.consumer_secret,
        auth.access_token,
        auth.token_secret
    );

    let mut client = goldcrest::ClientBuilder::new();
    client
        .authenticate(auth)
        .scheme(&sc_goldcrest.scheme)
        .socket(&sc_goldcrest.host, sc_goldcrest.port);
    if let Some(timeout) = sc_goldcrest.request_timeout_seconds {
        client.request_timeout(Duration::seconds(timeout));
    }
    if let Some(timeout) = sc_goldcrest.wait_timeout_seconds {
        client.wait_timeout(Duration::seconds(timeout));
    }

    let client = Arc::new(client.connect().await?);

    let group_ids = match opts.subcommand {
        Subcommand::Ids(ids_opts) => {
            if let Some(batch_size) = ids_opts.batch_size {
                if batch_size < 1 {
                    return Err(anyhow!("batch size must be at least 1"));
                }
            }

            // We will be accessing the database concurrently, so make a pool rather than just a
            // single connection
            let db_pool = {
                let db_url = env::var(DB_URL_VAR)?;
                PgPool::connect(&db_url).await?
            };
            
            let group_ids = {
                // Get the list of tweet ids from stdin
                let tweet_ids = read_tweet_ids()?;
                
                match ids_opts.batch_size {
                    // If a batch size was specified, split the tweet ids into batches
                    Some(batch_size) =>
                        batched_fetch_and_scribe(client, &db_pool, &tweet_ids, batch_size, opts.verbose)
                            .await?,
                    
                    // If no batch size was specified, process all tweet ids concurrently at once
                    None =>
                        fetch_and_scribe(client, &db_pool, &tweet_ids, opts.verbose)
                            .await?,
                }
            };

            // Manually close all of the pool's connections so that Postgres has an easier time
            // cleaning them up
            db_pool.close().await;

            group_ids
        },

        Subcommand::Timeline(timeline_opts) => {
            // All database access will be in series, so we can use a single connection
            let mut db_conn = {
                let db_url = env::var(DB_URL_VAR)?;
                PgConnection::connect(&db_url).await?
            };

            // Ignore the leading @ if one was included, e.g. @smolrobots -> smolrobots
            let handle = timeline_opts.user
                .strip_prefix("@")
                .map(str::to_owned)
                .unwrap_or(timeline_opts.user);
            let user = goldcrest::UserIdentifier::Handle(handle);

            // Traverse the user's timeline, parsing the tweets and adding new robots to the database
            let group_ids = scribe_timeline(&client, &mut db_conn, user, timeline_opts.page_length, timeline_opts.pages, opts.verbose)
                .await?;

            // Manually close the connection so Postgres has an easier time cleaning it up
            db_conn.close().await?;
            
            group_ids
        },
    };

    // Output the ids of the new robot groups, to be used by other processes
    for group_id in group_ids {
        println!("{}", group_id);
    }

    Ok(())
}

fn read_tweet_ids() -> anyhow::Result<Vec<u64>> {
    let stdin = io::stdin();
    let mut buffer = String::new();
    stdin.lock().read_to_string(&mut buffer)?;

    buffer
        .split_whitespace()
        .map(|id| id
            .parse::<u64>()
            .map_err(|_| anyhow!("could not parse tweet id from \"{}\"", id.to_owned())))
        .collect::<Result<Vec<_>, _>>()
}

async fn scribe_timeline(
    client: &goldcrest::Client,
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
            let mut tweets = client
                .user_timeline(user.clone(), timeline_options, tweet_options, true, true)
                .await?;
            tweets.retain(|tweet| tweet.id > 0);
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
            scribe_tweets(db_conn, tweets, verbose)
                .await?
                .into_iter());
    }

    Ok(group_ids)
}

/// Wrapper function around scribe_ids to put a limit on the number of tweets that can be in memory
/// at once. Each batch is requested, parsed and stored in series. All of the tweet ids within a
/// given batch will be requested, parsed and stored concurrently.
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
                        scribe_tweets(&mut pool_conn, tweets, verbose).await,
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

/// Parses and stores a collection of tweets in series, skipping any tweets that are not valid
/// small robots.
async fn scribe_tweets(
    db_conn: &mut PgConnection,
    tweets: Vec<Tweet>,
    verbose: bool
) -> Result<Vec<i32>, ScribeFailure>
{
    let mut group_ids = Vec::new();

    for tweet in tweets {
        let tweet_id = tweet.id;

        match scribe_tweet(db_conn, tweet).await {
            Ok(group_id) => group_ids.push(group_id),

            Err(NotScribed::InvalidTweet(err)) => if verbose {
                eprintln!("skip tweet {}: {}", tweet_id, err);
            },

            Err(NotScribed::ScribeFailure(err)) => return Err(err)
        }
    }

    Ok(group_ids)
}

/// Parses the given tweet, adds it to the database and returns the id of the new robot group.
async fn scribe_tweet(db_conn: &mut PgConnection, tweet: Tweet) -> Result<i32, NotScribed> {
    const TEXT_OPTIONS: TweetTextOptions = TweetTextOptions::all()
        .media(false)
        .urls(false);

    let tweet = tweet.original();
    let tweet_text = tweet.text(TEXT_OPTIONS);

    let group = match sbb_parse::parse_group(&tweet_text) {
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

    let mut tx = db_conn.begin().await?;

    let group_id = sqlx::query!(
        "INSERT INTO robot_groups \
            (tweet_id, tweet_time, image_url, body, alt, content_warning) \
        VALUES \
            ($1, $2, $3, $4, $5, $6) \
        ON CONFLICT (tweet_id) DO NOTHING \
        RETURNING id",
        /* $1 */ tweet.id as i64,
        /* $2 */ tweet.created_at,
        /* $3 */ media_url,
        /* $4 */ body,
        /* $5 */ alt,
        /* $6 */ group.cw,
    )
    .fetch_optional(&mut tx)
    .await?
    .ok_or(InvalidTweet::DuplicateTweetId(tweet.id))?
    .id;

    for robot in group.robots {
        let ident = robot.name.identifier();

        let _robot_id = sqlx::query!(
            "INSERT INTO robots \
                (group_id, robot_number, prefix, suffix, plural, ident) \
            VALUES \
                ($1, $2, $3, $4, $5, $6) \
            ON CONFLICT (robot_number, ident) DO NOTHING \
            RETURNING id",
            /* $1 */ group_id,
            /* $2 */ robot.number,
            /* $3 */ robot.name.prefix.as_ref(),
            /* $4 */ robot.name.suffix.as_ref(),
            /* $5 */ robot.name.plural.as_ref().map(Cow::as_ref),
            /* $6 */ ident.as_str(),
        )
        .fetch_optional(&mut tx)
        .await?
        .ok_or(InvalidTweet::DuplicateRobot(robot.number, ident))?
        .id;
        
        //TODO: log or output the robot id in some way
    }

    tx.commit().await?;

    Ok(group_id)
}

fn is_valid_robot_media(media: &Media) -> bool {
    media.media_type == "photo" || media.media_type == "animated_gif" || media.media_type == "video"
}
