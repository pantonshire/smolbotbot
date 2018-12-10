import robots
import thesaurus
import nltk


thank_keywords = ["thank", "thanks", "thx", "ty", "merci"]


def search(query):
    first_pass_results = search_first_pass(query)
    
    if len(first_pass_results) > 0:
       pass

    pass


def sanitise(query):
    return query.lower().strip()


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


def search_first_pass(query):
    pass


def search_second_pass(query):
    pass

