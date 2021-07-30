mod error;

use std::env;
use std::path::{PathBuf, Path};
use std::sync::Arc;
use std::time::Duration;

use anyhow::anyhow;
use clap::{Clap, crate_version, crate_authors, crate_description};
use tokio::io::AsyncReadExt;
use tokio::sync::Semaphore;
use sqlx::postgres::{PgConnection, PgPool};
use image::{ImageFormat, DynamicImage, GenericImageView, ImageEncoder};
use image::imageops::FilterType;
use image::codecs::jpeg;
use governor::{Quota, RateLimiter};
use nonzero_ext::nonzero;
use url::Url;

use error::{ImgError, ImgErrorCause};

#[derive(Clap)]
#[clap(version = crate_version!(), author = crate_authors!(), about = crate_description!())]
struct Opts {
    /// If set, download the images
    #[clap(short, long = "download")]
    download: bool,

    /// If set, generate image thumbnails
    #[clap(short, long = "thumb")]
    thumb: bool,

    /// The timeout in seconds for connecting to the image server. If not set, there is no timeout.
    #[clap(long)]
    connect_timeout: Option<u64>,

    /// The timeout in seconds for a request to complete. If not set, there is no timeout.
    #[clap(long)]
    request_timeout: Option<u64>,

    /// If set, use the this directory for storing / retrieving images instead of the current working
    /// directory
    dir: Option<PathBuf>,

    #[clap(subcommand)]
    subcommand: Subcommand,
}

#[derive(Clap)]
enum Subcommand {
    /// Read group ids from stdin
    Ids,
    /// Get groups without an image in the database
    Missing,
}

#[derive(Clone, Debug)]
struct GroupImageUrl {
    id: i32,
    image_url: String,
}

#[derive(Clone, Debug)]
struct GroupImagePath {
    id: i32,
    image_path: String,
}

#[derive(Clone, Debug)]
struct GroupImagePathOpt {
    id: i32,
    image_path: Option<String>,
}

const THUMB_SIZE: u32 = 192;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(feature = "dotenv")] {
        dotenv::dotenv().ok();
    }

    let opts = Opts::parse();

    // Exit early if the user did not provide any directories
    if !opts.download && !opts.thumb {
        return Err(anyhow!("no download directory or thumb directory provided, nothing to do"));
    }

    let dir = opts.dir.map(Arc::new);

    let db_pool = {
        let db_url = env::var("DATABASE_URL")?;
        PgPool::connect(&db_url).await?
    };

    let mut all_succeeded = true;

    let group_paths = match opts.download {
        // Download the images and store the paths in the database
        true => {
            let groups = {
                let mut db_conn = db_pool.acquire().await?;

                match opts.subcommand {
                    Subcommand::Ids => {
                        let group_ids = read_stdin_ids().await?;
                        get_image_urls(&mut db_conn, &group_ids).await?
                    },

                    Subcommand::Missing =>
                        get_image_urls_missing(&mut db_conn).await?,
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
                builder.build()?
            };

            let image_results = get_images(
                &db_pool,
                &http_client,
                dir.clone(),
                groups
            ).await;
            
            let mut successful_groups = Vec::new();
            for res in image_results {
                match res {
                    Ok(group) => successful_groups.push(group),
                    Err(err) => {
                        all_succeeded = false;
                        eprintln!("{}", err);
                    }
                }
            }
            
            successful_groups
        },

        // Retrieve the paths of the pre-downloaded images from the database
        false => {
            let mut db_conn = db_pool.acquire().await?;
            
            let opt_groups = match opts.subcommand {
                Subcommand::Ids => {
                    let group_ids = read_stdin_ids().await?;
                    get_image_paths(&mut db_conn, &group_ids).await?
                }

                Subcommand::Missing =>
                    get_image_paths_missing(&mut db_conn).await?,
            };

            let mut groups = Vec::new();
            for opt_group in opt_groups {
                match opt_group.image_path {
                    Some(image_path) => groups.push(GroupImagePath {
                        id: opt_group.id,
                        image_path,
                    }),

                    None => {
                        all_succeeded = false;
                        eprintln!("group {} has no image to generate a thumb from", opt_group.id);
                    },
                }
            }

            groups
        },
    };

    // Generate image thumbs and store the paths in the database
    if opts.thumb {
        let thumb_results = gen_thumbs(
            &db_pool,
            group_paths,
            dir,
            THUMB_SIZE
        ).await;

        for res in thumb_results {
            if let Err(err) = res {
                all_succeeded = false;
                eprintln!("{}", err);
            }
        }
    }

    db_pool.close().await;

    match all_succeeded {
        true => Ok(()),
        false => Err(anyhow!("failed for some groups")),
    }
}

