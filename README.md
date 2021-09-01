# Smolbotbot

[![Latest release](https://img.shields.io/github/v/release/pantonshire/smolbotbot)](https://github.com/Pantonshire/SmolBotBot/releases/latest)

A Twitter bot for looking up [Small Robots](https://twitter.com/smolrobots) robots.  
Running at [@smolbotbot](https://twitter.com/smolbotbot) on Twitter! 🤖

## Installation and setup
You will need:
- Docker and Docker Compose
- Twitter API credentials

### Building the images
First, clone the repository:
```sh
git clone --recurse-submodules git@github.com:Pantonshire/SmolBotBot.git
cd SmolBotBot
```

Create a `.env` file for Docker Compose to use, setting the following environment variables:
- `DATABASE_PASSWORD`: the password to use for the new PostgreSQL instance
- `TWITTER_CONSUMER_KEY`, `TWITTER_CONSUMER_SECRET`, `TWITTER_TOKEN` and `TWITTER_TOKEN_SECRET`: the [OAuth 1.0a credentials](https://developer.twitter.com/en/docs/authentication/oauth-1-0a) for the Twitter API

Build the images (this may take some time):
```sh
docker-compose build && docker-compose build sbb
```

Create and run the containers:
```sh
docker-compose up -d
```

The Small Robots Archive will now be running on port 8080. However, there will be nothing to show because we haven't fetched any of the Small Robots from Twitter yet!

### Getting robot data from Twitter
There's a couple of different ways to get the Small Robots from Twitter. The first is to search the @smolrobots timeline:
```sh
docker-compose run --rm sbb timeline.sh
```

Unfortunately, this can only be used to get recently-tweeted robots due to limitations of the Twitter API. For this reason, I maintain a complete list of Tweet IDs of robots, available at [https://smolbotbot.com/bootstrap/ids](https://smolbotbot.com/bootstrap/ids). You can use this to get all of the robots from Twitter:
```sh
docker-compose run --rm -e SBB_BOOTSTRAP_URL=https://smolbotbot.com/bootstrap/ids sbb bootstrap.sh
```

I recommend you run the bootstrap command once, then periodically run the timeline command using something like cron.

### Posting to Twitter
To post a "small robot of the day" to Twitter:
```sh
docker-compose run --rm sbb daily.sh
```
