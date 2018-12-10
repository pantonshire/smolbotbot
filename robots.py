import csv
import random
from collections import defaultdict


def setup():
    # Load the robot data from the file
    robots_file = open("data/robot-data.csv", "r")
    reader = csv.reader(robots_file)
    rows = [row for row in reader]
    robots_file.close()
    print("Loaded " + str(len(rows)) + " rows from csv file")

    for row in rows:
        add_robot(row)


# Adds the robot to the robots list and all of the secondary-key indexes.
def add_robot(attributes):
    global robots, shuffled_robots_daily, shuffled_robots_request, number_index, name_index, tag_index, mention_index
    try:
        if len(attributes) != 9:
            print("Invalid number of attributes supplied")
            return

        list_pos = len(robots)

        robot_number = int(attributes[0])
        robot_name = attributes[1]
        robot_tags = attributes[6].split(" ")
        robot_mentions = attributes[7].split(" ")

        robot = {
            "number":       robot_number,
            "name":         robot_name,
            "tweet_id":     int(attributes[2]),
            "description":  attributes[3],
            "image":        attributes[4],
            "alt":          attributes[5],
            "tags":         robot_tags,
            "mentions":     robot_mentions,
            "hashtags":     attributes[8].split(" ")
        }
        robots.append(robot)
        shuffled_robots_daily.insert(random.randrange(len(shuffled_robots_daily) + 1), list_pos)
        shuffled_robots_request.insert(random.randrange(len(shuffled_robots_request) + 1), list_pos)

        number_index[robot_number].append(list_pos)
        name_index[robot_name].append(list_pos)
        for tag in robot_tags:
            tag_index[tag].append(list_pos)
        for mention in robot_mentions:
            mention_index[mention].append(list_pos)

    except ValueError:
        print("Invalid data supplied:")
        print(attributes)
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
        str(robot["number"]),
        robot["name"],
        str(robot["tweet_id"]),
        robot["description"],
        robot["image"],
        robot["alt"],
        " ".join(robot["tags"]),
        " ".join(robot["mentions"]),
        " ".join(robot["hashtags"])
    ]


def next_daily_robot():
    global shuffled_robots_daily, current_daily
    robot = robots[shuffled_robots_daily[current_random]]
    current_daily += 1
    if current_daily >= len(shuffled_robots_daily):
        current_daily = 0
        shuffled_robots_daily = random.shuffle(shuffled_robots_daily)
    return robot


def next_random_robot():
    global robots, shuffled_robots_request, current_random
    robot = robots[shuffled_robots_request[current_random]]
    current_random += 1
    if current_random >= len(shuffled_robots_request):
        current_random = 0
        shuffled_robots_request = random.shuffle(shuffled_robots_request)
    return robot


robots = []
shuffled_robots_daily = []
shuffled_robots_request = []
number_index = defaultdict(list)
name_index = defaultdict(list)
tag_index = defaultdict(list)
mention_index = defaultdict(list)

current_daily = 0
current_random = 0

setup()