/// Reads a list of robot group ids from stdin.
async fn read_stdin_ids() -> anyhow::Result<Vec<i32>> {
    let mut buffer = String::new();
    tokio::io::stdin()
        .read_to_string(&mut buffer)
        .await?;
    
    buffer
        .split_whitespace()
        .map(|id| id
            .parse::<i32>()
            .map_err(|_| anyhow!("invalid robot group id \"{}\"", id)))
        .collect::<Result<Vec<_>, _>>()
}

/// Get the image urls of all of the robot groups with the given ids.
async fn get_image_urls(
    db_conn: &mut PgConnection,
    group_ids: &[i32]
) -> sqlx::Result<Vec<GroupImageUrl>>
{
    sqlx::query_as!(
        GroupImageUrl,
        "SELECT id, image_url FROM robot_groups \
        WHERE id = ANY($1)",
        group_ids
    )
    .fetch_all(db_conn)
    .await
}

/// Get the image urls of all of the robot groups which have no image path in the database.
async fn get_image_urls_missing(db_conn: &mut PgConnection) -> sqlx::Result<Vec<GroupImageUrl>> {
    sqlx::query_as!(
        GroupImageUrl,
        "SELECT id, image_url FROM robot_groups \
        WHERE image_path IS NULL",
    )
    .fetch_all(db_conn)
    .await
}

/// Get the image paths of all of the robot groups with the given ids.
async fn get_image_paths(
    db_conn: &mut PgConnection,
    group_ids: &[i32]
) -> sqlx::Result<Vec<GroupImagePathOpt>>
{
    sqlx::query_as!(
        GroupImagePathOpt,
        "SELECT id, image_path FROM robot_groups \
        WHERE id = ANY($1)",
        group_ids
    )
    .fetch_all(db_conn)
    .await
}

/// Get the image paths of all of the robot groups which have no image thumb path in the database.
async fn get_image_paths_missing(db_conn: &mut PgConnection) -> sqlx::Result<Vec<GroupImagePathOpt>> {
    sqlx::query_as!(
        GroupImagePathOpt,
        "SELECT id, image_path FROM robot_groups \
        WHERE image_thumb_path IS NULL",
    )
    .fetch_all(db_conn)
    .await
}

async fn get_images(
    db_pool: &PgPool,
    http_client: &reqwest::Client,
    dir: Option<Arc<PathBuf>>,
    groups: Vec<GroupImageUrl>
) -> Vec<Result<GroupImagePath, ImgError>>
{
    const MAX_CONCURRENT: usize = 16;
    const REQUESTS_PER_SECOND: u32 = 10;

    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT));

    let quota = Quota::per_second(nonzero!(REQUESTS_PER_SECOND));
    let limiter = Arc::new(RateLimiter::direct(quota));

    let mut join_handles = Vec::with_capacity(groups.len());

    for group in groups {
        let semaphore = semaphore.clone();
        let limiter = limiter.clone();
        let db_pool = db_pool.clone();
        let http_client = http_client.clone();
        let dir = dir.clone();

        let file_name = format!("orig_{}.png", group.id);

        join_handles.push((group.id, tokio::spawn(async move {
            match semaphore.acquire().await {
                Ok(_permit) => {
                    limiter.until_ready().await;
                    download_and_store(&db_pool, &http_client, &group, dir.as_deref(), &file_name)
                        .await
                        .map(move |_| GroupImagePath {
                            id: group.id,
                            image_path: file_name.to_owned(),
                        })
                },

                Err(err) => Err(ImgError::new(group.id, err.into())),
            }
        })));
    }

    let mut results = Vec::with_capacity(join_handles.len());

    for (group_id, join_handle) in join_handles {
        results.push(join_handle
            .await
            .unwrap_or_else(|err| Err(ImgError::new(group_id, err.into()))));
    }

    results
}

async fn download_and_store<P>(
    db_pool: &PgPool,
    http_client: &reqwest::Client,
    group: &GroupImageUrl,
    dir: Option<P>,
    file_name: &str,
) -> Result<(), ImgError>
where
    P: AsRef<Path>
{
    match dir.as_ref() {
        Some(dir) =>
            download_image(http_client, group, dir.as_ref().join(file_name)).await,
        None =>
            download_image(http_client, group, file_name).await,
    }?;
    
    let mut db_conn = db_pool
        .acquire()
        .await
        .map_err(|err| ImgError::new(group.id, err.into()))?;

    store_image_path(&mut db_conn, group, file_name).await
}

