CREATE TABLE past_dailies (
  id        SERIAL PRIMARY KEY,
  posted_on DATE NOT NULL,
  robot_id  INTEGER NOT NULL REFERENCES robots (id)
);
