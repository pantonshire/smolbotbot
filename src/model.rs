use sqlx::FromRow;

#[derive(FromRow)]
pub(crate) struct Id {
    pub(crate) id: i32,
}

#[derive(FromRow)]
pub(crate) struct TweetId {
    pub(crate) tweet_id: i64,
}

#[derive(FromRow)]
pub(crate) struct DailyRobot {
    pub(crate) id: i32,
    pub(crate) robot_number: i32,
    pub(crate) prefix: String,
    pub(crate) suffix: String,
    pub(crate) plural: Option<String>,
    pub(crate) tweet_id: i64,
    pub(crate) content_warning: Option<String>,
}

impl DailyRobot {
    pub(crate) fn full_name(&self) -> String {
        let mut name_buf = String::with_capacity(
            self.prefix.len()
            + self.suffix.len()
            + self.plural.as_deref().map_or(0, |plural| plural.len())
        );

        name_buf.push_str(&self.prefix);
        name_buf.push_str(&self.suffix);

        if let Some(ref plural) = self.plural {
            name_buf.push_str(plural);
        }

        name_buf
    }

    pub(crate) fn tweet_url(&self) -> String {
        format!("https://twitter.com/smolrobots/status/{}", self.tweet_id)
    }
}

#[derive(FromRow, Clone, Debug)]
pub(crate) struct RobotImageUrl {
    pub(crate) id: i32,
    pub(crate) image_url: String,
}

#[derive(FromRow, Clone, Debug)]
pub(crate) struct RobotImagePath {
    pub(crate) id: i32,
    pub(crate) image_path: String,
}

#[derive(FromRow, Clone, Debug)]
pub(crate) struct RobotImagePathOpt {
    pub(crate) id: i32,
    pub(crate) image_path: Option<String>,
}
