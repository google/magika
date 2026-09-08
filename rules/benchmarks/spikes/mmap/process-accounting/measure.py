# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Reconcile parent and child boundaries from the same Hyperfine iterations."""
import argparse
import hashlib
import json
import shlex
import statistics
import subprocess
from pathlib import Path


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run(hyperfine, reference, source, destination):
    destination.mkdir(exist_ok=False)
    ref = json.loads(reference.read_text())
    measurement = next(m for m in ref['measurements'] if m['tool'] == 'rules-mapped' and m['file_count'] == 1)
    command = measurement['command']
    result = {'schema': 1, 'hyperfine_sha256': sha(hyperfine), 'magika_sha256': sha(Path(command[0])),
              'reference_sha256': sha(reference), 'script_sha256': sha(Path(__file__)), 'runs': {}}
    for traced in (False, True):
        name = 'traced' if traced else 'untraced'
        env = dict(measurement['environment'], MAGIKA_STARTUP_TRACE=str(int(traced)), MAGIKA_PARENT_TRACE='1')
        invocation = [str(hyperfine), '--shell=none', '--runs', '20', '--warmup', '1',
                      '--output=inherit', '--export-json', str(destination / (name + '-hyperfine.json')),
                      shlex.join(command)]
        child = subprocess.run(invocation, env=env, cwd=source, capture_output=True, text=True, check=True)
        (destination / (name + '.stdout')).write_text(child.stdout)
        (destination / (name + '.stderr')).write_text(child.stderr)
        captured, trace = [], None
        for line in child.stderr.splitlines():
            if not line.startswith('{'):
                continue
            value = json.loads(line)
            if value.get('magika_startup_trace') == 1:
                trace = value
            elif value.get('hyperfine_boundary') == 1:
                row = dict(value)
                row['total_ns'] = value['end_unix_ns'] - value['start_unix_ns']
                assert abs(row['total_ns'] - value['elapsed_seconds'] * 1e9) < 100_000
                if traced:
                    assert trace is not None
                    row['before_main_ns'] = trace['main_entry_unix_ns'] - value['start_unix_ns']
                    row['main_ns'] = trace['main_elapsed_ns']
                    row['after_main_ns'] = value['end_unix_ns'] - trace['main_entry_unix_ns'] - trace['main_elapsed_ns']
                    assert min(row[k] for k in ('before_main_ns', 'main_ns', 'after_main_ns')) >= 0
                    assert sum(row[k] for k in ('before_main_ns', 'main_ns', 'after_main_ns')) == row['total_ns']
                    row['trace'] = trace
                captured.append(row)
                trace = None
        assert len(captured) == 21, len(captured)
        measured = captured[1:]
        exported = json.loads((destination / (name + '-hyperfine.json')).read_text())['results'][0]
        for row, elapsed in zip(measured, exported['times'], strict=True):
            assert abs(row['elapsed_seconds'] - elapsed) < 1e-8
        # Hyperfine inherited stdout includes benchmark text; retain and compare every JSON decision.
        decisions = [json.loads(line) for line in child.stdout.splitlines() if line.startswith('{')]
        assert len(decisions) == 21 and all(d == decisions[0] for d in decisions)
        fields = ['total_ns'] + (['before_main_ns', 'main_ns', 'after_main_ns'] if traced else [])
        result['runs'][name] = {'invocation': invocation, 'environment': env, 'captures': captured,
                                'means_ms': {k: statistics.mean(r[k] for r in measured) / 1e6 for k in fields},
                                'median_ms': statistics.median(r['elapsed_seconds'] for r in measured) * 1000,
                                'decision': decisions[0]}
    assert result['runs']['traced']['decision'] == result['runs']['untraced']['decision']
    (destination / 'accounting.json').write_text(json.dumps(result, indent=2) + '\n')
    traced = result['runs']['traced']
    lines = ['# Whole-process startup accounting', '',
             'Twenty measured iterations after one warmup. Parent timestamps are added directly to '
             'Hyperfine 1.20.0 around its existing spawn/wait timer; parent JSON is emitted after '
             'timing stops. Child timestamps come from Magika. Every traced iteration reconciles '
             'exactly using the same clock; arithmetic means below also add to the total.', '',
             '| Boundary | Mean ms |', '|---|---:|']
    for key, label in [('before_main_ns', 'Before main: process creation, loader and runtime startup'),
                       ('main_ns', 'Inside main: rules load, classification, output and local cleanup'),
                       ('after_main_ns', 'After main timer: trace emission, exit and parent wakeup'),
                       ('total_ns', 'Total traced process')]:
        lines.append(f"| {label} | {traced['means_ms'][key]:.4f} |")
    lines += ['', f"Untraced Hyperfine median: {result['runs']['untraced']['median_ms']:.4f} ms. "
              f"Traced median: {traced['median_ms']:.4f} ms. These remain separate observations; "
              'the traced component table is not retroactively assigned to the earlier 4.640 ms median.', '',
              'Before-main is not isolated dyld time. After-main includes trace emission. '
              'The trace comparison therefore quantifies instrumentation overhead rather than '
              'assuming it is free. Raw parent/child timestamps, stdout decisions and Hyperfine '
              'iteration timings are retained.', '']
    (destination / 'report.md').write_text('\n'.join(lines))


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    for name in ('hyperfine', 'reference', 'source', 'destination'):
        p.add_argument(name, type=Path)
    a = p.parse_args()
    run(a.hyperfine.resolve(), a.reference.resolve(), a.source.resolve(), a.destination.resolve())
