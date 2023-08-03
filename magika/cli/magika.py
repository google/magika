#!/usr/bin/env python3

import os
from pathlib import Path
import logging
import json
import sys
from tabulate import tabulate
from typing import List, Optional

import click


from magika import get_logger
from magika.magika import Magika


l = get_logger()
l.setLevel(logging.WARNING)


@click.command()
@click.argument('paths', type=click.Path(exists=True, file_okay=True, dir_okay=True, resolve_path=True, path_type=Path), required=False, nargs=-1)
@click.option('-r', '--recursive', 'recursive_flag', is_flag=True, help='Recursively scan every directory.')
@click.option('--json', 'json_output_flag', is_flag=True, help='Output in JSON format.')
@click.option('--jsonl', 'jsonl_output_flag', is_flag=True, help='Output in JSONL format.')
@click.option('-i', '--mime', 'mime_output_flag', is_flag=True, help='Output the MIME type instead of a content type label.')
@click.option('-p', '--with-probability', is_flag=True, help='Output the probability in addition to the content type.')
@click.option('-t', '--threshold', default=0.0)
@click.option('-v', '--verbose', is_flag=True, help='Enable more verbose output.')
@click.option('-vv', '-d', '--debug', is_flag=True, help='Enable debug logging.')
@click.option('-l', '--list', 'list_flag', is_flag=True, help='Show a list of supported content types')
@click.option('--model-dir', type=click.Path(exists=True, file_okay=False, dir_okay=True, resolve_path=True, path_type=Path), help='Use a custom model.')
def main(
    paths: List[Path],
    recursive_flag: bool,
    json_output_flag: bool,
    jsonl_output_flag: bool,
    mime_output_flag: bool,
    with_probability: bool,
    threshold: float,
    verbose: bool,
    debug: bool,
    list_flag: bool,
    model_dir: Optional[Path]):
    """
    Determine the content types of PATHS.

    PATHS is one or more paths. A path can point to a file or a directory: if it
    points to a directory, Magika processes all files within such directory.
    Note that by default Magika does not recursively process files in
    subdirectories (use -r to enable recursive processing).
    """

    if list_flag:
        if len(paths) > 0:
            l.error('You cannot pass any path when using the -l / --list option.')
            sys.exit(1)
        print_content_types_list()
        sys.exit(0)

    if len(paths) == 0:
        l.error('You need to pass at least one path')
        sys.exit(1)

    if verbose:
        l.setLevel(logging.INFO)
    if debug:
        l.setLevel(logging.DEBUG)

    # check CLI options
    if json_output_flag and jsonl_output_flag:
        l.error('You should use either --json or --jsonl, not both')
        sys.exit(1)

    if mime_output_flag:
        l.warning('Support for outputting MIME type is currently very limited!')

    # expand dirs
    expanded_paths = []
    for p in paths:
        if p.is_file():
            expanded_paths.append(p)
        elif p.is_dir():
            if recursive_flag:
                expanded_paths.extend(p.rglob('*'))
            else:
                expanded_paths.extend(p.iterdir())
    # consider only files
    paths = sorted(filter(lambda x: x.is_file(), expanded_paths))
    l.info(f'Considering {len(paths)} files')
    l.debug(f'Files: {paths}')

    m = Magika(
        model_dir=model_dir,
        threshold=threshold,
        mime_output_flag=mime_output_flag,
        verbose=verbose,
        debug=debug,
    )
    predictions = m.get_content_types(paths)
    if json_output_flag:
        print(json.dumps(predictions, indent=4))
    elif jsonl_output_flag:
        for entry in predictions:
            print(json.dumps(entry))
    else:
        for entry in predictions:
            path = entry['path']
            if mime_output_flag:
                output = entry['output']['mime_type']
            else:
                output = entry['output']['ct_label']
            score = entry['output']['score']
            if with_probability:
                print(f'{path}: {output} {score=}')
            else:
                print(f'{path}: {output}')


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
