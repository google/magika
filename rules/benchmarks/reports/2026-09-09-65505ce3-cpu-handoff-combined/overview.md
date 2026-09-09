# Tool-default benchmark

Dataset: **Adjudicated combined corpus**, version `snapshot-1411a5c0fd4a`. Measured at **2026-09-09T03:39:30.163738Z**; benchmark protocol `1.2.1`.

<details><summary>Exact tool configurations and build identities</summary>

- **Magika 1.1.0** — config `9be4a280`, build `4ce463f9f59f11a701cd9674e80103005915cd2ef5412a20c57aaaa305d27331`: `{"backend": "CPU", "model": "standard_v3_3", "release": "cli/v1.1.0", "resource_policy": "tool defaults"}`
- **Magika 2.0.0-dev @ 65505ce3** — config `cc3270f7`, build `083b90e2b9a41b98dffe682b295bfca00593a6c40c469aac90f5987eac5335fd`: `{"backend": "CPU", "model": "standard_v3_3", "resource_policy": "tool defaults", "rules": "off"}`
- **Magika 2.0.0-dev @ 65505ce3** — config `15e7a0f3`, build `083b90e2b9a41b98dffe682b295bfca00593a6c40c469aac90f5987eac5335fd`: `{"backend": "CPU", "model": "standard_v3_3", "resource_policy": "tool defaults", "rules": "enforce"}`
- **Magika 2.0.0-dev @ 65505ce3** — config `740ce2ef`, build `083b90e2b9a41b98dffe682b295bfca00593a6c40c469aac90f5987eac5335fd`: `{"backend": "GPU (Metal)", "gpu_admission": "ready", "model": "standard_v3_3", "resource_policy": "tool defaults", "rules": "off", "startup_backend": "CPU"}`
- **Magika 2.0.0-dev @ 65505ce3** — config `39d2c09a`, build `083b90e2b9a41b98dffe682b295bfca00593a6c40c469aac90f5987eac5335fd`: `{"backend": "GPU (Metal)", "gpu_admission": "ready", "model": "standard_v3_3", "resource_policy": "tool defaults", "rules": "enforce", "startup_backend": "CPU"}`
- **Magika 2.0.0-dev @ 65505ce3** — config `2bf84c20`, build `083b90e2b9a41b98dffe682b295bfca00593a6c40c469aac90f5987eac5335fd`: `{"backend": "Auto", "gpu_admission": "ready, at least eight files in the current batch", "model": "standard_v3_3", "resource_policy": "tool defaults", "rules": "off", "startup_backend": "CPU"}`
- **Magika 2.0.0-dev @ 65505ce3** — config `d8d1d1cf`, build `083b90e2b9a41b98dffe682b295bfca00593a6c40c469aac90f5987eac5335fd`: `{"backend": "Auto", "gpu_admission": "ready, at least eight files in the current batch", "model": "standard_v3_3", "resource_policy": "tool defaults", "rules": "enforce", "startup_backend": "CPU"}`
- **Magika 2.0.0-dev @ 65505ce3** — config `f8f02ebe`, build `083b90e2b9a41b98dffe682b295bfca00593a6c40c469aac90f5987eac5335fd`: `{"backend": "CPU", "model": null, "resource_policy": "tool defaults", "rules": "only"}`
- **libmagic 5.41** — config `b787da2e`, build `808833b8da07e23b5cd32ef92460cfc4b80472e8ce612ebc39d3c5580afb98b5`: `{"output": "NUL-framed MIME", "resource_policy": "tool defaults"}`
- **TrID 2.48** — config `477850ab`, build `25209e1003e1393dee001c56ff193a828c9a3e84ceac724ff58edd85543fcf9f`: `{"resource_policy": "tool defaults", "strings": true, "stringzilla": "off"}`

Full commands, environment, source revisions and executable/artifact hashes are retained in `overview.json`.

</details>

Configuration correction: TrID acceleration is labeled from saved runtime evidence. Earlier declared StringZilla settings were inaccurate; the raw records and timings are preserved unchanged. Correction details are recorded in `overview.json`.

