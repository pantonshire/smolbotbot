mod parse;
mod fetch;
mod export;
mod timeline;
mod images;
mod post;
mod scribe;
mod model;
mod error;
mod plural;

use std::default::Default;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;

use anyhow::Context;
use chrono::Duration;
use clap::{Clap, crate_version, crate_authors, crate_description};
use serde::Deserialize;
use sqlx::postgres::PgPool;

use error::{InvalidVarError, MissingVarError};

#[derive(Clap)]
#[clap(version = crate_version!(), author = crate_authors!(), about = crate_description!())]
struct Opts {
    #[clap(short, long)]
    config: Option<PathBuf>,

    #[clap(subcommand)]
    command: MainCommand,
}

#[derive(Clap)]
enum MainCommand {
    /// Retrieve, parse and store a list of robot Tweets.
    Fetch(fetch::Opts),

    /// Output a list of stored Tweet ids.
    Export(export::Opts),

    /// Read a user's timeline, searching for new robot Tweets.
    Timeline(timeline::Opts),

    /// Download robot images and/or generate thumbnails
    Image(images::Opts),

    /// Post a new Tweet.
    Post(post::Opts),
}

#[derive(Deserialize, Default)]
struct Config {
    database: Option<DatabaseConfig>,
    goldcrest: Option<GoldcrestConfig>,
}

#[derive(Deserialize, Default)]
struct DatabaseConfig {
    url: Option<String>,
}

#[derive(Deserialize, Default)]
struct GoldcrestConfig {
    scheme: Option<String>,
    host: Option<String>,
    port: Option<u16>,
    auth: Option<OAuth10aConfig>,
    request_timeout_seconds: Option<u32>,
    wait_timeout_seconds: Option<u32>,
}

#[derive(Deserialize, Default)]
struct OAuth10aConfig {
    consumer_key: Option<String>,
    consumer_secret: Option<String>,
    token: Option<String>,
    token_secret: Option<String>,
}

const DEFAULT_CONFIG_PATH: &str = "smolbotbot.yaml";

const VAR_CONFIG_PATH: &str = "SBB_CONFIG";

const VAR_DB_URL: &str = "DATABASE_URL";

const VAR_GOLDCREST_SCHEME: &str = "GOLDCREST_SCHEME";
const VAR_GOLDCREST_HOST: &str = "GOLDCREST_HOST";
const VAR_GOLDCREST_PORT: &str = "GOLDCREST_PORT";
const VAR_GOLDCREST_REQUEST_TIMEOUT: &str = "GOLDCREST_REQUEST_TIMEOUT";
const VAR_GOLDCREST_WAIT_TIMEOUT: &str = "GOLDCREST_WAIT_TIMEOUT";

const VAR_TWITTER_CONSUMER_KEY: &str = "TWITTER_CONSUMER_KEY";
const VAR_TWITTER_CONSUMER_SECRET: &str = "TWITTER_CONSUMER_SECRET";
const VAR_TWITTER_TOKEN: &str = "TWITTER_TOKEN";
const VAR_TWITTER_TOKEN_SECRET: &str = "TWITTER_TOKEN_SECRET";

fn main() -> anyhow::Result<()> {
    #[cfg(feature = "dotenv")] {
        dotenv::dotenv().ok();
    }

    let opts = Opts::parse();

    let config = match opts.config.as_deref() {
        Some(config_path) => load_config(config_path)
            .with_context(|| format!("failed to read config file {}", config_path.to_string_lossy()))?
            .with_context(|| format!("failed to parse config file {}", config_path.to_string_lossy()))?,
        
        None => match env::var_os(VAR_CONFIG_PATH) {
            Some(config_path) => load_config(config_path.as_ref())
                .with_context(|| format!("failed to read config file {}", config_path.to_string_lossy()))?
                .with_context(|| format!("failed to parse config file {}", config_path.to_string_lossy()))?,
            
            None => if cfg!(feature = "default-config-file") {
                load_config(DEFAULT_CONFIG_PATH.as_ref())
                    .ok()
                    .map(|res| res
                        .with_context(|| format!("failed to parse config file {}", DEFAULT_CONFIG_PATH)))
                    .transpose()?
                    .unwrap_or_default()
            } else {
                Config::default()
            },
        },
    };

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("failed to create tokio runtime")?
        .block_on(run(opts, config))
}

