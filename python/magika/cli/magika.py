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


import copy
import dataclasses
import hashlib
import json
import logging
import os
import sys
from pathlib import Path
from typing import List, Optional

import click
from tabulate import tabulate

from magika import Magika, MagikaError, PredictionMode, colors
from magika.content_types import ContentTypesManager
from magika.logger import get_logger
from magika.types import FeedbackReport, MagikaResult

VERSION = "0.5.1"

CONTACT_EMAIL = "magika-dev@google.com"

CONTEXT_SETTINGS = dict(help_option_names=["-h", "--help"])

HELP_EPILOG = f"""
Magika version: "{VERSION}"\f
Default model: "{Magika.get_default_model_name()}"

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
    "--generate-report",
    "generate_report_flag",
    is_flag=True,
    help="Generate report useful when reporting feedback.",
)
@click.option(
    "--version", "output_version", is_flag=True, help="Print the version and exit."
)
@click.option(
    "--list-output-content-types",
    "list_output_content_types",
    is_flag=True,
    help="Show a list of supported content types.",
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
    generate_report_flag: bool,
    output_version: bool,
    list_output_content_types: bool,
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
        _l.raw_print_to_stdout(f"Magika version: {VERSION}")
        _l.raw_print_to_stdout(f"Default model: {Magika.get_default_model_name()}")
        sys.exit(0)

    # check CLI arguments and options
    if list_output_content_types:
        if len(files_paths) > 0:
            _l.error("You cannot pass any path when using the -l / --list option.")
            sys.exit(1)
        print_output_content_types_list()
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
        files_paths = list(filter(lambda x: not x.is_dir(), expanded_paths))

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
    all_predictions: List[MagikaResult] = []

    # used only when the user decides to generate a feedback report
    report_entries: List[FeedbackReport] = []

    batches_num = len(files_paths) // batch_size
    if len(files_paths) % batch_size != 0:
        batches_num += 1
    for batch_idx in range(batches_num):
        files_ = files_paths[batch_idx * batch_size : (batch_idx + 1) * batch_size]

        if should_read_from_stdin(files_paths):
            batch_predictions = [get_magika_result_from_stdin(magika)]
        else:
            batch_predictions = magika.identify_paths(files_)

        if json_output:
            # we do not stream the output for JSON output
            all_predictions.extend(batch_predictions)
        elif jsonl_output:
            for magika_result in batch_predictions:
                _l.raw_print_to_stdout(json.dumps(dataclasses.asdict(magika_result)))
        else:
            for magika_result in batch_predictions:
                path = magika_result.path
                output_ct_label = magika_result.output.ct_label
                output_ct_description = magika_result.output.description
                output_ct_group = magika_result.output.group

                if mime_output:
                    # If the user requested the MIME type, we use the mime type
                    # regardless of the compatibility mode.
                    output = magika_result.output.mime_type
                elif label_output:
                    output = magika_result.output.ct_label
                elif magic_compatibility_mode:
                    output = magika_result.output.magic
                else:  # human-readable description
                    dl_ct_label = magika_result.dl.ct_label

                    output = f"{output_ct_description} ({output_ct_group})"

                    if dl_ct_label is not None and dl_ct_label != output_ct_label:
                        # it seems that we had a too-low confidence prediciton
                        # from the model. Let's warn the user about our best
                        # bet.
                        assert magika_result.dl.score is not None
                        dl_description = magika_result.dl.description
                        dl_group = magika_result.dl.group
                        dl_score = int(magika_result.dl.score * 100)
                        output += f" [Low-confidence model best-guess: {dl_description} ({dl_group}), score={dl_score}]"

                if with_colors:
                    start_color = color_by_group.get(output_ct_group, colors.WHITE)
                    end_color = colors.RESET

                if output_score:
                    score = int(magika_result.output.score * 100)
                    _l.raw_print_to_stdout(
                        f"{start_color}{path}: {output} {score}%{end_color}"
                    )
                else:
                    _l.raw_print_to_stdout(f"{start_color}{path}: {output}{end_color}")

        if generate_report_flag:
            for file_path, magika_result in zip(files_, batch_predictions):
                report_entries.append(
                    generate_feedback_report(magika, file_path, magika_result)
                )

    if json_output:
        _l.raw_print_to_stdout(
            json.dumps([dataclasses.asdict(res) for res in all_predictions], indent=4)
        )

    if generate_report_flag:
        print_feedback_report(magika=magika, reports=report_entries)


def should_read_from_stdin(files_paths: List[Path]) -> bool:
    return len(files_paths) == 1 and str(files_paths[0]) == "-"


def get_magika_result_from_stdin(magika: Magika) -> MagikaResult:
    content = sys.stdin.buffer.read()
    result = magika.identify_bytes(content)
    return result


def generate_feedback_report(
    magika: Magika, file_path: Path, magika_result: MagikaResult
) -> FeedbackReport:
    magika_result_copy = copy.copy(magika_result)
    magika_result_copy.path = "<REMOVED>"  # avoid PII
    features = Magika._extract_features_from_path(
        file_path,
        beg_size=magika._input_sizes["beg"],
        mid_size=magika._input_sizes["mid"],
        end_size=magika._input_sizes["end"],
        padding_token=magika._padding_token,
        block_size=magika._block_size,
    )
    return FeedbackReport(
        hash=hashlib.sha256(file_path.read_bytes()).hexdigest(),
        features=features,
        result=magika_result_copy,
    )


def print_feedback_report(magika: Magika, reports: List[FeedbackReport]) -> None:
    _l = get_logger()

    processed_reports = [
        {
            "hash": report.hash,
            "features": json.dumps(dataclasses.asdict(report.features)).replace(
                " ", ""
            ),
            "result": dataclasses.asdict(report.result),
        }
        for report in reports
    ]

    full_report = {
        "version": VERSION,
        "model_dir_name": magika.get_model_name(),
        "python_version": sys.version,
        "reports": processed_reports,
    }
    report_header = "REPORT"
    report_header_full_len = 40
    _l.raw_print("#" * report_header_full_len)
    _l.raw_print(
        "###"
        + (" " * ((report_header_full_len - 6 - len(report_header)) // 2))
        + report_header
        + (" " * ((report_header_full_len - 6 - len(report_header)) // 2))
        + "###",
    )
    _l.raw_print("#" * report_header_full_len)
    _l.raw_print(json.dumps(full_report))
    _l.raw_print("#" * report_header_full_len)
    _l.raw_print(
        f"Please copy/paste the above as a description of your issue. Open a GitHub issue or reach out at {CONTACT_EMAIL}.",
    )
    _l.raw_print(
        "Please include as many details as possible, e.g., what was the expected content type.",
    )
    _l.raw_print(
        "IMPORTANT: do NOT submit private information or PII! The extracted features include many bytes of the tested files!",
    )


def print_output_content_types_list() -> None:
    _l = get_logger()

    ctm = ContentTypesManager()
    content_types = ctm.get_output_content_types()

    headers = ["#", "Content Type Label", "Description"]
    rows = []
    for ct_idx, ct in enumerate(content_types):
        row = [
            ct_idx + 1,
            ct.name,
            "" if ct.description is None else ct.description,
        ]
        rows.append(row)
    _l.raw_print_to_stdout(tabulate(rows, headers=headers))


if __name__ == "__main__":
    main()