Accuracy on 25,421 files; 7 saved workloads, 3 measured runs after 1 warmup. Every tool uses its default worker, reader and internal batch policy. No thread-cap environment variables are set.

Whole-process median milliseconds, including startup, I/O, output and shutdown; warm OS caches. Default settings are measured here; this is not a claim that every tool has been exhaustively tuned.

| Tool | Version | Config | Accuracy | Precision | Coverage | Rule matches | 1 file | 2 files | 5 files | 10 files | 25 files | 100 files | 1,000 files |
|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Magika | 1.1.0 | CPU; ML (`9be4a280`); defaults | 57.73% | 70.23% | 82.20% | — | 53.78 | 54.16 | 54.53 | 55.39 | 59.62 | 93.17 | 410.84 |
| Magika | 2.0.0-dev @ 65505ce3 | CPU; ML (`cc3270f7`); defaults | 57.73% | 70.23% | 82.20% | — | 5.04 | 6.22 | 11.75 | 13.14 | 19.27 | 36.18 | 256.56 |
| Magika | 2.0.0-dev @ 65505ce3 | CPU; rules + ML (`15e7a0f3`); defaults | 78.93% | 84.39% | 93.52% | 37.86% | 2.92 | 5.97 | 11.91 | 12.35 | 16.07 | 28.13 | 164.70 |
| Magika | 2.0.0-dev @ 65505ce3 | Metal; ML; CPU warmup (`740ce2ef`); defaults | 57.73% | 70.23% | 82.20% | — | 6.13 | 6.18 | 12.40 | 15.11 | 20.41 | 37.76 | 169.54 |
| Magika | 2.0.0-dev @ 65505ce3 | Metal; rules + ML; CPU warmup (`39d2c09a`); defaults | 78.93% | 84.39% | 93.52% | 37.86% | 2.81 | 6.21 | 12.16 | 12.53 | 17.30 | 29.86 | 122.43 |
| Magika | 2.0.0-dev @ 65505ce3 | Auto; ML; CPU warmup (`2bf84c20`); defaults | 57.73% | 70.23% | 82.20% | — | 5.18 | 6.10 | 12.41 | 14.67 | 20.16 | 38.46 | 163.87 |
| Magika | 2.0.0-dev @ 65505ce3 | Auto; rules + ML; CPU warmup (`d8d1d1cf`); defaults | 78.93% | 84.39% | 93.52% | 37.86% | 2.77 | 6.27 | 12.29 | 12.58 | 17.46 | 28.61 | 128.80 |
| Magika | 2.0.0-dev @ 65505ce3 | CPU; rules-only (`f8f02ebe`); defaults | 37.86% | 100.00% | 37.86% | 37.86% | 2.67 | 2.73 | 2.82 | 3.15 | 4.10 | 9.00 | 64.83 |
| libmagic | 5.41 | MIME; serial (`b787da2e`); defaults | 25.46% | 93.24% | 27.31% | — | 1.26 | 1.53 | 3.87 | 4.93 | 11.68 | 33.35 | 336.20 |
| TrID | 2.48 | strings=True; StringZilla=off (`477850ab`); defaults | 55.25% | 78.32% | 70.55% | — | 70.52 | 96.69 | 71.43 | 97.90 | 81.54 | 272.84 | 4,850.05 |

## CPU/GPU configuration crossover

Winners below compare requested CPU and GPU configurations on the same natural workloads. GPU configurations include CPU warmup when their Config column says so. No crossover is interpolated between file counts. A rules-only hit with no inference is excluded from CPU/GPU crossover claims.

| Mode | 1 files | 2 files | 5 files | 10 files | 25 files | 100 files | 1,000 files | First measured GPU win |
|---|---|---|---|---|---|---|---|---|
| ML | CPU | GPU | CPU | CPU | CPU | CPU | GPU | 2 |
| rules + ML | no inference | CPU | CPU | CPU | CPU | CPU | GPU | 1000 |

Quality evaluation uses invocations of up to 128 files. For CPU warmup runs, per-workload validation metrics and inference differences are retained in the result JSON.

Raw observations, exact commands, executable/model/database hashes and per-workload timings are retained in the run JSON. GPU means Metal on this host.
