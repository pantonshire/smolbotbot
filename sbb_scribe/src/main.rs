mod error;

use std::env;
use std::sync::Arc;
use std::collections::HashSet;
use chrono::{prelude::*, Duration};
use diesel::prelude::*;
use diesel::{Connection, select, QueryDsl};
use diesel::result::{Error::DatabaseError, DatabaseErrorKind};
use clap::{Clap, crate_version, crate_authors, crate_description};

use goldcrest::request::{TweetOptions, TimelineOptions};

use sbb_parse::twitter::{parse_tweet, new_robot};
use sbb_data::Create;
use diesel::expression::exists::exists;
use goldcrest::data::Tweet;

use error::{ScribeError, ScribeResult};

#[derive(Clap)]
#[clap(version = crate_version!(), author = crate_authors!(), about = crate_description!())]
struct Opts {
    /// The scheme to use to connect to the Goldcrest server
    #[clap(short = 'x', long, default_value = "http")]
    scheme: String,
    /// The hostname of the Goldcrest server to connect to
    #[clap(short, long, default_value = "localhost")]
    host: String,
    /// The port of the Goldcrest server to connect to
    #[clap(short, long, default_value = "8000")]
    port: u32,
    /// Time maximum time to wait for a Goldcrest request in seconds
    #[clap(short, long, default_value = "30")]
    request_timeout: i64,
    /// Time maximum time to wait for Goldcrest resource availability in seconds
    #[clap(short, long, default_value = "1000")]
    wait_timeout: i64,
    /// Limit output from the command
    #[clap(short, long, parse(from_occurrences))]
    silent: u8,
    /// Display new robot Tweet ids when done
    #[clap(long)]
    show_new: bool,
    #[clap(subcommand)]
    subcommand: Subcommand,
}

#[derive(Clap)]
enum Subcommand {
    File(FileCommand),
    Timeline(TimelineCommand),
}

#[derive(Clap)]
struct FileCommand {
    /// The file from which to read the Tweet ids
    #[clap(short, long)]
    file: String,
    /// The maximum number of Tweets to retrieve before writing to the database
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = Opts::parse();

    let show_summary = opts.silent < 2;
    let show_progress = opts.silent < 1;

    let db_conn = sbb_data::connect_env()?;

    dotenv::dotenv().ok();

    let consumer_key = env::var("TWITTER_CONSUMER_KEY")
        .expect("TWITTER_CONSUMER_KEY not set");
    let consumer_secret = env::var("TWITTER_CONSUMER_SECRET")
        .expect("TWITTER_CONSUMER_SECRET not set");
    let access_token = env::var("TWITTER_ACCESS_TOKEN")
        .expect("TWITTER_ACCESS_TOKEN not set");
    let token_secret = env::var("TWITTER_TOKEN_SECRET")
        .expect("TWITTER_TOKEN_SECRET not set");

    let auth = goldcrest::Authentication::new(consumer_key, consumer_secret, access_token, token_secret);

    let mut client = goldcrest::ClientBuilder::new();
    client
        .authenticate(auth)
        .scheme(&opts.scheme)
        .host(&opts.host)
        .port(opts.port)
        .request_timeout(Duration::seconds(opts.request_timeout))
        .wait_timeout(Duration::seconds(opts.wait_timeout));

    let client = Arc::new(client.connect().await?);

    let (parsed, unparsed, existing, not_found) = match opts.subcommand {
        Subcommand::File(file_opts) => {
            use std::fs;

            let contents = fs::read_to_string(file_opts.file)
                .expect("Error reading file");

            let tweet_ids = contents
                .split_whitespace()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(|s| s.parse::<u64>().expect("Error parsing file"))
                .collect::<Vec<u64>>();
            
            let (parsed, unparsed, existing) = match file_opts.batch_size {
                None => scribe_ids(client, &db_conn, &tweet_ids, show_progress).await,
                Some(batch_size) => scribe_ids_batched(client, &db_conn, &tweet_ids, batch_size, show_progress).await
            }?;

            let mut found = HashSet::<u64>::new();
            found.extend(parsed.iter());
            found.extend(unparsed.iter());
            found.extend(existing.iter());

            let not_found = tweet_ids
                .iter()
                .copied()
                .filter(|id| !found.contains(id))
                .collect::<Vec<u64>>();

            (parsed, unparsed, existing, Some(not_found))
        },

        Subcommand::Timeline(timeline_opts) => {
            let handle = timeline_opts.user
                .strip_prefix("@")
                .map(str::to_owned)
                .unwrap_or(timeline_opts.user);
            let user = goldcrest::UserIdentifier::Handle(handle);

            let (parsed, unparsed, existing) = scribe_timeline(&client, &db_conn, user, timeline_opts.page_length, timeline_opts.pages, show_progress)
                .await?;
            
            (parsed, unparsed, existing, None)
        },
    };

    if show_summary {
        if show_progress {
            println!();
            println!("\u{1f916} Done!");
            println!();
        }

        println!("New robot tweets .............. {}", parsed.len());
        println!("Existing robot tweets ......... {}", existing.len());
        println!("Non-robot tweets .............. {}", unparsed.len());

        if let Some(not_found) = not_found {
            println!("Tweets not found by Twitter ... {}", not_found.len());
        }

        if opts.show_new {
            println!();
            println!("New robot tweet IDs: {:?}", parsed);
        }
    }

    Ok(())
}

