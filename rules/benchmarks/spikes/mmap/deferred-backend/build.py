"""Build one distribution containing CPU and GPU backends; never time a build."""
import argparse
import hashlib
import json
import os
import platform
import shutil
import subprocess
from pathlib import Path


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def build(root, target, destination, restore):
    destination.mkdir(parents=True, exist_ok=False)
    (destination / 'lib').mkdir()
    env = dict(os.environ, CARGO_TARGET_DIR=str(target))
    suffix = '.dylib' if platform.system() == 'Darwin' else '.so'
    gpu = 'metal' if platform.system() == 'Darwin' else 'cuda'
    sources = [p for crate in ['cli', 'lib', 'runtime', 'runtime-abi', 'runtime-plugin', 'tract-runtime']
               for p in (root / 'rust' / crate).rglob('*') if p.is_file() and
               (p.suffix == '.rs' or p.name in ['Cargo.toml', 'Cargo.lock'])]
    source_hashes = {str(p.relative_to(root)): digest(p) for p in sources}
    commands = []
    try:
        for backend in ['cpu', gpu]:
            cmd = ['cargo', 'build', '--release', '--locked', '--manifest-path', str(root / 'rust/runtime-plugin/Cargo.toml')]
            if backend != 'cpu':
                cmd += ['--features', backend]
            commands.append(cmd)
            with (destination / f'build-{backend}.log').open('w') as log:
                subprocess.run(cmd, cwd=root, env=env, stdout=log, stderr=subprocess.STDOUT, check=True)
            shutil.copy2(target / ('release/libmagika_runtime_plugin' + suffix), destination / 'lib' / f'libmagika_runtime_{backend}{suffix}')
        cmd = ['cargo', 'build', '--release', '--locked', '--manifest-path', str(root / 'rust/cli/Cargo.toml'), '--features', '_sha2-accel-spike']
        commands.append(cmd)
        with (destination / 'build-cli.log').open('w') as log:
            subprocess.run(cmd, cwd=root, env=env, stdout=log, stderr=subprocess.STDOUT, check=True)
        shutil.copy2(target / 'release/magika', destination / 'magika')
    finally:
        shutil.copy2(restore, target / 'release/magika')
    assert source_hashes == {str(p.relative_to(root)): digest(p) for p in sources}
    manifest = {'schema': 1, 'source_files': source_hashes, 'commands': commands, 'rustflags': env.get('RUSTFLAGS', ''),
                'rustc': subprocess.check_output(['rustc', '-Vv'], text=True),
                'source_head': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=root, text=True).strip(),
                'dirty_diff_sha256': hashlib.sha256(subprocess.check_output(['git', 'diff', 'HEAD'], cwd=root)).hexdigest(),
                'artifacts': {str(p.relative_to(destination)): digest(p) for p in [destination / 'magika', *sorted((destination / 'lib').iterdir())]}}
    (destination / 'build.json').write_text(json.dumps(manifest, indent=2) + '\n')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['root', 'target', 'destination', 'restore']:
        parser.add_argument(name, type=Path)
    args = parser.parse_args()
    build(args.root.resolve(), args.target.resolve(), args.destination.resolve(), args.restore.resolve())
