CREATE TABLE past_dailies (
  id        SERIAL8 PRIMARY KEY,
  posted_on DATE NOT NULL,
  robot_id  INT8 NOT NULL REFERENCES robots (id)
);
