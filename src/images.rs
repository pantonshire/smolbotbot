use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::fmt;
use std::error;
use std::io;
use std::time::Duration;

use anyhow::{anyhow, Context};
use clap::Parser;
use governor::{Quota, RateLimiter};
use image::{ImageFormat, DynamicImage, GenericImageView, ImageEncoder};
use image::imageops::FilterType;
use image::codecs::jpeg;
use nonzero_ext::nonzero;
use rand::Rng;
use reqwest::StatusCode;
use sqlx::postgres::{PgPool, PgConnection};
use tokio::io::AsyncReadExt;
use tokio::sync::Semaphore;
use url::Url;

use crate::model::{RobotImageUrl, RobotImagePath, RobotImagePathOpt};

#[derive(Parser, Debug)]
pub(crate) struct Opts {
    /// If set, download the images.
    #[clap(short, long = "download")]
    download: bool,

    /// If set, generate image thumbnails.
    #[clap(short, long = "thumb")]
    thumb: bool,

    /// The width and height of the thumbnails to generate, in pixels.
    #[clap(long, default_value = "128")]
    thumb_size: u32,

    /// The timeout in seconds for connecting to the image server. If not set, there is no timeout.
    #[clap(long)]
    connect_timeout: Option<u64>,

    /// The timeout in seconds for a request to complete. If not set, there is no timeout.
    #[clap(long)]
    request_timeout: Option<u64>,

    /// If set, use the this directory for storing / retrieving images instead of the current working
    /// directory.
    dir: Option<PathBuf>,

    #[clap(subcommand)]
    subcommand: Subcommand,
}

#[derive(Parser, Debug)]
enum Subcommand {
    Ids,
    Missing,
}

pub(crate) async fn run(db_pool: &PgPool, opts: Opts) -> anyhow::Result<()> {
    // Exit early if the user did not specify anything to do
    if !opts.download && !opts.thumb {
        return Err(anyhow!("neither -d nor -t flags provided, nothing to do"));
    }

    let dir = opts.dir.map(Arc::new);

    let mut all_succeeded = true;

    let robot_paths = match opts.download {
        // Download the images and store the paths in the database
        true => {
            let robots = {
                let mut db_conn = db_pool.acquire().await?;

                match opts.subcommand {
                    Subcommand::Ids => {
                        let robot_ids = read_stdin_ids()
                            .await
                            .context("failed to read robot ids from stdin")?;

                        get_image_urls(&mut db_conn, &robot_ids)
                            .await
                            .context("failed to retrieve robot data from database")?
                    },

                    Subcommand::Missing =>
                        get_image_urls_missing(&mut db_conn)
                            .await
                            .context("failed to retrieve robot data from database")?,
                }
            };

            let http_client = {
                let mut builder = reqwest::ClientBuilder::new();
                if let Some(connect_timeout) = opts.connect_timeout {
                    builder = builder.connect_timeout(Duration::from_secs(connect_timeout));
                }
                if let Some(request_timeout) = opts.request_timeout {
                    builder = builder.timeout(Duration::from_secs(request_timeout));
                }
                builder
                    .build()
                    .context("failed to create http client")?
            };

            let image_results = get_images(
                &db_pool,
                &http_client,
                dir.clone(),
                robots
            ).await;
            
            let mut successful_robots = Vec::new();
            for res in image_results {
                match res {
                    Ok(robot) => successful_robots.push(robot),
                    Err(err) => {
                        all_succeeded = false;
                        eprintln!("{}", err);
                    }
                }
            }
            
            successful_robots
        },

        // Retrieve the paths of the pre-downloaded images from the database
        false => {
            let mut db_conn = db_pool.acquire().await?;
            
            let opt_robots = match opts.subcommand {
                Subcommand::Ids => {
                    let robot_ids = read_stdin_ids()
                        .await
                        .context("failed to read robot ids from stdin")?;

                    get_image_paths(&mut db_conn, &robot_ids)
                        .await
                        .context("failed to retrieve robot data from database")?
                }

                Subcommand::Missing =>
                    get_image_paths_missing(&mut db_conn)
                        .await
                        .context("failed to retrieve robot data from database")?,
            };

            let mut robots = Vec::new();
            for opt_robot in opt_robots {
                match opt_robot.image_path {
                    Some(image_path) => robots.push(RobotImagePath {
                        id: opt_robot.id,
                        image_path,
                    }),

                    None => {
                        all_succeeded = false;
                        eprintln!("robot {} has no image to generate a thumb from", opt_robot.id);
                    },
                }
            }

            robots
        },
    };

    // Generate image thumbs and store the paths in the database
    if opts.thumb {
        let thumb_results = gen_thumbs(
            &db_pool,
            robot_paths,
            dir,
            opts.thumb_size
        ).await;

        for res in thumb_results {
            if let Err(err) = res {
                all_succeeded = false;
                eprintln!("{}", err);
            }
        }
    }

    match all_succeeded {
        true => Ok(()),
        false => Err(anyhow!("failed for some robots")),
    }
}

