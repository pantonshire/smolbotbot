use std::borrow::Cow;

use chrono::{Utc, DateTime};

use goldcrest::data::{Tweet, tweet::TweetTextOptions};

use crate::parse;
use crate::robot::Robot;

#[derive(Debug)]
pub struct RobotsTweet<'a> {
    tweet_id: i64,
    tweet_time: DateTime<Utc>,
    image_url: Cow<'a, str>,
    body: Cow<'a, str>,
    alt: Option<Cow<'a, str>>,
    content_warning: Option<Cow<'a, str>>,
    robots: Vec<Robot<'a>>,
}

pub fn parse_tweet<F, T>(tweet: &Tweet, handler: F) -> Option<T>
where
    F: Fn(RobotsTweet) -> T,
{
    let text_opts = TweetTextOptions::all()
        .media(false)
        .urls(false);

    let text = tweet.text(text_opts);

    let (robots, body, cw) = parse::parse_group(&text)?;
    let body = body.trim_end();

    let image = tweet.media
        .iter()
        .find(|media| {
            media.media_type == "photo" || media.media_type == "animated_gif" || media.media_type == "video"
        })?;

    let image_url = image.media_url.as_str();

    let alt = {
        let alt = image.alt.trim();
        if alt.is_empty() {
            None
        } else {
            Some(alt)
        }
    };
    
    Some(handler(RobotsTweet{
        tweet_id: tweet.id as i64,
        tweet_time: tweet.created_at,
        image_url: Cow::Borrowed(image_url),
        body: Cow::Borrowed(body),
        alt: alt.map(Cow::Borrowed),
        content_warning: cw.map(Cow::Borrowed),
        robots
    }))
}
