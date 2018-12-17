import robots
import re
import urllib.request
import urllib.error
import nltk
from bs4 import BeautifulSoup


bot_intro_re = re.compile("(\s*\-?\d+\)\s+[\w\-]+bot(?=\w*\W))")
at_re = re.compile("(?<=^|(?<=[^a-zA-Z0-9-\.]))@[A-Za-z_]+[A-Za-z0-9_]+")
hashtag_re = re.compile("(?<=^|(?<=[^a-zA-Z0-9-\.]))#[A-Za-z_]+[A-Za-z0-9_]+")
picture_re = re.compile("pic\.twitter\.com/\S+")
compound_word_re = re.compile("(?<=\w)[\-\/](?=\w)")
trailing_punctuation_re = re.compile("((^\W+)|(\W+$))")
special_char_re = re.compile("[\*]")

sanitise_expressions = [picture_re, at_re, hashtag_re, bot_intro_re, special_char_re]
polish_expressions = [picture_re, bot_intro_re]

blacklist_file = open("data/keyword-blacklist.txt", "r")
blacklist = [line.lower().strip() for line in blacklist_file]
blacklist_file.close()
del blacklist_file
stopwords = nltk.corpus.stopwords.words("english")
blacklist.extend(stopwords)

stemmer = nltk.stem.PorterStemmer()

key_token_types = ["N", "J"]


def generate_robot_data(tweet_text, tweet_id):
    global bot_intro_re, at_re, hashtag_re, stemmer, key_token_types, sanitise_expressions, polish_expressions

    id_str = str(tweet_id)

    # Check if the tweet starts with the classic robot intro: number) name
    bot_intro = bot_intro_re.match(tweet_text)
    if not bot_intro:
        return False

    # Check that the robot intro contains valid data about the robot's name and number
    intro_data = [data.strip() for data in bot_intro.groups()[0].split(")")]
    if len(intro_data) != 2 or not is_str_int(intro_data[0]):
        return False

    # Store the name and number as strings (no need to store the number as an integer)
    number = intro_data[0]
    name = intro_data[1]

    # Check if the robot is already indexed
    if robots.robot_exists(int(number), name):
        return False

    text = tweet_text
    src = ""
    alt = ""

    try:
        # Get the tweet page
        page = urllib.request.urlopen("https://twitter.com/smolrobots/status/" + id_str)
        content = page.read()
        dom = BeautifulSoup(content, features="lxml")

        # Get the text, image src and image alt from the page
        tweet_container = dom.body.find(class_="tweet", attrs={"data-associated-tweet-id": id_str})
        text = tweet_container.find(class_="tweet-text").text
        image = tweet_container.find(class_="AdaptiveMedia-container").find("img")
        src = image.get("src")
        alt = image.get("alt")

    except urllib.error.HTTPError:
        return False

    except AttributeError:
        print("AttributeError was raised while retrieving elements from dom")
        pass # Do not return false if an element was not found; some data may still be available

    # Sanitise the description and alt text
    sanitised_text = split_compound_words(sanitise(text, sanitise_expressions))
    sanitised_alt = split_compound_words(sanitise(alt, sanitise_expressions))

    polished_text = sanitise(text, polish_expressions)
    polished_alt = sanitise(alt, polish_expressions)

    # Find all mentions and hashtags
    mentions = at_re.findall(polished_text)
    hashtags = hashtag_re.findall(polished_text)

    # Tokenise description and alt text
    text_tokens = classify(tokenise(sanitised_text))
    alt_tokens = classify(tokenise(sanitised_alt))
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

    robot_data = [number, name, id_str, polished_text, src, polished_alt, tagstr, mentionstr, hashtagstr]

    # Add robot to list and lookup dictionaries
    robots.add_robot(robot_data)

    # Write the robot data to the csv
    robots.write_to_csv(robot_data)

    return True


def sanitise(text, expressions):
    sanitised = text.lower().strip()
    for expression in expressions:
        sanitised = expression.sub("", sanitised).strip()
    return sanitised


def split_compound_words(text):
    global compound_word_re
    return compound_word_re.sub(" ", text)


def clean_token(token):
    global trailing_punctuation_re
    return trailing_punctuation_re.sub("", token.strip())


def tokenise(text):
    global blacklist
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


def is_str_int(string):
    if string and string[0] == "-":
        return string[1:].isdigit()
    return string.isdigit()
