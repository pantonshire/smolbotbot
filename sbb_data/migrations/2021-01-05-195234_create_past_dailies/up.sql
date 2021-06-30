create table past_dailies (
    id         serial8 primary key,
    robot_id   int8 not null references robots (id) on delete cascade,
    posted_on  date not null
);
