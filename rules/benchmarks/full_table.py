# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Generate the eight-mode overview and historical arithmetic from saved JSON."""

import argparse
import gzip
import json
import tarfile
from pathlib import Path

from magika_rules_benchmark import comparison as c
from magika_rules_benchmark import corpus

TOOLS = [
    ('magika1', 'Magika 1 CPU', 'cpu'),
    ('magika2-ml', 'Magika 2 CPU ML', 'cpu'),
    ('magika2-rules', 'Magika 2 CPU rules + ML', 'cpu'),
    ('magika2-gpu-ml', 'Magika 2 GPU ML', 'gpu'),
    ('magika2-gpu-rules', 'Magika 2 GPU rules + ML', 'gpu'),
    ('magika2-rules-only', 'Magika 2 rules-only CPU', 'rules-only'),
    ('libmagic', 'libmagic', 'cpu'),
    ('trid', 'TrID', 'cpu'),
]
COUNTS = [1, 10, 100, 1000]


def observations(source):
    if (source / 'observations.json.gz').exists():
        return c.load_json(source / 'observations.json.gz')
    with tarfile.open(source / 'raw-output.tar.gz') as archive:
        return json.loads(gzip.decompress(archive.extractfile('observations.json.gz').read()))


def result_file(source):
    path = source / 'results.json'
    return path if path.exists() else source / 'results.json.gz'


def derive(source, previous, destination):
    result_path = result_file(source)
    current = c.load_json(result_path)
    assert current['status'] == 'complete'
    current_inputs = c.load_json(source / 'inputs.json.gz')
    samples = sorted(current_inputs['samples'], key=lambda s: s['sha256'])
    observed = observations(source)
    old = {name: c.load_json(result_file(path)) for name, path in previous.items()}
    old_observations = {name: observations(path) for name, path in previous.items()}
    for path in previous.values():
        assert c.input_identity(current_inputs) == c.input_identity(c.load_json(path / 'inputs.json.gz'))
        assert c.load_json(source / 'workloads.json') == c.load_json(path / 'workloads.json')
    summary = {'schema': 1, 'revision': current['revision'], 'files': len(samples),
               'runs': current['config']['runs'], 'warmup': current['config']['warmup'],
               'results_sha256': corpus.file_hash(result_path),
               'previous_results_sha256': {name: corpus.file_hash(result_file(path)) for name, path in previous.items()},
               'historical_compatibility': {name: c.compare_previous(current, value) for name, value in old.items()},
               'rows': []}
    controls = {'magika2-rules': 'magika2-ml', 'magika2-gpu-rules': 'magika2-gpu-ml',
                'magika2-rules-only': None}
    for tool, label, group in TOOLS:
        before = old[group]
        assert current['compatibility']['corpus'] == before['compatibility']['corpus']
        assert current['compatibility']['mappings'][tool] == before['compatibility']['mappings'][tool]
        quality = current['quality'][tool]
        assert quality['files'] == len(samples) and quality['errors'] == 0
        rule_matches = None
        if tool in controls:
            control = observed[controls[tool]] if controls[tool] else None
            rule_matches = len(c.rule_hit_hashes(samples, observed[tool], control)) / len(samples)
        times = {}
        for count in COUNTS:
            new = [m for m in current['measurements'] if m['tool'] == tool and m['file_count'] == count and m['requested_rule_hit_percent'] is None]
            prior = [m for m in before['measurements'] if m['tool'] == tool and m['file_count'] == count and m['requested_rule_hit_percent'] is None]
            assert len(new) == len(prior) == 1
            new, prior = new[0], prior[0]
            assert new['input_order_sha256'] == prior['input_order_sha256']
            times[str(count)] = {'before_ms': 1000 * prior['median_seconds'],
                'after_ms': 1000 * new['median_seconds'], 'min_ms': 1000 * new['min_seconds'],
                'max_ms': 1000 * new['max_seconds'], 'historical_ratio': prior['median_seconds'] / new['median_seconds'],
                'elapsed_reduction_percent': 100 * (1 - new['median_seconds'] / prior['median_seconds'])}
        summary['rows'].append({'id': tool, 'label': label, 'quality': quality,
            'rule_matches': rule_matches, 'timings': times,
            'observation_changes': sum(a != b for a, b in zip(observed[tool], old_observations[group][tool], strict=True)),
            'quality_changes': {key: quality[key] - before['quality'][tool][key]
                                for key in ['correct', 'wrong', 'errors', 'decisions']}})
    destination.mkdir(parents=True, exist_ok=True)
    corpus.atomic_json(destination / 'overview.json', summary)
    render(destination / 'overview.json', destination / 'overview.md')


def render(path, output):
    summary = c.load_json(path)
    lines = [f"# Magika benchmark — {summary['revision'][:8]}", '',
             f"Fresh accuracy evaluation on {summary['files']:,} files. All eight modes rerun on the same "
             f"30 saved workloads; {summary['runs']} measured runs after {summary['warmup']} warmup. "
             'Times are median milliseconds for natural file mixes, including startup and shutdown; warm OS caches.', '',
             '| Tool / mode | Accuracy | Precision | Coverage | Rule matches | 1 file | 10 files | 100 files | 1,000 files |',
             '|---|---:|---:|---:|---:|---:|---:|---:|---:|']
    for row in summary['rows']:
        q = row['quality']
        cells = [row['label'], *[f'{100*q[key]:.2f}%' for key in ['accuracy', 'precision', 'decision_coverage']],
                 '—' if row['rule_matches'] is None else f"{100*row['rule_matches']:.2f}%",
                 *[f"{row['timings'][str(n)]['after_ms']:,.2f}" for n in COUNTS]]
        lines.append('| ' + ' | '.join(cells) + ' |')
    lines += ['', '## Elapsed-time reduction versus the previous published table', '',
              'Positive percentages mean less time; negative percentages mean more time. '
              'These historical differences include all intervening changes and host variation. '
              'They are not an isolated measurement of asynchronous loading. The original measurements remain unchanged.', '',
              '| Tool / mode | 1 file | 10 files | 100 files | 1,000 files |', '|---|---:|---:|---:|---:|']
    for row in summary['rows']:
        lines.append('| ' + ' | '.join([row['label'], *[f"{row['timings'][str(n)]['elapsed_reduction_percent']:+.1f}%" for n in COUNTS]]) + ' |')
    lines += ['', 'Full precision, raw ranges, historical ratios, quality-count changes and compatibility checks are stored '
              'in `overview.json`. The strict historical compatibility check flags the changed reference/configuration and '
              'loader environment; the overview reports historical arithmetic rather than a controlled causal speedup. '
              'The corpus, labels, mapping and ordered file lists are verified identical. '
              'Magika 2 uses the deferred CPU/Metal distribution and the mapped rule pack. '
              'GPU means Metal; rules-only runs CPU signatures without model initialization.']
    Path(output).write_text('\n'.join(lines) + '\n')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path)
    parser.add_argument('--previous-cpu', type=Path)
    parser.add_argument('--previous-gpu', type=Path)
    parser.add_argument('--previous-rules-only', type=Path)
    parser.add_argument('--destination', type=Path)
    parser.add_argument('--render', type=Path)
    parser.add_argument('--output', type=Path)
    args = parser.parse_args()
    if args.render:
        render(args.render, args.output)
    else:
        derive(args.source, {'cpu': args.previous_cpu, 'gpu': args.previous_gpu, 'rules-only': args.previous_rules_only}, args.destination)
