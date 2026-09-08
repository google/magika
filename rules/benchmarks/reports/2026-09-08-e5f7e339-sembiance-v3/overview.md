# Sembiance v3 — e5f7e339

Dataset: **Sembiance**, version `v3`. Measured at **2026-09-08T23:11:38.675067Z**; benchmark protocol `1.1.0`.

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

2,400 eligible samples across 167 formats, from 33,421 verified files. 3 timed runs after 1 warmup; 29 seeded workloads per mode. Times are median milliseconds, including startup and shutdown, with warm OS caches.

| Tool | Version | Config | Accuracy | Precision | Coverage | Rule matches | Wrong decisions | 1 file | 10 files | 100 files | 1,000 files |
|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Magika | 1.1.0 | CPU; ML (`06b0f91c`) | 58.29% | 71.27% | 81.79% | — | 564 | 23.24 | 41.48 | 235.21 | 2,143.49 |
| Magika | 2.0.0-dev @ e5f7e339 | CPU; ML (`22cc1952`) | 58.29% | 71.27% | 81.79% | — | 564 | 8.22 | 16.18 | 68.77 | 622.32 |
| Magika | 2.0.0-dev @ e5f7e339 | CPU; rules + ML (`626cb9ea`) | 65.21% | 76.27% | 85.50% | 22.21% | 487 | 4.55 | 14.99 | 51.31 | 504.78 |
| Magika | 2.0.0-dev @ e5f7e339 | Metal; ML (`5f62a645`) | 58.29% | 71.27% | 81.79% | — | 564 | 79.61 | 91.17 | 124.00 | 251.66 |
| Magika | 2.0.0-dev @ e5f7e339 | Metal; rules + ML (`c0765229`) | 65.21% | 76.27% | 85.50% | 22.21% | 487 | 8.49 | 99.91 | 116.49 | 209.79 |
| Magika | 2.0.0-dev @ e5f7e339 | CPU; rules-only (`11776c00`) | 22.21% | 100.00% | 22.21% | 22.21% | 0 | 3.94 | 4.11 | 7.10 | 54.01 |
| libmagic | 5.41 | MIME; serial (`52c4f5d9`) | 33.29% | 96.61% | 34.46% | — | 28 | 2.23 | 12.25 | 78.63 | 869.56 |
| TrID | 2.48 | strings=True; StringZilla=5.1.2 (`62cc9f75`) | 42.58% | 90.68% | 46.96% | — | 105 | 88.24 | 92.98 | 220.49 | 1,774.83 |

The scoring subset is selected solely by the dataset lane’s existing `evaluation_eligible` flags: 1,487 structurally validated and 913 reviewed source mappings. Exact overlaps are excluded; near-duplicate independence is not certified. Detector aliases are frozen from the baseline benchmark, with six additional source-declared classes. No source labels, rules or model weights were changed for this run. This is an external evaluation; V56 remains the base dataset.
