use chrono::{NaiveDateTime, NaiveDate};

use crate::schema::*;

#[derive(Insertable, Clone, Debug)]
#[table_name="robot_groups"]
pub struct NewRobotGroup<'a> {
    pub tweet_id: i64,
    pub tweet_time: NaiveDateTime,
    pub image_url: Option<&'a str>,
    pub body: &'a str,
    pub alt: Option<&'a str>,
    pub content_warning: Option<&'a str>,
}

#[derive(Insertable, Clone, Debug)]
#[table_name="robots"]
pub struct NewRobot<'a> {
    pub robot_group_id: i32,
    pub robot_number: i32,
    pub prefix: &'a str,
    pub suffix: &'a str,
    pub plural: Option<&'a str>,
    pub ident: &'a str,
}

#[derive(Insertable, Clone, Debug)]
#[table_name="tags"]
pub struct NewTag<'a> {
    pub robot_group_id: i32,
    pub tag: &'a str,
}

#[derive(Insertable, Clone, Debug)]
#[table_name="past_dailies"]
pub struct NewPastDaily {
    pub posted_on: NaiveDate,
    pub robot_id: i32,
}

#[derive(Insertable, Clone, Debug)]
#[table_name="scheduled_dailies"]
pub struct NewScheduledDaily {
    pub post_on: NaiveDate,
    pub robot_id: i32,
}

#[derive(Insertable, Clone, Debug)]
#[table_name="reply_tweets"]
pub struct NewReplyTweet<'a> {
    pub request_tweet_id: i64,
    pub request_tweet_time: NaiveDateTime,
    pub reply_tweet_id: i64,
    pub reply_tweet_time: NaiveDateTime,
    pub user_id: i64,
    pub user_handle: &'a str,
    pub robot_id: i32,
}

