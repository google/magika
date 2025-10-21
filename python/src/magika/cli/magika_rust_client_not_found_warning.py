#!/usr/bin/env python
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

"""Placeholder script for the primary 'magika' command-line interface.

This module serves as a fallback entry point for the 'magika' command. **It is
included only in the pure-Python package.** If this script is executed, it
indicates that the user has installed the pure-Python package and not the
package that contains the native binary.

The script explicitly notifies the user that they are not using the binary
client and guides them to use the alternative Python client (`$
magika-python-client`) or to seek support.
"""

import sys


def main() -> None:  # noqa:  D103
    message = """
WARNING: you have attempted to run `$ magika` (the Rust client), but this is not
available in the python package you installed, likely because magika pipeline
does not currently build binary wheels compatible with your platform settings.

If you think this is a problem worth solving, please open an issue at
https://github.com/google/magika.

In the meantime, you can use the old python magika client with `$ magika-python-client`.
"""

    print(message.strip())
    sys.exit(1)


if __name__ == "__main__":
    main()
