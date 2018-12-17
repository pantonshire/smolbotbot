
log_path = "state/log.txt"
lines = []


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
