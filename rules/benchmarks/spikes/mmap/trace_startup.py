# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Capture actual startup spans and parent-observed process boundaries as JSON."""
import argparse
import json
import os
import random
import statistics
import subprocess
import time
from collections import defaultdict
from pathlib import Path

from magika_rules_benchmark import comparison as c
from magika_rules_benchmark import corpus


def run(source, destination, binary, packs):
    original = c.load_json(source / 'results.json')
    observations = c.load_json(source / 'observations.json.gz')['rules-mapped']
    samples = sorted(c.load_json(source / 'inputs.json.gz')['samples'], key=lambda s: s['sha256'])
    positions = {s['sha256']: i for i, s in enumerate(samples)}
    mapping = c.load_json(source / 'label-mappings.json')['rules-mapped']
    cases = [w for w in c.load_json(source / 'workloads.json')
             if w['file_count'] == 1 and w['requested_rule_hit_percent'] in (0, 100)]
    destination.mkdir(parents=True, exist_ok=False)
    env = {'PATH': os.defpath, 'HOME': str(source / 'home'), 'TMPDIR': str(source / 'tmp'),
           'MAGIKA_RULES_CACHE': str(destination / 'cache'), 'LC_ALL': 'C', 'LANG': 'C', 'TZ': 'UTC',
           **original['config']['environment'], 'MAGIKA_STARTUP_TRACE': '1'}
    tools = {}
    for mode in ['serialized', 'mapped']:
        pack = packs / mode / 'rules.yar'
        tools[mode] = {'command': [str(binary), '--jsonl', '--backend=cpu', '--readers=2',
                      '--threads=2', '--batch-size=8', '--rules=only', '--rules-file', str(pack)],
                      'pack': str(pack.with_suffix('.hsdb')), 'pack_sha256': corpus.file_hash(pack.with_suffix('.hsdb'))}
    jobs = [(mode, case, i) for i in range(10) for mode in tools for case in cases]
    random.Random(20260908).shuffle(jobs)
    captured = []
    for mode, case, repetition in jobs:
        env['MAGIKA_RULES_MMAP_SPIKE'] = '1' if mode == 'mapped' else '0'
        paths = ['files/' + h for h in case['samples']]
        command = [*tools[mode]['command'], '--', *paths]
        start_unix = time.time_ns()
        start = time.perf_counter_ns()
        child = subprocess.run(command, env=env, cwd=source, capture_output=True, timeout=30)
        elapsed = time.perf_counter_ns() - start
        end_unix = time.time_ns()
        assert child.returncode == 0, child.stderr.decode(errors='replace')
        values = c.parse_output(c.Adapter.MAGIKA, child.stdout, paths, mapping)
        assert values == [observations[positions[h]] for h in case['samples']]
        trace = json.loads(child.stderr)
        assert trace['magika_startup_trace'] == 1
        launch = trace['main_entry_unix_ns'] - start_unix
        tail = end_unix - trace['main_entry_unix_ns'] - trace['main_elapsed_ns']
        assert launch >= 0 and tail >= 0
        captured.append({'mode': mode, 'case': case['id'], 'repetition': repetition,
                         'command': command, 'environment': dict(env),
                         'parent_elapsed_ns': elapsed, 'launch_to_main_ns': launch,
                         'after_main_trace_ns': tail, 'stdout': child.stdout.decode(), 'trace': trace})
    groups = defaultdict(list)
    for row in captured:
        key = (row['mode'], row['case'])
        for name in ['parent_elapsed_ns', 'launch_to_main_ns', 'after_main_trace_ns']:
            groups[(*key, name)].append(row[name])
        groups[(*key, 'main_elapsed_ns')].append(row['trace']['main_elapsed_ns'])
        # One row per event occurrence, retaining overlap/thread identity in raw traces.
        for event in row['trace']['events']:
            groups[(*key, event['name'])].append(event['duration_ns'])
    summary = [{'mode': k[0], 'case': k[1], 'stage': k[2], 'observations': len(v),
                'median_ms': statistics.median(v) / 1e6, 'min_ms': min(v) / 1e6, 'max_ms': max(v) / 1e6}
               for k, v in sorted(groups.items())]
    result = {'schema': 1, 'binary': str(binary), 'binary_sha256': corpus.file_hash(binary),
              'source_results_sha256': corpus.file_hash(source / 'results.json'),
              'trace_script_sha256': corpus.file_hash(Path(__file__)), 'tools': tools,
              'host': original['host'], 'repetitions': 10, 'decision_parity': True,
              'samples': cases, 'captures': captured, 'summary': summary}
    for tool in tools.values():
        assert corpus.file_hash(tool['pack']) == tool['pack_sha256']
    corpus.atomic_json(destination / 'traces.json', result)
    table = ['# Rules-only startup spans', '',
             'Ten fresh-process traces per loader and hit/miss workload. Numbers are measured medians. '
             'Nested and concurrent intervals are inclusive and must not be added. '
             'Launch-to-main includes process creation, scheduling, dynamic loading and Rust startup. '
             'The tail includes trace JSON emission, process exit and parent wakeup; it is not pure teardown.', '',
             '| Mode | Case | Stage | Median ms | Min ms | Max ms |', '|---|---|---|---:|---:|---:|']
    for row in summary:
        table.append(f"| {row['mode']} | {row['case']} | {row['stage']} | {row['median_ms']:.4f} | {row['min_ms']:.4f} | {row['max_ms']:.4f} |")
    (destination / 'report.md').write_text('\n'.join(table) + '\n')


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    for name in ['source', 'destination', 'binary', 'packs']:
        p.add_argument(name, type=Path)
    a = p.parse_args()
    run(a.source.resolve(), a.destination.resolve(), a.binary.resolve(), a.packs.resolve())
