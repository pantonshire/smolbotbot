CREATE TABLE scheduled_dailies (
  id       SERIAL PRIMARY KEY,
  post_on  DATE NOT NULL UNIQUE,
  robot_id INTEGER NOT NULL REFERENCES robots (id)
);
