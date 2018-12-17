import robots
import search
import twitter
import schedule
import time


def morning_tweet():
    tweet_next_robot("Morning")


def noon_tweet():
    tweet_next_robot("Afternoon")


def tweet_next_robot(time_of_day):
    print(time_of_day + "tweet")


def check_new_robots():
    print("Check for new robots")


def check_tweets():
    print("Check tweets")


def check_direct_messages():
    print("Check DMs")


schedule.every().day.at("07:00").do(morning_tweet)
schedule.every().day.at("12:00").do(noon_tweet)
schedule.every().hour.do(check_new_robots)
schedule.every().minute.do(check_direct_messages())
schedule.every(15).seconds.do(check_tweets())

while True:
    try:
        schedule.run_pending()
    except:
        print("An error occurred in schedule loop")
    time.sleep(1)

    # Todo: accept DM commands from @pantonshire
