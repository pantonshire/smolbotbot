use anyhow::Context;
use chrono::{Utc, NaiveDate, Duration};
use clap::Parser;
use goldcrest::{TweetBuilder, TweetOptions};
use rand::seq::SliceRandom;
use sqlx::Executor;
use sqlx::postgres::{PgPool, Postgres};

use crate::model;

#[derive(Parser, Debug)]
pub(crate) struct Opts {
    #[clap(subcommand)]
    subcommand: Subcommand,
}

#[derive(Parser, Debug)]
enum Subcommand {
    /// Tweet the small robot of the day.
    Daily(DailyOpts),
}

#[derive(Parser, Debug)]
struct DailyOpts {
    /// The number of days before a robot group can be selected again after being selected.
    #[clap(short, long, default_value = "14")]
    no_repeat_days: i64,

    /// Delete old scheduled dailies.
    #[clap(short, long)]
    cleanup: bool,
}

pub(crate) async fn run(
    db_pool: &PgPool,
    au_client: &goldcrest::Client,
    opts: Opts
) -> anyhow::Result<()>
{
    match opts.subcommand {
        Subcommand::Daily(daily_opts) => {
            let now = Utc::now();
            let today = now.date().naive_utc();

            let greetings = lines(include_str!("data/greetings"));
            let intros = lines(include_str!("data/intros"));

            if daily_opts.cleanup {
                cleanup_old_scheduled(db_pool, today)
                    .await
                    .context("failed to clean up old scheduled daily robots")?;
            }

            let robot = match scheduled_robot(db_pool, today)
                .await
                .context("failed to check for scheduled daily robots")?
            {
                Some(scheduled) => scheduled,
                None => random_robot(db_pool, today, daily_opts.no_repeat_days)
                    .await
                    .context("failed to select random daily robot")?,
            };

            let today_string = today.format("%d/%m/%y").to_string();

            let (greeting, intro) = {
                let mut rng = rand::thread_rng();

                let greeting = greetings.choose(&mut rng)
                    .map(|greeting| *greeting)
                    .unwrap_or("");

                let intro = intros.choose(&mut rng)
                    .map(|intro| *intro)
                    .unwrap_or("");

                (greeting, intro)
            };

            let message = {
                let mut message = String::new();
    
                if let Some(ref content_warning) = robot.content_warning {
                    message.push_str("[CW: ");
                    message.push_str(content_warning);
                    message.push_str("]\n");
                }

                message.push_str(&today_string);
                message.push('\n');
                message.push_str(greeting);
                message.push(' ');
                message.push_str(intro);
                message.push_str(" #");
                message.push_str(&robot.robot_number.to_string());
                message.push_str(", ");
                message.push_str(&robot.full_name());
                message.push_str("!\n");
                message.push_str(&robot.tweet_url());
    
                message
            };
            
            let tweet = TweetBuilder::new(message);

            au_client.publish(tweet, TweetOptions::default())
                .await
                .context("failed to send tweet")?;

            record_past_daily(db_pool, today, robot.id)
                .await
                .context("failed to store daily robot in database")?;

            Ok(())
        }
    }
}

fn lines(text: &str) -> Vec<&str> {
    text
        .lines()
        .filter_map(|line| match line.trim() {
            s if s.is_empty() => None,
            s => Some(s),
        })
        .collect()
}

async fn record_past_daily<'e, E>(
    db_exec: E,
    date: NaiveDate,
    robot_id: i32
) -> sqlx::Result<()>
where
    E: Executor<'e, Database = Postgres>
{
    sqlx::query(
        "INSERT INTO past_dailies (posted_on, robot_id) VALUES ($1, $2)"
    )
    .bind(date)
    .bind(robot_id)
    .execute(db_exec)
    .await
    .map(|_| ())
}

async fn scheduled_robot<'e, E>(
    db_exec: E,
    today: NaiveDate
) -> sqlx::Result<Option<model::DailyRobot>>
where
    E: Executor<'e, Database = Postgres>
{
    sqlx::query_as(
        "SELECT \
            id, robot_number, prefix, suffix, plural, tweet_id, content_warning \
        FROM robots \
        WHERE EXISTS (\
            SELECT 1 FROM scheduled_dailies \
            WHERE \
                robots.id = scheduled_dailies.robot_id \
                AND scheduled_dailies.post_on = $1) \
        LIMIT 1",
    )
    .bind(today)
    .fetch_optional(db_exec)
    .await
}

async fn random_robot<'e, E>(
    db_exec: E,
    today: NaiveDate,
    no_repeat_days: i64
) -> sqlx::Result<model::DailyRobot>
where
    E: Executor<'e, Database = Postgres>
{
    let reuse_cutoff_date = today - Duration::days(no_repeat_days);

    sqlx::query_as(
        "SELECT \
            id, robot_number, prefix, suffix, plural, tweet_id, content_warning \
        FROM robots \
        WHERE NOT EXISTS (\
            SELECT 1 FROM past_dailies \
            WHERE \
                past_dailies.robot_id = robots.id \
                AND past_dailies.posted_on >= $1) \
        ORDER BY random() \
        LIMIT 1"
    )
    .bind(reuse_cutoff_date)
    .fetch_one(db_exec)
    .await
}

async fn cleanup_old_scheduled<'e, E>(
    db_exec: E,
    today: NaiveDate
) -> sqlx::Result<u64>
where
    E: Executor<'e, Database = Postgres>
{
    sqlx::query(
        "DELETE FROM scheduled_dailies WHERE post_on < $1"
    )
    .bind(today)
    .execute(db_exec)
    .await
    .map(|qr| qr.rows_affected())
}
