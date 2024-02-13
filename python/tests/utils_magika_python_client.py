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
from typing import List, Optional, Tuple


def run_magika_python_cli(
    samples_paths: List[Path],
    json_output: bool = False,
    jsonl_output: bool = False,
    mime_output: bool = False,
    label_output: bool = False,
    compatibility_mode: bool = False,
    output_probability: bool = False,
    batch_size: Optional[int] = None,
    with_colors: bool = False,
    verbose: bool = False,
    debug: bool = False,
    generate_report: bool = False,
    output_version: bool = False,
    list_output_content_types: bool = False,
    model_dir: Optional[Path] = None,
    extra_cli_options: Optional[List[str]] = None,
) -> Tuple[str, str]:
    cmd = [
        "magika",
    ]
    cmd.extend(map(str, samples_paths))
    if model_dir is not None:
        cmd.append(str(model_dir))
    if json_output is True:
        cmd.append("--json")
    if jsonl_output is True:
        cmd.append("--jsonl")
    if mime_output is True:
        cmd.append("--mime-type")
    if label_output is True:
        cmd.append("--label")
    if compatibility_mode is True:
        cmd.append("--compatibility-mode")
    if output_probability is True:
        cmd.append("--output-probability")
    if batch_size is not None:
        cmd.extend(["--batch-size", str(batch_size)])
    if with_colors:
        cmd.append("--colors")
    else:
        cmd.append("--no-colors")
    if verbose is True:
        cmd.append("--verbose")
    if debug is True:
        cmd.append("--debug")
    if generate_report is True:
        cmd.append("--generate-report")
    if output_version is True:
        cmd.append("--version")
    if list_output_content_types is True:
        cmd.append("--list-output-content-types")
    if extra_cli_options is not None:
        cmd.extend(extra_cli_options)

    p = subprocess.run(cmd, capture_output=True, text=True, check=True)
    return p.stdout, p.stderr
