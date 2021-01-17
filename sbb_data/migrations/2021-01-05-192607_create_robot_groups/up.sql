CREATE TABLE robot_groups (
  id         SERIAL PRIMARY KEY,
  tweet_id   BIGINT NOT NULL,
  tweet_time TIMESTAMP NOT NULL,
  image_url  TEXT,
  body       TEXT NOT NULL,
  alt        TEXT
);
