import robots
import robotdata
import search
import twitter
import schedule
import time


responded_tweets = []
responded_dms = []

saved_responded_tweets = open("state/responded-tweets.txt", "r")
for tweet_id in saved_responded_tweets:
    try:
        responded_tweets.append(int(tweet_id.strip()))
    except ValueError:
        continue
saved_responded_tweets.close()
del saved_responded_tweets
print("Loaded responded tweets: " + str(responded_tweets))

saved_responded_dms = open("state/responded-dms", "r")
for dm_id in saved_responded_dms:
    try:
        responded_dms.append(int(dm_id.strip()))
    except ValueError:
        continue
saved_responded_dms.close()
del saved_responded_dms
print("Loaded responded dms: " + str(responded_dms))


def morning_tweet():
    tweet_next_robot("Morning")


def noon_tweet():
    tweet_next_robot("Afternoon")


def tweet_next_robot(time_of_day):
    print(time_of_day + "tweet")


def check_new_robots():
    recent_tweets = twitter.recent_tweets("smolrobots", 10800)
    print(str(len(recent_tweets)) + " recent tweets found from @smolrobots, looking for new robots")
    for tweet in recent_tweets:
        if robotdata.generate_robot_data(tweet.text, tweet.id):
            print("Registered a new robot")


def check_tweets():
    print("Check tweets")


def check_direct_messages():
    print("Check DMs")


def close_bot():
    global responded_tweets, responded_dms

    tweets_file = open("state/responded-tweets.txt", "w")
    for tweet_id in responded_tweets:
        tweets_file.write(str(tweet_id))
    tweets_file.close()
    print("Saved responded tweet ids")

    dms_file = open("state/responded-dms", "w")
    for dm_id in responded_dms:
        dms_file.write(str(dm_id))
    dms_file.close()
    print("Saved responded dm ids")


schedule.every().day.at("07:00").do(morning_tweet)
schedule.every().day.at("12:00").do(noon_tweet)
schedule.every().hour.do(check_new_robots)
schedule.every().minute.do(check_direct_messages())
schedule.every(15).seconds.do(check_tweets())


while True:
    try:
        time.sleep(1)
        schedule.run_pending()
    except KeyboardInterrupt:
        print("Keyboard interrupt, stopping")
        close_bot()
        break
    except:
        print("An uncaught error occurred in schedule loop")

    # Todo: accept DM commands from @pantonshire
    # Todo: make sure to check for retweets and whether or not the tweet actually comes from @smolrobots

print("Goodbye!")