import botlookup as search
import twitter
print("> Logged in")
import codecs
import time
import re
import tweepy

print("> Loading robots from old-robot-table.csv")
robotfile = codecs.open("old-robot-table.csv", "r", "utf-8")
robots = [tuple(line.strip().split(",")) for line in robotfile]
robotfile.close()
print("> Loaded " + str(len(robots)) + " robots")

print("> Retrieving direct messages from last 24 hours")
dms = twitter.direct_messages(86400, [])
for dm in dms:
   print("[DM] " + dm["message_create"]["message_data"]["text"].replace("\n", "").strip())

print("> Direct messaging @PantonshireDev")
twitter.send_direct_message("1030814512851681280", "Setup test")

print("> Retrieving last 20 mentions from last 24 hours")
mentions = twitter.mentions(20, 86400, [])
for mention in mentions:
   print("[@] " + mention.text.replace("\n", "").strip())

print("\n> Input test query / exit")
while True:
   q = input()
   if q.strip().lower() == "exit":
      break
   else:
      print("[R] " + search.search(robots, q))

print("> Test complete")
