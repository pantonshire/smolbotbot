import tweepy
import os
import subprocess as sp
import json
import datetime as dt

keyfile = open(".api", "rt")
lines = [line.strip() for line in keyfile]
keyfile.close()

auth = tweepy.OAuthHandler(lines[0], lines[1])
auth.set_access_token(lines[2], lines[3])

del lines
del keyfile

api = tweepy.API(auth)

bot_id = 1045382175091290113
bot_id_str = str(bot_id)
bot_account = api.get_user(bot_id)


def tweet(text):
    global api
    try:
        api.update_status(text)
    except tweepy.TweepError:
        print("Failed to tweet: " + text)


def reply(replyto, text):
    global api
    try:
        api.update_status("@" + replyto.user.screen_name + " " + text, replyto.id)
    except tweepy.TweepError:
        print("Failed to reply to " + replyto.user.name)


def mentions(count, max_seconds_ago, id_blacklist):
    global api
    return [mention for mention in api.mentions_timeline(count=count) if not mention.id in id_blacklist and (dt.datetime.now() - mention.created_at).seconds < max_seconds_ago]


def recent_tweets(user, max_seconds_ago):
    global api
    return [(tweet.retweeted_status if hasattr(tweet, "retweeted_status") else tweet) for tweet in api.user_timeline("@" + user) if (dt.datetime.now() - tweet.created_at).seconds < max_seconds_ago]


def direct_messages(max_seconds_ago, id_blacklist):
    try:
        output = sp.run("/usr/local/bin/twurl -X GET /1.1/direct_messages/events/list.json".split(" "), stdout=sp.PIPE).stdout
        obj = json.loads(output)
        messages = obj["events"]
        return [message for message in messages if message["id"] not in id_blacklist and message["message_create"]["sender_id"] != bot_id_str and dt.datetime.now().timestamp() - (0.001 * float(message["created_timestamp"])) < max_seconds_ago]
    except:
        print("Error retrieving direct messages")
        return []


def send_direct_message(user_id, message):
    try:
        command = '/usr/local/bin/twurl -A \'Content-type: application/json\' -X POST /1.1/direct_messages/events/new.json -d\'{"event": {"type": "message_create", "message_create": {"target": {"recipient_id": "' + user_id + '"}, "message_data": {"text": "' + message + '"}}}}\''
        return os.system(command) == 0
    except:
        print("Failed to send direct message to " + user_id)
        return False
   
