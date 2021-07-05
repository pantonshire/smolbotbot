create table robot_groups (
    id               serial4 primary key,
    tweet_id         int8 not null unique,
    tweet_time       timestamp with time zone not null,
    image_url        text not null,
    body             text not null,
    alt              text,
    content_warning  text,
    tags             text array
);

create index ix_robot_groups_tags on robot_groups using gin (tags);

create table robots (
    id              serial4 primary key,
    robot_group_id  int4 not null references robot_groups (id) on delete cascade,
    robot_number    int4 not null,
    prefix          text not null,
    suffix          text not null,
    plural          text,
    ident           text not null,
    
    unique (robot_number, prefix)
);

create table past_dailies (
    id         serial4 primary key,
    robot_id   int4 not null references robots (id) on delete cascade,
    posted_on  date not null
);

create table scheduled_dailies (
    id        serial4 primary key,
    robot_id  int4 not null references robots (id) on delete cascade,
    post_on   date not null unique
);
