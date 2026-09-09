# Revision timing comparison

Dataset: **Adjudicated combined corpus**, version `snapshot-1411a5c0fd4a`.

Before: `9214192eda842d02c8d50d9af75f7d5a043901f3` at 2026-09-09T03:03:17.576954+00:00. After: `a8429b03b514730254e628a12c2b0cf48591a480` at 2026-09-09T03:31:45.835824+00:00.

Same host, dataset, exact natural input lists, command flags, default resources, Hyperfine, trials and warmups. Protocol 1.2.1 adds workload prediction validation; the 1.2.0 timing recipe is unchanged. CPU warmup and GPU admission are the implementation changes under test. This scoped comparison does not relax the general history compatibility gate.

Each cell is **before → after** median milliseconds, followed by percentage change. Negative changes mean less time. These are recorded observations on a shared host.

| Tool | Version before → after | Config after | 1 files | 2 files | 5 files | 10 files | 25 files | 100 files | 1,000 files |
|---|---|---|---:|---:|---:|---:|---:|---:|---:|
| Magika | 1.1.0 → 1.1.0 | CPU; ML | 57.57 → 63.71 (+10.7%) | 65.67 → 75.42 (+14.8%) | 61.49 → 59.29 (-3.6%) | 63.47 → 64.93 (+2.3%) | 72.25 → 65.46 (-9.4%) | 106.24 → 110.18 (+3.7%) | 493.08 → 539.25 (+9.4%) |
| Magika | 2.0.0-dev @ 9214192e → 2.0.0-dev @ a8429b03 | CPU; ML | 6.79 → 5.53 (-18.5%) | 7.04 → 7.72 (+9.6%) | 12.69 → 12.29 (-3.1%) | 15.87 → 16.03 (+1.0%) | 21.08 → 19.91 (-5.5%) | 44.01 → 46.49 (+5.6%) | 265.38 → 310.06 (+16.8%) |
| Magika | 2.0.0-dev @ 9214192e → 2.0.0-dev @ a8429b03 | CPU; rules + ML | 4.52 → 3.77 (-16.5%) | 7.21 → 7.57 (+5.1%) | 14.42 → 12.02 (-16.6%) | 14.67 → 12.13 (-17.3%) | 18.35 → 16.80 (-8.4%) | 30.72 → 30.79 (+0.2%) | 178.84 → 225.44 (+26.1%) |
| Magika | 2.0.0-dev @ 9214192e → 2.0.0-dev @ a8429b03 | Metal; ML; CPU warmup | 84.36 → 7.51 (-91.1%) | 86.61 → 8.79 (-89.8%) | 88.18 → 13.59 (-84.6%) | 87.71 → 17.93 (-79.6%) | 88.32 → 21.23 (-76.0%) | 89.65 → 45.14 (-49.6%) | 186.21 → 346.01 (+85.8%) |
| Magika | 2.0.0-dev @ 9214192e → 2.0.0-dev @ a8429b03 | Metal; rules + ML; CPU warmup | 6.07 → 3.63 (-40.2%) | 71.44 → 7.01 (-90.2%) | 80.57 → 16.35 (-79.7%) | 88.66 → 13.92 (-84.3%) | 88.74 → 28.51 (-67.9%) | 84.61 → 37.23 (-56.0%) | 134.27 → 250.52 (+86.6%) |
| Magika | 2.0.0-dev @ 9214192e → 2.0.0-dev @ a8429b03 | Auto; ML; CPU warmup | 5.87 → 6.83 (+16.3%) | 75.85 → 8.90 (-88.3%) | 81.81 → 14.50 (-82.3%) | 87.70 → 21.09 (-76.0%) | 91.20 → 24.36 (-73.3%) | 88.46 → 53.29 (-39.8%) | 194.14 → 335.48 (+72.8%) |
| Magika | 2.0.0-dev @ 9214192e → 2.0.0-dev @ a8429b03 | Auto; rules + ML; CPU warmup | 3.78 → 3.52 (-6.9%) | 83.97 → 7.13 (-91.5%) | 87.28 → 13.79 (-84.2%) | 110.39 → 13.46 (-87.8%) | 78.55 → 20.52 (-73.9%) | 93.31 → 38.14 (-59.1%) | 135.84 → 226.02 (+66.4%) |
| Magika | 2.0.0-dev @ 9214192e → 2.0.0-dev @ a8429b03 | CPU; rules-only | 3.35 → 3.21 (-4.3%) | 3.81 → 2.78 (-27.1%) | 3.43 → 3.85 (+12.4%) | 4.08 → 3.44 (-15.8%) | 5.64 → 4.49 (-20.3%) | 9.33 → 9.77 (+4.7%) | 67.09 → 83.14 (+23.9%) |
| libmagic | 5.41 → 5.41 | MIME; serial | 1.63 → 1.38 (-15.4%) | 2.05 → 2.07 (+0.8%) | 4.51 → 4.00 (-11.5%) | 6.09 → 5.62 (-7.7%) | 13.13 → 17.27 (+31.5%) | 35.27 → 36.27 (+2.8%) | 347.32 → 381.77 (+9.9%) |
| TrID | 2.48 → 2.48 | strings=True; StringZilla=5.1.2 | 83.06 → 84.24 (+1.4%) | 104.00 → 113.60 (+9.2%) | 76.64 → 79.07 (+3.2%) | 109.84 → 111.67 (+1.7%) | 100.82 → 105.36 (+4.5%) | 304.64 → 320.12 (+5.1%) | 5156.85 → 5433.15 (+5.4%) |

[Full current accuracy, precision, coverage and rule-match table](overview.md). Both sets of quality scores and all arithmetic above are retained in `comparison.json`.
