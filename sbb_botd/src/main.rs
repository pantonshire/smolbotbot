#[macro_use]
extern crate diesel;

mod error;

use std::env;
use diesel::prelude::*;
use chrono::{prelude::*, Duration};
use clap::{Clap, crate_version, crate_authors, crate_description};

use goldcrest::{TweetOptions, TweetBuilder};

use sbb_data::*;

use error::BotdError;

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
    /// The number of days before a robot group can be selected again after being selected
    #[clap(short, long, default_value = "14")]
    no_repeat_days: i32,
    /// Delete old scheduled dailies
    #[clap(short, long)]
    cleanup: bool,
    /// Print the selected bot rather than Tweeting it or creating a past_dailies record
    #[clap(short, long)]
    dry_run: bool,
}

mod function {
    use diesel::sql_types::*;
    no_arg_sql_function!(random, Integer, "SQL RANDOM() function");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    let opts: Opts = Opts::parse();

    let greetings = lines(include_str!("greetings"));
    let intros = lines(include_str!("intros"));

    let db_conn = sbb_data::connect_env()?;

    let client = if opts.dry_run {
        None
    } else {
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

        Some(runtime.block_on(client.connect())?)
    };

    if opts.cleanup {
        cleanup_old_scheduled(&db_conn)?;
    }

    let no_repeat_days = if opts.no_repeat_days <= 0 {
        None
    } else {
        Some(opts.no_repeat_days)
    };

    let robot = select_robot(&db_conn, no_repeat_days)?;
    let group = group_by_id(&db_conn, robot.robot_group_id)?;

    if let Some(client) = client {
        let greeting = greetings[fastrand::usize(0..greetings.len())];
        let intro = intros[fastrand::usize(0..intros.len())];

        let today = Utc::now().date().naive_utc();
        let today_str = today.format("%d/%m/%y");

        let message = format!("{}\n{} {} #{}, {}!\n{}",
                              today_str,
                              greeting,
                              intro,
                              robot.robot_number,
                              robot.full_name(),
                              group.tweet_link());

        let tweet = TweetBuilder::new(message);

        db_conn.transaction::<(), BotdError, _>(|| {
            record_past_daily(&db_conn, today, robot.id)?;
            runtime.block_on(client.publish(tweet, TweetOptions::default()))?;
            Ok(())
        })?;
    } else {
        println!("\u{1f916} Small Robot of the Day");
        println!("#{}: {}", robot.robot_number, robot.full_name());
        println!("Tweet link: {}", group.tweet_link());
    }

    Ok(())
}

fn lines(contents: &str) -> Vec<&str> {
    contents
        .split("\n")
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect()
}

fn select_robot(db_conn: &PgConnection, no_repeat_days: Option<i32>) -> QueryResult<Robot> {
    if let Some(robot) = scheduled_robot(db_conn)? {
        return Ok(robot);
    }
    random_robot(db_conn, no_repeat_days)
}

fn group_by_id(db_conn: &PgConnection, id: i32) -> QueryResult<RobotGroup> {
    use schema::*;
    robot_groups::table
        .filter(robot_groups::id.eq(id))
        .first(db_conn)
}

fn record_past_daily(db_conn: &PgConnection, date: NaiveDate, robot_id: i32) -> QueryResult<()> {
    NewPastDaily{
        posted_on: date,
        robot_id,
    }.create(db_conn)?;
    Ok(())
}

fn scheduled_robot(db_conn: &PgConnection) -> QueryResult<Option<Robot>> {
    use diesel::dsl::{now, date, exists};
    use schema::*;

    let res = robots::table.filter(
            exists(scheduled_dailies::table
                .filter(robots::id.eq(scheduled_dailies::robot_id)
                    .and(scheduled_dailies::post_on.eq(date(now))))))
        .first(db_conn);

    match res {
        Ok(robot) => Ok(Some(robot)),
        Err(diesel::NotFound) => Ok(None),
        Err(err) => Err(err),
    }
}

fn random_robot(db_conn: &PgConnection, no_repeat_days: Option<i32>) -> QueryResult<Robot> {
    use diesel::dsl::{now, date, not, IntervalDsl};
    use schema::*;

    let recent_groups: Vec<i32> = match no_repeat_days {
        Some(days) => past_dailies::table
            .inner_join(robots::table)
            .filter(past_dailies::posted_on.ge(date(now - days.days())))
            .select(robots::robot_group_id)
            .distinct()
            .load(db_conn)?,
        None => Vec::new(),
    };

    robots::table
        .filter(not(robots::robot_group_id.eq_any(&recent_groups)))
        .order(function::random)
        .first(db_conn)
}

fn cleanup_old_scheduled(db_conn: &PgConnection) -> QueryResult<()> {
    use diesel::dsl::{now, date, delete};
    use schema::*;

    delete(scheduled_dailies::table
            .filter(scheduled_dailies::post_on.lt(date(now))))
        .execute(db_conn)?;

    Ok(())
}
