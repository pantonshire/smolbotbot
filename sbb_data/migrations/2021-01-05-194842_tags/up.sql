CREATE TABLE tags (
  id             SERIAL PRIMARY KEY,
  robot_group_id INTEGER NOT NULL REFERENCES robot_groups (id),
  tag            TEXT NOT NULL
);
