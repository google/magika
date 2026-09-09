# TrID StringZilla comparison

Dataset: **Adjudicated combined corpus**, version `snapshot-1411a5c0fd4a`. Measured at **2026-09-09T22:07:12.019537+00:00**.

Both configurations use the same Python executable bytes, TrID script, definitions, input files and default resource policy. The accelerated configuration uses a Python virtual environment containing the verified StringZilla library; its startup is included. One warmup and three measured runs per configuration/workload, with shuffled execution order.

Normalized observations agree on **25,421 / 25,421 files**. Quality scores and exact identities are in the [full table](overview.md); any differences are retained in JSON.

Whole-process median milliseconds, with warm OS caches. Speedup is the unaccelerated median divided by the accelerated median.

| Files | TrID without StringZilla | TrID with StringZilla | Speedup | Time change |
|---:|---:|---:|---:|---:|
| 1 | 93.54 | 79.12 | 1.18× | -15.4% |
| 2 | 105.96 | 76.44 | 1.39× | -27.9% |
| 5 | 77.06 | 94.44 | 0.82× | +22.6% |
| 10 | 101.50 | 87.22 | 1.16× | -14.1% |
| 25 | 85.87 | 85.50 | 1.00× | -0.4% |
| 100 | 278.25 | 130.40 | 2.13× | -53.1% |
| 1,000 | 4775.78 | 1118.27 | 4.27× | -76.6% |
