# Performance handoff validation

Recorded: 2026-09-09T01:44:30Z. Apple M5 Max, release builds.

The original handoff is preserved unchanged. This table is generated from the linked raw JSON; whole-process, inference-loop, and component timings remain separate.

| Measurement | Before | Candidate |
|---|---:|---:|
| CPU inference loop, batch 1, files/s | 814 | 1,082 |
| CPU inference loop, batch 4, files/s | 794 | 1,103 |
| Native scratch allocation, µs | 578.88 | 188.71 |
| Fresh-process CPU prepare, µs | 2113.90 | 1948.00 |

| Follow-up whole-process timing | Before | Candidate |
|---|---:|---:|
| 5 files, ms | 13.19 | 12.98 |
| 1,000 files, ms | 606.04 | 611.26 |

| Default policy check, 1,000 files | 17 threads | 4 threads |
|---|---:|---:|
| Median wall time, ms | 268.15 | 365.66 |
| Mean user CPU, seconds | 3.667 | 1.487 |

Median session drop: 0.46 µs; runtime drop: 17.04 µs.

CRC oracle: 65,696 passing cases under ASan/UBSan. Maximum differing probe-score bits: 0.

The full 25,421-file comparison retained every normalized decision and recorded zero tool errors. [Full accuracy and timing table](../../results/v1/2026-09-09-754c063e-fusion/report.md). The follow-up timings above retain the confirmation run after outliers in the initial run; neither run was discarded.
