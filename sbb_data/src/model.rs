use chrono::{NaiveDateTime, NaiveDate};

use crate::schema::*;

#[derive(Identifiable, Queryable, Clone, Debug)]
pub struct RobotGroup {
    pub id: i64,
    pub tweet_id: i64,
    pub tweet_time: NaiveDateTime,
    pub image_url: Option<String>,
    pub body: String,
    pub alt: Option<String>,
    pub content_warning: Option<String>,
}

impl RobotGroup {
    /// Returns the Tweet ID of this robot group as an unsigned 64-bit integer.
    ///
    /// Note that this method should be used rather than the signed tweet_id field, the sign
    /// of which is only for compatibility with PostgreSQL; the bits should be interpreted
    /// as an unsigned integer.
    pub fn u_tweet_id(&self) -> u64 {
        self.tweet_id as u64
    }

    /// Returns a link to the Tweet associated with the robot group.
    pub fn tweet_link(&self) -> String {
        const LINK_BASE: &'static str = "https://twitter.com/smolrobots/status/";
        const LINK_BASE_LEN: usize = LINK_BASE.len();
        let tweet_id_str = self.u_tweet_id().to_string();
        let mut link = String::with_capacity(LINK_BASE_LEN + tweet_id_str.len());
        link.push_str(LINK_BASE);
        link.push_str(&tweet_id_str);
        link
    }
}

#[derive(Identifiable, Queryable, Associations, Clone, Debug)]
#[belongs_to(RobotGroup)]
pub struct Robot {
    pub id: i64,
    pub robot_group_id: i64,
    pub robot_number: i32,
    pub prefix: String,
    pub suffix: String,
    pub plural: Option<String>,
    pub ident: String,
}

impl Robot {
    /// Returns the full name of the robot as it appeared in the Tweet, consisting of a prefix
    /// followed by a "bot" suffix.
    ///
    /// The suffix may be plural, e.g. Mischiefbots or R.O.B.O.T.S
    ///
    /// ```
    /// use sbb_data::Robot;
    /// let robot = Robot{
    ///     id: 0,
    ///     robot_group_id: 0,
    ///     robot_number: 179,
    ///     prefix: "Mischief".to_owned(),
    ///     suffix: "bot".to_owned(),
    ///     plural: Some("s".to_owned()),
    ///     ident: "mischief".to_owned(),
    /// };
    /// assert_eq!(robot.full_name(), "Mischiefbots");
    /// ```
    pub fn full_name(&self) -> String {
        let mut name = String::with_capacity(self.prefix.len()
            + self.suffix.len()
            + self.plural.as_ref().map(String::len).unwrap_or(0));
        name.push_str(&self.prefix);
        name.push_str(&self.suffix);
        if let Some(ref plural) = self.plural {
            name.push_str(plural);
        }
        name
    }

    /// Returns the full name of the robot, consisting of a prefix followed by a "bot" suffix.
    /// The suffix will always be singular, meaning that the final character will always be
    /// either 'T' or 't', never 'S' or 's'.
    ///
    /// For example, this will return "Mischiefbot" for Mischiefbots.
    ///
    /// ```
    /// use sbb_data::Robot;
    /// let robot = Robot{
    ///     id: 0,
    ///     robot_group_id: 0,
    ///     robot_number: 179,
    ///     prefix: "Mischief".to_owned(),
    ///     suffix: "bot".to_owned(),
    ///     plural: Some("s".to_owned()),
    ///     ident: "mischief".to_owned(),
    /// };
    /// assert_eq!(robot.singular_full_name(), "Mischiefbot");
    /// ```
    pub fn singular_full_name(&self) -> String {
        let mut name = String::with_capacity(self.prefix.len() + self.suffix.len());
        name.push_str(&self.prefix);
        name.push_str(&self.suffix);
        name
    }
}

#[derive(Identifiable, Queryable, Associations, Clone, Debug)]
#[belongs_to(RobotGroup)]
pub struct Tag {
    pub id: i64,
    pub robot_group_id: i64,
    pub tag: String,
}

#[derive(Identifiable, Queryable, Clone, Debug)]
#[table_name="past_dailies"]
pub struct PastDaily {
    pub id: i64,
    pub posted_on: NaiveDate,
    pub robot_id: i64,
}

#[derive(Identifiable, Queryable, Clone, Debug)]
#[table_name="scheduled_dailies"]
pub struct ScheduledDaily {
    pub id: i64,
    pub post_on: NaiveDate,
    pub robot_id: i64,
}

#[derive(Identifiable, Queryable, Clone, Debug)]
pub struct ReplyTweet {
    pub id: i64,
    pub request_tweet_id: i64,
    pub request_tweet_time: NaiveDateTime,
    pub reply_tweet_id: i64,
    pub reply_tweet_time: NaiveDateTime,
    pub user_id: i64,
    pub user_handle: String,
    pub robot_id: i64,
}
