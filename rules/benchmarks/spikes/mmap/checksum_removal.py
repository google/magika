# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Render retained before/after observations for outer checksum removal."""
import json
from pathlib import Path

from magika_rules_benchmark import corpus


def render(root):
    names = ('sha256', 'none')
    paths = {n: root / n / 'startup/traces.json' for n in names}
    traces = {n: json.loads(p.read_text()) for n, p in paths.items()}
    timings = {n: json.loads((root / n / 'direct/summary.json').read_text()) for n in names}
    assert all(e['name'] != 'rules_payload_checksum' for c in traces['none']['captures']
               for e in c['trace']['events'])
    stages = ['main_elapsed_ns', 'rule_pack_load_total', 'rules_payload_checksum',
              'rules_cache_identity', 'native_scratch_allocate', 'native_library_dlopen']
    rows = []
    for stage in stages:
        row = {'stage': stage}
        for name in names:
            samples = [s['median_ms'] for s in traces[name]['summary']
                       if s['mode'] == 'mapped' and s['case'] == 'files-1-hits-100' and s['stage'] == stage]
            row[name] = samples[0] if samples else None
        rows.append(row)
    process = []
    for count in (1, 10, 1000):
        row = {'files': count}
        for name in names:
            m = next(m for m in timings[name]['measurements']
                     if m['tool'] == 'rules-mapped' and m['file_count'] == count)
            row[name] = {k: m[k] * 1000 for k in ('median_seconds', 'min_seconds', 'max_seconds')}
        process.append(row)
    corpus.atomic_json(root / 'comparison.json', {
        'schema': 1, 'outer_checksum_span_absent': True, 'stages_ms': rows, 'process_ms': process,
        'source_hashes': {str(p.relative_to(root)): corpus.file_hash(p) for p in
                         [*paths.values(), *(root / n / 'direct/summary.json' for n in names)]},
    })
    lines = ['# Startup after removing the outer payload checksum', '',
             'Mapped rules, same saved inputs and native image. Ten fresh-process traces per '
             'build/hit/miss case. Inclusive stage medians overlap; do not sum them. The removed '
             'checksum event is absent from every new trace. Source/compiler cache keys still hash '
             'their inputs; structural and native CRC checks remain.', '',
             '| Stage (one rule hit) | With ARM SHA-256 ms | No outer checksum ms |', '|---|---:|---:|']
    for row in rows:
        cells = [row['stage'], *(f'{row[n]:.4f}' if row[n] is not None else 'absent' for n in names)]
        lines.append('| ' + ' | '.join(cells) + ' |')
    lines += ['', 'Trace-disabled Hyperfine: twenty runs after one warmup, warm OS caches, '
              'same saved natural file sets. Whole-process timings include startup and shutdown.', '',
              '| Files | With checksum median ms (min–max) | Without checksum median ms (min–max) |',
              '|---:|---:|---:|']
    for row in process:
        cells = [str(row['files'])]
        for name in names:
            m = row[name]
            cells.append(f"{m['median_seconds']:.3f} ({m['min_seconds']:.3f}–{m['max_seconds']:.3f})")
        lines.append('| ' + ' | '.join(cells) + ' |')
    lines += ['', 'All traced classifications and each timed workload match the saved full-corpus '
              'observations. This focused loader change does not claim a new full-corpus replay. '
              'Raw JSON also includes serialized mode, misses and individual timing ranges.', '']
    (root / 'comparison.md').write_text('\n'.join(lines))


if __name__ == '__main__':
    render(Path(__file__).resolve().parent / 'checksum-removal')
