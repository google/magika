# TrID StringZilla comparison

Dataset: **Sembiance**, version `v3`. Measured at **2026-09-09T22:11:33.257060+00:00**.

Both configurations use the same Python executable bytes, TrID script, definitions, input files and default resource policy. The accelerated configuration uses a Python virtual environment containing the verified StringZilla library; its startup is included. One warmup and three measured runs per configuration/workload, with shuffled execution order.

Normalized observations agree on **2,400 / 2,400 files**. Quality scores and exact identities are in the [full table](overview.md); any differences are retained in JSON.

Whole-process median milliseconds, with warm OS caches. Speedup is the unaccelerated median divided by the accelerated median.

| Files | TrID without StringZilla | TrID with StringZilla | Speedup | Time change |
|---:|---:|---:|---:|---:|
| 1 | 69.24 | 68.44 | 1.01× | -1.2% |
| 2 | 72.68 | 72.54 | 1.00× | -0.2% |
| 5 | 88.71 | 70.11 | 1.27× | -21.0% |
| 10 | 87.84 | 95.55 | 0.92× | +8.8% |
| 25 | 166.84 | 88.67 | 1.88× | -46.9% |
| 100 | 179.31 | 112.53 | 1.59× | -37.2% |
| 1,000 | 1464.41 | 444.25 | 3.30× | -69.7% |
