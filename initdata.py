def read_lines(path):
    data_file = open(path, "r")
    lines = [line.strip() for line in data_file]
    data_file.close()
    return lines
