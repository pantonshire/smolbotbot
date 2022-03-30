-- TODO: table for most recent timeline tweet
-- TODO: review all queries in codebase
-- TODO: update archive, order by id rather than (robot_number, id)
-- TODO: constrain fields of id to be NOT NULL
-- TODO: elasticsearch

CREATE TYPE robot_ident AS (
    number  INT4,
    name    TEXT
);

CREATE TABLE robots (
    id                robot_ident PRIMARY KEY,
    prefix            TEXT NOT NULL,
    suffix            TEXT NOT NULL,
    plural            TEXT,
    tweet_id          INT8 NOT NULL,
    tweet_time        TIMESTAMP WITH TIME ZONE NOT NULL,
    image_url         TEXT NOT NULL,
    body              TEXT NOT NULL,
    alt               TEXT,
    content_warning   TEXT,
    custom_alt        TEXT,
    image_path        TEXT,
    image_thumb_path  TEXT
);

-- This is used for preempting duplicates, may not need this any more? (it's ok for there to be conflicts now)
CREATE INDEX ix_robots_tweet_id ON robots USING btree (tweet_id);

CREATE INDEX ix_robots_tweet_time ON robots USING btree (tweet_time);

-- TODO: replace with elasticsearch
-- CREATE INDEX ix_robots_ident_trgm ON robots USING gin (ident gin_trgm_ops);

CREATE TABLE past_dailies (
    id         SERIAL4 PRIMARY KEY,
    robot_id   robot_ident NOT NULL REFERENCES robots (id) ON DELETE CASCADE,
    posted_on  DATE NOT NULL
);

CREATE INDEX ix_past_dailies_robot_id ON past_dailies USING btree (robot_id);
CREATE INDEX ix_past_dailies_posted_on ON past_dailies USING btree (posted_on);

CREATE TABLE scheduled_dailies (
    id        SERIAL4 PRIMARY KEY,
    robot_id  robot_ident NOT NULL REFERENCES robots (id) ON DELETE CASCADE,
    post_on   DATE NOT NULL UNIQUE
);

CREATE INDEX ix_scheduled_dailies_post_on ON scheduled_dailies USING btree (post_on);
