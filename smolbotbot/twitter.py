from . import log, accounts, data

import tweepy
import subprocess as sp
import json
import os
import datetime as dt


api_data = data.read_json("data/.api")

auth = tweepy.OAuthHandler(api_data["key"], api_data["keySecret"])
auth.set_access_token(api_data["token"], api_data["tokenSecret"])

del api_data

api = tweepy.API(auth)


def tweet(message):
    try:
        api.update_status(message)
    except tweepy.TweepError:
        log.log_error("Failed to tweet: %s" % (message))


def reply(reply_to_tweet, message):
    try:
        api.update_status("@%s %s" % (reply_to_tweet.user.screen_name, message), reply_to_tweet.id)
    except tweepy.TweepError:
        log.log_error("Failed to reply to %s" (reply_to_tweet.user.name))


def mentions(count, max_seconds_ago, id_blacklist):
    return [mention for mention in api.mentions_timeline(count=count, tweet_mode="extended")
            if not mention.id in id_blacklist and (dt.datetime.now() - mention.created_at).seconds < max_seconds_ago]


def recent_tweets(user, max_seconds_ago):
    return [(tweet.retweeted_status if hasattr(tweet, "retweeted_status") else tweet)
            for tweet in api.user_timeline("@" + user, tweet_mode="extended", include_ext_alt_text=True)
            if tweet.user.screen_name == user and (dt.datetime.now() - tweet.created_at).seconds < max_seconds_ago]


def direct_messages(max_seconds_ago, id_blacklist):
    try:
        return [
            message for message in api.list_direct_messages()
            if message.id not in id_blacklist
            and message.message_create["sender_id"] != accounts.bot_id
            and dt.datetime.now().timestamp() - (0.001 * float(message.created_timestamp)) < max_seconds_ago
        ]
    except tweepy.TweepError:
        log.log_error("Failed to get direct messages")
        return []


def send_direct_message(user_id, message):
    try:
        api.send_direct_message(user_id, message)
        return True
    except tweepy.TweepError:
        log.log_error("Failed to send direct message to " + user_id)
        return False


# Function for safely getting a tweet's text.
# Returns either the tweet's full_text or text depending on whether tweet_mode="extended" was used or not.
# Returns an empty string if no text attribute could be found.
def tweet_text(tweet):
    if hasattr(tweet, "full_text"):
        return tweet.full_text
    elif hasattr(tweet, "text"):
        return tweet.text
    return ""


# Returns the text of the specified tweet as it would be displayed on the Twitter website.
def actual_tweet_text(tweet):
    text = tweet_text(tweet)
    display_range = tweet.display_text_range
    if len(display_range) < 2:
        return text
    return text[display_range[0]:display_range[1]]
