# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Measure compiler options on identical bytes; retain cold-pool and warm timings."""
import argparse
import hashlib
import json
import os
import platform
import random
import statistics
import subprocess
from pathlib import Path


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run(generic, native, pack, destination):
    destination.mkdir(exist_ok=False)
    variants = {'generic': generic, 'native': native}
    modes = ('sha256', 'blake3', 'blake3-rayon')
    jobs = [(variant, mode, repetition) for repetition in range(10)
            for variant in variants for mode in modes]
    random.Random(20260908).shuffle(jobs)
    captured = []
    for variant, mode, repetition in jobs:
        command = [str(variants[variant]), str(pack), mode]
        # Bound the optional parallel path, including pool initialization on the first hash.
        env = dict(os.environ, RAYON_NUM_THREADS='4')
        result = json.loads(subprocess.check_output(command, env=env))
        captured.append({'variant': variant, 'repetition': repetition, 'command': command, **result})
    # Parallel and sequential BLAKE3 must agree, as must both CPU-option builds.
    for family in ('sha256', 'blake3'):
        assert len({tuple(row['digest']) for row in captured if row['mode'].startswith(family)}) == 1
    summaries = []
    for variant in variants:
        for mode in modes:
            rows = [r for r in captured if r['variant'] == variant and r['mode'] == mode]
            cold = [r['durations_ns'][0] / 1e6 for r in rows]
            warm = [n / 1e6 for r in rows for n in r['durations_ns'][1:]]
            summaries.append({'variant': variant, 'mode': mode,
                              'first_median_ms': statistics.median(cold),
                              'first_min_ms': min(cold), 'first_max_ms': max(cold),
                              'warm_median_ms': statistics.median(warm),
                              'warm_min_ms': min(warm), 'warm_max_ms': max(warm)})
    evidence = {'schema': 1, 'host': platform.platform(), 'pack_sha256': sha(pack),
                'binary_sha256': {k: sha(v) for k, v in variants.items()},
                'script_sha256': sha(Path(__file__)), 'rayon_threads': 4,
                'generic_flags': {'RUSTFLAGS': '', 'CFLAGS': ''},
                'native_flags': {'RUSTFLAGS': '-C target-cpu=native', 'CFLAGS': '-mcpu=native'},
                'profile': {'opt_level': 3, 'lto': True, 'codegen_units': 1},
                'captures': captured, 'summary': summaries}
    (destination / 'measurements.json').write_text(json.dumps(evidence, indent=2) + '\n')
    lines = ['# Hash compiler-option audit', '',
             'The same manifest and native payload bytes are hashed by every variant. Both use '
             'release opt-level=3, LTO and one codegen unit. BLAKE3 NEON and SHA-256 ARM assembly '
             'are enabled. Native additionally sets Rust target-cpu=native and C -mcpu=native.', '',
             'Ten fresh processes per cell, each hashing 21 times. First-call medians include any '
             'Rayon pool initialization (four threads); warm medians use the remaining 20 hashes. '
             'File reading is outside the timer. These are in-memory hash costs, not CLI timings.', '',
             '| CPU flags | Algorithm | First hash ms | Warm hash ms |', '|---|---|---:|---:|']
    for s in summaries:
        lines.append(f"| {s['variant']} | {s['mode']} | {s['first_median_ms']:.4f} | {s['warm_median_ms']:.4f} |")
    (destination / 'report.md').write_text('\n'.join(lines) + '\n')


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    for name in ('generic', 'native', 'pack', 'destination'):
        p.add_argument(name, type=Path)
    a = p.parse_args()
    run(a.generic.resolve(), a.native.resolve(), a.pack.resolve(), a.destination.resolve())
