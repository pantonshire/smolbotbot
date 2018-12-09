import twitter

import codecs
import csv
import re
import urllib.request as request
import nltk
from bs4 import BeautifulSoup


def get_tweet_page(tweet_id):
    page = request.urlopen("https://twitter.com/smolrobots/status/" + str(tweet_id))
    content = page.read()
    return BeautifulSoup(content, features="lxml")


def get_tweet_data(tweet_id, dom):
    try:
        tweet_container = dom.body.find(class_="tweet", attrs={"data-associated-tweet-id": str(tweet_id)})
        text = tweet_container.find(class_="tweet-text").text
        image = tweet_container.find(class_="AdaptiveMedia-container").find("img")
        src = image.get("src")
        alt = image.get("alt")
        return text, src, alt
    except AttributeError:
        print("An element was not found")
        return "", "", ""


def sanitise(text, expressions):
    sanitised = text.lower().strip()
    for expression in expressions:
        sanitised = expression.sub("", sanitised).strip()
    return sanitised


def split_compound_words(text):
    return re.sub("(?<=\w)[\-\/](?=\w)", " ", text)


def clean_token(token):
    cleaned = re.sub("((^\W+)|(\W+$))", "", token)
    return cleaned


def tokenise(text, blacklist):
    sentences = nltk.sent_tokenize(text)
    tokens = []
    for sentence in sentences:
        words = [word for word in nltk.word_tokenize(sentence) if word not in blacklist]
        tokens.extend(words)
    return tokens


def classify(tokens):
    return nltk.pos_tag(tokens)


def is_valid_word(token):
    return len(nltk.corpus.wordnet.synsets(token)) > 0


at_regex = re.compile("(?<=^|(?<=[^a-zA-Z0-9-\.]))@[A-Za-z_]+[A-Za-z0-9_]+")
hashtag_regex = re.compile("(?<=^|(?<=[^a-zA-Z0-9-\.]))#[A-Za-z_]+[A-Za-z0-9_]+")
picture_regex = re.compile("pic\.twitter\.com/\S+")
bot_intro_regex = re.compile("(^|\s)(\-)?\d+\)\s[\w\-]+bot\w*[\.\:]?")
special_char_regex = re.compile("[\*]")

sanitise_expressions = [picture_regex, at_regex, hashtag_regex, bot_intro_regex, special_char_regex]
polish_expressions = [picture_regex, bot_intro_regex]

robotfile = codecs.open("ROBOT_TABLE", "r", "utf-8")
robots = [tuple(line.strip().split(",")) for line in robotfile]
robotfile.close()
del robotfile

robots = sorted(robots, key = lambda robot: int(robot[0]))

outputfile = open("robot-data.csv", "a")
writer = csv.writer(outputfile, delimiter=",", quotechar='"', quoting=csv.QUOTE_ALL)

blacklistfile = open("keyword-blacklist.txt", "r")
blacklist = [line.lower().strip() for line in blacklistfile]
blacklistfile.close()
del blacklistfile
stopwords = nltk.corpus.stopwords.words("english")
blacklist.extend(stopwords)

stemmer = nltk.stem.PorterStemmer()

key_token_types = ["N", "J"]

try:
    for robot in robots:
        tweet_id = robot[2]
        tweet_page = get_tweet_page(tweet_id)
        text, src, alt = get_tweet_data(tweet_id, tweet_page)

        print("#" + str(robot[0]) + ", " + str(robot[1]))

        # Sanitise the description and alt text
        sanitised_text = split_compound_words(sanitise(text, sanitise_expressions))
        sanitised_alt = split_compound_words(sanitise(alt, sanitise_expressions))

        polished_text = sanitise(text, polish_expressions)
        polished_alt = sanitise(alt, polish_expressions)

        # Find all mentions and hashtags
        mentions = at_regex.findall(polished_text)
        hashtags = hashtag_regex.findall(polished_text)

        # Tokenise description and alt text
        text_tokens = classify(tokenise(sanitised_text, blacklist))
        alt_tokens = classify(tokenise(sanitised_alt, blacklist))
        all_tokens = text_tokens + alt_tokens
        tags = [clean_token(token[0]) for token in all_tokens if token[1][0] in key_token_types]

        # Stem tags and check if they are valid words when stemmed
        stemmed = [stemmer.stem(tag) for tag in tags]
        valid_stemmed = [tag for tag in stemmed if is_valid_word(tag)]
        tags.extend(valid_stemmed)

        # Remove duplicates
        tags = list(dict.fromkeys(tags))
        
        # Join lists as single strings
        tagstr = " ".join(sorted(tags)).strip()
        mentionstr = " ".join(sorted(mentions))
        hashtagstr = " ".join(sorted(hashtags))

        print("Tags: " + tagstr)
        print("Mentions: " + mentionstr)
        print("Hashtags: " + hashtagstr)
        print()

        # Write to the csv
        writer.writerow([robot[0], robot[1], robot[2], polished_text, src, polished_alt, tagstr, mentionstr, hashtagstr])
        # Flush the csv
        outputfile.flush()

finally:
    # Always close the output file when the try block is exited
    outputfile.close()
