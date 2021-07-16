CREATE TABLE robot_groups (
    id                SERIAL4 PRIMARY KEY,
    tweet_id          INT8 NOT NULL UNIQUE,
    tweet_time        TIMESTAMP WITH TIME ZONE NOT NULL,
    image_url         TEXT NOT NULL,
    body              TEXT NOT NULL,
    alt               TEXT,
    content_warning   TEXT,
    tags              TEXT ARRAY,
    custom_alt        TEXT,
    image_path        TEXT,
    image_thumb_path  TEXT
);

CREATE INDEX ix_robot_groups_tags ON robot_groups USING gin (tags);

CREATE TABLE robots (
    id            SERIAL4 PRIMARY KEY,
    group_id      INT4 NOT NULL REFERENCES robot_groups (id) ON DELETE CASCADE,
    robot_number  INT4 NOT NULL,
    prefix        TEXT NOT NULL,
    suffix        TEXT NOT NULL,
    plural        TEXT,
    ident         TEXT NOT NULL,
    
    UNIQUE (ident, robot_number)
);

CREATE INDEX ix_robots_robot_number ON robots USING btree (robot_number);

CREATE TABLE past_dailies (
    id         SERIAL4 PRIMARY KEY,
    robot_id   INT4 NOT NULL REFERENCES robots (id) ON DELETE CASCADE,
    posted_on  DATE NOT NULL
);

CREATE INDEX ix_past_dailies_robot_id ON past_dailies USING hash (robot_id);
CREATE INDEX ix_past_dailies_posted_on ON past_dailies USING btree (posted_on);

CREATE TABLE scheduled_dailies (
    id        SERIAL4 PRIMARY KEY,
    robot_id  INT4 NOT NULL REFERENCES robots (id) ON DELETE CASCADE,
    post_on   DATE NOT NULL UNIQUE
);

CREATE INDEX ix_scheduled_dailies_post_on ON scheduled_dailies USING btree (post_on);
