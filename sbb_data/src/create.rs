use diesel::prelude::*;

use crate::{schema::*, model::*, new::*};

pub trait Create<T> {
    fn create(&self, conn: &PgConnection) -> QueryResult<T>;
}

macro_rules! insert {
    ($conn:expr, $table:ident, $values:expr) => {
        diesel::insert_into($table::table)
            .values($values)
            .get_result($conn)
    };
}

impl Create<RobotGroup> for NewRobotGroup<'_> {
    fn create(&self, conn: &PgConnection) -> QueryResult<RobotGroup> {
        insert!(conn, robot_groups, self)
    }
}

impl Create<Robot> for NewRobot<'_> {
    fn create(&self, conn: &PgConnection) -> QueryResult<Robot> {
        insert!(conn, robots, self)
    }
}

impl Create<Tag> for NewTag<'_> {
    fn create(&self, conn: &PgConnection) -> QueryResult<Tag> {
        insert!(conn, tags, self)
    }
}

impl Create<PastDaily> for NewPastDaily {
    fn create(&self, conn: &PgConnection) -> QueryResult<PastDaily> {
        insert!(conn, past_dailies, self)
    }
}

impl Create<ScheduledDaily> for NewScheduledDaily {
    fn create(&self, conn: &PgConnection) -> QueryResult<ScheduledDaily> {
        insert!(conn, scheduled_dailies, self)
    }
}

impl Create<ReplyTweet> for NewReplyTweet<'_> {
    fn create(&self, conn: &PgConnection) -> QueryResult<ReplyTweet> {
        insert!(conn, reply_tweets, self)
    }
}
