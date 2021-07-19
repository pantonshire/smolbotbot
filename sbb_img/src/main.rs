mod error;

use std::io::{self, Read};
use std::env;
use std::path::{PathBuf, Path, StripPrefixError};
use std::sync::Arc;

use anyhow::anyhow;
use clap::{Clap, crate_version, crate_authors, crate_description};
use tokio::io::AsyncWriteExt;
use tokio::sync::Semaphore;
use sqlx::postgres::{PgConnection, PgPool};
use image::{ImageFormat, DynamicImage, GenericImageView};
use image::imageops::FilterType;
use image::codecs::jpeg;
use governor::{Quota, RateLimiter};
use nonzero_ext::nonzero;
use url::Url;

use error::{ImgError, ImgErrorCause};

#[derive(Clap)]
#[clap(version = crate_version!(), author = crate_authors!(), about = crate_description!())]
struct Opts {
    /// If set, paths stored in the database will be relative to this path
    #[clap(short, long)]
    relative: Option<PathBuf>,

    /// If set, download the images and store them in this directory
    #[clap(short, long = "download")]
    download_dir: Option<PathBuf>,

    /// If set, generate image thumbnails and store them in this directory
    #[clap(short, long = "thumb")]
    thumb_dir: Option<PathBuf>,
}

#[derive(Clone, Debug)]
struct GroupImageInfo {
    id: i32,
    image_url: String,
}

//TODO: only download if download_dir specified
//TODO: set exit status to nonzero if there was a "report" error severity (but do not abort immediately)
//TODO: option to get images / thumbs for all robots who currently don't have one

fn main() -> anyhow::Result<()> {
    #[cfg(feature = "dotenv")] {
        dotenv::dotenv().ok();
    }

    let opts = Opts::parse();

    let group_ids = {
        let stdin = io::stdin();
        let mut buffer = String::new();
        stdin.lock().read_to_string(&mut buffer)?;
        
        buffer
            .split_whitespace()
            .map(|id| id
                .parse::<i32>()
                .map_err(|_| anyhow!("invalid robot group id \"{}\"", id)))
            .collect::<Result<Vec<_>, _>>()?
    };

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(run(opts, group_ids))
}

async fn run(opts: Opts, group_ids: Vec<i32>) -> anyhow::Result<()> {
    let download_dirs = opts.download_dir
        .as_deref()
        .map(|download_dir| image_dirs(download_dir, opts.relative.as_deref()))
        .transpose()?;

    let thumb_dirs = opts.thumb_dir
        .as_deref()
        .map(|thumb_dir| image_dirs(thumb_dir, opts.relative.as_deref()))
        .transpose()?;

    let db_pool = {
        let db_url = env::var("DATABASE_URL")?;
        PgPool::connect(&db_url).await?
    };

    let groups = {
        let mut db_conn = db_pool.acquire().await?;
        let groups = get_image_urls(&mut db_conn, &group_ids).await?;
        drop(group_ids);
        groups
    };

    let mut all_succeeded = true;

    let groups = match download_dirs {
        Some((download_dir, db_download_dir)) => {
            let http_client = reqwest::Client::new();

            let image_results = get_images(
                &db_pool,
                &http_client,
                download_dir,
                db_download_dir,
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

        None => groups,
    };

    println!("{:?}", groups);

    //TODO: generate thumbs concurrently

    db_pool.close().await;

    match all_succeeded {
        true => Ok(()),
        false => Err(anyhow!("failed for some groups")),
    }
}

async fn get_image_urls(
    db_conn: &mut PgConnection,
    group_ids: &[i32]
) -> sqlx::Result<Vec<GroupImageInfo>>
{
    sqlx::query_as!(
        GroupImageInfo,
        "SELECT id, image_url FROM robot_groups \
        WHERE id = ANY($1)",
        group_ids
    )
    .fetch_all(db_conn)
    .await
}

async fn get_images(
    db_pool: &PgPool,
    http_client: &reqwest::Client,
    output_dir: &Path,
    store_dir: &Path,
    groups: Vec<GroupImageInfo>
) -> Vec<Result<GroupImageInfo, ImgError>>
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

        let file_name = format!("group_{}_orig.png", group.id);

        let output_path = output_dir.join(&file_name);
        let store_path = store_dir.join(&file_name);

        join_handles.push((group.id, tokio::spawn(async move {
            match store_path.to_str() {
                Some(store_path) => match semaphore.acquire().await {
                    Ok(permit) => {
                        limiter.until_ready().await;
                        let res = download_and_store(&db_pool, &http_client, &group, &output_path, store_path)
                            .await
                            .map(move |_| group);
                        drop(permit);
                        res
                    },

                    Err(err) => Err(ImgError::new(group.id, err.into())),
                },

                None => Err(ImgError::new(group.id, ImgErrorCause::InvalidPath)),
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

async fn download_and_store(
    db_pool: &PgPool,
    http_client: &reqwest::Client,
    group: &GroupImageInfo,
    output_path: &Path,
    store_path: &str,
) -> Result<(), ImgError>
{
    download_image(http_client, group, output_path).await?;

    let mut db_conn = db_pool
        .acquire()
        .await
        .map_err(|err| ImgError::new(group.id, err.into()))?;

    store_image_path(&mut db_conn, group, store_path).await
}

async fn download_image(
    http_client: &reqwest::Client,
    group: &GroupImageInfo,
    output_path: &Path,
) -> Result<(), ImgError>
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

            write_file(&output_path, &image_data)
                .await
                .map_err(|err| ImgError::new(group.id, err.into()))
        },

        status => Err(ImgError::new(group.id, ImgErrorCause::HttpError(status))),
    }
}

async fn store_image_path(
    db_conn: &mut PgConnection,
    group: &GroupImageInfo,
    store_path: &str,
) -> Result<(), ImgError>
{
    sqlx::query!(
        "UPDATE robot_groups SET image_path = $1 WHERE id = $2",
        store_path,
        group.id
    )
    .execute(db_conn)
    .await
    .map_err(|err| ImgError::new(group.id, err.into()))?;

    Ok(())
}

async fn write_file<P>(path: P, bytes: &[u8]) -> io::Result<()>
where
    P: AsRef<Path>
{
    tokio::fs::File::create(path)
        .await?
        .write_all(bytes)
        .await
}

fn gen_thumb(original: &DynamicImage, size: u32, grayscale_threshold: f32) -> DynamicImage {
    // const GRAYSCALE_THRESHOLD: f32 = 0.001;

    let resized = original.resize_to_fill(size, size, FilterType::Lanczos3);

    if is_approx_grayscale(&resized, grayscale_threshold) {
        DynamicImage::ImageLuma8(resized.into_luma8())
    } else {
        DynamicImage::ImageRgb8(resized.into_rgb8())
    }
}

//TODO
async fn save_thumb() {
    todo!()
}

fn is_approx_grayscale(image: &DynamicImage, threshold: f32) -> bool {
    const STRIDE: u32 = 16;
    const CHANNEL_MAX: f32 = 255.0;
    const INV_SQRT3: f32 = 0.5773502692;

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

fn image_dirs<'p>(full_dir: &'p Path, relative: Option<&Path>) -> Result<(&'p Path, &'p Path), StripPrefixError> {
    match relative {
        Some(relative) => full_dir
            .strip_prefix(relative)
            .map(|db_dir| (full_dir, db_dir)),
        None => Ok((full_dir, full_dir)),
    }
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
