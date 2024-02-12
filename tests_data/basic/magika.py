#!/usr/bin/env python3

import hashlib
import json
import logging
import os
import sys
import tempfile
from pathlib import Path
from typing import List, Optional

import click

from magika import colors
from magika.logger import SimpleLogger
from magika.magika import Magika

VERSION = "0.4.1-dev"

DEFAULT_MODEL_NAME = "dense_v4_top_20230910"


log = None


@click.command()
@click.argument(
    "files",
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
    is_flag=True,
    help="Compatibility mode: output is as close as possible to `file` and colors are disabled.",
)
@click.option(
    "-p",
    "--output-probability",
    is_flag=True,
    help="Output the probability in addition to the content type.",
)
@click.option(
    "-m",
    "--prediction-mode",
    type=click.Choice(Magika.PREDICTION_MODES, case_sensitive=True),
    default="default",
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
    files: List[Path],
    recursive: bool,
    json_output: bool,
    jsonl_output: bool,
    mime_output: bool,
    label_output: bool,
    compatibility_mode: bool,
    output_probability: bool,
    prediction_mode: str,
    batch_size: int,
    no_dereference: bool,
    with_colors: bool,
    verbose: bool,
    debug: bool,
    generate_report_flag: bool,
    output_version: bool,
    list_output_content_types: bool,
    model_dir: Optional[Path],
):
    """
    Magika - Determine the type of files using deep-learning.

    Magika tests each argument to attempt to classify it.
    """

    global log

    if compatibility_mode:
        # In compatibility mode we disable colors.
        with_colors = False

    log = SimpleLogger(use_colors=with_colors)

    if verbose:
        log.setLevel(logging.INFO)
    if debug:
        log.setLevel(logging.DEBUG)

    if output_version:
        print(f"Magika version: {VERSION}")
        print(f"Default model name: {DEFAULT_MODEL_NAME}")
        sys.exit(0)

    # check CLI arguments and options
    if list_output_content_types:
        if len(files) > 0:
            log.error("You cannot pass any path when using the -l / --list option")
            sys.exit(1)
        print_output_content_types_list()
        sys.exit(0)

    if len(files) == 0:
        log.error("You need to pass at least one path, or - to read from stdin.")
        sys.exit(1)

    if batch_size <= 0 or batch_size > 512:
        log.error("Batch size needs to be greater than 0 and less or equal than 512")
        sys.exit(1)

    if json_output and jsonl_output:
        log.error("You should use either --json or --jsonl, not both")
        sys.exit(1)

    if int(mime_output) + int(label_output) + int(compatibility_mode) > 1:
        log.error("You should use only one of --mime, --label, --compatibility-mode")
        sys.exit(1)

    if recursive:
        # recursively enumerate files within directories
        expanded_paths = []
        for p in files:
            if p.is_file():
                expanded_paths.append(p)
            elif p.is_dir():
                expanded_paths.extend(sorted(p.rglob("*")))
        # the resulting list may still include some directories; thus, we filter them out.
        files = list(filter(lambda x: not x.is_dir(), expanded_paths))

    log.info(f"Considering {len(files)} files")
    log.debug(f"Files: {files}")

    if len(files) == 1 and str(files[0]) == "-":
        read_from_stdin = True
    else:
        read_from_stdin = False

    # Select the model using the following priority: CLI option, env variable,
    # default.
    if model_dir is None:
        model_dir_str = os.environ.get("MAGIKA_MODEL_DIR")
        if model_dir_str is not None and model_dir_str.strip() != "":
            model_dir = Path(model_dir_str)
        else:
            model_dir = Path(__file__).parent.parent / "models" / DEFAULT_MODEL_NAME

    m = Magika(
        model_dir=model_dir,
        prediction_mode=prediction_mode,
        no_dereference=no_dereference,
        verbose=verbose,
        debug=debug,
        use_colors=with_colors,
    )

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

    batches_num = len(files) // batch_size
    if len(files) % batch_size != 0:
        batches_num += 1
    all_predictions = []  # updated only when we need to output in JSON format

    report_entries = []
    for batch_idx in range(batches_num):
        files_ = files[batch_idx * batch_size : (batch_idx + 1) * batch_size]

        if read_from_stdin:
            # We do stdin processing within the batch logic to avoid code
            # duplication.
            with tempfile.NamedTemporaryFile() as tmp_file:
                tmp_file.write(sys.stdin.buffer.read())
                tmp_file.flush()
                entry = m.get_content_type(Path(tmp_file.name))
                # Patch the path to reflect -
                entry["path"] = "-"
                batch_predictions = [entry]
        else:
            batch_predictions = m.get_content_types(files_)

        if json_output:
            # we do not stream the output for JSON output
            all_predictions.extend(batch_predictions)
        elif jsonl_output:
            for entry in batch_predictions:
                print(json.dumps(entry))
        else:
            for entry in batch_predictions:
                path = entry["path"]

                ct_group = entry["output"]["group"]
                if mime_output:
                    # If the user requested the MIME type, we use the mime type
                    # regardless of the compatibility mode.
                    output = entry["output"]["mime_type"]
                elif label_output:
                    output = entry["output"]["ct_label"]
                elif compatibility_mode:
                    output = entry["output"]["magic"]
                else:  # human-readable description
                    output = f'{entry["output"]["description"]} ({ct_group})'

                score = int(entry["output"]["score"] * 100)

                if with_colors:
                    start_color = color_by_group.get(ct_group, colors.WHITE)
                    end_color = colors.RESET

                if output_probability:
                    print(f"{start_color}{path}: {output} {score}%{end_color}")
                else:
                    print(f"{start_color}{path}: {output}{end_color}")

        if generate_report_flag:
            for file_path, entry in zip(files_, batch_predictions):
                # remove information we don't need, e.g., paths
                entry_copy = entry.copy()
                entry_copy["path"] = "<REMOVED>"
                fs = m.extract_features_from_path(file_path)
                report_entry = {
                    "hash": hashlib.sha256(file_path.read_bytes()).hexdigest(),
                    "features": {
                        "beg": fs["beg"],
                        "mid": fs["mid"],
                        "end": fs["end"],
                    },
                    "output": entry_copy,
                }
                report_entries.append(report_entry)

    if json_output:
        print(json.dumps(all_predictions, indent=4))

    if generate_report_flag:
        report = {
            "version": VERSION,
            "model_dir": str(model_dir),
            "entries": report_entries,
        }
        report_header = "REPORT"
        report_header_full_len = 40
        print(
            "Please copy/paste the following as a description of your issue:",
            file=sys.stderr,
        )
        print("#" * report_header_full_len, file=sys.stderr)
        print(
            "###"
            + (" " * ((report_header_full_len - 6 - len(report_header)) // 2))
            + report_header
            + (" " * ((report_header_full_len - 6 - len(report_header)) // 2))
            + "###",
            file=sys.stderr,
        )
        print("#" * report_header_full_len, file=sys.stderr)
        print(json.dumps(report), file=sys.stderr)
        print("#" * report_header_full_len, file=sys.stderr)


def print_output_content_types_list():
    from tabulate import tabulate  # type: ignore

    from magika.content_types import ContentTypesManager

    ctm = ContentTypesManager()
    content_types = ctm.get_output_content_types()

    headers = ["#", "Content Type Label", "MIME Type", "Description"]
    rows = []
    for ct_idx, ct in enumerate(content_types):
        row = [
            ct_idx + 1,
            ct.name,
            "" if ct.mime_type is None else ct.mime_type,
            "" if ct.description is None else ct.description,
        ]
        rows.append(row)
    print(tabulate(rows, headers=headers))


if __name__ == "__main__":
    main()
