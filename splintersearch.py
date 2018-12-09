from splinter import Browser
import time
import re

robots = []

max_to_check = 652

inf = open("old-robot-table.csv", "rt")
numbers = [int(line.split(",")[0]) for line in inf]
tocheck = []
for x in range(1, max_to_check):
    if not x in numbers:
        tocheck.append(x)
inf.close()

with Browser() as browser:
    for no in tocheck:
        url = "https://twitter.com/search?l=&q=%22" + str(no) + ")%22%20from%3Asmolrobots&src=typd"
        browser.visit(url)
        
        attempts = 0
        found = False
        while attempts < 10 and not found:
            stream = browser.find_by_id("stream-items-id")
            try:
                tweets = stream.find_by_css(".tweet")
                for tweet in tweets:
                    content = tweet.find_by_css(".tweet-text")[0].text
                    search = re.search("(^|\s)" + str(no) + "\)\ [\w|\-]+bot\w*", content, re.IGNORECASE)
                    if search != None:
                        name = search.group().split(") ")[1].strip()
                        tweetid = tweet["data-tweet-id"]
                        robot = (no, name, tweetid)
                        robots.append(robot)
                        print("Robot #" + str(no) + ": " + name + ", id " + str(tweetid))
                        found = True
                        break
            except:
                print("An error occurred")
                break
            browser.execute_script("window.scrollTo(0, document.body.scrollHeight);")
            time.sleep(0.5)
            attempts += 1
        if not found:
            print("No robot #" + str(no) + " found")

outputfile = open("old-robot-table.csv", "at")
for robot in robots:
    outputfile.write(",".join([str(item) for item in robot]) + "\n")
outputfile.close()
