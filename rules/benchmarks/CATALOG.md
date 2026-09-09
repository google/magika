# Benchmark catalogue

UTC measurement timestamps and dataset versions come from saved evidence. The adjudicated combined snapshot has a content-addressed version; it is not the full V56 release. Sembiance v3 records the eligible scoring subset and the complete source size.

| Dataset | Version | Scored / source files | Measured at (UTC) | Protocol | Revision | Run |
|---|---|---:|---|---|---|---|
| Adjudicated combined corpus | `snapshot-1411a5c0fd4a` | 25,421 / 25,421 | 2026-09-08T18:22:05.748172Z | 1.0.0 | `425a5e38` | [2026-09-08-425a5e38-cpu](results/v1/2026-09-08-425a5e38-cpu/report.md) |
| Adjudicated combined corpus | `snapshot-1411a5c0fd4a` | 25,421 / 25,421 | 2026-09-08T18:30:12.121784Z | 1.0.0 | `425a5e38` | [2026-09-08-425a5e38-gpu](results/v1/2026-09-08-425a5e38-gpu/report.md) |
| Adjudicated combined corpus | `snapshot-1411a5c0fd4a` | 25,421 / 25,421 | 2026-09-08T18:55:28.159122Z | 1.1.0 | `2d6c3992` | [2026-09-08-2d6c3992-rules-only](results/v1/2026-09-08-2d6c3992-rules-only/report.md) |
| Adjudicated combined corpus | `snapshot-1411a5c0fd4a` | 25,421 / 25,421 | 2026-09-08T22:36:35.070300Z | 1.1.0 | `e5f7e339` | [2026-09-08-e5f7e339-all](reports/2026-09-08-e5f7e339-all/overview.md) |
| Sembiance | `v3` | 2,400 / 33,421 | 2026-09-08T23:11:38.675067Z | 1.1.0 | `e5f7e339` | [2026-09-08-e5f7e339-sembiance-v3](reports/2026-09-08-e5f7e339-sembiance-v3/overview.md) |
| Adjudicated combined corpus — natural workload subset | `snapshot-1411a5c0fd4a-natural-workloads` | 1,018 / 25,421 | 2026-09-09T00:50:31.823098Z | 1.1.0 | `7a0a720c` | [2026-09-09-7a0a720c-small-files](results/v1/2026-09-09-7a0a720c-small-files/report.md) |
| Adjudicated combined corpus — natural workload subset | `snapshot-1411a5c0fd4a-natural-workloads` | 1,018 / 25,421 | 2026-09-09T00:53:12.166857Z | 1.1.0 | `dd99c69c` | [2026-09-09-dd99c69c-small-files](reports/2026-09-09-dd99c69c-small-files/overview.md) |

The [JSON index](results/v1/index.json) retains full revisions, corpus fingerprints and artifact receipts. Each overview separates Tool, Version and Config; its JSON retains exact commands, settings, environment and binary/database hashes.
