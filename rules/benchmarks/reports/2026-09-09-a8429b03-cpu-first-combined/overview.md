# Tool-default benchmark

Dataset: **Adjudicated combined corpus**, version `snapshot-1411a5c0fd4a`. Measured at **2026-09-09T03:31:45.835824Z**; benchmark protocol `1.2.1`.

<details><summary>Exact tool configurations and build identities</summary>

- **Magika 1.1.0** — config `9be4a280`, build `4ce463f9f59f11a701cd9674e80103005915cd2ef5412a20c57aaaa305d27331`: `{"backend": "CPU", "model": "standard_v3_3", "release": "cli/v1.1.0", "resource_policy": "tool defaults"}`
- **Magika 2.0.0-dev @ a8429b03** — config `cc3270f7`, build `91846b0625999f212d276c1e0be9516bb0517a4f7eab66e4f8bd3bb37db4c23c`: `{"backend": "CPU", "model": "standard_v3_3", "resource_policy": "tool defaults", "rules": "off"}`
- **Magika 2.0.0-dev @ a8429b03** — config `15e7a0f3`, build `91846b0625999f212d276c1e0be9516bb0517a4f7eab66e4f8bd3bb37db4c23c`: `{"backend": "CPU", "model": "standard_v3_3", "resource_policy": "tool defaults", "rules": "enforce"}`
- **Magika 2.0.0-dev @ a8429b03** — config `740ce2ef`, build `91846b0625999f212d276c1e0be9516bb0517a4f7eab66e4f8bd3bb37db4c23c`: `{"backend": "GPU (Metal)", "gpu_admission": "ready", "model": "standard_v3_3", "resource_policy": "tool defaults", "rules": "off", "startup_backend": "CPU"}`
- **Magika 2.0.0-dev @ a8429b03** — config `39d2c09a`, build `91846b0625999f212d276c1e0be9516bb0517a4f7eab66e4f8bd3bb37db4c23c`: `{"backend": "GPU (Metal)", "gpu_admission": "ready", "model": "standard_v3_3", "resource_policy": "tool defaults", "rules": "enforce", "startup_backend": "CPU"}`
- **Magika 2.0.0-dev @ a8429b03** — config `b05bf365`, build `91846b0625999f212d276c1e0be9516bb0517a4f7eab66e4f8bd3bb37db4c23c`: `{"backend": "Auto", "gpu_admission": "ready with queued work", "model": "standard_v3_3", "resource_policy": "tool defaults", "rules": "off", "startup_backend": "CPU"}`
- **Magika 2.0.0-dev @ a8429b03** — config `b8e04b12`, build `91846b0625999f212d276c1e0be9516bb0517a4f7eab66e4f8bd3bb37db4c23c`: `{"backend": "Auto", "gpu_admission": "ready with queued work", "model": "standard_v3_3", "resource_policy": "tool defaults", "rules": "enforce", "startup_backend": "CPU"}`
- **Magika 2.0.0-dev @ a8429b03** — config `f8f02ebe`, build `91846b0625999f212d276c1e0be9516bb0517a4f7eab66e4f8bd3bb37db4c23c`: `{"backend": "CPU", "model": null, "resource_policy": "tool defaults", "rules": "only"}`
- **libmagic 5.41** — config `b787da2e`, build `808833b8da07e23b5cd32ef92460cfc4b80472e8ce612ebc39d3c5580afb98b5`: `{"output": "NUL-framed MIME", "resource_policy": "tool defaults"}`
- **TrID 2.48** — config `cf7a7915`, build `25209e1003e1393dee001c56ff193a828c9a3e84ceac724ff58edd85543fcf9f`: `{"resource_policy": "tool defaults", "strings": true, "stringzilla": "5.1.2"}`

Full commands, environment, source revisions and executable/artifact hashes are retained in `overview.json`.

</details>

Accuracy on 25,421 files; 7 saved workloads, 3 measured runs after 1 warmup. Every tool uses its default worker, reader and internal batch policy. No thread-cap environment variables are set.

