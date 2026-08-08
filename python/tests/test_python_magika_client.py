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

import os
import subprocess
from pathlib import Path


def _python_magika_test_env() -> dict[str, str]:
    python_root_dir = Path(__file__).parent.parent.resolve()
    env = dict(os.environ)
    src_path = str(python_root_dir / "src")
    env["PYTHONPATH"] = (
        src_path
        if not env.get("PYTHONPATH")
        else f"{src_path}:{env['PYTHONPATH']}"
    )
    return env


def test_python_magika_client() -> None:
    python_root_dir = Path(__file__).parent.parent
    python_magika_client_path = (
        python_root_dir / "src" / "magika" / "cli" / "magika_client.py"
    ).resolve()

    # quick test to check there are no obvious problems
    cmd = [str(python_magika_client_path), "--help"]
    subprocess.run(cmd, capture_output=True, check=True, env=_python_magika_test_env())

    # quick test to check there are no crashes
    cmd = [str(python_magika_client_path), str(python_magika_client_path)]
    subprocess.run(cmd, capture_output=True, check=True, env=_python_magika_test_env())


def test_python_magika_client_group_colors_are_readable() -> None:
    colors_path = (
        Path(__file__).parent.parent / "src" / "magika" / "colors.py"
    ).resolve()
    namespace: dict[str, object] = {}
    exec(colors_path.read_text(), namespace)

    color_for_group = namespace["color_for_group"]

    assert color_for_group("text") == namespace["CYAN"]
    assert color_for_group("unknown") == namespace["DARK_GRAY"]
    assert color_for_group(None) == namespace["CYAN"]