async fn run(opts: Opts, config: Config) -> anyhow::Result<()> {
    match opts.command {
        MainCommand::Fetch(opts) => {
            let db_pool = connect_db(config.database.unwrap_or_default()).await?;
            let au_client = Arc::new(connect_goldcrest(config.goldcrest.unwrap_or_default()).await?);
            let res = fetch::run(&db_pool, au_client, opts).await;
            db_pool.close().await;
            res
        },

        MainCommand::Export(opts) => {
            let db_pool = connect_db(config.database.unwrap_or_default()).await?;
            let res = export::run(&db_pool, opts).await;
            db_pool.close().await;
            res
        },

        MainCommand::Timeline(opts) => {
            let db_pool = connect_db(config.database.unwrap_or_default()).await?;
            let au_client = connect_goldcrest(config.goldcrest.unwrap_or_default()).await?;
            let res = timeline::run(&db_pool, &au_client, opts).await;
            db_pool.close().await;
            res
        },

        MainCommand::Image(opts) => {
            let db_pool = connect_db(config.database.unwrap_or_default()).await?;
            let res = images::run(&db_pool, opts).await;
            db_pool.close().await;
            res
        },

        MainCommand::Post(opts) => {
            let db_pool = connect_db(config.database.unwrap_or_default()).await?;
            let au_client = connect_goldcrest(config.goldcrest.unwrap_or_default()).await?;
            let res = post::run(&db_pool, &au_client, opts).await;
            db_pool.close().await;
            res
        },
    }
}

fn load_config(path: &Path) -> io::Result<serde_yaml::Result<Config>> {
    fs::read_to_string(path)
        .map(|contents| serde_yaml::from_str(&contents))
}

async fn connect_db(config: DatabaseConfig) -> anyhow::Result<PgPool> {
    let db_url = env_var(VAR_DB_URL)?
        .or(config.url)
        .ok_or(MissingVarError("database url"))?;

    // PgPoolOptions::new()
    //     .connect_with(connect_opts)
    //     .await
    //     .with_context(|| format!("failed to connect to database at {}", db_url))

    PgPool::connect(&db_url)
        .await
        .with_context(|| format!("failed to connect to database at {}", db_url))
}

async fn connect_goldcrest(config: GoldcrestConfig) -> anyhow::Result<goldcrest::Client> {
    let mut client_builder = goldcrest::ClientBuilder::new();

    let auth = get_twitter_auth(config.auth.unwrap_or_default())?;
    client_builder.authenticate(auth);

    if let Some(scheme) = env_var(VAR_GOLDCREST_SCHEME)
        .context("failed to read goldcrest scheme")?
        .or(config.scheme)
    {
        client_builder.scheme(&scheme);
    }

    if let Some(host) = env_var(VAR_GOLDCREST_HOST)
        .context("failed to read goldcrest host")?
        .or(config.host)
    {
        client_builder.host(&host);
    }

    if let Some(port) = env_var_parse::<u16>(VAR_GOLDCREST_PORT)
        .context("failed to read goldcrest port")?
        .or(config.port)
    {
        client_builder.port(port as u32);
    }

    if let Some(timeout) = env_var_parse::<u32>(VAR_GOLDCREST_REQUEST_TIMEOUT)
        .context("failed to read goldcrest request timeout")?
        .or(config.request_timeout_seconds)
    {
        client_builder.request_timeout(Duration::seconds(timeout as i64));
    }

    if let Some(timeout) = env_var_parse::<u32>(VAR_GOLDCREST_WAIT_TIMEOUT)
        .context("failed to read goldcrest wait timeout")?
        .or(config.wait_timeout_seconds)
    {
        client_builder.wait_timeout(Duration::seconds(timeout as i64));
    }

    client_builder
        .connect()
        .await
        .context("failed to connect to goldcrest")
}

fn get_twitter_auth(config: OAuth10aConfig) -> anyhow::Result<goldcrest::Authentication> {
    let consumer_key = env_var(VAR_TWITTER_CONSUMER_KEY)
        .context("failed to read twitter consumer key")?
        .or(config.consumer_key)
        .ok_or(MissingVarError("twitter consumer key"))?;

    let consumer_secret = env_var(VAR_TWITTER_CONSUMER_SECRET)
        .context("failed to read twitter consumer secret")?
        .or(config.consumer_secret)
        .ok_or(MissingVarError("twitter consumer secret"))?;

    let token = env_var(VAR_TWITTER_TOKEN)
        .context("failed to read twitter token")?
        .or(config.token)
        .ok_or(MissingVarError("twitter token"))?;

    let token_secret = env_var(VAR_TWITTER_TOKEN_SECRET)
        .context("failed to read twitter token secret")?
        .or(config.token_secret)
        .ok_or(MissingVarError("twitter token secret"))?;

    Ok(goldcrest::Authentication::new(consumer_key, consumer_secret, token, token_secret))
}

fn env_var(key: &str) -> Result<Option<String>, InvalidVarError> {
    match env::var(key) {
        Ok(val) => Ok(Some(val)),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(env::VarError::NotUnicode(bad_str)) => Err(InvalidVarError::invalid_utf8(bad_str)),
    }
}

fn env_var_parse<T>(key: &str) -> Result<Option<T>, InvalidVarError>
where
    T: FromStr,
{
    env_var(key)
        .and_then(|val| val
            .map(|val| val
                .parse::<T>()
                .map_err(|_| InvalidVarError::parse_error(val.into())))
            .transpose())
}
