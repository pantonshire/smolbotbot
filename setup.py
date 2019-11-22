from setuptools import setup, find_packages
import json


def input_setup_data(data_name):
    return input("> Enter %s (leave blank for none): " % (data_name)).strip()


with open("LICENSE", "r") as license_file:
    license = license_file.read()

with open("README.md", "r") as readme_file:
    readme = readme_file.read()


setup(
    name="smolbotbot",
    version="3.0.0",
    url="https://github.com/Pantonshire/SmolBotBot",
    license=license,
    author="Tom Panton",
    author_email="pantonshire@gmail.com",
    description="A Twitter chatbot for looking up Small Robots robots.",
    long_description=readme,
    long_description_content_type='text/markdown',
    python_requires=">=3.5.0",
    packages=find_packages(exclude=["tests"]),
    install_requires=["tweepy", "beautifulsoup4", "lxml", "nltk", "schedule", "sqlalchemy"],
    extras_require=["vocabulary"],
    include_package_data=True
)

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
