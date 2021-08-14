# Smolbotbot

[![Latest release](https://img.shields.io/github/v/release/pantonshire/smolbotbot)](https://github.com/Pantonshire/SmolBotBot/releases/latest)

A Twitter bot for looking up [Small Robots](https://twitter.com/smolrobots) robots.  
Running at [@smolbotbot](https://twitter.com/smolbotbot) on Twitter! 🤖

## Installing with Docker
1. Run `git clone git@github.com:Pantonshire/SmolBotBot.git` to clone this repository
2. Create a `.env` file, setting the following environment variables:
    - `DATABASE_PASSWORD`: the password to use for the new PostgreSQL instance
    - `TWITTER_CONSUMER_KEY`, `TWITTER_CONSUMER_SECRET`, `TWITTER_TOKEN` and `TWITTER_TOKEN_SECRET`: the [OAuth credentials](https://developer.twitter.com/en/docs/authentication/oauth-1-0a) for the Twitter API
    - `SBB_TIMELINE`: set to `true` if you want to periodically search the Twitter timeline for new robots
    - `SBB_DAILY`: set to `true` if you want to post a "robot of the day" Tweet every day
3. Run `docker-compose up`

The Docker Compose application contains both Smolbotbot and the [Small Robots Archive](https://github.com/Pantonshire/small_robots_archive).
