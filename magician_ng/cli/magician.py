#!/usr/bin/env python3

import os
from pathlib import Path
import logging
import json
import subprocess
import sys
import tarfile
from typing import List, Optional

import click


os.environ['TF_CPP_MIN_LOG_LEVEL'] = '3'

from magician_ng import get_logger
from magician_ng.magician import Magician

l = get_logger()
l.setLevel(logging.WARNING)


@click.command()
@click.argument('paths', type=click.Path(exists=True, file_okay=True, dir_okay=True, resolve_path=True, path_type=Path), required=True, nargs=-1)
@click.option('--model-dir', type=click.Path(exists=True, file_okay=False, dir_okay=True, resolve_path=True, path_type=Path))
@click.option('-t', '--threshold', default=0.0)
@click.option('--json', 'json_output_flag', is_flag=True, help='Output in JSON format.')
@click.option('--jsonl', 'jsonl_output_flag', is_flag=True, help='Output in JSONL format.')
@click.option('-s', '--with-score', 'with_score', is_flag=True, help='Output the score in addition to the content type.')
@click.option('-r', '--recursive', 'recursive_flag', is_flag=True, help='Recursively scan every directory.')
@click.option('-v', '--verbose', is_flag=True, help='Enable more verbose output.')
@click.option('-vv', '-d', '--debug', is_flag=True, help='Enable debug logging.')
def main(
    paths: List[Path],
    model_dir: Optional[Path],
    threshold: float,
    with_score: bool,
    json_output_flag: bool,
    jsonl_output_flag: bool,
    recursive_flag: bool,
    verbose: bool,
    debug: bool):

    if verbose:
        l.setLevel(logging.INFO)
    if debug:
        l.setLevel(logging.DEBUG)

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

    m = Magician(
        model_dir=model_dir,
        threshold=threshold,
        verbose=verbose,
        debug=debug,
    )
    predictions = m.get_content_types(paths)
    if json_output_flag:
        print(json.dumps(
            sorted(predictions.items()),
            indent=4,
        ))
    elif jsonl_output_flag:
        for _, v in sorted(predictions.items()):
            print(json.dumps(v))
    else:
        for path, v in sorted(predictions.items()):
            ct = v['output']['ct_label']
            score = v['output']['score']
            if with_score:
                print(f'{path}: {ct} {score=}')
            else:
                print(f'{path}: {ct}')


if __name__ == '__main__':
    main()
