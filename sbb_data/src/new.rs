use crate::schema::*;

use chrono::prelude::*;

#[derive(Insertable, Clone, Debug)]
#[table_name="robot_groups"]
pub struct NewRobotGroup<'a> {
    pub tweet_id: i64, //Cast u64 to i64 to obtain this, then cast back to u64 when retrieving the group
    pub tweet_time: DateTime<Utc>,
    pub image_url: Option<&'a str>,
    pub body: &'a str,
    pub alt: Option<&'a str>,
    pub content_warning: Option<&'a str>,
}

#[derive(Insertable, Clone, Debug)]
#[table_name="robots"]
pub struct NewRobot<'a> {
    pub robot_group_id: i64,
    pub robot_number: i32,
    pub prefix: &'a str,
    pub suffix: &'a str,
    pub plural: Option<&'a str>,
    pub ident: &'a str,
}

#[derive(Insertable, Clone, Debug)]
#[table_name="tags"]
pub struct NewTag<'a> {
    pub robot_group_id: i64,
    pub tag: &'a str,
}

#[derive(Insertable, Clone, Debug)]
#[table_name="past_dailies"]
pub struct NewPastDaily {
    pub robot_id: i64,
    pub posted_on: NaiveDate,
}

#[derive(Insertable, Clone, Debug)]
#[table_name="scheduled_dailies"]
pub struct NewScheduledDaily {
    pub robot_id: i64,
    pub post_on: NaiveDate,
}

#[derive(Insertable, Clone, Debug)]
#[table_name="reply_tweets"]
pub struct NewReplyTweet<'a> {
    pub request_tweet_id: i64,
    pub request_tweet_time: DateTime<Utc>,
    pub reply_tweet_id: i64,
    pub reply_tweet_time: DateTime<Utc>,
    pub user_id: i64,
    pub user_handle: &'a str,
    pub robot_id: i64,
}

#[derive(Insertable, Clone, Debug)]
#[table_name="tagged_markers"]
pub struct NewTaggedMarker {
    pub robot_group_id: i64,
}
