import tweepy
import datetime as dt

keyfile = open(".api", "rt")
lines = [line.strip() for line in keyfile]
keyfile.close()

auth = tweepy.OAuthHandler(lines[0], lines[1])
auth.set_access_token(lines[2], lines[3])

del lines
del keyfile

api = tweepy.API(auth)


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


   
