use chrono::{NaiveDateTime, NaiveDate};

use crate::schema::*;

#[derive(Identifiable, Queryable, Clone, Debug)]
pub struct RobotGroup {
    pub id: i32,
    pub tweet_id: i64,
    pub tweet_time: NaiveDateTime,
    pub image_url: Option<String>,
    pub original_names: String,
    pub body: String,
    pub original_body: String,
    pub alt: Option<String>,
}

#[derive(Identifiable, Queryable, Associations, Clone, Debug)]
#[belongs_to(RobotGroup)]
pub struct Robot {
    pub id: i32,
    pub robot_group_id: i32,
    pub robot_number: i32,
    pub prefix: String,
    pub suffix: String,
    pub suffix_singular: String,
    pub name_raw: String,
}

#[derive(Identifiable, Queryable, Associations, Clone, Debug)]
#[belongs_to(RobotGroup)]
pub struct Tag {
    pub id: i32,
    pub robot_group_id: i32,
    pub tag: String,
}

#[derive(Identifiable, Queryable, Clone, Debug)]
#[table_name="past_dailies"]
pub struct PastDaily {
    pub id: i32,
    pub posted_on: NaiveDate,
    pub robot_id: i32,
}

#[derive(Identifiable, Queryable, Clone, Debug)]
#[table_name="scheduled_dailies"]
pub struct ScheduledDaily {
    pub id: i32,
    pub post_on: NaiveDate,
    pub robot_id: i32,
}

#[derive(Identifiable, Queryable, Clone, Debug)]
pub struct ReplyTweet {
    pub id: i32,
    pub request_tweet_id: i64,
    pub request_tweet_time: NaiveDateTime,
    pub reply_tweet_id: i64,
    pub reply_tweet_time: NaiveDateTime,
    pub user_id: i64,
    pub user_handle: String,
    pub robot_id: i32,
}
