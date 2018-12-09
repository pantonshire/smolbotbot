import csv
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


# Adds the robot to the robots list and all of the secondary-key indexes
def add_robot(attributes):
    global robots, number_index, name_index, tag_index, mention_index
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


# Returns the robot's data as a list of strings which can be written to the csv file
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


robots = []
number_index = defaultdict(list)
name_index = defaultdict(list)
tag_index = defaultdict(list)
mention_index = defaultdict(list)

setup()