// Reads a list of robot robot ids from stdin.
async fn read_stdin_ids() -> anyhow::Result<Vec<i32>> {
    let mut buffer = String::new();
    tokio::io::stdin()
        .read_to_string(&mut buffer)
        .await?;
    
    buffer
        .split_whitespace()
        .map(|id| id
            .parse::<i32>()
            .map_err(|_| anyhow!("invalid robot robot id \"{}\"", id)))
        .collect::<Result<Vec<_>, _>>()
}

/// Get the image urls of all of the robots with the given ids.
async fn get_image_urls(
    db_conn: &mut PgConnection,
    robot_ids: &[i32]
) -> sqlx::Result<Vec<RobotImageUrl>>
{
    sqlx::query_as("SELECT id, image_url FROM robots WHERE id = ANY($1)")
        .bind(robot_ids)
        .fetch_all(db_conn)
        .await
}

/// Get the image urls of all of the robots which have no image path in the database.
async fn get_image_urls_missing(
    db_conn: &mut PgConnection
) -> sqlx::Result<Vec<RobotImageUrl>>
{
    sqlx::query_as("SELECT id, image_url FROM robots WHERE image_path IS NULL")
        .fetch_all(db_conn)
        .await
}

/// Get the image paths of all of the robots with the given ids.
async fn get_image_paths(
    db_conn: &mut PgConnection,
    robot_ids: &[i32]
) -> sqlx::Result<Vec<RobotImagePathOpt>>
{
    sqlx::query_as("SELECT id, image_path FROM robots WHERE id = ANY($1)")
        .bind(robot_ids)
        .fetch_all(db_conn)
        .await
}

/// Get the image paths of all of the robots which have no image thumb path in the database.
async fn get_image_paths_missing(
    db_conn: &mut PgConnection
) -> sqlx::Result<Vec<RobotImagePathOpt>>
{
    sqlx::query_as("SELECT id, image_path FROM robots WHERE image_thumb_path IS NULL")
        .fetch_all(db_conn)
        .await
}

async fn get_images(
    db_pool: &PgPool,
    http_client: &reqwest::Client,
    dir: Option<Arc<PathBuf>>,
    robots: Vec<RobotImageUrl>
) -> Vec<Result<RobotImagePath, ImgError>>
{
    const MAX_CONCURRENT: usize = 16;
    const REQUESTS_PER_SECOND: u32 = 10;

    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT));

    let quota = Quota::per_second(nonzero!(REQUESTS_PER_SECOND));
    let limiter = Arc::new(RateLimiter::direct(quota));

    let mut join_handles = Vec::with_capacity(robots.len());

    for robot in robots {
        let semaphore = semaphore.clone();
        let limiter = limiter.clone();
        let db_pool = db_pool.clone();
        let http_client = http_client.clone();
        let dir = dir.clone();

        let file_name = gen_image_file_name("orig", robot.id, "png");

        join_handles.push((robot.id, tokio::spawn(async move {
            match file_name {
                Ok(file_name) => match semaphore.acquire().await {
                    Ok(_permit) => {
                        limiter.until_ready().await;
                        download_and_store(&db_pool, &http_client, &robot, dir.as_deref(), &file_name)
                            .await
                            .map(move |_| RobotImagePath {
                                id: robot.id,
                                image_path: file_name.to_owned(),
                            })
                    },

                    Err(err) => Err(ImgError::new(robot.id, err.into())),
                },

                Err(err) => Err(ImgError::new(robot.id, ImgErrorCause::GenFileNameError(Box::new(err)))),
            }
        })));
    }

    let mut results = Vec::with_capacity(join_handles.len());

    for (robot_id, join_handle) in join_handles {
        results.push(join_handle
            .await
            .unwrap_or_else(|err| Err(ImgError::new(robot_id, err.into()))));
    }

    results
}

