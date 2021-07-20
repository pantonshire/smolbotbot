mod error;

use std::io::{self, Read};
use std::env;
use std::path::{PathBuf, Path};
use std::sync::Arc;

use anyhow::anyhow;
use clap::{Clap, crate_version, crate_authors, crate_description};
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
    /// If set, download the images and store them in this directory
    #[clap(short, long = "download")]
    download_dir: Option<PathBuf>,

    /// If set, generate image thumbnails and store them in this directory
    #[clap(short, long = "thumb")]
    thumb_dir: Option<PathBuf>,
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

//TODO: option to get images / thumbs for all robots who currently don't have one

const THUMB_SIZE: u32 = 192;

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
    let db_pool = {
        let db_url = env::var("DATABASE_URL")?;
        PgPool::connect(&db_url).await?
    };

    let mut all_succeeded = true;

    let groups = match opts.download_dir {
        // Download the images and store the paths in the database
        Some(download_dir) => {
            let groups = {
                let mut db_conn = db_pool.acquire().await?;
                let groups = get_image_urls(&mut db_conn, &group_ids).await?;
                drop(group_ids);
                groups
            };

            let http_client = reqwest::Client::new();

            let image_results = get_images(
                &db_pool,
                &http_client,
                &download_dir,
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
        None => {
            let mut db_conn = db_pool.acquire().await?;
            
            let opt_groups = get_image_paths(&mut db_conn, &group_ids).await?;
            drop(group_ids);

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
    if let Some(thumb_dir) = opts.thumb_dir {
        let thumb_results = gen_thumbs(
            &db_pool,
            groups,
            &thumb_dir,
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

async fn get_image_paths(
    db_conn: &mut PgConnection,
    group_ids: &[i32]
) -> sqlx::Result<Vec<GroupImagePathOpt>>
{
    sqlx::query_as!(
        GroupImagePathOpt,
        "SELECT id, image_path FROM robot_groups \
        WHERE id = ANY($1) \
        AND image_path IS NOT NULL",
        group_ids
    )
    .fetch_all(db_conn)
    .await
}

async fn get_images<P>(
    db_pool: &PgPool,
    http_client: &reqwest::Client,
    output_dir: P,
    groups: Vec<GroupImageUrl>
) -> Vec<Result<GroupImagePath, ImgError>>
where
    P: AsRef<Path>
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
        let output_path = output_dir.as_ref().join(&file_name);

        join_handles.push((group.id, tokio::spawn(async move {
            match output_path.to_str() {
                Some(output_path) => match semaphore.acquire().await {
                    Ok(_permit) => {
                        limiter.until_ready().await;
                        download_and_store(&db_pool, &http_client, &group, output_path)
                            .await
                            .map(move |_| GroupImagePath {
                                id: group.id,
                                image_path: output_path.to_owned(),
                            })
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
    group: &GroupImageUrl,
    output_path: &str,
) -> Result<(), ImgError>
{
    download_image(http_client, group, output_path).await?;

    let mut db_conn = db_pool
        .acquire()
        .await
        .map_err(|err| ImgError::new(group.id, err.into()))?;

    store_image_path(&mut db_conn, group, output_path).await
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
    path: &str,
) -> Result<(), ImgError>
{
    let rows_affected = sqlx::query!(
        "UPDATE robot_groups SET image_path = $1 WHERE id = $2",
        path,
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

async fn gen_thumbs<P>(
    db_pool: &PgPool,
    groups: Vec<GroupImagePath>,
    output_dir: P,
    thumb_size: u32
) -> Vec<Result<(), ImgError>>
where
    P: AsRef<Path>
{
    const MAX_CONCURRENT: usize = 16;
    const GRAYSCALE_THRESHOLD: f32 = 0.005;
    const JPEG_QUALITY: u8 = 50;

    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT));
    let mut join_handles = Vec::with_capacity(groups.len());

    for group in groups {
        let semaphore = semaphore.clone();
        let db_pool = db_pool.clone();

        let file_name = format!("group_{}_thumb.jpg", group.id);
        let output_path = output_dir.as_ref().join(&file_name);

        join_handles.push((group.id, tokio::spawn(async move {
            match output_path.to_str() {
                Some(output_path) => match semaphore.acquire().await {
                    Ok(_permit) => gen_thumb(
                        &db_pool,
                        &group,
                        output_path,
                        thumb_size,
                        JPEG_QUALITY,
                        GRAYSCALE_THRESHOLD
                    ).await,

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

async fn gen_thumb(
    db_pool: &PgPool,
    group: &GroupImagePath,
    output_path: &str,
    size: u32,
    quality: u8,
    grayscale_threshold: f32
) -> Result<(), ImgError>
{
    let original = {
        let image_data = tokio::fs::read(&group.image_path)
            .await
            .map_err(|err| ImgError::new(group.id, err.into()))?;

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

    tokio::fs::write(output_path, buffer)
        .await
        .map_err(|err| ImgError::new(group.id, err.into()))?;

    let mut db_conn = db_pool
        .acquire()
        .await
        .map_err(|err| ImgError::new(group.id, err.into()))?;

    let rows_affected = sqlx::query!(
        "UPDATE robot_groups SET image_thumb_path = $1 WHERE id = $2",
        output_path,
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
