# Revision timing comparison

Dataset: **Adjudicated combined corpus**, version `snapshot-1411a5c0fd4a`.

Before: `9214192eda842d02c8d50d9af75f7d5a043901f3` at 2026-09-09T03:03:17.576954+00:00. After: `65505ce38363b63d2c07d1370168a95451cf339a` at 2026-09-09T03:39:30.163738+00:00.

Same host, dataset, exact natural input lists, command flags, default resources, Hyperfine, trials and warmups. Protocol 1.2.1 adds workload prediction validation; the 1.2.0 timing recipe is unchanged. CPU warmup and GPU admission are the implementation changes under test. This scoped comparison does not relax the general history compatibility gate.

Each cell is **before → after** median milliseconds, followed by percentage change. Negative changes mean less time. These are recorded observations on a shared host.

| Tool | Version before → after | Config after | 1 files | 2 files | 5 files | 10 files | 25 files | 100 files | 1,000 files |
|---|---|---|---:|---:|---:|---:|---:|---:|---:|
| Magika | 1.1.0 → 1.1.0 | CPU; ML | 57.57 → 53.78 (-6.6%) | 65.67 → 54.16 (-17.5%) | 61.49 → 54.53 (-11.3%) | 63.47 → 55.39 (-12.7%) | 72.25 → 59.62 (-17.5%) | 106.24 → 93.17 (-12.3%) | 493.08 → 410.84 (-16.7%) |
| Magika | 2.0.0-dev @ 9214192e → 2.0.0-dev @ 65505ce3 | CPU; ML | 6.79 → 5.04 (-25.7%) | 7.04 → 6.22 (-11.7%) | 12.69 → 11.75 (-7.4%) | 15.87 → 13.14 (-17.2%) | 21.08 → 19.27 (-8.6%) | 44.01 → 36.18 (-17.8%) | 265.38 → 256.56 (-3.3%) |
| Magika | 2.0.0-dev @ 9214192e → 2.0.0-dev @ 65505ce3 | CPU; rules + ML | 4.52 → 2.92 (-35.4%) | 7.21 → 5.97 (-17.2%) | 14.42 → 11.91 (-17.4%) | 14.67 → 12.35 (-15.8%) | 18.35 → 16.07 (-12.4%) | 30.72 → 28.13 (-8.4%) | 178.84 → 164.70 (-7.9%) |
| Magika | 2.0.0-dev @ 9214192e → 2.0.0-dev @ 65505ce3 | Metal; ML; CPU warmup | 84.36 → 6.13 (-92.7%) | 86.61 → 6.18 (-92.9%) | 88.18 → 12.40 (-85.9%) | 87.71 → 15.11 (-82.8%) | 88.32 → 20.41 (-76.9%) | 89.65 → 37.76 (-57.9%) | 186.21 → 169.54 (-8.9%) |
| Magika | 2.0.0-dev @ 9214192e → 2.0.0-dev @ 65505ce3 | Metal; rules + ML; CPU warmup | 6.07 → 2.81 (-53.6%) | 71.44 → 6.21 (-91.3%) | 80.57 → 12.16 (-84.9%) | 88.66 → 12.53 (-85.9%) | 88.74 → 17.30 (-80.5%) | 84.61 → 29.86 (-64.7%) | 134.27 → 122.43 (-8.8%) |
| Magika | 2.0.0-dev @ 9214192e → 2.0.0-dev @ 65505ce3 | Auto; ML; CPU warmup | 5.87 → 5.18 (-11.7%) | 75.85 → 6.10 (-92.0%) | 81.81 → 12.41 (-84.8%) | 87.70 → 14.67 (-83.3%) | 91.20 → 20.16 (-77.9%) | 88.46 → 38.46 (-56.5%) | 194.14 → 163.87 (-15.6%) |
| Magika | 2.0.0-dev @ 9214192e → 2.0.0-dev @ 65505ce3 | Auto; rules + ML; CPU warmup | 3.78 → 2.77 (-26.7%) | 83.97 → 6.27 (-92.5%) | 87.28 → 12.29 (-85.9%) | 110.39 → 12.58 (-88.6%) | 78.55 → 17.46 (-77.8%) | 93.31 → 28.61 (-69.3%) | 135.84 → 128.80 (-5.2%) |
| Magika | 2.0.0-dev @ 9214192e → 2.0.0-dev @ 65505ce3 | CPU; rules-only | 3.35 → 2.67 (-20.3%) | 3.81 → 2.73 (-28.3%) | 3.43 → 2.82 (-17.6%) | 4.08 → 3.15 (-22.8%) | 5.64 → 4.10 (-27.2%) | 9.33 → 9.00 (-3.5%) | 67.09 → 64.83 (-3.4%) |
| libmagic | 5.41 → 5.41 | MIME; serial | 1.63 → 1.26 (-22.9%) | 2.05 → 1.53 (-25.5%) | 4.51 → 3.87 (-14.2%) | 6.09 → 4.93 (-19.0%) | 13.13 → 11.68 (-11.0%) | 35.27 → 33.35 (-5.5%) | 347.32 → 336.20 (-3.2%) |
| TrID | 2.48 → 2.48 | strings=True; StringZilla=5.1.2 | 83.06 → 70.52 (-15.1%) | 104.00 → 96.69 (-7.0%) | 76.64 → 71.43 (-6.8%) | 109.84 → 97.90 (-10.9%) | 100.82 → 81.54 (-19.1%) | 304.64 → 272.84 (-10.4%) | 5156.85 → 4850.05 (-5.9%) |

[Full current accuracy, precision, coverage and rule-match table](overview.md). Both sets of quality scores and all arithmetic above are retained in `comparison.json`.
