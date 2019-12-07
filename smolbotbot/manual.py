from . import twitter, database, robots, robotdata, search, contentgen

import time


def add_robot(tweetid, update=False):
    database.accessdb(
        robotdata.generate_robot_data,
        twitter.api.get_status(tweetid, tweet_mode="extended", include_ext_alt_text=True),
        update
    )


def result_search(query):
    return database.accessdb(
        lambda session, q : contentgen.make_console_response(search.search(session, q)),
        query
    )


def data_search(query):
    return database.accessdb(
        lambda session, q : [robot.as_dict for robot in search.search(session, q)["robots"]],
        query
    )


def all_robot_data():
    database.accessdb(lambda session : [robot.as_dict for robot in robots.query(session).all()])


def time_task(task, *args):
    start = time.time()
    task(*args)
    end = time.time()
    print("Completed in %f seconds" % (round(end - start, 4)))
