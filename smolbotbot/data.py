import os
import json


package_path = os.path.dirname(__file__)


def internal_path(path):
    return os.path.join(package_path, path)


def read_lines(path):
    with open(internal_path(path), "r") as data_file:
        return [line.strip() for line in data_file]


def read_json(path):
    with open(internal_path(path), "r") as json_file:
        return json.load(json_file)
