"""Paired saved-workload replay; reports are generated only from recorded JSON."""
import argparse
import gzip
import json
import os
import platform
import random
import shlex
import subprocess
from pathlib import Path

from magika_rules_benchmark import comparison as c
from magika_rules_benchmark import corpus

MODES = {'rules-only': ('cpu', 'only'), 'cpu-ml': ('cpu', 'off'),
         'cpu-rules-ml': ('cpu', 'enforce'), 'gpu-ml': ('gpu', 'off'),
         'gpu-rules-ml': ('gpu', 'enforce')}


def invoke(command, environment, source):
    return subprocess.run(command, cwd=source, env=environment, capture_output=True, check=True, timeout=120)


def record(source, baseline, distribution, pack, hyperfine, destination):
    destination.mkdir(parents=True, exist_ok=False)
    (destination / 'raw').mkdir()
    original = c.load_json(source / 'results.json')
    workloads = c.load_json(source / 'workloads.json')
    # Existing file lists, including a one-file rule miss to force first inference.
    cases = [w for w in workloads if (w['requested_rule_hit_percent'] is None and
             w['file_count'] in [1, 2, 5, 10, 100, 1000]) or
             (w['file_count'] == 1 and w['requested_rule_hit_percent'] == 0)]
    env = {'PATH': os.defpath, 'HOME': str(source / 'home'), 'TMPDIR': str(source / 'tmp'),
           'MAGIKA_RULES_CACHE': str(source / 'rule-cache'), 'LC_ALL': 'C', 'LANG': 'C', 'TZ': 'UTC',
           **original['config']['environment'], 'MAGIKA_RULES_MMAP_SPIKE': '1'}
    variants = {'eager': baseline, 'deferred': distribution / 'magika'}
    artifacts = [baseline, distribution / 'magika', *sorted((distribution / 'lib').iterdir()),
                 pack, pack.with_suffix('.hsdb'), Path(env['MAGIKA_VECTORSCAN_LIBRARY']), hyperfine]
    identities = {str(path): corpus.file_hash(path) for path in artifacts}
    summary = {'schema': 1, 'experiment': 'deferred-tract-backends', 'runs': 20, 'warmup': 1,
               'environment': env, 'cwd': str(source), 'host': original['host'],
               'platform_at_measurement': platform.platform(), 'artifacts': identities,
               'workloads_sha256': corpus.file_hash(source / 'workloads.json'),
               'code_sha256': corpus.file_hash(Path(__file__)), 'measurements': [], 'parity': []}
    commands = {}
    observations = {}
    for mode, (backend, rules) in MODES.items():
        for case in cases:
            paths = ['files/' + h for h in case['samples']]
            for variant, binary in variants.items():
                command = [str(binary), '--jsonl', '--backend=' + backend, '--readers=2',
                           '--threads=2', '--batch-size=8', '--rules=' + rules]
                if rules != 'off':
                    command += ['--rules-file', str(pack)]
                command += ['--', *paths]
                name = '-'.join([variant, mode, case['id']])
                commands[name] = (command, case, mode, variant)
                result = invoke(command, env, source)
                (destination / 'raw' / (name + '.jsonl')).write_bytes(result.stdout)
                # Compare the entire classification JSON (scores and decisions), not just labels.
                observations[name] = [json.loads(row) for row in result.stdout.splitlines()]
                assert len(observations[name]) == len(paths)
            left = observations['-'.join(['eager', mode, case['id']])]
            right = observations['-'.join(['deferred', mode, case['id']])]
            assert left == right, (mode, case['id'], 'output changed')
            summary['parity'].append({'mode': mode, 'workload': case['id'], 'files': len(paths), 'identical': True})
    # Backend-ready diagnostic, separate from complete classification timings.
    for variant, binary in variants.items():
        for backend in ['cpu', 'gpu']:
            name = f'{variant}-{backend}-backend-ready'
            command = [str(binary), '--backend=' + backend, '--batch-size=8', '--rules=off', '--backend-info']
            result = invoke(command, env, source)
            expected = b'cpu (tract-cpu)' if backend == 'cpu' else b'gpu (tract-metal)'
            assert result.stdout.strip().lower() == expected
            commands[name] = (command, None, backend + '-backend-ready', variant)
    jobs = list(commands.items())
    random.Random(20260908).shuffle(jobs)
    for name, (command, case, mode, variant) in jobs:
        export = destination / 'raw' / (name + '.hyperfine.json')
        invoke([str(hyperfine), '--shell=none', '--style=none', '--runs', '20', '--warmup', '1',
                '--export-json', str(export), shlex.join(command)], env, source)
        raw = c.load_json(export)['results'][0]
        count = case['file_count'] if case else 0
        summary['measurements'].append({'name': name, 'variant': variant, 'mode': mode,
            'workload': case, 'file_count': count, 'command': command,
            'raw_sha256': corpus.file_hash(export), **c.timing_summary(raw, count)})
        print(name, round(raw['median'] * 1000, 3), flush=True)
        corpus.atomic_json(destination / 'progress.json', summary)
    assert identities == {str(path): corpus.file_hash(path) for path in artifacts}
    corpus.atomic_json(destination / 'summary.json', summary)
    render(destination)


