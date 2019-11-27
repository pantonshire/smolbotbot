import json
import os
from pathlib import Path


def input_setup_data(data_name):
    return input("> Enter %s (leave blank for none): " % (data_name)).strip()


def touch(path):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    Path(path).touch()


api_path = "smolbotbot/data/.api"
db_path = "smolbotbot/data/.db"

print("The following data will be stored in %s" % (api_path))

api_data = {
    "key": input_setup_data("Twitter API key"),
    "keySecret": input_setup_data("Twitter API secret key"),
    "token": input_setup_data("Twitter API access token"),
    "tokenSecret": input_setup_data("Twitter API access token secret")
}

with open(api_path, "w") as api_file:
    json.dump(api_data, api_file)

print("The following data will be stored in %s" % (db_path))

db_data = {
    "uri": input_setup_data("database URI")
}

with open(db_path, "w") as db_file:
    json.dump(db_data, db_file)

os.makedirs("smolbotbot/state", exist_ok=True)
Path("smolbotbot/state/log.txt").touch()
Path("smolbotbot/state/responded-tweets.txt").touch()
Path("smolbotbot/state/responded-dms.txt").touch()
