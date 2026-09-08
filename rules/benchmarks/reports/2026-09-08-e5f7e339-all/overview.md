# Magika benchmark — e5f7e339

Dataset: **Adjudicated combined corpus**, version `snapshot-1411a5c0fd4a`. Measured at **2026-09-08T22:36:35.070300Z**; benchmark protocol `1.1.0`.

<details><summary>Exact tool configurations and build identities</summary>

- **Magika 1.1.0** — config `06b0f91c`, build `188e3ace44592119f89741af19c3636d42daea5239da85adc684647267fc1b31`: `{"backend": "CPU", "internal_batch": "default 1", "intra_threads": 2, "model": "standard_v3_3", "release": "cli/v1.1.0", "tasks": 1}`
- **Magika 2.0.0-dev @ e5f7e339** — config `22cc1952`, build `a0289752a5c2a98347e7e7c3276d3a8b4701748d4a6b43cf0df53bc0918bd667`: `{"backend": "CPU", "internal_batch": 8, "model": "standard_v3_3", "readers": 2, "rules": "off", "threads": 2}`
- **Magika 2.0.0-dev @ e5f7e339** — config `626cb9ea`, build `30eb9bb8ebd44e4ad4ef6685c42255f0c8b7449cba1da90c5ac101a9376e5453`: `{"backend": "CPU", "internal_batch": 8, "model": "standard_v3_3", "readers": 2, "rules": "enforce", "threads": 2}`
- **Magika 2.0.0-dev @ e5f7e339** — config `5f62a645`, build `a0289752a5c2a98347e7e7c3276d3a8b4701748d4a6b43cf0df53bc0918bd667`: `{"backend": "GPU-required (Metal on macOS)", "internal_batch": 8, "model": "standard_v3_3", "readers": 2, "rules": "off", "threads": 2}`
- **Magika 2.0.0-dev @ e5f7e339** — config `c0765229`, build `30eb9bb8ebd44e4ad4ef6685c42255f0c8b7449cba1da90c5ac101a9376e5453`: `{"backend": "GPU-required (Metal on macOS)", "internal_batch": 8, "model": "standard_v3_3", "readers": 2, "rules": "enforce", "threads": 2}`
- **Magika 2.0.0-dev @ e5f7e339** — config `11776c00`, build `338103194deb1c2ea4bbf4ed104a391212d272b51c6d189a25116fda8e75c81e`: `{"backend": "none (CPU signatures)", "internal_batch": null, "model": null, "queue_batch_flag": 8, "queue_threads_flag": 2, "readers": 2, "rules": "only", "threads": 0}`
- **libmagic 5.41** — config `52c4f5d9`, build `808833b8da07e23b5cd32ef92460cfc4b80472e8ce612ebc39d3c5580afb98b5`: `{"decompression": false, "output": "NUL-framed MIME", "threads": 1}`
- **TrID 2.48** — config `62cc9f75`, build `aaf5f8699486e5351eb37114c7d8441dbbe228e39a2bb4d8df43b88f36748885`: `{"max_results": 5, "strings": true, "stringzilla": "5.1.2", "threads": 1}`

Full commands, environment, source revisions and executable/artifact hashes are retained in `overview.json`.

</details>

Fresh accuracy evaluation on 25,421 files. All eight modes rerun on the same 30 saved workloads; 3 measured runs after 1 warmup. Times are median milliseconds for natural file mixes, including startup and shutdown; warm OS caches.

| Tool | Version | Config | Accuracy | Precision | Coverage | Rule matches | 1 file | 10 files | 100 files | 1,000 files |
|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Magika | 1.1.0 | CPU; ML (`06b0f91c`) | 57.73% | 70.23% | 82.20% | — | 18.45 | 35.10 | 199.78 | 1,851.52 |
| Magika | 2.0.0-dev @ e5f7e339 | CPU; ML (`22cc1952`) | 57.73% | 70.23% | 82.20% | — | 5.98 | 13.34 | 63.44 | 590.38 |
| Magika | 2.0.0-dev @ e5f7e339 | CPU; rules + ML (`626cb9ea`) | 78.93% | 84.39% | 93.52% | 37.86% | 3.74 | 11.85 | 43.65 | 376.89 |
| Magika | 2.0.0-dev @ e5f7e339 | Metal; ML (`5f62a645`) | 57.73% | 70.22% | 82.21% | — | 56.87 | 68.44 | 87.63 | 203.52 |
| Magika | 2.0.0-dev @ e5f7e339 | Metal; rules + ML (`c0765229`) | 78.93% | 84.38% | 93.53% | 37.86% | 5.36 | 73.02 | 83.80 | 159.18 |
| Magika | 2.0.0-dev @ e5f7e339 | CPU; rules-only (`11776c00`) | 37.86% | 100.00% | 37.86% | 37.86% | 2.84 | 3.14 | 6.04 | 34.92 |
| libmagic | 5.41 | MIME; serial (`52c4f5d9`) | 25.46% | 93.24% | 27.31% | — | 1.39 | 5.16 | 32.52 | 323.31 |
| TrID | 2.48 | strings=True; StringZilla=5.1.2 (`62cc9f75`) | 55.25% | 78.32% | 70.55% | — | 70.05 | 101.95 | 284.60 | 4,846.14 |

## Elapsed-time reduction versus the previous published table

Positive percentages mean less time; negative percentages mean more time. These historical differences include all intervening changes and host variation. They are not an isolated measurement of asynchronous loading. The original measurements remain unchanged.

| Tool | Version | Config | 1 file | 10 files | 100 files | 1,000 files |
|---|---|---|---:|---:|---:|---:|
| Magika | 1.1.0 | CPU; ML (`06b0f91c`) | +7.9% | +6.5% | +8.6% | +6.4% |
| Magika | 2.0.0-dev @ e5f7e339 | CPU; ML (`22cc1952`) | +26.3% | +21.9% | +11.4% | +4.7% |
| Magika | 2.0.0-dev @ e5f7e339 | CPU; rules + ML (`626cb9ea`) | +64.4% | +35.6% | +12.9% | +5.2% |
| Magika | 2.0.0-dev @ e5f7e339 | Metal; ML (`5f62a645`) | +19.3% | +13.7% | +9.9% | +10.1% |
| Magika | 2.0.0-dev @ e5f7e339 | Metal; rules + ML (`c0765229`) | +48.8% | +13.2% | +4.1% | +6.8% |
| Magika | 2.0.0-dev @ e5f7e339 | CPU; rules-only (`11776c00`) | +62.8% | +68.4% | +48.8% | +27.8% |
| libmagic | 5.41 | MIME; serial (`52c4f5d9`) | +17.4% | +14.8% | +6.7% | +17.8% |
| TrID | 2.48 | strings=True; StringZilla=5.1.2 (`62cc9f75`) | +13.6% | +9.6% | +5.0% | +5.7% |

Full precision, raw ranges, historical ratios, quality-count changes and compatibility checks are stored in `overview.json`. The strict historical compatibility check flags the changed reference/configuration and loader environment; the overview reports historical arithmetic rather than a controlled causal speedup. The corpus, labels, mapping and ordered file lists are verified identical. Magika 2 uses the deferred CPU/Metal distribution and the mapped rule pack. GPU means Metal; rules-only runs CPU signatures without model initialization.
