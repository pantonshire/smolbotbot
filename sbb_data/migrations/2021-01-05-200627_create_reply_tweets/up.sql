CREATE TABLE reply_tweets (
  id                 SERIAL PRIMARY KEY,
  request_tweet_id   BIGINT NOT NULL UNIQUE,
  request_tweet_time TIMESTAMP NOT NULL,
  reply_tweet_id     BIGINT NOT NULL UNIQUE,
  reply_tweet_time   TIMESTAMP NOT NULL,
  user_id            BIGINT NOT NULL,
  user_handle        TEXT NOT NULL,
  robot_id           INTEGER NOT NULL REFERENCES robots (id)
);
