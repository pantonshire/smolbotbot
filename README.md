# Smolbotbot
A Twitter chatbot for looking up [Small Robots](https://twitter.com/smolrobots) robots.  
This repository also includes the (now deprecated) API for accessing the data collected by the bot.

The bot is written in Rust and the API is written in Go.

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
(TODO)

## Changelog
[The changelog can be found here](https://github.com/Pantonshire/SmolBotBot/blob/main/changelog.md#changelog). I try to update it whenever I add or change something, but it may sometimes be a bit behind!

## Dependencies
(TODO)
