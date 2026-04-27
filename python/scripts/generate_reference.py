# Copyright 2025 Google LLC
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

import click

python_root_dir = Path(__file__).parent.parent


@click.command()
def main():
    test_scripts_paths = [
        python_root_dir / "tests" / "test_features_extraction_vs_reference.py",
        python_root_dir / "tests" / "test_inference_vs_reference.py",
    ]

    for test_script_path in test_scripts_paths:
        assert test_script_path.is_file()
        cmd = [
            "uv",
            "run",
            str(test_script_path),
            "generate-tests",
        ]

        print(f"Running CMD: {' '.join(cmd)}")
        subprocess.run(
            cmd,
            cwd=python_root_dir,
            check=True,
        )

    print("Everything went good.")


if __name__ == "__main__":
    main()
