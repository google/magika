# Tool-default benchmark

Dataset: **Adjudicated combined corpus**, version `snapshot-1411a5c0fd4a`. Measured at **2026-09-09T03:03:17.576954Z**; benchmark protocol `1.2.0`.

<details><summary>Exact tool configurations and build identities</summary>

- **Magika 1.1.0** — config `9be4a280`, build `4ce463f9f59f11a701cd9674e80103005915cd2ef5412a20c57aaaa305d27331`: `{"backend": "CPU", "model": "standard_v3_3", "release": "cli/v1.1.0", "resource_policy": "tool defaults"}`
- **Magika 2.0.0-dev @ 9214192e** — config `cc3270f7`, build `37b7a08529533473cfbed9e91ad8e5a61f907a07191b9c82467992710c87ac93`: `{"backend": "CPU", "model": "standard_v3_3", "resource_policy": "tool defaults", "rules": "off"}`
- **Magika 2.0.0-dev @ 9214192e** — config `15e7a0f3`, build `37b7a08529533473cfbed9e91ad8e5a61f907a07191b9c82467992710c87ac93`: `{"backend": "CPU", "model": "standard_v3_3", "resource_policy": "tool defaults", "rules": "enforce"}`
- **Magika 2.0.0-dev @ 9214192e** — config `7e8b755b`, build `37b7a08529533473cfbed9e91ad8e5a61f907a07191b9c82467992710c87ac93`: `{"backend": "GPU (Metal)", "model": "standard_v3_3", "resource_policy": "tool defaults", "rules": "off"}`
- **Magika 2.0.0-dev @ 9214192e** — config `0dec6350`, build `37b7a08529533473cfbed9e91ad8e5a61f907a07191b9c82467992710c87ac93`: `{"backend": "GPU (Metal)", "model": "standard_v3_3", "resource_policy": "tool defaults", "rules": "enforce"}`
- **Magika 2.0.0-dev @ 9214192e** — config `11c2b636`, build `37b7a08529533473cfbed9e91ad8e5a61f907a07191b9c82467992710c87ac93`: `{"backend": "Auto", "model": "standard_v3_3", "resource_policy": "tool defaults", "rules": "off"}`
- **Magika 2.0.0-dev @ 9214192e** — config `f28555ab`, build `37b7a08529533473cfbed9e91ad8e5a61f907a07191b9c82467992710c87ac93`: `{"backend": "Auto", "model": "standard_v3_3", "resource_policy": "tool defaults", "rules": "enforce"}`
- **Magika 2.0.0-dev @ 9214192e** — config `f8f02ebe`, build `37b7a08529533473cfbed9e91ad8e5a61f907a07191b9c82467992710c87ac93`: `{"backend": "CPU", "model": null, "resource_policy": "tool defaults", "rules": "only"}`
- **libmagic 5.41** — config `b787da2e`, build `808833b8da07e23b5cd32ef92460cfc4b80472e8ce612ebc39d3c5580afb98b5`: `{"output": "NUL-framed MIME", "resource_policy": "tool defaults"}`
- **TrID 2.48** — config `cf7a7915`, build `25209e1003e1393dee001c56ff193a828c9a3e84ceac724ff58edd85543fcf9f`: `{"resource_policy": "tool defaults", "strings": true, "stringzilla": "5.1.2"}`

Full commands, environment, source revisions and executable/artifact hashes are retained in `overview.json`.

</details>

Accuracy on 25,421 files; 30 saved workloads, 3 measured runs after 1 warmup. Every tool uses its default worker, reader and internal batch policy. No thread-cap environment variables are set.

Whole-process median milliseconds, including startup, I/O, output and shutdown; warm OS caches. Default settings are measured here; this is not a claim that every tool has been exhaustively tuned.

| Tool | Version | Config | Accuracy | Precision | Coverage | Rule matches | 1 file | 2 files | 5 files | 10 files | 25 files | 100 files | 1,000 files |
|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Magika | 1.1.0 | CPU; ML (`9be4a280`); defaults | 57.73% | 70.23% | 82.20% | — | 57.57 | 65.67 | 61.49 | 63.47 | 72.25 | 106.24 | 493.08 |
| Magika | 2.0.0-dev @ 9214192e | CPU; ML (`cc3270f7`); defaults | 57.73% | 70.23% | 82.20% | — | 6.79 | 7.04 | 12.69 | 15.87 | 21.08 | 44.01 | 265.38 |
| Magika | 2.0.0-dev @ 9214192e | CPU; rules + ML (`15e7a0f3`); defaults | 78.93% | 84.39% | 93.52% | 37.86% | 4.52 | 7.21 | 14.42 | 14.67 | 18.35 | 30.72 | 178.84 |
| Magika | 2.0.0-dev @ 9214192e | Metal; ML (`7e8b755b`); defaults | 57.73% | 70.22% | 82.21% | — | 84.36 | 86.61 | 88.18 | 87.71 | 88.32 | 89.65 | 186.21 |
| Magika | 2.0.0-dev @ 9214192e | Metal; rules + ML (`0dec6350`); defaults | 78.93% | 84.38% | 93.53% | 37.86% | 6.07 | 71.44 | 80.57 | 88.66 | 88.74 | 84.61 | 134.27 |
| Magika | 2.0.0-dev @ 9214192e | Auto; ML (`11c2b636`); defaults | 57.73% | 70.22% | 82.21% | — | 5.87 | 75.85 | 81.81 | 87.70 | 91.20 | 88.46 | 194.14 |
| Magika | 2.0.0-dev @ 9214192e | Auto; rules + ML (`f28555ab`); defaults | 78.93% | 84.38% | 93.53% | 37.86% | 3.78 | 83.97 | 87.28 | 110.39 | 78.55 | 93.31 | 135.84 |
| Magika | 2.0.0-dev @ 9214192e | CPU; rules-only (`f8f02ebe`); defaults | 37.86% | 100.00% | 37.86% | 37.86% | 3.35 | 3.81 | 3.43 | 4.08 | 5.64 | 9.33 | 67.09 |
| libmagic | 5.41 | MIME; serial (`b787da2e`); defaults | 25.46% | 93.24% | 27.31% | — | 1.63 | 2.05 | 4.51 | 6.09 | 13.13 | 35.27 | 347.32 |
| TrID | 2.48 | strings=True; StringZilla=5.1.2 (`cf7a7915`); defaults | 55.25% | 78.32% | 70.55% | — | 83.06 | 104.00 | 76.64 | 109.84 | 100.82 | 304.64 | 5,156.85 |

## CPU/GPU configuration crossover

Winners below compare requested CPU and GPU configurations on the same natural workloads. GPU configurations include CPU warmup when their Config column says so. No crossover is interpolated between file counts. A rules-only hit with no inference is excluded from CPU/GPU crossover claims.

| Mode | 1 files | 2 files | 5 files | 10 files | 25 files | 100 files | 1,000 files | First measured GPU win |
|---|---|---|---|---|---|---|---|---|
| ML | CPU | CPU | CPU | CPU | CPU | CPU | GPU | 1000 |
| rules + ML | no inference | CPU | CPU | CPU | CPU | CPU | GPU | 1000 |

Quality evaluation uses invocations of up to 128 files. For CPU warmup runs, per-workload validation metrics and inference differences are retained in the result JSON.

Raw observations, exact commands, executable/model/database hashes and per-workload timings are retained in the run JSON. GPU means Metal on this host.
