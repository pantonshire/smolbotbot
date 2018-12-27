import time

log_path = "state/log.txt"
lines = []


def log(message):
    message_time = time.strftime("[%d/%m/%y] [%H:%M:%S]")
    full_message = message_time + " " + message
    print(full_message)
    write(full_message)


def write(message):
    global lines
    lines += message


def flush():
    global lines, log_path
    file = open(log_path, "a")
    for line in lines:
        file.write(line)
    file.close()
    lines = []
