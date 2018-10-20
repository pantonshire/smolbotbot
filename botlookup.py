import re
import random


def sanitise(word):
    return word.replace(".", "").replace(",", "").replace("!", "").replace("?", "").replace("-", "").strip().lower()


def is_numerical(word):
    try:
        asint = int(word)
        return True
    except ValueError:
        return False


def blacklisted(word):
    sanitised = sanitise(word)
    return "@" in word or "#" in word or sanitised == "need" or sanitised == "robot" or sanitised == "make" or sanitised == "can"


def requesting_random(word):
    return word == "random" or word == "randombot"


def grateful(word):
    return "thank" in word


# Returns the text to be shown if the robot was successfully found
def success_text(robot):
    return "I found robot no. " + str(robot[0]) + ", " + robot[1] + ": " + linkto(robot)


# Returns the link to the robot's original tweet
def linkto(robot):
    return "https://twitter.com/smolrobots/status/" + str(robot[2])

    
def search(robots, query):
    # Regex for matching the last occurence of "bot" and everything after it
    tail_re = re.compile(r"(bot)(?:.(?!(bot)))*$", re.IGNORECASE)
    
    # Special case flags
    randbot = False
    thanked = False
    missingbot273 = False
    
    words = [sanitise(word) for word in re.split("\s+", query) if not blacklisted(word)]

    estimate = None

    # Pass 1: search words for bot names in list
    for index, word in enumerate(words):
        numerical = is_numerical(word)
        numerical_val = int(word) if numerical else 0

        # Skip 1 and 2 letter words
        if not numerical and len(word) < 3:
            continue
        
        for robot in robots:
            name = sanitise(robot[1])
            name_no_tail = tail_re.sub("", name)
            
            if name == word:
                #print("Exit point 1")
                return success_text(robot)
            elif (name_no_tail == word or name_no_tail == tail_re.sub("", word)) and estimate == None:
                #print("Estimate point 1")
                estimate = robot
            elif (len(word) >= 4 and word in name) and estimate == None:
                #print("Estimate point 2")
                estimate = robot
            elif numerical and numerical_val == int(robot[0]):
                return success_text(robot)

        if requesting_random(word):
            randbot = True
        elif grateful(word):
            thanked = True
        elif numerical and numerical_val == 273:
            missingbot273 = True

    if estimate != None:
        #print("Exit point 2")
        return success_text(estimate)

    # Special cases
    # Random robot. Roll the dicebots!
    if randbot:
        chosen = random.choice(robots)
        return "Here\'s your randomly-chosen robot, " + chosen[1] + ": " + linkto(chosen)
    # Express gratitude to gain the humans' trust
    elif thanked:
        return "You\'re welcome!"
    # There's no robot 273! Uh-oh!
    elif missingbot273:
        return "Small robot number 273 doesn\'t exist due to an error at the small robot development lab! https://twitter.com/smolrobots/status/961900525506695168"

    # Pass 2: concatenate all words and check all bot names against the amalgam
    amalgam = "".join(word for word in words)
    for robot in robots:
        name = sanitise(robot[1])
        name_no_tail = tail_re.sub("", name)
        if name in amalgam or (len(name_no_tail) >= 4 and name_no_tail in amalgam):
            #print("Exit point 3")
            return success_text(robot)
        
    # Return a failurebot
    return "Sorry, I couldn\'t find the robot you\'re looking for. This could be because the robot isn\'t indexed yet, or because your request is too complicated for me."

