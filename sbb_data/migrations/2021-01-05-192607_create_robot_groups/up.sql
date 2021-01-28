create table robot_groups (
    id               serial8 primary key,
    tweet_id         int8 not null unique,
    tweet_time       timestamp with time zone not null,
    image_url        text,
    body             text not null,
    alt              text,
    content_warning  text
);
