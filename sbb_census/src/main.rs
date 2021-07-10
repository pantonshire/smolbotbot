use std::fs::File;
use std::io::{Write, BufWriter};
use std::path::Path;
use std::env;

use clap::{Clap, crate_version, crate_authors, crate_description};
use sqlx::{Connection, postgres::PgConnection};

#[derive(Clap)]
#[clap(version = crate_version!(), author = crate_authors!(), about = crate_description!())]
struct Opts {
    /// The file to write the Tweet IDs to. If excluded, write to stdout instead
    #[clap(short, long)]
    output_path: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "dotenv")] {
        dotenv::dotenv().ok();
    }

    let opts: Opts = Opts::parse();

    let db_url = env::var("DATABASE_URL")?;

    let tweet_ids = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(async move {
            match PgConnection::connect(&db_url).await {
                Ok(mut db_conn) => get_tweet_ids(&mut db_conn).await,
                Err(err) => Err(err),
            }
        })?;

    match opts.output_path {
        Some(path) => {
            let path = Path::new(&path);
            let file = File::create(&path)?;
            let mut file = BufWriter::new(&file);
            for tweet_id in tweet_ids {
                writeln!(&mut file, "{}", tweet_id)?;
            }
            file.flush()?;
        },

        None => {
            for tweet_id in tweet_ids {
                println!("{}", tweet_id);
            }
        },
    }
    Ok(())
}

async fn get_tweet_ids(db_conn: &mut PgConnection) -> sqlx::Result<Vec<i64>> {
    sqlx::query!("SELECT DISTINCT tweet_id FROM robot_groups ORDER BY tweet_id ASC")
        .fetch_all(db_conn)
        .await
        .map(|rows| rows
            .into_iter()
            .map(|row| row.tweet_id)
            .collect())
}
