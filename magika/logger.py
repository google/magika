#!/usr/bin/python
#
# Copyright 2023 Google, LLC
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#      http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

"""TODO: High-level file comment."""

import sys


def main(argv):
    pass


if __name__ == '__main__':
    main(sys.argv)
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
