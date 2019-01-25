import time
import traceback

log_path = "state/log.txt"
lines = []

begin_exception = ("-" * 15) + " BEGIN EXCEPTION " + ("-" * 15)
end_exception = ("-" * 15) + " END EXCEPTION " + ("-" * 15)


def log(message):
    message_time = time.strftime("[%d/%m/%y] [%H:%M:%S]")
    full_message = message_time + " " + message
    print(full_message)
    write(full_message)


def log_error(message):
    global begin_exception, end_exception
    exception = traceback.format_exc()
    log("!! ERROR !! " + message + "\n" + begin_exception + "\n" + exception + end_exception)


def write(message):
    global lines
    lines.append(message)


def flush():
    global lines, log_path
    if lines:
        file = open(log_path, "a")
        for line in lines:
            file.write(line + "\n")
        file.close()
        lines.clear()
    
