CREATE TABLE robots (
  id              SERIAL PRIMARY KEY,
  robot_group_id  INTEGER NOT NULL REFERENCES robot_groups (id),
  robot_number    INTEGER NOT NULL,
  prefix          TEXT NOT NULL,
  suffix          TEXT NOT NULL,
  plural          TEXT,
  ident           TEXT NOT NULL
);
