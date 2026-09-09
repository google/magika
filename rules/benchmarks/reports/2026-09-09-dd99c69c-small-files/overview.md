# Small-file CPU batching

Dataset: **Adjudicated combined corpus — natural workload subset**, version `snapshot-1411a5c0fd4a-natural-workloads`. Measured at **2026-09-09T00:53:12.166857Z**; benchmark protocol `1.1.0`.

<details><summary>Exact tool configurations and build identities</summary>

- **Magika 2.0.0-dev @ 1848ae9b** — config `04acfd97`, build `52da2f310ed4674d78a3bc5a8da7a3468edc87aab7745f395b22a7043f45c55f`: `{"backend": "CPU", "internal_batch": 8, "model": "standard_v3_3", "readers": 2, "rules": "off", "threads": 2}`
- **Magika 2.0.0-dev @ dd99c69c** — config `cd71f59c`, build `d24bc3238331b7376451332c894eb2e52645ca9af8fb41dac7ac8a4787e90813`: `{"backend": "CPU", "internal_batch": 8, "model": "standard_v3_3", "readers": 2, "rules": "off", "threads": 2}`

Full commands, environment, source revisions and executable/artifact hashes are retained in `overview.json`.

</details>

1,018 identical normalized decisions, with zero tool errors. 5 measured runs after 1 warmup. Whole-process median milliseconds, warm OS caches. The subset is the union of saved natural workloads; it is not a new full-corpus accuracy evaluation.

| Tool | Version | Config | 1 files | 2 files | 5 files | 10 files | 1,000 files |
|---|---|---|---:|---:|---:|---:|---:|
| Magika | 2.0.0-dev @ 1848ae9b | CPU; ML (`04acfd97`) | 5.99 | 12.14 | 12.30 | 13.54 | 611.86 |
| Magika | 2.0.0-dev @ dd99c69c | CPU; ML (`cd71f59c`) | 6.04 | 7.58 | 12.16 | 13.46 | 604.55 |

| Files | Elapsed-time reduction |
|---:|---:|
| 1 | -0.90% |
| 2 | +37.52% |
| 5 | +1.13% |
| 10 | +0.61% |
| 1,000 | +1.20% |

Both binaries use identical commands and the same deferred CPU runtime library. Only explicit CPU invocations with two or three non-recursive paths receive the new cap. Single-file, GPU, recursive and larger-workload policies remain unchanged. Small movements in unchanged workloads measure run-to-run host variation.
