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


from pathlib import Path

import tomli
import tomli_w


def main() -> None:
    pyproject_toml_path = Path(__file__).parent.parent / "pyproject.toml"

    pyproject_content = tomli.loads(pyproject_toml_path.read_text())

    # Remove entry about maturin, we don't need it
    _ = pyproject_content["tool"].pop("maturin")

    # Tell uv we want to use the hatchling build system
    pyproject_content["build-system"] = {
        "requires": ["hatchling"],
        "build-backend": "hatchling.build",
    }

    # Make the python's magika client available as a script
    pyproject_content["project"]["scripts"] = {
        "magika-python-client": "magika.cli.magika_client:main",
        "magika": "magika.cli.magika_rust_client_not_found_warning:main",
    }

    pyproject_toml_path.write_text(tomli_w.dumps(pyproject_content))


if __name__ == "__main__":
    main()
