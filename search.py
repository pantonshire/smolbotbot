import robots
import thesaurus
import nltk
import re


thank_keywords = ("thank", "thanks", "thx", "ty", "merci")

tokenize_expression_first = re.compile("(\s+|((?<=\w)\-(?=\w)))")
sanitise_expression_first = re.compile("[^\w\-]]")


# Search for robots given the query.
# Media should be removed from the query before it is passed into the function.
# Mentions will be removed by the function.
def search(query):
    global thank_keywords
    tokens = tokenize_first_pass(query)

    by_name = search_by_name(tokens)
    if by_name:
        return robot_list_result(by_name)


# First pass tokenization of the query
# Splits into tokens by whitespace and hyphens then removes punctuation.
def tokenize_first_pass(query):
    global tokenize_expression_first
    return [sanitise_token(token) for token in tokenize_expression_first.split(query)]


# Sanitise the token by converting it to lowercase and removing any trailing whitespace.
def sanitise_token(token):
    global sanitise_expression_first
    return sanitise_expression_first.sub("", token.lower().strip())


# Returns whether or not the string represents an integer.
# Very performant compared to the commonly used try-except technique.
# https://stackoverflow.com/a/9859202
def is_str_int(string):
    if string[0] == "-":
        return string[1:].isdigit()
    return string.isdigit()


def search_by_name(tokens):
    found = []
    for index, token in enumerate(tokens):
        found.extend(robots.get_by_name(token))
        if token == "bot":
            for x in range(0, index):
                found.extend(robots.get_by_name("".join(tokens[x:index])))
    return list(dict.fromkeys(found))


def search_by_number(tokens):
    found = []
    for token in tokens:
        if is_str_int(token):
            found.extend(robots.get_by_number(int(token)))
    return list(dict.fromkeys(found))


def is_asking_for_random(tokens):
    for token in tokens:
        if "random" in token:
            return True
    return False


def is_thanking(tokens):
    global thank_keywords
    for keyword in thank_keywords:
        if keyword in tokens:
            return True
    return False


def robot_list_result(positions):
    top_five = positions[0:5]
    results_text = "\n".join([robots.link_to_robot_by_position(position) for position in top_five])
    if len(top_five) > 2:
        return "I found a few different robots:\n" + results_text
    if len(top_five) > 1:
        return "I found a couple of robots:\n" + results_text
    return "I found " + results_text