def render(destination):
    summary = c.load_json(destination / 'summary.json')
    groups = {}
    for row in summary['measurements']:
        workload = row['workload']['id'] if row['workload'] else 'backend-ready'
        groups.setdefault((row['mode'], row['file_count'], workload), {})[row['variant']] = row
    lines = ['# Deferred tract backend spike', '',
             'Identical saved file lists, model, rules, thread settings and dependency versions. '
             'Both distributions support CPU and Metal. Twenty fresh-process Hyperfine runs after one warmup; '
             'warm OS caches; direct execution without a shell. Timings include startup and shutdown. '
             'Natural subsets and the one-file rule miss are reported separately.', '',
             '| Mode | Files / workload | Eager median ms | Deferred median ms | Eager / deferred |',
             '|---|---:|---:|---:|---:|']
    for (mode, count, workload), pair in sorted(groups.items()):
        before = pair['eager']['median_seconds'] * 1000
        after = pair['deferred']['median_seconds'] * 1000
        label = 'backend ready' if not count else str(count) + (' (rule miss)' if workload.endswith('hits-0') else '')
        lines.append(f'| {mode} | {label} | {before:.3f} | {after:.3f} | {before/after:.2f}× |')
    unique = {h for m in summary['measurements'] if m['workload'] for h in m['workload']['samples']}
    lines += ['', f"Exact JSON output parity passed for {len(summary['parity'])} paired mode/workload checks "
              f"covering {len(unique)} distinct files. This is a focused replay, not a new full-corpus accuracy run.", '',
              'Backend-ready rows include library loading, plan construction and the unchanged GPU correctness probe, '
              'but no user-file inference. ML-only one-file rows include first inference. Thousand-file rows '
              'include startup and processing; they are not isolated persistent-session throughput. '
              'Raw timings, ranges, commands, input hashes and artifact hashes are retained in JSON.']
    lines += ['', '| Workload | Rules-only decisions / files | Coverage |', '|---|---:|---:|']
    cases = {m['workload']['id']: m['workload'] for m in summary['measurements'] if m['workload']}
    for name, _case in sorted(cases.items(), key=lambda pair: (pair[1]['file_count'], pair[0])):
        path = destination / 'raw' / ('eager-rules-only-' + name + '.jsonl')
        raw = path.read_bytes() if path.exists() else gzip.decompress(path.with_suffix('.jsonl.gz').read_bytes())
        decisions = [json.loads(line) for line in raw.splitlines()]
        hits = sum(row['result']['status'] == 'ok' and row['result']['value']['output']['label'] != 'unknown' for row in decisions)
        lines.append(f"| {name} | {hits} / {len(decisions)} | {100 * hits / len(decisions):.1f}% |")
    (destination / 'report.md').write_text('\n'.join(lines) + '\n')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--render', type=Path)
    for arg in ['source', 'baseline', 'distribution', 'pack', 'hyperfine', 'destination']:
        parser.add_argument('--' + arg, type=Path)
    args = parser.parse_args()
    if args.render:
        render(args.render)
    else:
        record(*[getattr(args, arg).resolve() for arg in ['source', 'baseline', 'distribution', 'pack', 'hyperfine', 'destination']])
