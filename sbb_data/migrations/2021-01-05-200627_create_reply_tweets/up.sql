CREATE TABLE reply_tweets (
  id                 SERIAL8 PRIMARY KEY,
  request_tweet_id   INT8 NOT NULL UNIQUE,
  request_tweet_time TIMESTAMP NOT NULL,
  reply_tweet_id     INT8 NOT NULL UNIQUE,
  reply_tweet_time   TIMESTAMP NOT NULL,
  user_id            INT8 NOT NULL,
  user_handle        TEXT NOT NULL,
  robot_id           INT8 NOT NULL REFERENCES robots (id)
);
