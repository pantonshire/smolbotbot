import log
import accounts
import tweepy
import subprocess as sp
import json
import os
import datetime as dt


key_file = open("data/.api", "r")
lines = [line.strip() for line in key_file]
key_file.close()

auth = tweepy.OAuthHandler(lines[0], lines[1])
auth.set_access_token(lines[2], lines[3])

del lines
del key_file

api = tweepy.API(auth)


def tweet(message):
    global api
    try:
        api.update_status(message)
    except tweepy.TweepError:
        log.log_error("Failed to tweet: " + message)


def reply(reply_to_tweet, message):
    global api
    try:
        api.update_status("@" + reply_to_tweet.user.screen_name + " " + message, reply_to_tweet.id)
    except tweepy.TweepError:
        log.log_error("Failed to reply to " + reply_to_tweet.user.name)


def mentions(count, max_seconds_ago, id_blacklist):
    global api
    return [mention for mention in api.mentions_timeline(count=count, tweet_mode="extended")
            if not mention.id in id_blacklist and (dt.datetime.now() - mention.created_at).seconds < max_seconds_ago]


def recent_tweets(user, max_seconds_ago):
    global api
    return [(tweet.retweeted_status if hasattr(tweet, "retweeted_status") else tweet)
            for tweet in api.user_timeline("@" + user, tweet_mode="extended")
            if tweet.user.screen_name == user and (dt.datetime.now() - tweet.created_at).seconds < max_seconds_ago]


def direct_messages(max_seconds_ago, id_blacklist):
    try:
        output = sp.run("/usr/local/bin/twurl -X GET /1.1/direct_messages/events/list.json".split(" "), stdout=sp.PIPE).stdout
        obj = json.loads(output.decode("utf-8"))
        messages = obj["events"]
        return [message for message in messages
                if message["id"] not in id_blacklist
                and message["message_create"]["sender_id"] != accounts.bot_id
                and dt.datetime.now().timestamp() - (0.001 * float(message["created_timestamp"])) < max_seconds_ago]
    except:
        print("Error retrieving direct messages")
        return []


def send_direct_message(user_id, message):
    try:
        command = '/usr/local/bin/twurl -A \'Content-type: application/json\' -X POST /1.1/direct_messages/events/new.json -d\'{"event": {"type": "message_create", "message_create": {"target": {"recipient_id": "' + user_id + '"}, "message_data": {"text": "' + message + '"}}}}\''
        return os.system(command) == 0
    except:
        log.log_error("Failed to send direct message to " + user_id)
        return False
   
