#!/usr/bin/env python
# Copyright 2023-2024 Google LLC
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     https://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

"""
This script should only rely on dependencies installed with `pip install
magika`; this script is used as part of "build & test package" github action,
and the dev dependencies are not available.
"""

import statistics
import sys
import time
from pathlib import Path

import click

from magika import ContentTypeLabel, Magika, PredictionMode


@click.command()
@click.option("--print-inference-stats", is_flag=True, help="Print inference stats.")
@click.option("--repeat", default=1, help="Number of times to run the test set.")
def main(print_inference_stats: bool, repeat: int) -> None:
    m = Magika(prediction_mode=PredictionMode.HIGH_CONFIDENCE)

    print(f"Magika instance details: {m}")

    res = m.identify_bytes(b"text")
    assert res.dl.label == ContentTypeLabel.UNDEFINED
    assert res.output.label == ContentTypeLabel.TXT
    assert res.score == 1.0

    res = m.identify_bytes(b"\xff\xff\xff")
    assert res.dl.label == ContentTypeLabel.UNDEFINED
    assert res.output.label == ContentTypeLabel.UNKNOWN
    assert res.score == 1.0

    basic_tests_dir = (
        Path(__file__).parent.parent.parent / "tests_data" / "basic"
    ).resolve()

    files_paths = sorted(filter(lambda p: p.is_file(), basic_tests_dir.rglob("*")))

    latencies = []

    with_error = False
    for i in range(repeat):
        for file_path in files_paths:
            start_time = time.perf_counter()
            res = m.identify_path(file_path)
            end_time = time.perf_counter()
            latencies.append((end_time - start_time) * 1000)

            # Check for misprediction only on the first run.
            if i == 0:
                output_label = res.output.label
                expected_label = file_path.parent.name
                if expected_label != output_label:
                    with_error = True
                    print(
                        f"ERROR: Misprediction for {file_path}: expected_label={expected_label}, output_label={output_label}"
                    )

    if with_error:
        print("ERROR: There was at least one misprediction")
        sys.exit(1)

    print("All examples were predicted correctly")

    if print_inference_stats and latencies:
        print(f"Inference stats over {len(latencies)} files (repeat={repeat}):")
        print(f"  Min: {min(latencies):.4f} ms")
        print(f"  Max: {max(latencies):.4f} ms")
        print(f"  Mean: {statistics.mean(latencies):.4f} ms")
        print(f"  Median: {statistics.median(latencies):.4f} ms")
        print(f"  Total: {sum(latencies):.4f} ms")


if __name__ == "__main__":
    main()
