#[macro_use]
extern crate diesel;

mod error;

use error::BotdError;

use services::GoldcrestService;
use sbb_data::*;

use goldcrest::{TweetOptions, TweetBuilder};

use std::env;
use diesel::prelude::*;
use chrono::{prelude::*, Duration};
use clap::{Clap, crate_version, crate_authors, crate_description};

#[derive(Clap)]
#[clap(version = crate_version!(), author = crate_authors!(), about = crate_description!())]
struct Opts {
    /// The path to the services YAML. If omitted, "services.yaml" will be used.
    #[clap(long)]
    services: Option<String>,
    /// The services YAML key corresponding to the Goldcrest Twitter authentication data to use.
    /// If omitted, the key "default" will be used.
    #[clap(long)]
    goldcrest_auth: Option<String>,
    /// The number of days before a robot group can be selected again after being selected
    #[clap(short, long, default_value = "14")]
    no_repeat_days: i32,
    /// Delete old scheduled dailies
    #[clap(short, long)]
    cleanup: bool,
    /// Print the selected bot without Tweeting it or creating a past_dailies record
    #[clap(short, long)]
    dry_run: bool,
}

mod function {
    use diesel::sql_types::*;
    no_arg_sql_function!(random, Integer, "SQL RANDOM() function");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts: Opts = Opts::parse();

    let greetings = lines(include_str!("greetings"));
    let intros = lines(include_str!("intros"));

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    let client = if opts.dry_run {
        None
    } else {
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

        Some(runtime.block_on(client.connect())?)
    };

    let db_conn = sbb_data::connect_env()?;

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

fn group_by_id(db_conn: &PgConnection, id: i64) -> QueryResult<RobotGroup> {
    use schema::*;
    robot_groups::table
        .filter(robot_groups::id.eq(id))
        .first(db_conn)
}

fn record_past_daily(db_conn: &PgConnection, date: NaiveDate, robot_id: i64) -> QueryResult<()> {
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

    let recent_groups: Vec<i64> = match no_repeat_days {
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
