import robots
import nltk
import re
import random

blacklist_file = open("data/request-blacklist.txt", "r")
blacklist = set([line.lower().strip() for line in blacklist_file])
blacklist_file.close()
del blacklist_file

welcome_phrases = ("You\'re welcome!", "You\'re welcome!", "No problem!", "Just doing my job!", "My pleasure!")

random_keywords = ["random"]
thank_keywords = ["thank", "thanks", "thx", "ty"]
thank_keywords_fr = ["merci"]

hyphen_re = re.compile("(\-(?=\D))|((?<=\S)\-)")
at_re = re.compile("(?<=^|(?<=[^a-zA-Z0-9-\.]))#[A-Za-z_]+[A-Za-z0-9_]+")
hashtag_re = re.compile("(?<=^|(?<=[^a-zA-Z0-9-\.]))@[A-Za-z_]+[A-Za-z0-9_]+")
sanitise_re = re.compile("([^\w\-\'’]|(^\W)|(\W$))")
bot_ending_re = re.compile("bot(s)?$")
plural_re = re.compile("s$")

stemmer = nltk.stem.PorterStemmer()

token_scores = {
    "BOT": 10.0,
    "JJ": 8.0, "JJR": 8.0, "JJS": 8.0,
    "NN": 10.0, "NNS": 10.0,
    "VB": 5.0, "VBD": 5.0, "VBG": 5.0, "VBN": 5.0, "VBP": 5.0, "VBZ": 5.0,
}


# Search for robots given the query.
# Media should be removed from the query before it is passed into the function.
# Mentions will be removed by the function.
def search(query):
    tokens = [token for token in tokenise(query) if token]

    # State of bang!
    if contains_consecutive_keywords(tokens, ["state", "of", "bang"]):
        return result_output(result_type="bang")

    # Check if the query asks for a robot by name
    by_name = search_by_name(tokens)
    if by_name:
        # return robot_list_result(by_name)
        return result_output(robot_positions=by_name)

    # Check if the query asks for a robot by number
    by_number = search_by_number(tokens)
    if by_number:
        # return robot_list_result(by_number)
        return result_output(robot_positions=by_number)

    # Check if the query asks for a random robot
    if contains_keyword(tokens, random_keywords):
        return result_output(robot_objects=[robots.next_random_robot()], result_type="random")

    # Check if the query is thanking the smolbotbot in French (it's happened before!)
    if contains_exact_keyword(tokens, thank_keywords_fr):
        return result_output(result_type="welcome-fr")

    # Check if the query is thanking the smolbotbot
    if contains_exact_keyword(tokens, thank_keywords):
        # return random.choice(welcome_phrases)
        return result_output(result_type="welcome")

    # Check each token in the query against robot tags
    by_tags = search_by_tags(tokens)
    if by_tags:
        # return robot_list_result(by_tags)
        return result_output(robot_positions=by_tags)

    # Return an empty result
    return result_output()
    # return "Sorry, I couldn\'t find the robot you\'re looking for. This might be because the robot isn\'t indexed yet, or because your request is too complicated for me."


# First pass tokenisation of the query
# Splits into tokens by whitespace and hyphens then removes punctuation.
def tokenise(query):
    cleaned = hyphen_re.sub(" ", query)
    cleaned = at_re.sub("", cleaned)
    cleaned = hashtag_re.sub("", cleaned)
    return [sanitise_token(token) for token in cleaned.split()]


# Sanitise the token by converting it to lowercase and removing any trailing whitespace.
def sanitise_token(token):
    cleaned = token.lower().strip().replace("’", "'")
    return sanitise_re.sub("", cleaned)


# Returns whether or not the string represents an integer.
# Very performant compared to the commonly used try-except technique.
# https://stackoverflow.com/a/9859202
def is_str_int(string):
    if string and string[0] == "-":
        return string[1:].isdigit()
    return string.isdigit()


# Searches for a robot's name in the given tokens. Returns the positions of the robots whose
# names were found.
def search_by_name(tokens):
    found = []
    for index, token in enumerate(tokens):
        if "bot" in token:
            stripped_token = bot_ending_re.sub("", token)

            if stripped_token:
                found.extend(robots.get_by_name(stripped_token))
                found.extend(robots.get_by_name(stripped_token + "s"))
                found.extend(robots.get_by_name(plural_re.sub("", stripped_token)))

            if token in ("bot", "bots"):
                for x in range(0, index):
                    found.extend(robots.get_by_name("".join(tokens[x:index])))

    return list(dict.fromkeys(found))


# Searches for a robot's number in the given tokens. Returns the positions of the robots whose
# numbers were found.
def search_by_number(tokens):
    found = []
    for token in tokens:
        if is_str_int(token):
            found.extend(robots.get_by_number(int(token)))
    return list(dict.fromkeys(found))


