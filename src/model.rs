use sqlx::FromRow;

#[derive(FromRow)]
pub(crate) struct Id {
    pub(crate) id: i32,
}

#[derive(FromRow)]
pub(crate) struct TweetId {
    pub(crate) tweet_id: i64,
}