async fn download_and_store<P>(
    db_pool: &PgPool,
    http_client: &reqwest::Client,
    robot: &RobotImageUrl,
    dir: Option<P>,
    file_name: &str,
) -> Result<(), ImgError>
where
    P: AsRef<Path>
{
    match dir.as_ref() {
        Some(dir) =>
            download_image(http_client, robot, dir.as_ref().join(file_name)).await,
        None =>
            download_image(http_client, robot, file_name).await,
    }?;
    
    let mut db_conn = db_pool
        .acquire()
        .await
        .map_err(|err| ImgError::new(robot.id, err.into()))?;

    store_image_path(&mut db_conn, robot, file_name).await
}

async fn download_image<P>(
    http_client: &reqwest::Client,
    robot: &RobotImageUrl,
    path: P,
) -> Result<(), ImgError>
where
    P: AsRef<Path>
{
    let image_url = image_large_png_url(&robot.image_url)
        .map_err(|err| ImgError::new(robot.id, err.into()))?;

    let resp = http_client.get(image_url)
        .send()
        .await
        .map_err(|err| ImgError::new(robot.id, err.into()))?;

    match resp.status() {
        status if status.is_success() => {
            let image_data = resp
                .bytes()
                .await
                .map_err(|err| ImgError::new(robot.id, err.into()))?;

            tokio::fs::write(path, &image_data)
                .await
                .map_err(|err| ImgError::new(robot.id, err.into()))
        },

        status => Err(ImgError::new(robot.id, ImgErrorCause::HttpError(status))),
    }
}

async fn store_image_path(
    db_conn: &mut PgConnection,
    robot: &RobotImageUrl,
    file_name: &str,
) -> Result<(), ImgError>
{
    let rows_affected = sqlx::query("UPDATE robots SET image_path = $1 WHERE id = $2")
        .bind(file_name)
        .bind(robot.id)
        .execute(db_conn)
        .await
        .map_err(|err| ImgError::new(robot.id, err.into()))?
        .rows_affected();

    if rows_affected < 1 {
        Err(ImgError::new(robot.id, ImgErrorCause::NoRowsUpdated))
    } else {
        Ok(())
    }
}

async fn gen_thumbs(
    db_pool: &PgPool,
    robots: Vec<RobotImagePath>,
    dir: Option<Arc<PathBuf>>,
    thumb_size: u32
) -> Vec<Result<(), ImgError>>
{
    const MAX_CONCURRENT: usize = 16;
    const GRAYSCALE_THRESHOLD: f32 = 0.005;
    const JPEG_QUALITY: u8 = 50;

    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT));
    let mut join_handles = Vec::with_capacity(robots.len());

    for robot in robots {
        let semaphore = semaphore.clone();
        let db_pool = db_pool.clone();
        let dir = dir.clone();

        let file_name = gen_image_file_name("thumb", robot.id, "jpg");

        join_handles.push((robot.id, tokio::spawn(async move {
            match file_name {
                Ok(file_name) => match semaphore.acquire().await {
                    Ok(_permit) => gen_thumb(
                        &db_pool,
                        &robot,
                        dir.as_deref(),
                        &file_name,
                        thumb_size,
                        JPEG_QUALITY,
                        GRAYSCALE_THRESHOLD
                    ).await,

                    Err(err) => Err(ImgError::new(robot.id, err.into())),
                },

                Err(err) => Err(ImgError::new(robot.id, ImgErrorCause::GenFileNameError(Box::new(err)))),
            }
        })));
    }

    let mut results = Vec::with_capacity(join_handles.len());

    for (robot_id, join_handle) in join_handles {
        results.push(join_handle
            .await
            .unwrap_or_else(|err| Err(ImgError::new(robot_id, err.into()))));
    }

    results
}