def search_by_tags(tokens):
    tagged_tokens = nltk.pos_tag(tokens)

    tagged_tokens.extend([
        (bot_ending_re.sub("", token_data[0]), "BOT") for token_data in tagged_tokens
        if bot_ending_re.search(token_data[0])
    ])

    tagged_tokens = [(token_data[0], stemmer.stem(token_data[0]), token_data[1]) for token_data in tagged_tokens]

    # Remove unnecessary phrases
    # TODO: Read this data from a file
    tagged_tokens = without_all_consecutive_tokens(tagged_tokens, [
        "can i have".split(),
        "can i see".split(),
        "may i have".split(),
        "may i see".split(),
        "can you find".split(),
        "can you".split(),
        "show me".split(),
        "are there any".split(),
        "is there anything".split(),
        "is there a".split(),
        "is there one".split(),
        "i would like".split()
    ])

    print(tagged_tokens)

    scores = {}

    # Score robots by partial names for each token
    for token_data in tagged_tokens:
        name_results = get_by_partial_name(token_data[0])

        if not name_results:
            continue

        score = get_token_score(token_data[2]) * 2

        for result in name_results:
            add_score(scores, result, score)

    compound_name_score = 20.0

    # Score robots by compound token names
    for no_words in range(2, 4):
        compound_name_results = search_for_compound_partial_name(tokens, no_words)

        for result in compound_name_results:
            add_score(scores, result, compound_name_score)

    # Allowed tokens for checking against robot tags
    allowed_tagged_tokens = [token_data for token_data in tagged_tokens if token_data[0] not in blacklist or token_data[1] == "BOT"]

    # Score robots by tags for each token
    for token_data in allowed_tagged_tokens:
        full_token = token_data[0]
        stemmed_token = token_data[1]

        full_results = robots.get_by_tag(full_token)
        stemmed_results = [result for result in robots.get_by_tag(stemmed_token) if result not in full_results]

        if not full_results and not stemmed_results:
            continue

        token_type = token_data[2]
        full_score = get_token_score(token_type)
        stemmed_score = full_score * 0.5

        all_results = ((full_results, full_score), (stemmed_results, stemmed_score))

        for results_data in all_results:
            results = results_data[0]
            score = results_data[1]
            for result in results:
                add_score(scores, result, score)

    score_list = sorted([(position, scores[position]) for position in scores], key = lambda result: -result[1])

    highest_score = score_list[0][1] if score_list else 0
    max_delta_score = 5.0

    top_results = [result[0] for result in score_list if highest_score - result[1] <= max_delta_score]

    return top_results


def add_score(scores, robot, score):
    if robot in scores:
        scores[robot] += score
    else:
        scores[robot] = score


def get_by_partial_name(token):
    results_full = robots.get_by_name(token)
    results_singular = robots.get_by_name(plural_re.sub("", token))
    results_plural = robots.get_by_name(token + "s")
    return list(dict.fromkeys(results_full + results_singular + results_plural))


def search_for_compound_partial_name(tokens, no_words):
    results = []
    for i in range(len(tokens) - (no_words - 1)):
        joined = "".join(tokens[i : i + no_words])
        compound_results = get_by_partial_name(joined)

        results += compound_results
    return list(dict.fromkeys(results))


def get_token_score(token_type):
    return token_scores[token_type] if token_type in token_scores else 1.0


def without_consecutive_tokens(token_data, to_remove):
    no_to_remove = len(to_remove)
    no_remove_from = len(token_data)
    if no_to_remove > no_remove_from:
        return token_data
    for x in range(0, no_remove_from - no_to_remove + 1):
        if [data[0] for data in token_data[x:x+no_to_remove]] == to_remove:
            return token_data[:x] + token_data[x+no_to_remove:]
    return token_data


def without_all_consecutive_tokens(token_data, consecutive_token_list):
    if len(consecutive_token_list) == 0:
        return token_data
    if len(consecutive_token_list) == 1:
        return without_consecutive_tokens(token_data, consecutive_token_list[0])
    return without_all_consecutive_tokens(without_consecutive_tokens(token_data, consecutive_token_list[0]), consecutive_token_list[1:])


# def is_asking_for_random(tokens):
#     for token in tokens:
#         if "random" in token:
#             return True
#     return False


# def is_thanking(tokens):
#     for keyword in thank_keywords:
#         if keyword in tokens:
#             return True
#     return False


# Checks if any of the keywords are in the list of tokens.
def contains_exact_keyword(tokens, keywords):
    for keyword in keywords:
        if keyword in tokens:
            return True
    return False


# Checks if any of the tokens contain any of the keywords.
def contains_keyword(tokens, keywords):
    for keyword in keywords:
        for token in tokens:
            if keyword in token:
                return True
    return False


def contains_consecutive_keywords(tokens, keywords):
    no_keywords = len(keywords)
    no_tokens = len(tokens)
    if no_keywords > no_tokens:
        return False
    for x in range(0, no_tokens - no_keywords + 1):
        if tokens[x:x+no_tokens] == keywords:
            return True
    return False


def robot_list_result(positions):
    top_results = positions[0:4]
    results_text = "\n".join([robots.link_to_robot_by_position(position, True) for position in top_results])
    if len(top_results) > 2:
        return "I found a few different robots:\n" + results_text
    if len(top_results) > 1:
        return "I found a couple of robots:\n" + results_text
    return "I found " + results_text


# def random_result():
#     # next_random_robot = robots.next_random_robot()
#     # return "Here\'s your randomly chosen robot, " + robots.link_to_robot(next_random_robot, False)
#     return result_output(robots=[robots.next_random_robot], result_type="random")


def result_output(robot_positions=[], robot_objects=[], result_type="search"):
    return {
        "robots": [robot for robot in [robots.robot_data(position) for position in robot_positions] if robot] + robot_objects,
        "type": result_type
    }
