import logging
import sys
from typing import Optional

from magika import colors


class Logger:
    def __init__(self, use_colors: bool = False):
        self.level = logging.WARNING
        self.use_colors = use_colors

    def setLevel(self, level):
        self.level = level

    def debug(self, msg):
        if logging.DEBUG >= self.level:
            if self.use_colors:
                print(f"{colors.GREEN}DEBUG: {msg}{colors.RESET}")
            else:
                print(f"DEBUG: {msg}")

    def info(self, msg):
        if logging.INFO >= self.level:
            print(f"INFO: {msg}")

    def warning(self, msg):
        if logging.WARNING >= self.level:
            if self.use_colors:
                print(f"{colors.YELLOW}WARNING: {msg}{colors.RESET}")
            else:
                print(f"WARNING: {msg}")

    def error(self, msg):
        if logging.ERROR >= self.level:
            if self.use_colors:
                print(f"{colors.RED}ERROR: {msg}{colors.RESET}")
            else:
                print(f"ERROR: {msg}")
