import schedule
import time


def tweet_next_robot():
    print("Tweet!")


schedule.every().day.at("07:00").do(tweet_next_robot)
schedule.every().day.at("12:00").do(tweet_next_robot)

while True:
    schedule.run_pending()
    time.sleep(1)
