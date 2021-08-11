use std::io;
use std::error;
use std::fmt::{self, Write};
use std::path::PathBuf;

use anyhow::Context;
use clap::Clap;
use sqlx::postgres::PgPool;
use tokio::io::AsyncWriteExt;

use crate::model;

#[derive(Clap)]
pub(crate) struct Opts {
    /// The file to write the Tweet ids to.
    /// If omitted, they will be written to stdout instead.
    file: Option<PathBuf>,
}

//TODO: test this with flamegraph

pub(crate) async fn run(db_pool: &PgPool, opts: Opts) -> anyhow::Result<()> {
    let ids = fetch_ids(&db_pool).await?;

    match opts.file {
        Some(output_path) => {
            let file = tokio::fs::File::create(&output_path)
                .await
                .with_context(|| format!("failed to open output file {}", output_path.to_string_lossy()))?;

            let mut buf_writer = tokio::io::BufWriter::new(file);
            
            write_ids(&mut buf_writer, &ids)
                .await
                .with_context(|| format!("failed to write to output file {}", output_path.to_string_lossy()))
        },

        None => {
            let mut stdout = tokio::io::stdout();

            write_ids(&mut stdout, &ids)
                .await
                .context("failed to write to stdout")
        },
    }
}

async fn fetch_ids(db_pool: &PgPool) -> anyhow::Result<Vec<model::TweetId>> {
    sqlx::query_as("SELECT DISTINCT tweet_id FROM robots")
        .fetch_all(db_pool)
        .await
        .context("failed to retrieve tweet ids from database")
}

async fn write_ids<W>(writer: &mut W, ids: &[model::TweetId]) -> Result<(), WriteError>
where
    W: AsyncWriteExt + Unpin,
{
    let mut format_buf = String::new();

    for id in ids {
        write!(&mut format_buf, "{}\n", id.tweet_id)?;
        writer.write_all(format_buf.as_bytes()).await?;
        format_buf.clear();
    }

    Ok(())
}

#[derive(Debug)]
enum WriteError {
    FmtError(Box<fmt::Error>),
    IoError(Box<io::Error>),
}

impl fmt::Display for WriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FmtError(err) => err.fmt(f),
            Self::IoError(err) => err.fmt(f),
        }
    }
}

impl error::Error for WriteError {}

impl From<fmt::Error> for WriteError {
    fn from(err: fmt::Error) -> Self {
        Self::FmtError(Box::new(err))
    }
}

impl From<io::Error> for WriteError {
    fn from(err: io::Error) -> Self {
        Self::IoError(Box::new(err))
    }
}
