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

import subprocess
import sys
from pathlib import Path

CLIENT = (
    Path(__file__).parent.parent / "src" / "magika" / "cli" / "magika_client.py"
).resolve()


def _run(*args: str) -> subprocess.CompletedProcess:
    return subprocess.run(
        [sys.executable, str(CLIENT), *args], capture_output=True, text=True
    )


def test_python_magika_client() -> None:
    # quick test to check there are no obvious problems
    subprocess.run([sys.executable, str(CLIENT), "--help"], capture_output=True, check=True)

    # quick test to check there are no crashes
    subprocess.run([sys.executable, str(CLIENT), str(CLIENT)], capture_output=True, check=True)


def test_compatibility_mode_removed() -> None:
    result = _run("--compatibility-mode", str(CLIENT))
    assert result.returncode != 0, "--compatibility-mode should no longer be accepted"


def test_no_colors_flag() -> None:
    result = _run("--no-colors", str(CLIENT))
    assert result.returncode == 0
    assert "\x1b[" not in result.stdout, "--no-colors should produce no ANSI escape codes"


def test_colors_flag() -> None:
    result = _run("--colors", str(CLIENT))
    assert result.returncode == 0
