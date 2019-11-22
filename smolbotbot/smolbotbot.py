from . import robots, robotdata, search, contentgen, accounts, twitter, database, log

import schedule
import time
import random
import re


running = True

responded_tweets = []
responded_dms = []


saved_responded_tweets = open("state/responded-tweets.txt", "r")
for tweet_id in saved_responded_tweets:
    try:
        responded_tweets.append(int(tweet_id.strip()))
    except ValueError:
        continue
saved_responded_tweets.close()
del saved_responded_tweets
log.log("Loaded responded tweets: " + str(responded_tweets))

saved_responded_dms = open("state/responded-dms.txt", "r")
for dm_id in saved_responded_dms:
    dm_id_stripped = dm_id.strip()
    if dm_id_stripped:
        responded_dms.append(dm_id_stripped) # DM ids are stored as strings for convenience
saved_responded_dms.close()
del saved_responded_dms
log.log("Loaded responded dms: " + str(responded_dms))


greeting_phrases = [""]
introduction_phrases = [""]


ignore_re = re.compile("\([^\(]*ignore[^\)]*\)")


def daily_robot():
    database.accessdb(tweet_next_robot)


# TODO: avoid duplicates
def tweet_next_robot(session):
    robot = robots.random_robot(session)
    name = robot.name
    date = time.strftime("%d/%m/%y")
    greeting = random.choice(greeting_phrases)
    introduction = random.choice(introduction_phrases)
    link = robot.get_link()
    text = date + "\n" + greeting + " " + introduction + " " + name + "!" + link
    twitter.tweet(text)


def check_new_robots():
    recent_tweets = twitter.recent_tweets("smolrobots", 10800)
    log.log("%d recent tweets found from @smolrobots, looking for new robots" % (len(recent_tweets)))

    if recent_tweets:
        database.accessdb(check_tweets_for_robots, recent_tweets)


def check_tweets_for_robots(session, tweets):
    for tweet in tweets:
        if robotdata.generate_robot_data(session, tweet):
            log.log("Registered a new robot from tweet id " + str(tweet.id))


def check_mentions():
    mentions = twitter.mentions(20, 10800, responded_tweets)
    if mentions:
        database.accessdb(respond_mentions, mentions)


def respond_mentions(session, mentions):
    global responded_tweets

    for mention in mentions:
        text = mention.full_text

        if is_probably_request(mention) and not contains_ignore(text):
            search_result = search.search(session, text)
            response = contentgen.make_tweet_response(search_result)
            twitter.reply(mention, response)
            log.log("Tweet @" + mention.user.screen_name + ":" + str(mention.id))

        responded_tweets.append(mention.id)

        if len(responded_tweets) > 1024:
            responded_tweets = responded_tweets[1:]


def check_direct_messages():
    dms = twitter.direct_messages(7200, responded_dms)
    if dms:
        database.accessdb(respond_dms, dms)


def respond_dms(session, dms):
    global responded_dms

    for dm in dms:
        text = dm.message_create["message_data"]["text"]
        sender_id = dm.message_create["sender_id"]

        if not contains_ignore(text):
            response = []

            if sender_id in accounts.admin_ids and text.startswith("$"):
                response = [do_command(text[1:].lower().strip())]

            else:
                search_result = search.search(session, text)
                response = contentgen.make_dms_response(search_result)

            success = True

            for message in response:
                if not twitter.send_direct_message(sender_id, message):
                    log.log("DM user " + sender_id + ":" + dm.id + " failed")
                    success = False
                    break

            if success:
                log.log("DM user " + sender_id + ":" + dm.id)

        responded_dms.append(dm.id)
        if len(responded_dms) > 1024:
            responded_dms = responded_dms[1:]


def do_command(command):
    global running
    if command == "help":
        return "Valid commands: $help, $ldphrases, $ldrobots, $stop"
    elif command == "ldrobots":
        loaded = robots.reload()
        return "Loaded " + str(loaded) + " robots"
    elif command == "ldphrases":
        load_phrases()
        return "Reloaded phrases"
    elif command == "stop":
        running = False
        return "Stopping at end current loop"
    return "Unrecognised command"


# Returns true if the mention is either replying to @smolbotbot or nobody, or the mention explicitly
# contains the bot's @.
def is_probably_request(mention):
    return not mention.in_reply_to_user_id_str \
        or mention.in_reply_to_user_id_str == accounts.bot_id \
        or accounts.bot_handle.lower() in twitter.actual_tweet_text(mention).lower()


def contains_ignore(query):
    return ignore_re.search(query)


def load_phrases():
    global greeting_phrases, introduction_phrases

    greetings_file = open("data/greetings.txt", "r")
    greeting_phrases = [line.strip() for line in greetings_file]
    greetings_file.close()

    if not greeting_phrases:
        greeting_phrases = ["[INTERNAL ERROR]"]

    intros_file = open("data/botd-intros.txt", "r")
    introduction_phrases = [line.strip() for line in intros_file]
    intros_file.close()

    if not introduction_phrases:
        introduction_phrases = ["[INTERNAL ERROR]"]


def close_bot():
    tweets_file = open("state/responded-tweets.txt", "w")
    for tweet_id in responded_tweets:
        tweets_file.write(str(tweet_id) + "\n")
    tweets_file.close()
    log.log("Saved responded tweet ids")

    dms_file = open("state/responded-dms.txt", "w")
    for dm_id in responded_dms:
        dms_file.write(dm_id + "\n")
    dms_file.close()
    log.log("Saved responded dm ids")

    log.log("Stopping")


load_phrases()

schedule.every().day.at("07:00").do(daily_robot)
schedule.every().hour.do(check_new_robots)
schedule.every(90).seconds.do(check_direct_messages)
schedule.every(15).seconds.do(check_mentions)


log.log("Starting")

while running:
    try:
        time.sleep(1)
        schedule.run_pending()
        log.flush()
    except KeyboardInterrupt:
        log.log("Keyboard interrupt, stopping")
        break
    except:
        log.log_error("An uncaught error occurred in schedule loop")

close_bot()
print("Goodbye!")
