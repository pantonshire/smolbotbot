CREATE TABLE robots (
  id              SERIAL8 PRIMARY KEY,
  robot_group_id  INT8 NOT NULL REFERENCES robot_groups (id),
  robot_number    INT4 NOT NULL,
  prefix          TEXT NOT NULL,
  suffix          TEXT NOT NULL,
  plural          TEXT,
  ident           TEXT NOT NULL,
  UNIQUE (robot_number, prefix)
);
