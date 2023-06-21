#!/usr/bin/env python3

import os
from pathlib import Path
import logging
import json
import subprocess
import sys
import tarfile

import click

os.environ['TF_CPP_MIN_LOG_LEVEL'] = '3'

from magician_ng import get_logger
from magician_ng.magician import Magician

l = get_logger()


@click.command()
@click.argument('paths', type=click.Path(exists=True, file_okay=True, dir_okay=True, resolve_path=True, path_type=Path), required=True, nargs=-1)
@click.option('--gbt-model-dir', type=click.Path(exists=True, file_okay=False, dir_okay=True, resolve_path=True, path_type=Path))
@click.option('--dl-model-dir', type=click.Path(exists=True, file_okay=False, dir_okay=True, resolve_path=True, path_type=Path))
@click.option('--gbt-t', default=0.9)
@click.option('--dl-t', default=0.9)
@click.option('--mode', type=click.Choice(Magician.MODES, case_sensitive=True), default=Magician.MODES[0], help='Select operation mode.')
@click.option('--combine-policy', type=click.Choice(Magician.COMBINE_POLICY, case_sensitive=True), default=Magician.COMBINE_POLICY[0], help='Select combine policy.')
@click.option('--json', 'json_output_flag', is_flag=True, help='Output in JSON format.')
@click.option('--jsonl', 'jsonl_output_flag', is_flag=True, help='Output in JSONL format.')
@click.option('-r', '--recursive', 'recursive_flag', is_flag=True, help='Recursively scan every directory.')
@click.option('-d', '--debug', is_flag=True, help='Enable debug logging.')
def main(
    paths: Path,
    gbt_model_dir: Path,
    dl_model_dir: Path,
    gbt_t: float,
    dl_t: float,
    mode: str,
    combine_policy: str,
    json_output_flag: bool,
    jsonl_output_flag: bool,
    recursive_flag: bool,
    debug: bool):

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
        gbt_model_dir=gbt_model_dir,
        dl_model_dir=dl_model_dir,
        gbt_t=gbt_t,
        dl_t=dl_t,
        mode=mode,
        combine_policy=combine_policy,
        debug=debug,
    )
    predictions = m.get_content_types(paths)
    if json_output_flag:
        print(json.dumps(
            predictions,
            indent=4,
        ))
    elif jsonl_output_flag:
        for _, v in sorted(predictions.items()):
            print(json.dumps(v))
    else:
        for path, v in sorted(predictions.items()):
            ct = v['output']['ct_label']
            score = v['output']['score']
            print(f'{path}: {ct} {score=}')


def check_and_unpack_models():
    l.debug(f'Checking and unpacking models (if needed)')
    models_data_dir = Path(__file__).parent.parent / 'models-data'
    assert models_data_dir.is_dir()

    models_info_path = Path(__file__).parent.parent / 'config' / 'models_info.json'
    with open(models_info_path) as f:
        models_info = json.load(f)

    for name in ['gbt-exact', 'gbt-standalone', 'gbt-first-stage', 'dl-text']:
        tgz_fname = f'{name}.tgz'
        tgz_path = models_data_dir / tgz_fname
        if not tgz_path.is_file():
            l.info(f'Could not find model {name} locally, downloading it to {tgz_path}')
            tgz_hash = models_info[tgz_fname]['sha256']
            tgz_url = f'https://storage.googleapis.com/magician-ng/{name}-{tgz_hash}.tgz'
            utils.download_payload(tgz_url, tgz_path)
            assert tgz_path.is_file()
        dst_model_dir = models_data_dir / name
        if not dst_model_dir.is_dir():
            l.info(f'Unpacking {tgz_path} to {models_data_dir}')
            with tarfile.open(tgz_path, 'r:gz') as tf:
                tf.extractall(models_data_dir)
    l.debug('Models checked and unpacked successfully')


if __name__ == '__main__':
    main()
