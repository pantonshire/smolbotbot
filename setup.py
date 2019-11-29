from setuptools import setup, find_packages


with open("LICENSE", "r") as license_file:
    license = license_file.read()

with open("README.md", "r") as readme_file:
    readme = readme_file.read()


setup(
    name="smolbotbot",
    version="3.1.0",
    url="https://github.com/Pantonshire/SmolBotBot",
    license=license,
    author="Tom Panton",
    author_email="pantonshire@gmail.com",
    description="A Twitter chatbot for looking up Small Robots robots.",
    long_description=readme,
    long_description_content_type='text/markdown',
    python_requires=">=3.5.0",
    packages=find_packages(exclude=["tests"]),
    install_requires=["tweepy", "nltk", "schedule", "sqlalchemy"],
    extras_require={ "thesaurus": ["vocabulary"] },
    include_package_data=True,
    package_data={ "": ["data/*", "data/.api", "data/.db", "state/*"] }
)
