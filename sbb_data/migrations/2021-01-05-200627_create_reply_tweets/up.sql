create table reply_tweets (
    id                  serial8 primary key,
    request_tweet_id    int8 not null unique,
    request_tweet_time  timestamp with time zone not null,
    reply_tweet_id      int8 not null unique,
    reply_tweet_time    timestamp with time zone not null,
    user_id             int8 not null,
    user_handle         text not null,
    robot_id            int8 not null references robots (id)
);
