# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Render startup attribution from saved software/hardware SHA observations."""
import json
from pathlib import Path

from magika_rules_benchmark import corpus

root = Path(__file__).resolve().parent
sources = {
    'software': root / 'startup-traces/traces.json',
    'accelerated': root / 'startup-traces-accelerated/traces.json',
}
traces = {name: json.loads(path.read_text()) for name, path in sources.items()}
stages = [
    'main_elapsed_ns', 'rule_pack_load_total', 'rules_payload_checksum',
    'rules_cache_identity', 'native_scratch_allocate', 'native_library_dlopen',
    'pipeline_thread_launch', 'native_scan', 'output_format_and_write',
]
rows = []
for stage in stages:
    row = {'stage': stage}
    for name, trace in traces.items():
        row[name + '_ms'] = next(s['median_ms'] for s in trace['summary']
            if s['mode'] == 'mapped' and s['case'] == 'files-1-hits-100' and s['stage'] == stage)
    rows.append(row)
result = {'schema': 1, 'loader': 'mapped', 'case': 'files-1-hits-100', 'trials_per_build': 10,
          'sources': {name: corpus.file_hash(path) for name, path in sources.items()}, 'stages': rows}
corpus.atomic_json(root / 'attribution.json', result)
lines = ['# Startup attribution', '',
         'Mapped loader, one signature hit, ten traces per build. Inclusive stage medians in milliseconds. '
         'Nested intervals overlap; do not sum this table. The accelerated build enables sha2 0.10.9 asm '
         'support on ARM; cache/image validation is retained.', '',
         '| Stage | Software SHA | Accelerated SHA |', '|---|---:|---:|']
for r in rows:
    lines.append(f"| {r['stage']} | {r['software_ms']:.4f} | {r['accelerated_ms']:.4f} |")
lines += ['', 'Main elapsed time excludes launch-to-main and process teardown. Parent-observed boundaries '
          'are retained in the raw traces; they include launcher/scheduler overhead. Trace JSON emission '
          'is outside main elapsed time. These spans are diagnostic, not replacements for Hyperfine measurements.', '',
          '[Software SHA direct-execution timings](direct-exec/report.md) · '
          '[Accelerated SHA direct-execution timings](direct-exec-accelerated/report.md)', '',
          'The later direct-execution run has wider timing ranges on the shared host. '
          'Retain its raw ranges rather than treating every end-to-end delta as a stable speedup.']
(root / 'attribution.md').write_text('\n'.join(lines) + '\n')