async fn scribe_timeline(client: &goldcrest::Client, db_conn: &PgConnection, user: goldcrest::UserIdentifier, page_length: u32, pages: usize, show_progress: bool) -> ScribeResult<(Vec<u64>, Vec<u64>, Vec<u64>)> {
    let mut parsed_ids = Vec::new();
    let mut unparsed_ids = Vec::new();
    let mut existing_ids = Vec::new();

    let mut max_id = None;

    for i in 0..pages {
        if show_progress {
            println!("Page {}", i + 1);
        }

        let tweet_options = TweetOptions::default();

        let timeline_options = TimelineOptions::default().count(page_length);
        let timeline_options = match max_id {
            None     => timeline_options,
            Some(id) => timeline_options.max_id(id),
        };

        let mut tweets = client
            .user_timeline(user.clone(), timeline_options, tweet_options, true, true)
            .await?;
        tweets.retain(|tweet| tweet.id > 0);

        if tweets.is_empty() {
            if show_progress {
                println!("Empty page, cannot continue");
            }
            break;
        }

        max_id = Some(tweets
            .iter()
            .map(|tweet| tweet.id)
            .min()
            .unwrap() - 1); //Subtract 1 because, at the time of writing, max_id is inclusive

        let (page_parsed_ids, page_unparsed_ids, page_existing_ids) = scribe_all(db_conn, tweets, show_progress)?;

        parsed_ids.extend(page_parsed_ids.into_iter());
        unparsed_ids.extend(page_unparsed_ids.into_iter());
        existing_ids.extend(page_existing_ids.into_iter());

        if show_progress {
            println!();
        }
    }

    Ok((parsed_ids, unparsed_ids, existing_ids))
}

async fn scribe_ids_batched(client: Arc<goldcrest::Client>, db_conn: &PgConnection, tweet_ids: &[u64], batch_size: usize, show_progress: bool) -> ScribeResult<(Vec<u64>, Vec<u64>, Vec<u64>)> {
    let mut parsed = Vec::new();
    let mut unparsed = Vec::new();
    let mut existing = Vec::new();
    let n = tweet_ids.len();
    let mut i: usize = 0;
    let mut batch_no = 0;
    while i < n {
        if show_progress {
            println!("Batch {}", batch_no + 1);
        }
        let next_i = (i + batch_size).min(n);
        let (batch_parsed, batch_unparsed, batch_existing) = scribe_ids(client.clone(), db_conn, &tweet_ids[i..next_i], show_progress).await?;
        parsed.extend(batch_parsed.into_iter());
        unparsed.extend(batch_unparsed.into_iter());
        existing.extend(batch_existing.into_iter());
        i = next_i;
        batch_no += 1;
    }
    Ok((parsed, unparsed, existing))
}

async fn scribe_ids(client: Arc<goldcrest::Client>, db_conn: &PgConnection, tweet_ids: &[u64], show_progress: bool) -> ScribeResult<(Vec<u64>, Vec<u64>, Vec<u64>)> {
    use tokio::task::JoinHandle;

    let mut tweet_ids = tweet_ids.to_vec();
    tweet_ids.sort();
    tweet_ids.dedup();
    let n_tweet_ids = tweet_ids.len();

    let mut join_handles = Vec::<JoinHandle<ScribeResult<Vec<Tweet>>>>::new();
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
                    .map_err(|err| ScribeError::from(err))
            }));
            assigned = max_id;
        }
    }

    let mut tweets = Vec::new();
    for join_handle in join_handles {
        tweets.extend(join_handle.await??.into_iter());
    }

    scribe_all(db_conn, tweets, show_progress)
}

fn scribe_all(db_conn: &PgConnection, tweets: Vec<Tweet>, show_progress: bool) -> ScribeResult<(Vec<u64>, Vec<u64>, Vec<u64>)> {
    let mut parsed_ids = Vec::new();
    let mut unparsed_ids = Vec::new();
    let mut existing_ids = Vec::new();

    let n_tweets = tweets.len();
    let mut n_tweets_done = 0;
    let mut last_shown = None;

    if show_progress {
        println!();
    }

    for tweet in tweets {
        let tweet_id = tweet.id;
        let res = scribe(db_conn, tweet);
        match res {
            Ok(true) => &mut parsed_ids,
            Ok(false) => &mut unparsed_ids,
            Err(err) => match err {
                ScribeError::DbError(DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => &mut existing_ids,
                ScribeError::TweetAlreadyExists => &mut existing_ids,
                ScribeError::RobotAlreadyExists => &mut existing_ids,
                err => return Err(err),
            },
        }.push(tweet_id);

        n_tweets_done += 1;
        if show_progress && (n_tweets_done == n_tweets || last_shown.is_none() || Utc::now() - last_shown.unwrap() > Duration::milliseconds(250)) {
            println!("{}{}Progress: {} / {}", termion::cursor::Up(1), termion::clear::CurrentLine, n_tweets_done, n_tweets);
            last_shown = Some(Utc::now());
        }
    }

    Ok((parsed_ids, unparsed_ids, existing_ids))
}

fn scribe(db_conn: &PgConnection, tweet: Tweet) -> ScribeResult<bool> {
    let tweet = tweet.original();

    db_conn.transaction::<bool, ScribeError, _>(|| {
        let parse_res = parse_tweet::<_, ScribeResult<()>>(&tweet, |group, robots| {
            // Check for uniqueness after parsing, since parsing is faster than DB access
            if !tweet_unique(db_conn, &tweet)? {
                return Err(ScribeError::TweetAlreadyExists);
            }
            let group_id = group.create(db_conn)?.id;
            for ref robot in robots {
                if !robot_unique(db_conn, robot)? {
                    return Err(ScribeError::RobotAlreadyExists);
                }
                new_robot(robot, group_id, |robot| {
                    robot.create(db_conn)
                })?;
            }
            Ok(())
        });
        match parse_res {
            Some(Ok(())) => Ok(true),
            Some(Err(err)) => Err(err),
            None => Ok(false),
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
