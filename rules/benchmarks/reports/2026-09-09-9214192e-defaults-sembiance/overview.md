# Tool-default benchmark

Dataset: **Sembiance**, version `v3`. Measured at **2026-09-09T03:08:57.821771Z**; benchmark protocol `1.2.0`.

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
- **TrID 2.48** — config `477850ab`, build `25209e1003e1393dee001c56ff193a828c9a3e84ceac724ff58edd85543fcf9f`: `{"resource_policy": "tool defaults", "strings": true, "stringzilla": "off"}`

Full commands, environment, source revisions and executable/artifact hashes are retained in `overview.json`.

</details>

Configuration correction: TrID acceleration is labeled from saved runtime evidence. Earlier declared StringZilla settings were inaccurate; the raw records and timings are preserved unchanged. Correction details are recorded in `overview.json`.

Accuracy on 2,400 files; 29 saved workloads, 3 measured runs after 1 warmup. Every tool uses its default worker, reader and internal batch policy. No thread-cap environment variables are set.

Whole-process median milliseconds, including startup, I/O, output and shutdown; warm OS caches. Default settings are measured here; this is not a claim that every tool has been exhaustively tuned.

| Tool | Version | Config | Accuracy | Precision | Coverage | Rule matches | 1 file | 2 files | 5 files | 10 files | 25 files | 100 files | 1,000 files |
|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Magika | 1.1.0 | CPU; ML (`9be4a280`); defaults | 58.29% | 71.27% | 81.79% | — | 60.71 | 70.39 | 59.13 | 60.79 | 99.65 | 146.30 | 1,051.82 |
| Magika | 2.0.0-dev @ 9214192e | CPU; ML (`cc3270f7`); defaults | 58.29% | 71.27% | 81.79% | — | 6.28 | 7.27 | 17.46 | 14.61 | 20.93 | 41.96 | 277.03 |
| Magika | 2.0.0-dev @ 9214192e | CPU; rules + ML (`15e7a0f3`); defaults | 65.21% | 76.27% | 85.50% | 22.21% | 3.17 | 11.73 | 13.41 | 12.84 | 29.68 | 36.70 | 221.60 |
| Magika | 2.0.0-dev @ 9214192e | Metal; ML (`7e8b755b`); defaults | 58.29% | 71.27% | 81.79% | — | 79.55 | 103.24 | 80.02 | 103.44 | 108.27 | 96.64 | 210.56 |
| Magika | 2.0.0-dev @ 9214192e | Metal; rules + ML (`0dec6350`); defaults | 65.21% | 76.27% | 85.50% | 22.21% | 4.79 | 88.69 | 84.18 | 86.09 | 89.12 | 106.79 | 178.92 |
| Magika | 2.0.0-dev @ 9214192e | Auto; ML (`11c2b636`); defaults | 58.29% | 71.27% | 81.79% | — | 8.05 | 87.16 | 86.96 | 80.45 | 86.16 | 116.58 | 186.10 |
| Magika | 2.0.0-dev @ 9214192e | Auto; rules + ML (`f28555ab`); defaults | 65.21% | 76.27% | 85.50% | 22.21% | 3.68 | 88.64 | 93.41 | 81.69 | 88.75 | 117.99 | 172.30 |
| Magika | 2.0.0-dev @ 9214192e | CPU; rules-only (`f8f02ebe`); defaults | 22.21% | 100.00% | 22.21% | 22.21% | 3.20 | 3.46 | 6.49 | 6.11 | 4.67 | 8.99 | 67.93 |
| libmagic | 5.41 | MIME; serial (`b787da2e`); defaults | 33.29% | 96.61% | 34.46% | — | 1.60 | 10.36 | 9.12 | 16.41 | 9.99 | 77.63 | 626.60 |
| TrID | 2.48 | strings=True; StringZilla=off (`477850ab`); defaults | 42.58% | 90.68% | 46.96% | — | 95.11 | 75.13 | 75.03 | 99.48 | 186.17 | 197.24 | 1,685.32 |

## CPU/GPU configuration crossover

Winners below compare requested CPU and GPU configurations on the same natural workloads. GPU configurations include CPU warmup when their Config column says so. No crossover is interpolated between file counts. A rules-only hit with no inference is excluded from CPU/GPU crossover claims.

| Mode | 1 files | 2 files | 5 files | 10 files | 25 files | 100 files | 1,000 files | First measured GPU win |
|---|---|---|---|---|---|---|---|---|
| ML | CPU | CPU | CPU | CPU | CPU | CPU | GPU | 1000 |
| rules + ML | no inference | CPU | CPU | CPU | CPU | CPU | GPU | 1000 |

Quality evaluation uses invocations of up to 128 files. For CPU warmup runs, per-workload validation metrics and inference differences are retained in the result JSON.

Raw observations, exact commands, executable/model/database hashes and per-workload timings are retained in the run JSON. GPU means Metal on this host.
