# Revision timing comparison

Dataset: **Sembiance**, version `v3`.

Before: `9214192eda842d02c8d50d9af75f7d5a043901f3` at 2026-09-09T03:08:57.821771+00:00. After: `65505ce38363b63d2c07d1370168a95451cf339a` at 2026-09-09T03:42:18.100538+00:00.

Same host, dataset, exact natural input lists, command flags, default resources, Hyperfine, trials and warmups. Protocol 1.2.1 adds workload prediction validation; the 1.2.0 timing recipe is unchanged. CPU warmup and GPU admission are the implementation changes under test. This scoped comparison does not relax the general history compatibility gate.

Each cell is **before → after** median milliseconds, followed by percentage change. Negative changes mean less time. These are recorded observations on a shared host.

| Tool | Version before → after | Config after | 1 files | 2 files | 5 files | 10 files | 25 files | 100 files | 1,000 files |
|---|---|---|---:|---:|---:|---:|---:|---:|---:|
| Magika | 1.1.0 → 1.1.0 | CPU; ML | 60.71 → 52.79 (-13.0%) | 70.39 → 53.23 (-24.4%) | 59.13 → 54.49 (-7.8%) | 60.79 → 56.13 (-7.7%) | 99.65 → 58.45 (-41.3%) | 146.30 → 90.90 (-37.9%) | 1051.82 → 416.06 (-60.4%) |
| Magika | 2.0.0-dev @ 9214192e → 2.0.0-dev @ 65505ce3 | CPU; ML | 6.28 → 5.43 (-13.5%) | 7.27 → 6.00 (-17.5%) | 17.46 → 12.00 (-31.3%) | 14.61 → 12.95 (-11.4%) | 20.93 → 18.40 (-12.1%) | 41.96 → 38.05 (-9.3%) | 277.03 → 252.80 (-8.7%) |
| Magika | 2.0.0-dev @ 9214192e → 2.0.0-dev @ 65505ce3 | CPU; rules + ML | 3.17 → 2.64 (-16.5%) | 11.73 → 5.99 (-48.9%) | 13.41 → 11.69 (-12.8%) | 12.84 → 11.61 (-9.6%) | 29.68 → 15.78 (-46.8%) | 36.70 → 31.57 (-14.0%) | 221.60 → 199.86 (-9.8%) |
| Magika | 2.0.0-dev @ 9214192e → 2.0.0-dev @ 65505ce3 | Metal; ML; CPU warmup | 79.55 → 6.19 (-92.2%) | 103.24 → 5.97 (-94.2%) | 80.02 → 12.39 (-84.5%) | 103.44 → 14.36 (-86.1%) | 108.27 → 20.44 (-81.1%) | 96.64 → 37.85 (-60.8%) | 210.56 → 163.18 (-22.5%) |
| Magika | 2.0.0-dev @ 9214192e → 2.0.0-dev @ 65505ce3 | Metal; rules + ML; CPU warmup | 4.79 → 2.69 (-43.7%) | 88.69 → 6.53 (-92.6%) | 84.18 → 12.16 (-85.6%) | 86.09 → 12.50 (-85.5%) | 89.12 → 16.55 (-81.4%) | 106.79 → 31.18 (-70.8%) | 178.92 → 139.44 (-22.1%) |
| Magika | 2.0.0-dev @ 9214192e → 2.0.0-dev @ 65505ce3 | Auto; ML; CPU warmup | 8.05 → 4.89 (-39.3%) | 87.16 → 6.34 (-92.7%) | 86.96 → 12.01 (-86.2%) | 80.45 → 14.12 (-82.5%) | 86.16 → 20.24 (-76.5%) | 116.58 → 38.33 (-67.1%) | 186.10 → 178.22 (-4.2%) |
| Magika | 2.0.0-dev @ 9214192e → 2.0.0-dev @ 65505ce3 | Auto; rules + ML; CPU warmup | 3.68 → 2.80 (-24.0%) | 88.64 → 6.23 (-93.0%) | 93.41 → 12.17 (-87.0%) | 81.69 → 12.53 (-84.7%) | 88.75 → 17.78 (-80.0%) | 117.99 → 31.23 (-73.5%) | 172.30 → 140.41 (-18.5%) |
| Magika | 2.0.0-dev @ 9214192e → 2.0.0-dev @ 65505ce3 | CPU; rules-only | 3.20 → 2.47 (-22.7%) | 3.46 → 2.76 (-20.1%) | 6.49 → 2.74 (-57.8%) | 6.11 → 3.21 (-47.5%) | 4.67 → 4.05 (-13.3%) | 8.99 → 7.80 (-13.3%) | 67.93 → 53.64 (-21.0%) |
| libmagic | 5.41 → 5.41 | MIME; serial | 1.60 → 1.10 (-31.0%) | 10.36 → 7.73 (-25.4%) | 9.12 → 7.94 (-13.0%) | 16.41 → 10.98 (-33.1%) | 9.99 → 8.24 (-17.6%) | 77.63 → 78.83 (+1.5%) | 626.60 → 630.30 (+0.6%) |
| TrID | 2.48 → 2.48 | strings=True; StringZilla=off | 95.11 → 66.85 (-29.7%) | 75.13 → 66.33 (-11.7%) | 75.03 → 68.20 (-9.1%) | 99.48 → 83.65 (-15.9%) | 186.17 → 173.63 (-6.7%) | 197.24 → 181.24 (-8.1%) | 1685.32 → 1286.28 (-23.7%) |

[Full current accuracy, precision, coverage and rule-match table](overview.md). Both sets of quality scores and all arithmetic above are retained in `comparison.json`.
