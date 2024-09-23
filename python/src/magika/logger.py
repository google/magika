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

from __future__ import annotations

import logging
import sys
from typing import Optional, TextIO

from magika import colors

_logger: Optional[SimpleLogger] = None


class SimpleLogger:
    """
    We implement a simple logger to not rely on additional python packages,
    e.g., rich. This is written in way that, by default, log messages (e.g.,
    debug/info/...) are sent to stderr.
    """

    def __init__(self, use_colors: bool = False):
        self.level = logging.WARNING
        self.use_colors = use_colors

    def setLevel(self, level: int) -> None:
        self.level = level

    def raw_print_to_stdout(self, msg: str) -> None:
        self.raw_print(msg, file=sys.stdout)

    def raw_print(
        self, msg: str, file: Optional[TextIO] = None, flush: bool = True
    ) -> None:
        if file is None:
            # We avoid using a default value for the `file` argument because we
            # need to get the reference to the "current" stderr; if we used a
            # default argument, we would just store the "current at
            # instantiation time" stderr, which may not be the current one.
            # This, in turn, could create problems for testing.
            file = sys.stderr
        print(msg, file=file, flush=flush)

    def debug(self, msg: str) -> None:
        if logging.DEBUG >= self.level:
            if self.use_colors:
                self.raw_print(f"{colors.GREEN}DEBUG: {msg}{colors.RESET}")
            else:
                self.raw_print(f"DEBUG: {msg}")

    def info(self, msg: str) -> None:
        if logging.INFO >= self.level:
            self.raw_print(f"INFO: {msg}")

    def warning(self, msg: str) -> None:
        if logging.WARNING >= self.level:
            if self.use_colors:
                self.raw_print(f"{colors.YELLOW}WARNING: {msg}{colors.RESET}")
            else:
                self.raw_print(f"WARNING: {msg}")

    def error(self, msg: str) -> None:
        if logging.ERROR >= self.level:
            if self.use_colors:
                self.raw_print(f"{colors.RED}ERROR: {msg}{colors.RESET}")
            else:
                self.raw_print(f"ERROR: {msg}")


def get_logger(use_colors: bool = False) -> SimpleLogger:
    global _logger

    if _logger is None:
        _logger = SimpleLogger(use_colors=use_colors)

    return _logger
