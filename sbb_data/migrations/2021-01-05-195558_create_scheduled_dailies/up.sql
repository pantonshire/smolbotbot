CREATE TABLE scheduled_dailies (
  id       SERIAL8 PRIMARY KEY,
  post_on  DATE NOT NULL UNIQUE,
  robot_id INT8 NOT NULL REFERENCES robots (id)
);