async fn gen_thumb<P>(
    db_pool: &PgPool,
    robot: &RobotImagePath,
    dir: Option<P>,
    file_name: &str,
    size: u32,
    quality: u8,
    grayscale_threshold: f32
) -> Result<(), ImgError>
where
    P: AsRef<Path>
{
    let original = {
        let image_data = match dir.as_ref(){
            Some(dir) => tokio::fs::read(dir.as_ref().join(&robot.image_path)).await,
            None => tokio::fs::read(&robot.image_path).await,
        }.map_err(|err| ImgError::new(robot.id, err.into()))?;

        match ImageFormat::from_path(&robot.image_path).ok() {
            Some(image_format) => image::load_from_memory_with_format(&image_data, image_format),
            None => image::load_from_memory(&image_data),
        }.map_err(|err| ImgError::new(robot.id, err.into()))?
    };

    let thumb = original.resize_to_fill(size, size, FilterType::Lanczos3);

    let thumb = match is_approx_grayscale(&thumb, grayscale_threshold) {
        true => DynamicImage::ImageLuma8(thumb.into_luma8()),
        false => DynamicImage::ImageRgb8(thumb.into_rgb8()),
    };

    let mut buffer = Vec::new();
    
    jpeg::JpegEncoder::new_with_quality(&mut buffer, quality)
        .write_image(thumb.as_bytes(), thumb.width(), thumb.height(), thumb.color())
        .map_err(|err| ImgError::new(robot.id, err.into()))?;

    drop(thumb);

    match dir.as_ref() {
        Some(dir) => tokio::fs::write(dir.as_ref().join(file_name), buffer).await,
        None => tokio::fs::write(file_name, buffer).await,
    }.map_err(|err| ImgError::new(robot.id, err.into()))?;

    let mut db_conn = db_pool
        .acquire()
        .await
        .map_err(|err| ImgError::new(robot.id, err.into()))?;

    let rows_affected = sqlx::query("UPDATE robots SET image_thumb_path = $1 WHERE id = $2")
        .bind(file_name)
        .bind(robot.id)
        .execute(&mut db_conn)
        .await
        .map_err(|err| ImgError::new(robot.id, err.into()))?
        .rows_affected();

    if rows_affected < 1 {
        Err(ImgError::new(robot.id, ImgErrorCause::NoRowsUpdated))
    } else {
        Ok(())
    }
}

fn is_approx_grayscale(image: &DynamicImage, threshold: f32) -> bool {
    const STRIDE: u32 = 16;
    const CHANNEL_MAX: f32 = 255.0;
    const INV_SQRT3: f32 = 0.577_350_26;

    let channel_sum = image
        .pixels()
        .filter(|(x, y, _)| x % STRIDE == 0 && y % STRIDE == 0)
        .fold((0f32, 0f32, 0f32), |(r, g, b), (_, _, pixel)| (
            r + pixel[0] as f32 / CHANNEL_MAX,
            g + pixel[1] as f32 / CHANNEL_MAX,
            b + pixel[2] as f32 / CHANNEL_MAX
        ));

    let magnitude = (
        (channel_sum.0 * channel_sum.0) 
        + (channel_sum.1 * channel_sum.1) 
        + (channel_sum.2 * channel_sum.2)
    ).sqrt();

    ((channel_sum.0 / magnitude) - INV_SQRT3).abs() < threshold
    && ((channel_sum.1 / magnitude) - INV_SQRT3).abs() < threshold
    && ((channel_sum.2 / magnitude) - INV_SQRT3).abs() < threshold
}

