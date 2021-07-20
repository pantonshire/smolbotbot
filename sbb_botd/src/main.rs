use goldcrest::{TweetOptions, TweetBuilder};

use std::env;
use lazy_static::lazy_static;
use chrono::{Utc, NaiveDate, Duration};
use clap::{Clap, crate_version, crate_authors, crate_description};
use sqlx::{Connection, postgres::PgConnection, FromRow};

use services::GoldcrestService;

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
    no_repeat_days: i64,
    /// Delete old scheduled dailies
    #[clap(short, long)]
    cleanup: bool,
    /// Print the selected bot without Tweeting it or creating a past_dailies record
    #[clap(short, long)]
    dry_run: bool,
}

#[derive(FromRow)]
struct DailyRobot {
    id: i32,
    robot_number: i32,
    prefix: String,
    suffix: String,
    plural: Option<String>,
    tweet_id: i64,
    content_warning: Option<String>,
}

impl DailyRobot {
    fn full_name(&self) -> String {
        let mut name_buf = String::with_capacity(
            self.prefix.len()
            + self.suffix.len()
            + self.plural.as_deref().map_or(0, |plural| plural.len())
        );

        name_buf.push_str(&self.prefix);
        name_buf.push_str(&self.suffix);

        if let Some(ref plural) = self.plural {
            name_buf.push_str(plural);
        }

        name_buf
    }

    fn tweet_url(&self) -> String {
        format!("https://twitter.com/smolrobots/status/{}", self.tweet_id)
    }
}

lazy_static! {
    static ref TODAY_UTC: NaiveDate = Utc::now().date().naive_utc();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "dotenv")] {
        dotenv::dotenv().ok();
    }

    let opts: Opts = Opts::parse();

    let greetings = lines(include_str!("greetings"));
    let intros = lines(include_str!("intros"));

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

        Some(client.connect().await?)
    };

    let mut db_conn = {
        let db_url = env::var("DATABASE_URL")?;
        PgConnection::connect(&db_url).await?
    };

    if opts.cleanup {
        cleanup_old_scheduled(&mut db_conn).await?;
    }

    let robot = select_robot(&mut db_conn, opts.no_repeat_days).await?;

    if let Some(client) = client {
        let greeting = greetings[fastrand::usize(0..greetings.len())];
        let intro = intros[fastrand::usize(0..intros.len())];

        let today = Utc::now().date().naive_utc();
        let today_str = today.format("%d/%m/%y");

        let message = {
            let mut message = String::new();

            if let Some(ref content_warning) = robot.content_warning {
                message.push_str("[CW: ");
                message.push_str(content_warning);
                message.push_str("]\n");
            }

            message.push_str(&today_str.to_string());
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

        let mut tx = db_conn.begin().await?;
        record_past_daily(&mut tx, today, robot.id).await?;
        client.publish(tweet, TweetOptions::default()).await?;
        tx.commit().await?;
    } else {
        println!("\u{1f916} Small Robot of the Day");
        println!("#{}: {}", robot.robot_number, robot.full_name());
        println!("Tweet link: {}", robot.tweet_url());
    }

    db_conn.close().await?;

    Ok(())
}

fn lines(contents: &str) -> Vec<&str> {
    contents
        .lines()
        .filter_map(|line| match line.trim() {
            s if s.is_empty() => None,
            s => Some(s),
        })
        .collect()
}

async fn select_robot(db_conn: &mut PgConnection, no_repeat_days: i64) -> sqlx::Result<DailyRobot> {
    match scheduled_robot(db_conn).await? {
        Some(scheduled) => Ok(scheduled),
        None => random_robot(db_conn, no_repeat_days).await,
    }
}

async fn record_past_daily(db_conn: &mut PgConnection, date: NaiveDate, robot_id: i32) -> sqlx::Result<()> {
    sqlx::query!(
        "INSERT INTO past_dailies (posted_on, robot_id) VALUES ($1, $2)",
        date,
        robot_id,
    )
    .execute(db_conn)
    .await
    .map(|_| ())
}

async fn scheduled_robot(db_conn: &mut PgConnection) -> sqlx::Result<Option<DailyRobot>> {
    sqlx::query_as!(
        DailyRobot,
        "SELECT \
            robots.id, robots.robot_number, robots.prefix, robots.suffix, robots.plural, \
            robot_groups.tweet_id, robot_groups.content_warning \
        FROM robots INNER JOIN robot_groups ON robots.group_id = robot_groups.id \
        WHERE EXISTS (\
            SELECT 1 FROM scheduled_dailies \
            WHERE \
                robots.id = scheduled_dailies.robot_id \
                AND scheduled_dailies.post_on = $1) \
        LIMIT 1",
        *TODAY_UTC
    )
    .fetch_optional(db_conn)
    .await
}

async fn random_robot(db_conn: &mut PgConnection, no_repeat_days: i64) -> sqlx::Result<DailyRobot> {
    let reuse_cutoff_date = *TODAY_UTC - Duration::days(no_repeat_days);

    sqlx::query_as!(
        DailyRobot,
        "SELECT \
            robots.id, robots.robot_number, robots.prefix, robots.suffix, robots.plural, \
            robot_groups.tweet_id, robot_groups.content_warning \
        FROM robots INNER JOIN robot_groups ON robots.group_id = robot_groups.id \
        WHERE NOT EXISTS (\
            SELECT 1 FROM past_dailies \
            WHERE \
                past_dailies.robot_id = robots.id \
                AND past_dailies.posted_on >= $1) \
        ORDER BY random() \
        LIMIT 1",
        reuse_cutoff_date
    )
    .fetch_one(db_conn)
    .await
}

async fn cleanup_old_scheduled(db_conn: &mut PgConnection) -> sqlx::Result<u64> {
    sqlx::query!(
        "DELETE FROM scheduled_dailies WHERE post_on < $1",
        *TODAY_UTC
    )
    .execute(db_conn)
    .await
    .map(|qr| qr.rows_affected())
}
