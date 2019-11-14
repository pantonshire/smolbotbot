import twitter
import database
import robots
import robotdata


def add_robot(tweetid):
    database.accessdb(robotdata.generate_robot_data, twitter.api.get_status(tweetid, tweet_mode="extended"))
