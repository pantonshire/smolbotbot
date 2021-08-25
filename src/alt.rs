use std::path::PathBuf;

use anyhow::Context;
use clap::Clap;
use serde::{Serialize, Deserialize};
use sqlx::postgres::PgPool;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::model;

#[derive(Clap)]
pub(crate) struct Opts {
    #[clap(subcommand)]
    subcommand: Subcommand,
}

#[derive(Clap)]
enum Subcommand {
    Import(ImportOpts),
    Export(ExportOpts),
}

#[derive(Clap)]
struct ImportOpts {
    file: Option<PathBuf>,
}

#[derive(Clap)]
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
        let (number, ident) = entry.robot.split_once('/')
            .with_context(|| format!("invalid number/ident pair: {}", entry.robot))?;

        let number = number.parse::<i32>()
            .with_context(|| format!("invalid robot number: {}", number))?;

        sqlx::query("UPDATE robots SET custom_alt = $1 WHERE (ident, robot_number) = ($2, $3)")
            .bind(entry.alt)
            .bind(ident)
            .bind(number)
            .execute(db_pool)
            .await
            .with_context(|| format!("failed to update alt text for {}/{}", number, ident))?;
    }

    Ok(())
}

async fn export_alt(db_pool: &PgPool, opts: ExportOpts) -> anyhow::Result<()> {
    let robots: Vec<model::RobotCustomAltExport> = sqlx::query_as(
        "SELECT robot_number, ident, custom_alt FROM robots \
        WHERE custom_alt IS NOT NULL"
    )
    .fetch_all(db_pool)
    .await
    .context("failed to get robot alt text from database")?;

    let entries = robots
        .into_iter()
        .map(|robot| AltEntry {
            robot: format!("{}/{}", robot.robot_number, robot.ident),
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
