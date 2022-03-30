use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;
use serde::{Serialize, Deserialize};
use sqlx::postgres::PgPool;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::model::{self, IdentBuf};

#[derive(Parser, Debug)]
pub(crate) struct Opts {
    #[clap(subcommand)]
    subcommand: Subcommand,
}

#[derive(Parser, Debug)]
enum Subcommand {
    Import(ImportOpts),
    Export(ExportOpts),
}

#[derive(Parser, Debug)]
struct ImportOpts {
    file: Option<PathBuf>,
}

#[derive(Parser, Debug)]
struct ExportOpts {
    file: Option<PathBuf>,
}

type AltEntries = Vec<AltEntry>;

#[derive(Serialize, Deserialize, Debug)]
struct AltEntry {
    robot: String,
    alt: String,
}

pub(crate) async fn run(db_pool: &PgPool, opts: Opts) -> anyhow::Result<()> {
    match opts.subcommand {
        Subcommand::Import(opts) => import_alt(db_pool, opts).await,
        Subcommand::Export(opts) => export_alt(db_pool, opts).await,
    }
}

async fn import_alt(db_pool: &PgPool, opts: ImportOpts) -> anyhow::Result<()> {
    let input = match opts.file {
        Some(input_path) =>
            tokio::fs::read_to_string(&input_path)
                .await
                .with_context(|| format!("failed to read input file {}", input_path.to_string_lossy()))?,

        None => {
            let mut buf = String::new();
            tokio::io::stdin()
                .read_to_string(&mut buf)
                .await
                .context("failed to read from stdin")?;
            buf
        },
    };

    let entries = serde_json::from_str::<AltEntries>(&input)
        .context("invalid alt text json")?;

    drop(input);

    for entry in entries {
        let id = entry.robot.parse::<IdentBuf>()
            .with_context(|| format!("invalid robot id: {}", entry.robot))?;

        sqlx::query("UPDATE robots SET custom_alt = $1 WHERE id = $2")
            .bind(entry.alt)
            .bind(&id)
            .execute(db_pool)
            .await
            .with_context(|| format!("failed to update alt text for {}", id))?;
    }

    Ok(())
}

async fn export_alt(db_pool: &PgPool, opts: ExportOpts) -> anyhow::Result<()> {
    let robots: Vec<model::RobotCustomAltExport> = sqlx::query_as(
        "SELECT id, custom_alt FROM robots \
        WHERE custom_alt IS NOT NULL"
    )
    .fetch_all(db_pool)
    .await
    .context("failed to get robot alt text from database")?;

    let entries = robots
        .into_iter()
        .map(|robot| AltEntry {
            robot: robot.id.to_string(),
            alt: robot.custom_alt,
        })
        .collect::<Vec<_>>();

    let entries_json = serde_json::to_string(&entries)
        .context("failed to serialize alt text as json")?;

    drop(entries);

    match opts.file {
        Some(output_path) => {
            let mut file = tokio::fs::File::create(&output_path)
                .await
                .with_context(|| format!("failed to open output file {}", output_path.to_string_lossy()))?;

            file
                .write_all(entries_json.as_bytes())
                .await
                .with_context(|| format!("failed to write to output file {}", output_path.to_string_lossy()))?;
        },

        None => {
            let mut stdout = tokio::io::stdout();

            stdout
                .write_all(entries_json.as_bytes())
                .await
                .context("failed to write to stdout")?;
        },
    }

    Ok(())
}
