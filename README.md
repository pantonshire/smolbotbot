# Smolbotbot
A Twitter chatbot for looking up [Small Robots](https://twitter.com/smolrobots) robots.  
Written for Python 3.

## Overview
- On Twitter, mention [@smolbotbot](https://twitter.com/smolbotbot) in a tweet or send a direct message with some search terms for the small robot you're looking for. The bot will find the best matches (up to 4) and return links to each of the original tweets.
- If you don't know which small robot you want, type "random" and the bot will choose for you!
- Every day at 07:00 GMT, the bot will tweet a different "small robot of the day".
- The bot stores data about all of the existing small robots, checking for new robots each hour and updating the file when new ones are found. Tags are automatically generated for each robot, which are used for searching.
- If you don't want the bot to reply to a tweet you've mentioned it in, type (ignore) in parentheses.
- Try thanking the bot for its hard work!

## Contibuting
If you want to make a contribution, feel free to submit a pull request.

## Installing
First, run the `data_setup.py` file.

You will be prompted to provide the following:
- Twitter API keys
  - API key
  - API secret key
- Twitter API access tokens
  - Access token
  - Access token secret
- A database URI

If you wish to change any of these later, you can set the Twitter data in `data/.api` the database data in `data/.db`.

Once you have done this, install using the `setup.py` file:
- `python3 setup.py build`
- `python3 setup.py install`

## Changelog
[The changelog can be found here](https://github.com/Pantonshire/SmolBotBot/blob/master/changelog.md#changelog). I try to update it whenever I add or change something, but it may sometimes be a bit behind!

## Dependencies
- [Tweepy](https://github.com/tweepy/tweepy)
- [NLTK](http://www.nltk.org/)
- [Schedule](https://github.com/dbader/schedule)
- [SQLAlchemy](https://www.sqlalchemy.org)
