CREATE TABLE IF NOT EXISTS robots (
    id                SERIAL4 PRIMARY KEY,
    robot_number      INT4 NOT NULL,
    prefix            TEXT NOT NULL,
    suffix            TEXT NOT NULL,
    plural            TEXT,
    ident             TEXT NOT NULL,
    tweet_id          INT8 NOT NULL,
    tweet_time        TIMESTAMP WITH TIME ZONE NOT NULL,
    image_url         TEXT NOT NULL,
    body              TEXT NOT NULL,
    alt               TEXT,
    content_warning   TEXT,
    tags              TEXT ARRAY,
    custom_alt        TEXT,
    image_path        TEXT,
    image_thumb_path  TEXT,
    
    UNIQUE (ident, robot_number)
);

CREATE INDEX IF NOT EXISTS ix_robots_robot_number_id ON robots USING btree (robot_number, id);
CREATE INDEX IF NOT EXISTS ix_robots_tweet_id ON robots USING btree (tweet_id);
CREATE INDEX IF NOT EXISTS ix_robots_tags ON robots USING gin (tags);

CREATE TABLE IF NOT EXISTS past_dailies (
    id         SERIAL4 PRIMARY KEY,
    robot_id   INT4 NOT NULL REFERENCES robots (id) ON DELETE CASCADE,
    posted_on  DATE NOT NULL
);

CREATE INDEX IF NOT EXISTS ix_past_dailies_robot_id ON past_dailies USING btree (robot_id);
CREATE INDEX IF NOT EXISTS ix_past_dailies_posted_on ON past_dailies USING btree (posted_on);

CREATE TABLE IF NOT EXISTS scheduled_dailies (
    id        SERIAL4 PRIMARY KEY,
    robot_id  INT4 NOT NULL REFERENCES robots (id) ON DELETE CASCADE,
    post_on   DATE NOT NULL UNIQUE
);

CREATE INDEX IF NOT EXISTS ix_scheduled_dailies_post_on ON scheduled_dailies USING btree (post_on);