fn image_large_png_url(url: &str) -> Result<Url, url::ParseError> {
    let mut image_url = Url::parse(url)?;

    // Path will always start with a slash unless the URL is a cannot-be-a-base URL
    let url_path = image_url.path().to_owned();

    // Remove the extension from the last part of the path
    let path_no_extension = url_path
        .rfind('/')
        .map(|last_slash| last_slash + 1)
        .and_then(|last_path_component| url_path[last_path_component ..]
            .rfind('.')
            .map(|last_dot| &url_path[.. last_path_component + last_dot]));

    if let Some(new_path) = path_no_extension {
        image_url.set_path(new_path);
    }

    image_url.query_pairs_mut()
        .clear()
        .append_pair("format", "png")
        .append_pair("name", "large");

    Ok(image_url)
}

fn gen_image_file_name(image_type: &str, id: i32, extension: &str) -> Result<String, fmt::Error> {
    use fmt::Write;

    let mut name = String::new();

    name.push_str(image_type);
    name.push('_');

    write!(&mut name, "{}", id)?;
    name.push('_');
    
    let rand_token = rand::thread_rng().gen::<u128>();
    write!(&mut name, "{:x}", rand_token)?;

    name.push('.');
    name.push_str(extension);

    Ok(name)
}

#[derive(Debug)]
struct ImgError {
    robot_id: i32,
    cause: ImgErrorCause,
}

#[derive(Debug)]
enum ImgErrorCause {
    RequestError(Box<reqwest::Error>),
    ImageError(Box<image::ImageError>),
    IoError(Box<io::Error>),
    DbError(Box<sqlx::Error>),
    SemaphoreError(Box<tokio::sync::AcquireError>),
    TaskPanicked(Box<tokio::task::JoinError>),
    InvalidUrl(Box<url::ParseError>),
    HttpError(StatusCode),
    NoRowsUpdated,
    GenFileNameError(Box<fmt::Error>),
}

impl ImgError {
    const fn new(robot_id: i32, cause: ImgErrorCause) -> Self {
        ImgError {
            robot_id,
            cause,
        }
    }
}

impl fmt::Display for ImgError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error at robot {}: {}", self.robot_id, self.cause)
    }
}

impl fmt::Display for ImgErrorCause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RequestError(err) => err.fmt(f),
            Self::ImageError(err) => err.fmt(f),
            Self::IoError(err) => err.fmt(f),
            Self::DbError(err) => err.fmt(f),
            Self::SemaphoreError(err) => err.fmt(f),
            Self::TaskPanicked(err) => err.fmt(f),
            Self::InvalidUrl(err) => err.fmt(f),
            Self::HttpError(status) => status.fmt(f),
            Self::NoRowsUpdated => write!(f, "no rows affected by update"),
            Self::GenFileNameError(err) => write!(f, "error generating file name: {}", err),
        }
    }
}

impl error::Error for ImgError {}

impl From<reqwest::Error> for ImgErrorCause {
    fn from(err: reqwest::Error) -> Self {
        Self::RequestError(Box::new(err))
    }
}

impl From<image::ImageError> for ImgErrorCause {
    fn from(err: image::ImageError) -> Self {
        Self::ImageError(Box::new(err))
    }
}

impl From<io::Error> for ImgErrorCause {
    fn from(err: io::Error) -> Self {
        Self::IoError(Box::new(err))
    }
}

impl From<sqlx::Error> for ImgErrorCause {
    fn from(err: sqlx::Error) -> Self {
        Self::DbError(Box::new(err))
    }
}

impl From<tokio::sync::AcquireError> for ImgErrorCause {
    fn from(err: tokio::sync::AcquireError) -> Self {
        Self::SemaphoreError(Box::new(err))
    }
}

impl From<tokio::task::JoinError> for ImgErrorCause {
    fn from(err: tokio::task::JoinError) -> Self {
        Self::TaskPanicked(Box::new(err))
    }
}

impl From<url::ParseError> for ImgErrorCause {
    fn from(err: url::ParseError) -> Self {
        Self::InvalidUrl(Box::new(err))
    }
}
