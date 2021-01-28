CREATE TABLE robot_groups (
  id              SERIAL8 PRIMARY KEY,
  tweet_id        INT8 NOT NULL UNIQUE,
  tweet_time      TIMESTAMP NOT NULL,
  image_url       TEXT,
  body            TEXT NOT NULL,
  alt             TEXT,
  content_warning TEXT
);
