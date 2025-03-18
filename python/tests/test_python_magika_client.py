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
from pathlib import Path


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
