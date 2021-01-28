CREATE TABLE tags (
  id             SERIAL8 PRIMARY KEY,
  robot_group_id INT8 NOT NULL REFERENCES robot_groups (id),
  tag            TEXT NOT NULL
);
