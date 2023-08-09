#!/usr/bin/env python3

import os
from pathlib import Path
import logging
import json
import sys
from typing import List, Optional

import click

from magika.magika import Magika, MODEL_HASH
from magika import colors
from magika.logger import Logger


VERSION = '0.2.0'


log = None


@click.command()
@click.argument('files', type=click.Path(exists=True, file_okay=True, dir_okay=True, resolve_path=True, path_type=Path), required=False, nargs=-1)
@click.option('-r', '--recursive', 'recursive_flag', is_flag=True, help='Recursively scan every directory.')
@click.option('--json', 'json_output_flag', is_flag=True, help='Output in JSON format.')
@click.option('--jsonl', 'jsonl_output_flag', is_flag=True, help='Output in JSONL format.')
@click.option('-i', '--mime', 'mime_output_flag', is_flag=True, help='Output the MIME type instead of a content type label.')
@click.option('-p', '--with-probability', is_flag=True, help='Output the probability in addition to the content type.')
@click.option('-c', '--compatibility-mode', 'compatibility_mode_flag', is_flag=True, help='Compatibility mode: output is as close as possible to `file` and colors are disabled.')
@click.option('--guess', '--output-highest-probability', 'output_highest_probability', is_flag=True, help='Output the content type with the highest probability, regardless of thresholds.')
@click.option('-b', '--batch-size', default=32, help='How many files to process in one batch.')
@click.option('--colors/--no-colors', 'with_colors_flag', is_flag=True, default=True, help='Enable/disable use of colors.')
@click.option('-v', '--verbose', is_flag=True, help='Enable more verbose output.')
@click.option('-vv', '-d', '--debug', is_flag=True, help='Enable debug logging.')
@click.option('--version', 'version_flag', is_flag=True, help='Print the version and exit.')
@click.option('-l', '--list', 'list_flag', is_flag=True, help='Show a list of supported content types.')
@click.option('--model-dir', type=click.Path(exists=True, file_okay=False, dir_okay=True, resolve_path=True, path_type=Path), help='Use a custom model.')
def main(
    files: List[Path],
    recursive_flag: bool,
    json_output_flag: bool,
    jsonl_output_flag: bool,
    mime_output_flag: bool,
    with_probability: bool,
    compatibility_mode_flag: bool,
    output_highest_probability: bool,
    batch_size: int,
    with_colors_flag: bool,
    verbose: bool,
    debug: bool,
    version_flag: bool,
    list_flag: bool,
    model_dir: Optional[Path]):
    """
    Magika - Determine the type of files using deep-learning.

    Magika tests each argument to attempt to classify it. If a directory is
    passed, Magika processes all files within such directory.

    Note: by default Magika does not recursively process files in subdirectories
    (use -r to enable recursive processing).
    """

    global log

    if compatibility_mode_flag:
        # In compatibility mode we disable colors.
        with_colors_flag = False

    log = Logger(use_colors=with_colors_flag)

    if version_flag:
        print(f'Magika version: {VERSION}')
        print(f'Model hash: {MODEL_HASH}')
        sys.exit(0)

    if list_flag:
        if len(files) > 0:
            log.error('You cannot pass any path when using the -l / --list option')
            sys.exit(1)
        print_content_types_list()
        sys.exit(0)

    if len(files) == 0:
        log.error('You need to pass at least one path')
        sys.exit(1)

    if batch_size <= 0 or batch_size > 512:
        log.error('Batch size needs to be greater than 0 and less or equal than 512')
        sys.exit(1)

    if verbose:
        log.setLevel(logging.INFO)
    if debug:
        log.setLevel(logging.DEBUG)

    # check CLI options
    if json_output_flag and jsonl_output_flag:
        log.error('You should use either --json or --jsonl, not both')
        sys.exit(1)

    if compatibility_mode_flag:
        log.warning('Support for compatibility mode is currently very limited, many outputs will be "Unknown"')

    # expand dirs
    expanded_paths = []
    for p in files:
        if p.is_file():
            expanded_paths.append(p)
        elif p.is_dir():
            if recursive_flag:
                expanded_paths.extend(p.rglob('*'))
            else:
                expanded_paths.extend(p.iterdir())
    # consider only files
    files = sorted(filter(lambda x: x.is_file(), expanded_paths))
    log.info(f'Considering {len(files)} files')
    log.debug(f'Files: {files}')

    m = Magika(
        model_dir=model_dir,
        mime_output_flag=mime_output_flag,
        compatibility_mode_flag=compatibility_mode_flag,
        output_highest_probability=output_highest_probability,
        verbose=verbose,
        debug=debug,
        use_colors=with_colors_flag,
    )

    start_color = ''
    end_color = ''

    color_by_group = {
        'doc': colors.LIGHT_PURPLE,
        'executable': colors.LIGHT_GREEN,
        'archive': colors.LIGHT_RED,
        'audio': colors.YELLOW,
        'image': colors.YELLOW,
        'video': colors.YELLOW,
        'code': colors.LIGHT_BLUE,
    }

    batches_num = len(files) // batch_size
    if len(files) % batch_size != 0:
        batches_num += 1
    all_predictions = []  # updated only when we need to output in JSON format
    for batch_idx in range(batches_num):
        files_ = files[batch_idx*batch_size:(batch_idx+1)*batch_size]
        batch_predictions = m.get_content_types(files_)
        if json_output_flag:
            # we do not stream the output for JSON output
            all_predictions.extend(batch_predictions)
        elif jsonl_output_flag:
            for entry in batch_predictions:
                print(json.dumps(entry))
        else:
            for entry in batch_predictions:
                path = entry['path']
                ct_label = entry['output']['ct_label']
                ct_group = entry['output']['group']
                if mime_output_flag:
                    # If the user requested the MIME type, we use the mime type
                    # regardless of the compatibility mode.
                    output = entry['output']['mime_type']
                elif compatibility_mode_flag:
                    output = entry['output']['magic']
                else:
                    output = f'{ct_group}::{ct_label}'
                score = int(entry['output']['score']*100)

                if with_colors_flag:
                    start_color = color_by_group.get(ct_group, colors.WHITE)
                    end_color = colors.RESET

                if with_probability:
                    print(f'{start_color}{path}: {output} {score}%{end_color}')
                else:
                    print(f'{start_color}{path}: {output}{end_color}')

    if json_output_flag:
        print(json.dumps(all_predictions, indent=4))


def print_content_types_list():
    from magika.content_types import ContentTypesManager
    from tabulate import tabulate

    ctm = ContentTypesManager()
    content_types = ctm.select()

    headers = ['#', 'Content Type Label', 'MIME Type', 'Description']
    rows = []
    for ct_idx, ct in enumerate(content_types):
        row = [
            ct_idx + 1,
            ct.name,
            '' if ct.mime_type is None else ct.mime_type,
            '',
        ]
        rows.append(row)
    print(tabulate(rows, headers=headers))


if __name__ == '__main__':
    main()
