use std::convert::TryInto;

use goldcrest::data::{Tweet, tweet::TweetTextOptions};
use sbb_data::new::{NewRobotGroup, NewRobot};

use crate::parse::*;

pub fn parse_tweet<F, T>(tweet: &Tweet, handler: F) -> Option<T> where F: Fn(NewRobotGroup, Vec<Robot>) -> T {
    let tweet_id = tweet.id.try_into().ok()?;

    let text_opts = TweetTextOptions::all()
        .media(false)
        .urls(false);

    let text = tweet.text(text_opts);

    let (robots, body) = parse_group(&text)?;

    let image = tweet.media
        .iter()
        .filter(|&media| {
            media.media_type == "photo" || media.media_type == "animated_gif" || media.media_type == "video"
        })
        .next();

    let image_url = image.map(|image| image.media_url.as_str());

    let alt = image.and_then(|image| {
        let alt = image.alt.trim();
        if alt.is_empty() {
            None
        } else {
            Some(alt)
        }
    });

    let group = NewRobotGroup{
        tweet_id,
        tweet_time: tweet.created_at.naive_utc(),
        image_url,
        body: body.trim_end(),
        alt,
    };
    
    Some(handler(group, robots))
}

pub fn new_robot(robot: Robot, group_id: i32) -> NewRobot {
    NewRobot{
        robot_group_id: group_id,
        robot_number: robot.number,
        prefix: robot.name.prefix,
        suffix: robot.name.suffix,
        plural: robot.name.plural,
    }
}
