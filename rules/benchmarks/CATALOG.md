# Benchmark catalogue

UTC measurement timestamps and dataset versions come from saved evidence. The adjudicated combined snapshot has a content-addressed version; it is not the full V56 release. Sembiance v3 records the eligible scoring subset and the complete source size.

| Dataset | Version | Scored / source files | Measured at (UTC) | Protocol | Revision | Run |
|---|---|---:|---|---|---|---|
| Adjudicated combined corpus | `snapshot-1411a5c0fd4a` | 25,421 / 25,421 | 2026-09-09T03:03:17.576954Z | 1.2.0 | `9214192e` | [2026-09-09-9214192e-defaults-combined](reports/2026-09-09-9214192e-defaults-combined/overview.md) |
| Sembiance | `v3` | 2,400 / 33,421 | 2026-09-09T03:08:57.821771Z | 1.2.0 | `9214192e` | [2026-09-09-9214192e-defaults-sembiance](reports/2026-09-09-9214192e-defaults-sembiance/overview.md) |
| Adjudicated combined corpus | `snapshot-1411a5c0fd4a` | 25,421 / 25,421 | 2026-09-09T03:39:30.163738Z | 1.2.1 | `65505ce3` | [2026-09-09-65505ce3-cpu-handoff-combined](reports/2026-09-09-65505ce3-cpu-handoff-combined/overview.md) |
| Sembiance | `v3` | 2,400 / 33,421 | 2026-09-09T03:42:18.100538Z | 1.2.1 | `65505ce3` | [2026-09-09-65505ce3-cpu-handoff-sembiance](reports/2026-09-09-65505ce3-cpu-handoff-sembiance/overview.md) |
| Adjudicated combined corpus | `snapshot-1411a5c0fd4a` | 25,421 / 25,421 | 2026-09-09T22:07:12.019537Z | 1.2.2 | `7599ca96` | [2026-09-09-7599ca96-trid-combined](reports/2026-09-09-7599ca96-trid-combined/overview.md) |
| Sembiance | `v3` | 2,400 / 33,421 | 2026-09-09T22:11:33.257060Z | 1.2.2 | `7599ca96` | [2026-09-09-7599ca96-trid-sembiance](reports/2026-09-09-7599ca96-trid-sembiance/overview.md) |

The [JSON index](results/v1/index.json) retains full revisions, corpus fingerprints and artifact receipts. Each overview separates Tool, Version and Config. Compressed run JSON retains commands, settings, environment and binary/database hashes; derived JSON can be regenerated locally.
