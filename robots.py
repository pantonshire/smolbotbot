import log

import csv
import random
import re
from collections import defaultdict

from sqlalchemy import Column, Integer, BigInteger, String
from sqlalchemy.ext.declarative import declarative_base
from sqlalchemy import func, or_


Base = declarative_base()


class Robot(Base):
    __tablename__ = "robots"

    id = Column(Integer, primary_key=True)
    number = Column(Integer)
    name = Column(String)
    prefix = Column(String)
    tweetid = Column(BigInteger)
    timestamp = Column(BigInteger)
    description = Column(String)
    imgurl = Column(String)
    alt = Column(String)
    tags = Column(String)

    def __repr__(self):
        return self.get_full_title()

    def get_full_title(self):
        return "no. %d, %s" % (self.number, self.name)

    def get_link(self):
        return "https://twitter.com/smolrobots/status/%d" % (self.tweetid)

    def as_dict(self):
        return {
            "id": self.id,
            "number": self.number,
            "name": self.name,
            "prefix": self.prefix,
            "tweetid": self.tweetid,
            "timestamp": self.timestamp,
            "description": self.description,
            "imgurl": self.imgurl,
            "alt": self.alt,
            "tags": self.tags,
            "title": self.get_full_title(),
            "link": self.get_link()
        }


def query(session):
    return session.query(Robot)


def by_id(session, id):
    return query(session).filter_by(id=id).first()


def by_number(session, number):
    return query(session).filter_by(number=number).all()


def by_numbers(session, numbers):
    if not numbers:
        return []
    return query(session).filter(Robot.number.in_(numbers)).all()


def by_name(session, name):
    return query(session).filter(func.lower(Robot.number) == name.lower()).all()


def by_prefix(session, prefix):
    return query(session).filter_by(prefix=prefix.lower()).all()


def by_prefixes(session, prefixes):
    if not prefixes:
        return []
    return query(session).filter(Robot.prefix.in_(prefixes)).all()


def by_tag(session, tag):
    return query(session).filter(or_(
        Robot.tags.ilike("% " + tag + " %"),
        Robot.tags.ilike("% " + tag),
        Robot.tags.ilike(tag + " %"),
        func.lower(Robot.tags) == tag.lower()
    )).all()


def random_robot(session):
    return random.choice(query(session).all())


def exists(session, number, name):
    return bool(query(session).filter_by(number=number, name=name).all())


def add(session, number, name, tweet_id, timestamp, description, img_url, alt, tags):
    robot = Robot(
        number=number,
        name=name,
        prefix=get_name_prefix(name),
        tweetid=tweet_id,
        timestamp=timestamp,
        description=description,
        imgurl=img_url,
        alt=alt,
        tags=" ".join(sorted(tags)).lower()
    )
    session.add(robot)


def get_name_prefix(name):
    return bot_suffix_re.sub("", name.lower())


bot_suffix_re = re.compile("bot(s)?$")
