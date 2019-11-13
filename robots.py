import log

import csv
import random
import re
from collections import defaultdict

from sqlalchemy import Column, Integer, BigInteger, String
from sqlalchemy.ext.declarative import declarative_base
from sqlalchemy import func


Base = declarative_base()


class Robot(Base):
    __tablename__ = "robots"

    id = Column(Integer, primary_key=True)
    number = Column(Integer)
    name = Column(String)
    prefix = Column(String)
    tweetid = Column(BigInteger)
    description = Column(String)
    imgurl = Column(String)
    alt = Column(String)
    tags = Column(String)

    def __repr__(self):
        return self.get_full_title()

    def get_full_title(self):
        return "no. %d, %s" % (self.number, self.name)

    def get_link(self):
        return "https://twitter.com/smolrobots/status/%d" % (self.tweet_id)


def query(session):
    return session.query(Robot)


def by_id(session, id):
    return query(session).filter_by(id=id).first()


def by_number(session, number):
    return query(session).filter_by(number=number).all()


def by_numbers(session, numbers):
    return query(session).filter(Robot.number.in_(numbers)).all()


def by_name(session, name):
    return query(session).filter(func.lower(Robot.number) == name.lower()).all()


def by_prefix(session, prefix):
    return query(session).filter_by(prefix=prefix.lower()).all()


def by_prefixes(session, prefixes):
    return query(session).filter(Robot.prefix.in_(prefixes)).all()


def exists(session, number, name):
    return bool(query(session).filter_by(number=number, name=name).all())


def add(session, number, name, tweet_id, description, img_url, alt, tags):
    robot = Robot(
        number=number,
        name=name,
        prefix=get_name_prefix(name),
        tweetid=tweet_id,
        description=description,
        imgurl=img_url,
        alt=alt,
        tags=" ".join(tags).lower()
    )
    session.add(robot)


def get_name_prefix(name):
    return bot_suffix_re.sub("", name.lower())


def setup():
    # Load the robot data from the file
    robots_file = open("data/robot-data.csv", "r")
    reader = csv.reader(robots_file)
    rows = [row for row in reader if row]
    robots_file.close()
    log.log("Loaded " + str(len(rows)) + " rows from csv file")

    for row in rows:
        add_robot(row)


def reload():
    global robots
    robots.clear()
    setup()
    return len(robots)


# Adds the robot to the robots list and all of the secondary-key indexes.
def add_robot(attributes):
    global robots, shuffled_robots_daily, shuffled_robots_request,\
        number_index, name_index, tag_index, mention_index, bot_suffix_re

    try:
        if len(attributes) != 9:
            print("Invalid number of attributes supplied: " + str(attributes))
            return

        list_pos = len(robots)

        robot_number = int(attributes[0])
        robot_name = attributes[1]
        robot_tags = attributes[6].split(" ")
        robot_mentions = attributes[7].split(" ")

        robot = Robot(
            number=robot_number,
            name=robot_name,
            tweet_id=int(attributes[2]),
            description=attributes[3],
            image=attributes[4],
            alt=attributes[5],
            tags=robot_tags,
        )

        robots.append(robot)
        shuffled_robots_daily.insert(random.randrange(len(shuffled_robots_daily) + 1), list_pos)
        shuffled_robots_request.insert(random.randrange(len(shuffled_robots_request) + 1), list_pos)

        number_index[robot_number].append(list_pos)

        index_name = bot_suffix_re.sub("", robot_name.lower())
        name_index[index_name].append(list_pos)

        for tag in robot_tags:
            index_tag = tag.lower()
            tag_index[index_tag].append(list_pos)

        for mention in robot_mentions:
            index_mention = mention.lower()
            mention_index[index_mention].append(list_pos)

    except ValueError:
        log.log_error("Invalid data supplied: " + str(attributes))
        return


# Writes a new robots's data to the csv file.
# Data should be supplied as a list, not a dictionary.
# Should not be used for bulk writing, since the file is opened and closed with each call.
def write_to_csv(robot_data):
    robots_file = open("data/robot-data.csv", "a")
    writer = csv.writer(robots_file, delimiter=",", quotechar='"', quoting=csv.QUOTE_ALL)
    writer.writerow(robot_data)
    robots_file.close()


# Returns the robot's data as a list of strings which can be written to the csv file.
def robot_to_row(robot):
    return [
        str(robot.number),
        robot.name,
        str(robot.tweet_id),
        robot.description,
        robot.image,
        robot.alt,
        " ".join(robot.tags),
        " ".join(robot.mentions),
        " ".join(robot.hashtags)
    ]


def next_daily_robot():
    global shuffled_robots_daily, current_daily
    robot = robots[shuffled_robots_daily[current_daily]]
    current_daily += 1
    if current_daily >= len(shuffled_robots_daily):
        current_daily = 0
        random.shuffle(shuffled_robots_daily)
    return robot


def next_random_robot():
    global shuffled_robots_request, current_random
    robot = robots[shuffled_robots_request[current_random]]
    current_random += 1
    if current_random >= len(shuffled_robots_request):
        current_random = 0
        random.shuffle(shuffled_robots_request)
    return robot


def get_by_number(number):
    return number_index[number]


def get_by_name(name):
    return name_index[name]


def get_by_tag(tag):
    return tag_index[tag]


def robot_exists(number, name):
    return number in number_index and bot_suffix_re.sub("", name.lower()) in name_index


def robot_data(position):
    if position in range(0, len(robots)):
        return robots[position]
    return None


def robot_name(position):
    if position in range(0, len(robots)):
        return robots[position].name
    return ""


def link_to_robot_by_position(position, include_number=False):
    if position in range(0, len(robots)):
        return link_to_robot(robots[position], include_number)
    return ""


def link_to_robot(robot, include_number=False):
    return ("no. " + str(robot.number) + ", " if include_number else "") + robot.name + ": " +\
           "https://twitter.com/smolrobots/status/" + str(robot.tweet_id)


robots = []
shuffled_robots_daily = []
shuffled_robots_request = []
number_index = defaultdict(list)
name_index = defaultdict(list)
tag_index = defaultdict(list)
mention_index = defaultdict(list)

current_daily = 0
current_random = 0

bot_suffix_re = re.compile("bot(s)?$")

# setup()
