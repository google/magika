# Tool-default benchmark

Dataset: **Adjudicated combined corpus**, version `snapshot-1411a5c0fd4a`. Measured at **2026-09-09T22:07:12.019537Z**; benchmark protocol `1.2.2`.

<details><summary>Exact tool configurations and build identities</summary>

- **Magika 2.0.0-dev @ 65505ce3** — config `f8f02ebe`, build `083b90e2b9a41b98dffe682b295bfca00593a6c40c469aac90f5987eac5335fd`: `{"backend": "CPU", "model": null, "resource_policy": "tool defaults", "rules": "only"}`
- **TrID 2.48** — config `477850ab`, build `0d4f1c391a399c8dda6259ff17251008fc082f8f1a695dab0b60c22f279b2dc7`: `{"resource_policy": "tool defaults", "strings": true, "stringzilla": "off"}`
- **TrID 2.48** — config `a84d82e2`, build `25209e1003e1393dee001c56ff193a828c9a3e84ceac724ff58edd85543fcf9f`: `{"resource_policy": "tool defaults", "strings": true, "stringzilla": "5.1.2"}`

Full commands, environment, source revisions and executable/artifact hashes are retained in `overview.json`.

</details>

Accuracy on 25,421 files; 7 saved workloads, 3 measured runs after 1 warmup. Every tool uses its default worker, reader and internal batch policy. No thread-cap environment variables are set.

Whole-process median milliseconds, including startup, I/O, output and shutdown; warm OS caches. Default settings are measured here; this is not a claim that every tool has been exhaustively tuned.

| Tool | Version | Config | Accuracy | Precision | Coverage | Rule matches | 1 file | 2 files | 5 files | 10 files | 25 files | 100 files | 1,000 files |
|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Magika | 2.0.0-dev @ 65505ce3 | CPU; rules-only (`f8f02ebe`); defaults | 37.86% | 100.00% | 37.86% | 37.86% | 2.78 | 3.46 | 3.34 | 3.24 | 4.12 | 8.59 | 64.96 |
| TrID | 2.48 | strings=True; StringZilla=off (`477850ab`); defaults | 55.25% | 78.32% | 70.55% | — | 93.54 | 105.96 | 77.06 | 101.50 | 85.87 | 278.25 | 4,775.78 |
| TrID | 2.48 | strings=True; StringZilla=5.1.2 (`a84d82e2`); defaults | 55.25% | 78.32% | 70.55% | — | 79.12 | 76.44 | 94.44 | 87.22 | 85.50 | 130.40 | 1,118.27 |

Quality evaluation uses invocations of up to 128 files. For CPU warmup runs, per-workload validation metrics and inference differences are retained in the result JSON.

Raw observations, exact commands, executable/model/database hashes and per-workload timings are retained in the run JSON. GPU means Metal on this host.
