use sbb_data::schema::*;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;
use std::path::Path;
use diesel::prelude::*;
use clap::{Clap, crate_version, crate_authors, crate_description};

#[derive(Clap)]
#[clap(version = crate_version!(), author = crate_authors!(), about = crate_description!())]
struct Opts {
    /// The file to write the Tweet IDs to. If excluded, write to stdout instead
    #[clap(short, long)]
    output_path: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts: Opts = Opts::parse();

    let db_conn = sbb_data::connect_env()?;

    let tweet_ids: Vec<i64> = robot_groups::table
        .select(robot_groups::tweet_id)
        .distinct()
        .load(&db_conn)?;

    let tweet_ids: Vec<u64> = tweet_ids
        .into_iter()
        .map(|id| id as u64)
        .collect();

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
