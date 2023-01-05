from pathlib import Path
import re

def is_path(line):
    m = re.search(r"\.?(/[^\r\n]+)+\.\w+", line)
    if m:
        return True
    return False

# def is_new_file(line):
#     m = re.search(r"\(\.?(/[^\r\n]+)+\.\w+", line)
#     if m:
#         if m.span() == (0, len(line)):
#             return m.group()[1:]
#     return(False)

def get_error_message(lines):
    MAX_EMPTY_LINES = 2

    text = "\n".join(lines)

    # Remove interactive prompt.
    text = re.sub("See the \\w+ package documentation for explanation.\nType  H <return>  for immediate help.\n ...\\s+", "", text)

    # fix strange newline
    text = re.sub(r"([^\.])\s+({.+)\n", r"\1\2\n\n", text)

    # fix 80 horizontal char limit
    text = re.sub(r"([a-z ])\n([a-z ])", r"\1\2", text)

    error_lines = []

    empty_lines = 0
    for line in text.split("\n"):

        # Check if the line consists of nothing but spaces
        if re.search(r"^\s*$", line):
            empty_lines += 1
            continue

        if empty_lines > MAX_EMPTY_LINES:
            break

        error_lines.append(line)

    return(error_lines)

def get_warning_message(text):
    raise NotImplementedError()

def print_with_level(line, level):
    print("".join(["|   " for _ in range(level)]) + line)
    

def parse_log_file(log_file: Path):

    log = log_file.read_text()#.replace("(", "\n(\n").replace(")", "\n)\n")

    lines = log.split("\n")


    stack = []

    errors = []

    for n, line in enumerate(lines):

        SEARCH_LINES = 30

        if line.count("(") > 0 or line.count(")") > 0:
            for c in line:
                if c == ")":
                    stack.pop()
                elif c == "(":
                    stack.append(line)

        # Detect package errors
        m = re.search(r"! Package (\w+) Error: ", line)
        if m:
            error_name = m.group(1)
            lines = get_error_message(lines[n:n+SEARCH_LINES])

            errors.append({
                "type": "Package",
                "package_name": error_name,
                "message": lines,
                "where": stack.copy()
                })

        print_with_level(line, len(stack))

        # TODO detect regular errors

        # TODO detect warnings

    print(errors)

