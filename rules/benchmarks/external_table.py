# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Render external-corpus results without cross-dataset speedup claims."""

import argparse
from collections import Counter
from pathlib import Path

from full_table import COUNTS, TOOLS, observations, result_file
from magika_rules_benchmark import comparison as c
from magika_rules_benchmark import corpus


def derive(source, selection, destination):
    path = result_file(source)
    result = c.load_json(path)
    assert result['status'] == 'complete'
    inputs = c.load_json(source / 'inputs.json.gz')
    samples = sorted(inputs['samples'], key=lambda s: s['sha256'])
    annotation = {r['sha256']: r for r in c.load_json(selection / 'annotations.json.gz')}
    observed = observations(source)
    summary = {'schema': 1, 'dataset': 'Sembiance v3', 'revision': result['revision'],
               'selection': c.load_json(selection / 'selection.json'),
               'results_sha256': corpus.file_hash(path), 'workloads': len(c.load_json(source / 'workloads.json')),
               'runs': result['config']['runs'], 'warmup': result['config']['warmup'], 'rows': []}
    controls = {'magika2-rules': 'magika2-ml', 'magika2-gpu-rules': 'magika2-gpu-ml', 'magika2-rules-only': None}
    wrong, confusions = {}, {}
    for tool, label, _ in TOOLS:
        q = result['quality'][tool]
        assert q['files'] == len(samples) and not q['errors']
        hits = None
        if tool in controls:
            control = observed[controls[tool]] if controls[tool] else None
            hits = len(c.rule_hit_hashes(samples, observed[tool], control)) / len(samples)
        by_basis = {}
        for basis in ['validated_auto', 'reviewed_source_mapping']:
            indices = [i for i, s in enumerate(samples) if annotation[s['sha256']]['evaluation_basis'] == basis]
            by_basis[basis] = c.quality_metrics([samples[i] for i in indices], [observed[tool][i] for i in indices])
        wrong[tool] = [dict(sha256=s['sha256'], truth=s['truth'], prediction=row['prediction'],
            raw_labels=row['raw_labels'], evaluation_basis=annotation[s['sha256']]['evaluation_basis'],
            source_claims=annotation[s['sha256']]['source_claims'])
            for s, row in zip(samples, observed[tool], strict=True)
            if row['prediction'] is not None and row['prediction'] != s['truth']]
        confusions[tool] = [{'truth': truth, 'prediction': prediction, 'files': n}
                           for (truth, prediction), n in Counter((r['truth'], r['prediction']) for r in wrong[tool]).most_common()]
        times = {}
        for count in COUNTS:
            rows = [m for m in result['measurements'] if m['tool'] == tool and m['file_count'] == count and m['requested_rule_hit_percent'] is None]
            assert len(rows) == 1
            times[str(count)] = rows[0]
        summary['rows'].append({'id': tool, 'label': label, 'quality': q, 'by_label_basis': by_basis,
                                'rule_matches': hits, 'timings': times})
    destination.mkdir(parents=True, exist_ok=False)
    corpus.atomic_json(destination / 'overview.json', summary)
    corpus.atomic_json(destination / 'confusions.json', confusions)
    c.save_gzip(destination / 'wrong-decisions.json.gz', wrong)
    render(destination / 'overview.json', destination / 'overview.md')


def render(path, output):
    d = c.load_json(path)
    s = d['selection']
    lines = [f"# Sembiance v3 — {d['revision'][:8]}", '',
             f"{s['eligible_samples']:,} eligible samples across {s['eligible_classes']} formats, from {s['samples']:,} verified files. "
             f"{d['runs']} timed runs after {d['warmup']} warmup; {d['workloads']} seeded workloads per mode. "
             'Times are median milliseconds, including startup and shutdown, with warm OS caches.', '',
             '| Tool / mode | Accuracy | Precision | Coverage | Rule matches | Wrong decisions | 1 file | 10 files | 100 files | 1,000 files |',
             '|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|']
    for row in d['rows']:
        q = row['quality']
        cells = [row['label'], *[f'{100*q[key]:.2f}%' for key in ['accuracy', 'precision', 'decision_coverage']],
                 '—' if row['rule_matches'] is None else f"{100*row['rule_matches']:.2f}%", str(q['wrong']),
                 *[f"{1000*row['timings'][str(n)]['median_seconds']:,.2f}" for n in COUNTS]]
        lines.append('| ' + ' | '.join(cells) + ' |')
    lines += ['', 'The scoring subset is selected solely by the dataset lane’s existing `evaluation_eligible` flags: '
              f"{s['label_bases']['validated_auto']:,} structurally validated and {s['label_bases']['reviewed_source_mapping']:,} reviewed source mappings. "
              'Exact overlaps are excluded; near-duplicate independence is not certified. '
              'Detector aliases are frozen from the baseline benchmark, with six additional source-declared classes. '
              'No source labels, rules or model weights were changed for this run. '
              'This is an external evaluation; V56 remains the base dataset.']
    Path(output).write_text('\n'.join(lines) + '\n')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['source', 'selection', 'destination', 'render', 'output']:
        parser.add_argument('--' + name, type=Path)
    a = parser.parse_args()
    if a.render:
        render(a.render, a.output)
    else:
        derive(a.source, a.selection, a.destination)
