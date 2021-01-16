CREATE TABLE robot_groups (
  id             SERIAL PRIMARY KEY,
  tweet_id       BIGINT NOT NULL,
  tweet_time     TIMESTAMP NOT NULL,
  image_url      TEXT,
  original_names TEXT NOT NULL,
  body           TEXT NOT NULL,
  original_body  TEXT NOT NULL,
  alt            TEXT
);
