import robots
import nltk
import re
import random

blacklist_file = open("data/request-blacklist.txt", "r")
blacklist = set([line.lower().strip() for line in blacklist_file])
blacklist_file.close()
del blacklist_file

welcome_phrases = ("You\'re welcome!", "You\'re welcome!", "No problem!", "Just doing my job!", "My pleasure!")

thank_keywords = ("thank", "thanks", "thx", "ty", "merci")

hyphen_re = re.compile("(\-(?=\D))|((?<=\S)\-)")
at_re = re.compile("(?<=^|(?<=[^a-zA-Z0-9-\.]))#[A-Za-z_]+[A-Za-z0-9_]+")
hashtag_re = re.compile("(?<=^|(?<=[^a-zA-Z0-9-\.]))@[A-Za-z_]+[A-Za-z0-9_]+")
sanitise_re = re.compile("([^\w\-\'’]|(^\W)|(\W$))")
bot_ending_re = re.compile("bot(s)?$")
plural_re = re.compile("s$")

stemmer = nltk.stem.PorterStemmer()

token_scores = {
    "JJ": 8.0, "JJR": 8.0, "JJS": 8.0,
    "NN": 10.0, "NNS": 10.0,
    "VB": 5.0, "VBD": 5.0, "VBG": 5.0, "VBN": 5.0, "VBP": 5.0, "VBZ": 5.0,
}


# Search for robots given the query.
# Media should be removed from the query before it is passed into the function.
# Mentions will be removed by the function.
def search(query):
    tokens = [token for token in tokenize(query) if token]

    by_name = search_by_name(tokens)
    if by_name:
        return robot_list_result(by_name)

    by_number = search_by_number(tokens)
    if by_number:
        return robot_list_result(by_number)

    if is_asking_for_random(tokens):
        return random_result()

    if is_thanking(tokens):
        return random.choice(welcome_phrases)

    by_tags = search_by_tags(tokens)
    if by_tags:
        return robot_list_result(by_tags)

    return "Temporary failure message :("


# First pass tokenization of the query
# Splits into tokens by whitespace and hyphens then removes punctuation.
def tokenize(query):
    global hyphen_re
    cleaned = hyphen_re.sub(" ", query)
    cleaned = at_re.sub("", cleaned)
    cleaned = hashtag_re.sub("", cleaned)
    return [sanitise_token(token) for token in cleaned.split()]


# Sanitise the token by converting it to lowercase and removing any trailing whitespace.
def sanitise_token(token):
    global sanitise_re
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
    global bot_ending_re

    found = []
    for index, token in enumerate(tokens):
        if "bot" in token:
            stripped_token = bot_ending_re.sub("", token)

            if stripped_token:
                found.extend(robots.get_by_name(stripped_token))
                found.extend(robots.get_by_name(stripped_token + "s"))

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
    global stemmer, blacklist

    print("Searching by tags")
    print("Input tokens:")
    print(tokens)

    stemmed_tokens = [stemmer.stem(token) for token in tokens]
    tagged_tokens = [(token_data[0], stemmed_tokens[index], token_data[1])
                     for index, token_data in enumerate(nltk.pos_tag(tokens))]
    allowed_tagged_tokens = [token_data for token_data in tagged_tokens if token_data[0] not in blacklist]

    print("Tagged tokens:")
    print(tagged_tokens)

    print("Allowed tokens:")
    print(allowed_tagged_tokens)

    scores = {}

    print("Searching for partial names")

    # Score robots by partial names for each token
    for token_data in tagged_tokens:
        name_results = get_by_partial_name(token_data[0])

        if not name_results:
            print("No name results for " + str(token_data))
            continue

        score = get_token_score(token_data[2]) * 2

        for result in name_results:
            print(str(result) + " scored " + str(score))
            add_score(scores, result, score)

    compound_name_score = 20.0

    # Score robots by compound token names
    for no_words in range(2, 4):
        print("Searching for names comprised of " + str(no_words) + " contiguous tokens")
        compound_name_results = search_for_compound_partial_name(tokens, no_words)

        for result in compound_name_results:
            print(str(result) + " scored " + str(compound_name_score))
            add_score(scores, result, compound_name_score)

    print("Searching for tags")

    # Score robots by tags for each token
    for token_data in allowed_tagged_tokens:
        full_token = token_data[0]
        stemmed_token = token_data[1]

        full_results = robots.get_by_tag(full_token)
        stemmed_results = [result for result in robots.get_by_tag(stemmed_token) if result not in full_results]

        if not full_results and not stemmed_results:
            print("No tag results for " + str(token_data))
            continue

        token_type = token_data[2]
        full_score = get_token_score(token_type)
        stemmed_score = full_score * 0.5

        all_results = ((full_results, full_score), (stemmed_results, stemmed_score))

        print("Tag results for " + str(token_data))

        for results_data in all_results:
            results = results_data[0]
            score = results_data[1]
            for result in results:
                print(str(result) + " scored " + str(score))
                add_score(scores, result, score)

    score_list = sorted([(position, scores[position]) for position in scores], key = lambda result: -result[1])
    print(score_list)

    highest_score = score_list[0][1] if score_list else 0
    max_delta_score = 5.0

    top_results = [result[0] for result in score_list if highest_score - result[1] <= max_delta_score]
    print(top_results)

    return top_results


def add_score(scores, robot, score):
    if robot in scores:
        scores[robot] += score
    else:
        scores[robot] = score


def get_by_partial_name(token):
    global plural_re
    results_full = robots.get_by_name(token)
    results_singular = robots.get_by_name(plural_re.sub("", token))
    results_plural = robots.get_by_name(token + "s")
    return list(dict.fromkeys(results_full + results_singular + results_plural))


def search_for_compound_partial_name(tokens, no_words):
    results = []
    for i in range(len(tokens) - (no_words - 1)):
        joined = "".join(tokens[i : i + no_words])
        compound_results = get_by_partial_name(joined)
        if compound_results:
            print("Name match for " + joined)
        results += compound_results
    return list(dict.fromkeys(results))


def get_token_score(token_type):
    global token_scores
    return token_scores[token_type] if token_type in token_scores else 1.0


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
    results_text = "\n".join([robots.link_to_robot_by_position(position, True) for position in top_five])
    if len(top_five) > 2:
        return "I found a few different robots:\n" + results_text
    if len(top_five) > 1:
        return "I found a couple of robots:\n" + results_text
    return "I found " + results_text


def random_result():
    next_random_robot = robots.next_random_robot()
    return "Here\'s you\'re randomly chosen robot, " + robots.link_to_robot(next_random_robot, False)
