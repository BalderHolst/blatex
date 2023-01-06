from os import stat
from pathlib import Path
import re

import click

class Error():
    def __init__(self, message, trace) -> None:
        self.message = message
        self.trace = trace

    def __repr__(self) -> str:
        return f"Error({self.trace})"

class PackageError(Error):
    def __init__(self,package_name, message, trace) -> None:
        super().__init__(message, trace)
        self.package_name = package_name

    def __repr__(self) -> str:
        return f"PackageError({self.package_name!r}, {self.trace})"

class Warning():
    def __init__(self, message, trace) -> None:
        self.message = message
        self.trace = trace

    def __repr__(self) -> str:
        return f"Warning({self.trace})"

class HboxWarning(Warning):
    def __init__(self, type, message, trace) -> None:
        super().__init__(message, trace)
        self.type = type

    def __repr__(self) -> str:
        return f"HboxWarning({self.type}, {self.trace})"

def is_path(line):
    m = re.search(r"\.?(/[^\r\n]+)+\.\w+", line)
    if m:
        return True
    return False

def clean_error_text(text):
    # Remove interactive prompt.
    text = re.sub("See the LaTeX manual or LaTeX Companion for explanation.\nType  H <return>  for immediate help.\n ... ", "", text)

    # fix strange newline
    text = re.sub(r"([^\.])\s+({.+)\n", r"\1\2\n\n", text)

    # fix 80 horizontal char limit
    text = re.sub(r"([a-z ,])\n([a-z ,])", r"\1\2", text)

    # Make all newlines after "." double
    text = re.sub(r"\.\n\s*\n", r".\n", text)
    text = re.sub(r"\.\s*(\n|\r)", r".\n\n", text)

    return(text)

def get_error_message(lines):
    MAX_EMPTY_LINES = 2

    text = "\n".join(lines)

    text = clean_error_text(text)

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


def get_package_error_message(lines):

    MAX_EMPTY_LINES = 4

    text = "\n".join(lines)

    text = clean_error_text(text)

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

def echo_with_level(line, level):
    click.echo("".join(["|   " for _ in range(level)]) + line)
    
def extract_file(line):
    m = re.search(r"(\.?(/[^/]+)+$)", line)
    if m:
        return(m.group())
    return(line)

def parse_log_file(log_file: Path, echo_logs = False):

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
                    stack.append(extract_file(line))

        # Detect package errors
        m = re.search(r"! Package ([^/]+) Error: ", line)
        if m:
            error_name = m.group(1)
            error_lines = get_package_error_message(lines[n:n+SEARCH_LINES])

            errors.append(PackageError(error_name, "\n".join(error_lines), stack.copy()))

        # Detect errors
        m = re.search("! LaTeX Error:", line)
        if m:
            error_lines = get_error_message(lines[n:n+SEARCH_LINES])
            errors.append(Error("\n".join(error_lines), stack.copy()))

        # TODO detect warnings

        m = re.search(r"(Overfull|Underfull) \\hbox", line)
        if m:
            errors.append(HboxWarning(
                        type=f"{m.group(1)}",
                        message=line,
                        trace=stack.copy()
                        ))

        if echo_logs:
            echo_with_level(line, len(stack))



    return(errors)

