# Tool-default benchmark

Dataset: **Sembiance**, version `v3`. Measured at **2026-09-09T03:42:18.100538Z**; benchmark protocol `1.2.1`.

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

Accuracy on 2,400 files; 7 saved workloads, 3 measured runs after 1 warmup. Every tool uses its default worker, reader and internal batch policy. No thread-cap environment variables are set.

Whole-process median milliseconds, including startup, I/O, output and shutdown; warm OS caches. Default settings are measured here; this is not a claim that every tool has been exhaustively tuned.

| Tool | Version | Config | Accuracy | Precision | Coverage | Rule matches | 1 file | 2 files | 5 files | 10 files | 25 files | 100 files | 1,000 files |
|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Magika | 1.1.0 | CPU; ML (`9be4a280`); defaults | 58.29% | 71.27% | 81.79% | — | 52.79 | 53.23 | 54.49 | 56.13 | 58.45 | 90.90 | 416.06 |
| Magika | 2.0.0-dev @ 65505ce3 | CPU; ML (`cc3270f7`); defaults | 58.29% | 71.27% | 81.79% | — | 5.43 | 6.00 | 12.00 | 12.95 | 18.40 | 38.05 | 252.80 |
| Magika | 2.0.0-dev @ 65505ce3 | CPU; rules + ML (`15e7a0f3`); defaults | 65.21% | 76.27% | 85.50% | 22.21% | 2.64 | 5.99 | 11.69 | 11.61 | 15.78 | 31.57 | 199.86 |
| Magika | 2.0.0-dev @ 65505ce3 | Metal; ML; CPU warmup (`740ce2ef`); defaults | 58.29% | 71.27% | 81.79% | — | 6.19 | 5.97 | 12.39 | 14.36 | 20.44 | 37.85 | 163.18 |
| Magika | 2.0.0-dev @ 65505ce3 | Metal; rules + ML; CPU warmup (`39d2c09a`); defaults | 65.21% | 76.27% | 85.50% | 22.21% | 2.69 | 6.53 | 12.16 | 12.50 | 16.55 | 31.18 | 139.44 |
| Magika | 2.0.0-dev @ 65505ce3 | Auto; ML; CPU warmup (`2bf84c20`); defaults | 58.29% | 71.27% | 81.79% | — | 4.89 | 6.34 | 12.01 | 14.12 | 20.24 | 38.33 | 178.22 |
| Magika | 2.0.0-dev @ 65505ce3 | Auto; rules + ML; CPU warmup (`d8d1d1cf`); defaults | 65.21% | 76.27% | 85.50% | 22.21% | 2.80 | 6.23 | 12.17 | 12.53 | 17.78 | 31.23 | 140.41 |
| Magika | 2.0.0-dev @ 65505ce3 | CPU; rules-only (`f8f02ebe`); defaults | 22.21% | 100.00% | 22.21% | 22.21% | 2.47 | 2.76 | 2.74 | 3.21 | 4.05 | 7.80 | 53.64 |
| libmagic | 5.41 | MIME; serial (`b787da2e`); defaults | 33.29% | 96.61% | 34.46% | — | 1.10 | 7.73 | 7.94 | 10.98 | 8.24 | 78.83 | 630.30 |
| TrID | 2.48 | strings=True; StringZilla=off (`477850ab`); defaults | 42.58% | 90.68% | 46.96% | — | 66.85 | 66.33 | 68.20 | 83.65 | 173.63 | 181.24 | 1,286.28 |

## CPU/GPU configuration crossover

Winners below compare requested CPU and GPU configurations on the same natural workloads. GPU configurations include CPU warmup when their Config column says so. No crossover is interpolated between file counts. A rules-only hit with no inference is excluded from CPU/GPU crossover claims.

| Mode | 1 files | 2 files | 5 files | 10 files | 25 files | 100 files | 1,000 files | First measured GPU win |
|---|---|---|---|---|---|---|---|---|
| ML | CPU | GPU | CPU | CPU | CPU | GPU | GPU | 2 |
| rules + ML | no inference | CPU | CPU | CPU | CPU | GPU | GPU | 100 |

Quality evaluation uses invocations of up to 128 files. For CPU warmup runs, per-workload validation metrics and inference differences are retained in the result JSON.

Raw observations, exact commands, executable/model/database hashes and per-workload timings are retained in the run JSON. GPU means Metal on this host.
