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
import tempfile
from pathlib import Path

from magika import colors


def test_python_magika_client() -> None:
    python_root_dir = Path(__file__).parent.parent
    python_magika_client_path = (
        python_root_dir / "src" / "magika" / "cli" / "magika_client.py"
    ).resolve()

    # quick test to check there are no obvious problems
    cmd = [str(python_magika_client_path), "--help"]
    subprocess.run(cmd, capture_output=True, check=True)

    # quick test to check there are no crashes
    cmd = [str(python_magika_client_path), str(python_magika_client_path)]
    subprocess.run(cmd, capture_output=True, check=True)


def test_colored_output_visible_on_light_background_terminals() -> None:
    """Regression test for https://github.com/google/magika/issues/1243.

    The fallback color for content type groups not explicitly mapped in
    color_by_group must not be bright white (\033[1;37m), which is nearly
    invisible on light-background terminals. DARK_GRAY (\033[1;30m) is
    visible on both dark and light backgrounds.
    """
    python_root_dir = Path(__file__).parent.parent
    python_magika_client_path = (
        python_root_dir / "src" / "magika" / "cli" / "magika_client.py"
    ).resolve()

    # Write a plain text file — classified as group="text", which previously
    # fell through to the bright-white fallback color.
    with tempfile.NamedTemporaryFile(suffix=".txt", mode="w", delete=False) as tmp:
        tmp.write("Hello, world!\n")
        tmp_path = Path(tmp.name)

    try:
        result = subprocess.run(
            [str(python_magika_client_path), "--colors", str(tmp_path)],
            capture_output=True,
            text=True,
            check=True,
        )
        output = result.stdout
        # The output must not contain the bright-white ANSI code, which is
        # invisible on white/light-background terminals.
        assert colors.WHITE not in output, (
            f"Output used bright-white ({repr(colors.WHITE)}), which is "
            "invisible on light-background terminals. Use DARK_GRAY instead."
        )
        # DARK_GRAY must be present so that content types in the "text" group
        # (and any other unmapped group) are readable on light terminals.
        assert colors.DARK_GRAY in output, (
            f"Expected DARK_GRAY ({repr(colors.DARK_GRAY)}) in colored output "
            "for a plain-text file, but it was not found."
        )
    finally:
        tmp_path.unlink(missing_ok=True)
