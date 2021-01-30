create table scheduled_dailies (
    id        serial8 primary key,
    robot_id  int8 not null references robots (id),
    post_on   date not null unique
);
