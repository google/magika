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

import enum

from magika.types.strenum import LowerCaseStrEnum


class Status(LowerCaseStrEnum):
    OK = enum.auto()

    # Used when a file path does not exist.
    FILE_NOT_FOUND_ERROR = enum.auto()

    # Used when a file path exists, but there are permission issues, e.g., can't
    # read file.
    PERMISSION_ERROR = enum.auto()

    # Represents a generic error-like unknown status.
    UNKNOWN = enum.auto()
