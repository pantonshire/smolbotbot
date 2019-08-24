import accounts
import random


welcome_phrases = ("You\'re welcome!", "You\'re welcome!", "No problem!", "My pleasure!")


# Converts a search result dictionary to a string that can be tweeted.
def make_tweet_response(search_results):
    result_type = search_results["type"]
    robot_data = search_results["robots"]

    if result_type == "search":
        if robot_data:
            top_results = robot_data[:4]
            no_results = len(top_results)
            results_text = "\n".join([robot.get_full_title() + ": " + robot.get_link() for robot in top_results])
            if no_results == 2:
                return "I found a couple of robots:\n" + results_text
            if no_results > 2:
                return "I found a few different robots:\n" + results_text
            return "I found " + results_text
        else:
            return "Sorry, I couldn\'t find the robot you\'re looking for. This might be because the robot isn\'t indexed yet, or because your request is too complicated for me."

    if result_type == "random":
        if robot_data:
            random_robot = robot_data[0]
            return "Here\'s your randomly chosen robot, " + random_robot.name + ": " + random_robot.get_link()
        else:
            return "I tried to find a random robot for you, but something went wrong. Tagging " + accounts.support_handle + " to let them know. Sorry for the inconvenience!"

    if result_type == "welcome":
        return random.choice(welcome_phrases)
    
    if result_type == "welcome-fr":
        return "De rien!"
    
    if result_type == "bang":
        return "STATE OF BANG! https://twitter.com/smolrobots/status/1019836378534858752"
    
    return "Your request caused an unhandled result type: " + result_type + ". Tagging " + accounts.support_handle + " to let them know. Sorry for the inconvenience!"


# Converts a search result dictionary to a list of strings that can be sent by direct message.
def make_dms_response(search_results):
    result_type = search_results["type"]
    robot_data = search_results["robots"]

    if result_type == "search":
        if robot_data:
            top_results = robot_data[:3]
            no_results = len(top_results)
            results_text = "\n".join([robot.get_full_title() for robot in top_results])
            links = [robot.get_link() for robot in top_results]
            if no_results == 2:
                return ["I found a couple of robots:\n" + results_text] + links
            if no_results > 2:
                return ["I found a few different robots:\n" + results_text] + links
            return ["I found " + results_text] + links
        else:
            return ["Sorry, I couldn\'t find the robot you\'re looking for. This might be because the robot isn\'t indexed yet, or because your request is too complicated for me."]

    if result_type == "random":
        if robot_data:
            random_robot = robot_data[0]
            return ["Here\'s your randomly chosen robot, " + random_robot.name + ": ", random_robot.get_link()]
        else:
            return ["I tried to find a random robot for you, but something went wrong. Please let " + accounts.support_handle + " know. Sorry for the inconvenience!"]

    if result_type == "welcome":
        return [random.choice(welcome_phrases)]
    
    if result_type == "welcome-fr":
        return ["De rien!"]
    
    if result_type == "bang":
        return ["STATE OF BANG!", "https://twitter.com/smolrobots/status/1019836378534858752"]
    
    return ["Your request caused an unhandled result type: " + result_type + ". Please let " + accounts.support_handle + " know. Sorry for the inconvenience!"]


# Converts a search result dictionary to a string for printing to the console or log file.
def make_console_response(search_results):
    result_type = search_results["type"]
    robot_data = search_results["robots"]

    if result_type == "search":
        if robot_data:
            return "\n".join([robot.get_full_title() for robot in robot_data])
        else:
            return "No search results"

    if result_type == "random":
        if robot_data:
            return "Random robot: " + robot_data[0].name
        else:
            return "No random robot available"

    if result_type == "welcome":
        return "You\'re welcome"
    
    if result_type == "welcome-fr":
        return "De rien"
    
    if result_type == "bang":
        return "\"State of bang\" easter egg"
    
    return "Unhandled result type"