async fn download_image<P>(
    http_client: &reqwest::Client,
    group: &GroupImageUrl,
    path: P,
) -> Result<(), ImgError>
where
    P: AsRef<Path>
{
    let image_url = image_large_png_url(&group.image_url)
        .map_err(|err| ImgError::new(group.id, err.into()))?;

    let resp = http_client.get(image_url)
        .send()
        .await
        .map_err(|err| ImgError::new(group.id, err.into()))?;

    match resp.status() {
        status if status.is_success() => {
            let image_data = resp
                .bytes()
                .await
                .map_err(|err| ImgError::new(group.id, err.into()))?;

            tokio::fs::write(path, &image_data)
                .await
                .map_err(|err| ImgError::new(group.id, err.into()))
        },

        status => Err(ImgError::new(group.id, ImgErrorCause::HttpError(status))),
    }
}

async fn store_image_path(
    db_conn: &mut PgConnection,
    group: &GroupImageUrl,
    file_name: &str,
) -> Result<(), ImgError>
{
    let rows_affected = sqlx::query!(
        "UPDATE robot_groups SET image_path = $1 WHERE id = $2",
        file_name,
        group.id
    )
    .execute(db_conn)
    .await
    .map_err(|err| ImgError::new(group.id, err.into()))?
    .rows_affected();

    if rows_affected < 1 {
        Err(ImgError::new(group.id, ImgErrorCause::NoRowsUpdated))
    } else {
        Ok(())
    }
}

async fn gen_thumbs(
    db_pool: &PgPool,
    groups: Vec<GroupImagePath>,
    dir: Option<Arc<PathBuf>>,
    thumb_size: u32
) -> Vec<Result<(), ImgError>>
{
    const MAX_CONCURRENT: usize = 16;
    const GRAYSCALE_THRESHOLD: f32 = 0.005;
    const JPEG_QUALITY: u8 = 50;

    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT));
    let mut join_handles = Vec::with_capacity(groups.len());

    for group in groups {
        let semaphore = semaphore.clone();
        let db_pool = db_pool.clone();
        let dir = dir.clone();

        let file_name = format!("thumb_{}.jpg", group.id);

        join_handles.push((group.id, tokio::spawn(async move {
            match semaphore.acquire().await {
                Ok(_permit) => gen_thumb(
                    &db_pool,
                    &group,
                    dir.as_deref(),
                    &file_name,
                    thumb_size,
                    JPEG_QUALITY,
                    GRAYSCALE_THRESHOLD
                ).await,

                Err(err) => Err(ImgError::new(group.id, err.into())),
            }
        })));
    }

    let mut results = Vec::with_capacity(join_handles.len());

    for (group_id, join_handle) in join_handles {
        results.push(join_handle
            .await
            .unwrap_or_else(|err| Err(ImgError::new(group_id, err.into()))));
    }

    results
}

async fn gen_thumb<P>(
    db_pool: &PgPool,
    group: &GroupImagePath,
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
            Some(dir) => tokio::fs::read(dir.as_ref().join(&group.image_path)).await,
            None => tokio::fs::read(&group.image_path).await,
        }.map_err(|err| ImgError::new(group.id, err.into()))?;

        match ImageFormat::from_path(&group.image_path).ok() {
            Some(image_format) => image::load_from_memory_with_format(&image_data, image_format),
            None => image::load_from_memory(&image_data),
        }.map_err(|err| ImgError::new(group.id, err.into()))?
    };

    let thumb = original.resize_to_fill(size, size, FilterType::Lanczos3);

    let thumb = match is_approx_grayscale(&thumb, grayscale_threshold) {
        true => DynamicImage::ImageLuma8(thumb.into_luma8()),
        false => DynamicImage::ImageRgb8(thumb.into_rgb8()),
    };

    let mut buffer = Vec::new();
    
    jpeg::JpegEncoder::new_with_quality(&mut buffer, quality)
        .write_image(thumb.as_bytes(), thumb.width(), thumb.height(), thumb.color())
        .map_err(|err| ImgError::new(group.id, err.into()))?;

    drop(thumb);

    match dir.as_ref() {
        Some(dir) => tokio::fs::write(dir.as_ref().join(file_name), buffer).await,
        None => tokio::fs::write(file_name, buffer).await,
    }.map_err(|err| ImgError::new(group.id, err.into()))?;

    let mut db_conn = db_pool
        .acquire()
        .await
        .map_err(|err| ImgError::new(group.id, err.into()))?;

    let rows_affected = sqlx::query!(
        "UPDATE robot_groups SET image_thumb_path = $1 WHERE id = $2",
        file_name,
        group.id
    )
    .execute(&mut db_conn)
    .await
    .map_err(|err| ImgError::new(group.id, err.into()))?
    .rows_affected();

    if rows_affected < 1 {
        Err(ImgError::new(group.id, ImgErrorCause::NoRowsUpdated))
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
