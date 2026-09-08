# Sembiance v3 — e5f7e339

2,400 eligible samples across 167 formats, from 33,421 verified files. 3 timed runs after 1 warmup; 29 seeded workloads per mode. Times are median milliseconds, including startup and shutdown, with warm OS caches.

| Tool / mode | Accuracy | Precision | Coverage | Rule matches | Wrong decisions | 1 file | 10 files | 100 files | 1,000 files |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Magika 1 CPU | 58.29% | 71.27% | 81.79% | — | 564 | 23.24 | 41.48 | 235.21 | 2,143.49 |
| Magika 2 CPU ML | 58.29% | 71.27% | 81.79% | — | 564 | 8.22 | 16.18 | 68.77 | 622.32 |
| Magika 2 CPU rules + ML | 65.21% | 76.27% | 85.50% | 22.21% | 487 | 4.55 | 14.99 | 51.31 | 504.78 |
| Magika 2 GPU ML | 58.29% | 71.27% | 81.79% | — | 564 | 79.61 | 91.17 | 124.00 | 251.66 |
| Magika 2 GPU rules + ML | 65.21% | 76.27% | 85.50% | 22.21% | 487 | 8.49 | 99.91 | 116.49 | 209.79 |
| Magika 2 rules-only CPU | 22.21% | 100.00% | 22.21% | 22.21% | 0 | 3.94 | 4.11 | 7.10 | 54.01 |
| libmagic | 33.29% | 96.61% | 34.46% | — | 28 | 2.23 | 12.25 | 78.63 | 869.56 |
| TrID | 42.58% | 90.68% | 46.96% | — | 105 | 88.24 | 92.98 | 220.49 | 1,774.83 |

The scoring subset is selected solely by the dataset lane’s existing `evaluation_eligible` flags: 1,487 structurally validated and 913 reviewed source mappings. Exact overlaps are excluded; near-duplicate independence is not certified. Detector aliases are frozen from the baseline benchmark, with six additional source-declared classes. No source labels, rules or model weights were changed for this run. This is an external evaluation; V56 remains the base dataset.
