# Changelog

## Version 3.1.0
Released 29/11/19
- Removed the following dependencies: BeautifulSoup4, LXML.
- Robot images and alt texts are now found using the Twitter API.
- Added support for animated GIFs in robot data.

## Version 3.0.1
Released 27/11/19
- `data_setup.py` file now creates the `state/` directory and the necessary files in it.

## Version 3.0.0
Released 22/11/19
- Restructured the project so that it is contained in a Python package.
- Added a `setup.py` file for setting up the project.
- The `data/.api` and `data/.db` files now use a json format.
- Fixed an issue with ignoring phrases in requests.

## Version 3.0-pre.1
Released 15/11/19
- A database is now used to store all robot data, the URI of which is specified in data/.db.
- The timestamp at which the robots were published to Twitter is now stored.
- Robot name prefixes are now stored separately from the name; this is the lowercase name with the "bot" suffix removed.
- Mentions and hashtags relating to robots are no longer stored.
- robot-data.csv is now obsolete (but is still included for the time being).
- Added manual.py to allow some of the tasks performed by the bot to be performed manually.

## Version 2.1.2
Released 01/09/19
- Changed direct message check frequency to every 90 seconds due to API limits.

## Version 2.1.1
Released 01/09/19
- Tweets which explicitly contain the bot's @ will now always be responded to.
- Fixed a bug which caused only the first instance of a blacklisted phrase to be ignored.

## Version 2.1.0
Released 24/08/19
- The bot now only responds to tweets which mention the bot but are not replying to anybody but the bot.
- Phrases such as "can I have" are now removed from queries.
- Each link to a small robot is now sent as a separate direct message.
- Direct messages are now checked every 30 seconds rather than every minute.
- Admin account is now tagged if certain errors occur.
- Direct messages are now handled using Tweepy rather than Twurl, as Tweepy 3.8.0 seems to fix direct messages.
- Small robot data is now stored as objects rather than dictionaries.
- The bot has learned a little bit of French.
- Easter eggs!

## Version 2.0.3
- Fixed a bug which caused each character to be written to a separate line in the log file.

## Version 2.0.2
- Increased the maximum size of the blacklist lists from 20 items to 1024 items.
- The date and time of sending a tweet or direct message is now written to a log file.

## Version 2.0.1
- Fixed an issue in which responded tweet and dm ids were written to a single line in their respective text files.

## Version 2.0.0
- Improved robot data collection algorithm. The image URL, image alt text, mentions and hashtags are now collected and stored.
- Tags are now generated for each robot, based on nouns and adjectives appearing in the text and alt text.
- Implemented a daily robot feature; a different small robot is tweeted each day at 07:00 GMT.
- Random selection of robots is now biased to prevent the same robot from being selected twice.
- Improved search algorithm; as well as searching by name and number, robots can now be searched by their tags.
- Removed "amalgam" search method since it was returning many unexpected results.
- Up to 4 small robots may be returned per request.
- Scheduling is now done using [schedule](https://github.com/dbader/schedule) module.
- A few different admin commands may be now be supplied to the robot via direct messages.
- Robots are now indexed in dictionaries by name, number, tags and mentions for quick lookups.
- Replied tweet and DM ids are now saved to disk so that restarting does not cause the bot to re-reply to recent requests.
- Tweets and DMs containing the word "ignore" in parentheses, along with any other characters inside the parentheses, will now be ignored by the bot.
