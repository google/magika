#!/usr/bin/env python3
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

import importlib.metadata
import json
import logging
import os
import sys
from pathlib import Path
from typing import List, Optional, Tuple

import click

from magika import Magika, MagikaError, PredictionMode, colors
from magika.logger import get_logger
from magika.types import ContentTypeLabel, MagikaResult
from magika.types.overwrite_reason import OverwriteReason

VERSION = importlib.metadata.version("magika")

CONTACT_EMAIL = "magika-dev@google.com"

CONTEXT_SETTINGS = dict(help_option_names=["-h", "--help"])

HELP_EPILOG = f"""
Magika version: "{VERSION}"\f
Default model: "{Magika._get_default_model_name()}"

Send any feedback to {CONTACT_EMAIL} or via GitHub issues.
"""


@click.command(
    context_settings=CONTEXT_SETTINGS,
    epilog=HELP_EPILOG,
)
@click.argument(
    "file",
    type=click.Path(exists=False, readable=False, path_type=Path),
    required=False,
    nargs=-1,
)
@click.option(
    "-r",
    "--recursive",
    is_flag=True,
    help='When passing this option, magika scans every file within directories, instead of outputting "directory"',
)
@click.option("--json", "json_output", is_flag=True, help="Output in JSON format.")
@click.option("--jsonl", "jsonl_output", is_flag=True, help="Output in JSONL format.")
@click.option(
    "-i",
    "--mime-type",
    "mime_output",
    is_flag=True,
    help="Output the MIME type instead of a verbose content type description.",
)
@click.option(
    "-l",
    "--label",
    "label_output",
    is_flag=True,
    help="Output a simple label instead of a verbose content type description. Use --list-output-content-types for the list of supported output.",
)
@click.option(
    "-c",
    "--compatibility-mode",
    "magic_compatibility_mode",
    is_flag=True,
    help="Compatibility mode: output is as close as possible to `file` and colors are disabled.",
)
@click.option(
    "-s",
    "--output-score",
    is_flag=True,
    help="Output the prediction's score in addition to the content type.",
)
@click.option(
    "-m",
    "--prediction-mode",
    "prediction_mode_str",
    type=click.Choice(PredictionMode.get_valid_prediction_modes(), case_sensitive=True),
    default=PredictionMode.HIGH_CONFIDENCE,
)
@click.option(
    "--batch-size", default=32, help="How many files to process in one batch."
)
@click.option(
    "--no-dereference",
    is_flag=True,
    help="This option causes symlinks not to be followed. By default, symlinks are dereferenced.",
)
@click.option(
    "--colors/--no-colors",
    "with_colors",
    is_flag=True,
    default=True,
    help="Enable/disable use of colors.",
)
@click.option("-v", "--verbose", is_flag=True, help="Enable more verbose output.")
@click.option("-vv", "--debug", is_flag=True, help="Enable debug logging.")
@click.option(
    "--version", "output_version", is_flag=True, help="Print the version and exit."
)
@click.option(
    "--model-dir",
    type=click.Path(
        exists=True, file_okay=False, dir_okay=True, resolve_path=True, path_type=Path
    ),
    help="Use a custom model.",
)
def main(
    file: List[Path],
    recursive: bool,
    json_output: bool,
    jsonl_output: bool,
    mime_output: bool,
    label_output: bool,
    magic_compatibility_mode: bool,
    output_score: bool,
    prediction_mode_str: str,
    batch_size: int,
    no_dereference: bool,
    with_colors: bool,
    verbose: bool,
    debug: bool,
    output_version: bool,
    model_dir: Optional[Path],
) -> None:
    """
    Magika - Determine type of FILEs with deep-learning.
    """

    # click uses the name of the variable to determine how it will show up in
    # the --help. Since we don't like to see "file_paths" in the help, we name
    # the argument "file" (which is ugly) and we re-assign it as soon as we can.
    files_paths = file

    if magic_compatibility_mode:
        # In compatibility mode we disable colors.
        with_colors = False

    _l = get_logger(use_colors=with_colors)

    if verbose:
        _l.setLevel(logging.INFO)
    if debug:
        _l.setLevel(logging.DEBUG)

    if output_version:
        _l.raw_print_to_stdout("Magika python client")
        _l.raw_print_to_stdout(f"Magika version: {VERSION}")
        _l.raw_print_to_stdout(f"Default model: {Magika._get_default_model_name()}")
        sys.exit(0)

    if len(files_paths) == 0:
        _l.error("You need to pass at least one path, or - to read from stdin.")
        sys.exit(1)

    read_from_stdin = False
    for p in files_paths:
        if str(p) == "-":
            read_from_stdin = True
        elif not p.exists():
            _l.error(f'File or directory "{str(p)}" does not exist.')
            sys.exit(1)
    if read_from_stdin:
        if len(files_paths) > 1:
            _l.error('If you pass "-", you cannot pass anything else.')
            sys.exit(1)
        if recursive:
            _l.error('If you pass "-", recursive scan is not meaningful.')
            sys.exit(1)

    if batch_size <= 0 or batch_size > 512:
        _l.error("Batch size needs to be greater than 0 and less or equal than 512.")
        sys.exit(1)

    if json_output and jsonl_output:
        _l.error("You should use either --json or --jsonl, not both.")
        sys.exit(1)

    if int(mime_output) + int(label_output) + int(magic_compatibility_mode) > 1:
        _l.error("You should use only one of --mime, --label, --compatibility-mode.")
        sys.exit(1)

    if recursive:
        # recursively enumerate files within directories
        expanded_paths = []
        for p in files_paths:
            if p.exists():
                if p.is_file():
                    expanded_paths.append(p)
                elif p.is_dir():
                    expanded_paths.extend(sorted(p.rglob("*")))
            elif str(p) == "-":
                # this is "read from stdin", that's OK
                pass
            else:
                _l.error(f'File or directory "{str(p)}" does not exist.')
                sys.exit(1)
        # the resulting list may still include some directories; thus, we filter them out.
        files_paths: List[Path] = list(filter(lambda x: not x.is_dir(), expanded_paths))  # type: ignore[no-redef]

    _l.info(f"Considering {len(files_paths)} files")
    _l.debug(f"Files: {files_paths}")

    # Select an alternative model checking: 1) CLI option, 2) env variable.
    # If none of these is set, model_dir is left to None, and the Magika module
    # will use the default model.
    if model_dir is None:
        model_dir_str = os.environ.get("MAGIKA_MODEL_DIR")
        if model_dir_str is not None and model_dir_str.strip() != "":
            model_dir = Path(model_dir_str)

    try:
        magika = Magika(
            model_dir=model_dir,
            prediction_mode=PredictionMode(prediction_mode_str),
            no_dereference=no_dereference,
            verbose=verbose,
            debug=debug,
            use_colors=with_colors,
        )
    except MagikaError as mr:
        _l.error(str(mr))
        sys.exit(1)

    start_color = ""
    end_color = ""

    color_by_group = {
        "document": colors.LIGHT_PURPLE,
        "executable": colors.LIGHT_GREEN,
        "archive": colors.LIGHT_RED,
        "audio": colors.YELLOW,
        "image": colors.YELLOW,
        "video": colors.YELLOW,
        "code": colors.LIGHT_BLUE,
    }

    # updated only when we need to output in JSON format
    all_predictions: List[Tuple[Path, MagikaResult]] = []

    batches_num = len(files_paths) // batch_size
    if len(files_paths) % batch_size != 0:
        batches_num += 1
    for batch_idx in range(batches_num):
        batch_files_paths = files_paths[
            batch_idx * batch_size : (batch_idx + 1) * batch_size
        ]

        if should_read_from_stdin(files_paths):
            batch_predictions = [get_magika_result_from_stdin(magika)]
        else:
            batch_predictions = magika.identify_paths(batch_files_paths)

        if json_output:
            # we do not stream the output for JSON output
            all_predictions.extend(zip(batch_files_paths, batch_predictions))
        elif jsonl_output:
            for file_path, result in zip(batch_files_paths, batch_predictions):
                _l.raw_print_to_stdout(json.dumps(result.asdict()))
        else:
            for file_path, result in zip(batch_files_paths, batch_predictions):
                if result.ok:
                    if mime_output:
                        # If the user requested the MIME type, we use the mime type
                        # regardless of the compatibility mode.
                        output = result.prediction.output.mime_type
                    elif label_output:
                        output = str(result.prediction.output.label)
                    else:  # human-readable description
                        output = f"{result.prediction.output.description} ({result.prediction.output.group})"

                        if (
                            result.prediction.dl.label != ContentTypeLabel.UNDEFINED
                            and result.prediction.dl.label
                            != result.prediction.output.label
                            and result.prediction.overwrite_reason
                            == OverwriteReason.LOW_CONFIDENCE
                        ):
                            # It seems that we had a low-confidence prediction
                            # from the model. Let's warn the user about our best
                            # bet.
                            output += (
                                " [Low-confidence model best-guess: "
                                f"{result.prediction.dl.description} ({result.prediction.dl.group}), "
                                f"score={result.prediction.score}]"
                            )

                    if with_colors:
                        start_color = color_by_group.get(
                            result.prediction.output.group, colors.WHITE
                        )
                        end_color = colors.RESET
                else:
                    output = result.status
                    start_color = ""
                    end_color = ""

                if output_score and result.ok:
                    score = int(result.prediction.score * 100)
                    _l.raw_print_to_stdout(
                        f"{start_color}{file_path}: {output} {score}%{end_color}"
                    )
                else:
                    _l.raw_print_to_stdout(
                        f"{start_color}{file_path}: {output}{end_color}"
                    )

    if json_output:
        _l.raw_print_to_stdout(
            json.dumps(
                [result.asdict() for (_, result) in all_predictions],
                indent=4,
            )
        )


def should_read_from_stdin(files_paths: List[Path]) -> bool:
    return len(files_paths) == 1 and str(files_paths[0]) == "-"


def get_magika_result_from_stdin(magika: Magika) -> MagikaResult:
    content = sys.stdin.buffer.read()
    result = magika.identify_bytes(content)
    return result


if __name__ == "__main__":
    main()
