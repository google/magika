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

import dotenv
from rich.console import Console
from rich.logging import RichHandler

dotenv.load_dotenv(dotenv.find_dotenv())


def get_logger(logger_name="magika"):
    FORMAT = "%(name)s: %(message)s"
    logging.basicConfig(
        level="INFO",
        format=FORMAT,
        datefmt="[%x %X]",
        handlers=[RichHandler(console=Console(stderr=True))],
    )
    _l = logging.getLogger(logger_name)
    return _l


# disable annoying logging from google library: https://github.com/googleapis/google-api-python-client/issues/299
logging.getLogger("googleapiclient.discovery_cache").setLevel(logging.ERROR)
