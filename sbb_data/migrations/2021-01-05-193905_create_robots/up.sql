create table robots (
    id              serial8 primary key,
    robot_group_id  int8 not null references robot_groups (id),
    robot_number    int4 not null,
    prefix          text not null,
    suffix          text not null,
    plural          text,
    ident           text not null,
    unique (robot_number, prefix)
);
