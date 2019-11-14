import twitter
import database
import robots
import robotdata
import search
import contentgen

import time


def add_robot(tweetid):
    database.accessdb(robotdata.generate_robot_data, twitter.api.get_status(tweetid, tweet_mode="extended"))


def search(query):
    database.accessdb(_do_search, query)


def _do_search(session, query):
    print(contentgen.make_console_response(search.search(session, query)))


def time_task(task, *args):
    start = time.time()
    task(*args)
    end = time.time()
    print("Completed in %f seconds" % (round(end - start, 4)))