Whole-process median milliseconds, including startup, I/O, output and shutdown; warm OS caches. Default settings are measured here; this is not a claim that every tool has been exhaustively tuned.

| Tool | Version | Config | Accuracy | Precision | Coverage | Rule matches | 1 file | 2 files | 5 files | 10 files | 25 files | 100 files | 1,000 files |
|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Magika | 1.1.0 | CPU; ML (`9be4a280`); defaults | 57.73% | 70.23% | 82.20% | — | 63.71 | 75.42 | 59.29 | 64.93 | 65.46 | 110.18 | 539.25 |
| Magika | 2.0.0-dev @ a8429b03 | CPU; ML (`cc3270f7`); defaults | 57.73% | 70.23% | 82.20% | — | 5.53 | 7.72 | 12.29 | 16.03 | 19.91 | 46.49 | 310.06 |
| Magika | 2.0.0-dev @ a8429b03 | CPU; rules + ML (`15e7a0f3`); defaults | 78.93% | 84.39% | 93.52% | 37.86% | 3.77 | 7.57 | 12.02 | 12.13 | 16.80 | 30.79 | 225.44 |
| Magika | 2.0.0-dev @ a8429b03 | Metal; ML; CPU warmup (`740ce2ef`); defaults | 57.73% | 70.23% | 82.20% | — | 7.51 | 8.79 | 13.59 | 17.93 | 21.23 | 45.14 | 346.01 |
| Magika | 2.0.0-dev @ a8429b03 | Metal; rules + ML; CPU warmup (`39d2c09a`); defaults | 78.93% | 84.39% | 93.52% | 37.86% | 3.63 | 7.01 | 16.35 | 13.92 | 28.51 | 37.23 | 250.52 |
| Magika | 2.0.0-dev @ a8429b03 | Auto; ML; CPU warmup (`b05bf365`); defaults | 57.73% | 70.23% | 82.20% | — | 6.83 | 8.90 | 14.50 | 21.09 | 24.36 | 53.29 | 335.48 |
| Magika | 2.0.0-dev @ a8429b03 | Auto; rules + ML; CPU warmup (`b8e04b12`); defaults | 78.93% | 84.39% | 93.52% | 37.86% | 3.52 | 7.13 | 13.79 | 13.46 | 20.52 | 38.14 | 226.02 |
| Magika | 2.0.0-dev @ a8429b03 | CPU; rules-only (`f8f02ebe`); defaults | 37.86% | 100.00% | 37.86% | 37.86% | 3.21 | 2.78 | 3.85 | 3.44 | 4.49 | 9.77 | 83.14 |
| libmagic | 5.41 | MIME; serial (`b787da2e`); defaults | 25.46% | 93.24% | 27.31% | — | 1.38 | 2.07 | 4.00 | 5.62 | 17.27 | 36.27 | 381.77 |
| TrID | 2.48 | strings=True; StringZilla=5.1.2 (`cf7a7915`); defaults | 55.25% | 78.32% | 70.55% | — | 84.24 | 113.60 | 79.07 | 111.67 | 105.36 | 320.12 | 5,433.15 |

## CPU/GPU configuration crossover

Winners below compare requested CPU and GPU configurations on the same natural workloads. GPU configurations include CPU warmup when their Config column says so. No crossover is interpolated between file counts. A rules-only hit with no inference is excluded from CPU/GPU crossover claims.

| Mode | 1 files | 2 files | 5 files | 10 files | 25 files | 100 files | 1,000 files | First measured GPU win |
|---|---|---|---|---|---|---|---|---|
| ML | CPU | CPU | CPU | CPU | CPU | GPU | CPU | 100 |
| rules + ML | no inference | GPU | CPU | CPU | CPU | CPU | CPU | 2 |

Quality evaluation uses invocations of up to 128 files. For CPU warmup runs, per-workload validation metrics and inference differences are retained in the result JSON.

Raw observations, exact commands, executable/model/database hashes and per-workload timings are retained in the run JSON. GPU means Metal on this host.
