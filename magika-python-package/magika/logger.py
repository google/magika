# Copyright 2024 Google LLC
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

import logging
import sys

from magika import colors

# We implement a simple logger to not rely on additional python packages, e.g.,
# rich.


class SimpleLogger:
    def __init__(self, use_colors: bool = False):
        self.level = logging.WARNING
        self.use_colors = use_colors

    def setLevel(self, level: int) -> None:
        self.level = level

    def _print(self, msg: str) -> None:
        print(msg, file=sys.stderr)

    def debug(self, msg: str) -> None:
        if logging.DEBUG >= self.level:
            if self.use_colors:
                self._print(f"{colors.GREEN}DEBUG: {msg}{colors.RESET}")
            else:
                self._print(f"DEBUG: {msg}")

    def info(self, msg: str) -> None:
        if logging.INFO >= self.level:
            self._print(f"INFO: {msg}")

    def warning(self, msg: str) -> None:
        if logging.WARNING >= self.level:
            if self.use_colors:
                self._print(f"{colors.YELLOW}WARNING: {msg}{colors.RESET}")
            else:
                self._print(f"WARNING: {msg}")

    def error(self, msg: str) -> None:
        if logging.ERROR >= self.level:
            if self.use_colors:
                self._print(f"{colors.RED}ERROR: {msg}{colors.RESET}")
            else:
                self._print(f"ERROR: {msg}")
