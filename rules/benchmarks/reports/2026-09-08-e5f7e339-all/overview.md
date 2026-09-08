# Magika benchmark — e5f7e339

Fresh accuracy evaluation on 25,421 files. All eight modes rerun on the same 30 saved workloads; 3 measured runs after 1 warmup. Times are median milliseconds for natural file mixes, including startup and shutdown; warm OS caches.

| Tool / mode | Accuracy | Precision | Coverage | Rule matches | 1 file | 10 files | 100 files | 1,000 files |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Magika 1 CPU | 57.73% | 70.23% | 82.20% | — | 18.45 | 35.10 | 199.78 | 1,851.52 |
| Magika 2 CPU ML | 57.73% | 70.23% | 82.20% | — | 5.98 | 13.34 | 63.44 | 590.38 |
| Magika 2 CPU rules + ML | 78.93% | 84.39% | 93.52% | 37.86% | 3.74 | 11.85 | 43.65 | 376.89 |
| Magika 2 GPU ML | 57.73% | 70.22% | 82.21% | — | 56.87 | 68.44 | 87.63 | 203.52 |
| Magika 2 GPU rules + ML | 78.93% | 84.38% | 93.53% | 37.86% | 5.36 | 73.02 | 83.80 | 159.18 |
| Magika 2 rules-only CPU | 37.86% | 100.00% | 37.86% | 37.86% | 2.84 | 3.14 | 6.04 | 34.92 |
| libmagic | 25.46% | 93.24% | 27.31% | — | 1.39 | 5.16 | 32.52 | 323.31 |
| TrID | 55.25% | 78.32% | 70.55% | — | 70.05 | 101.95 | 284.60 | 4,846.14 |

## Elapsed-time reduction versus the previous published table

Positive percentages mean less time; negative percentages mean more time. These historical differences include all intervening changes and host variation. They are not an isolated measurement of asynchronous loading. The original measurements remain unchanged.

| Tool / mode | 1 file | 10 files | 100 files | 1,000 files |
|---|---:|---:|---:|---:|
| Magika 1 CPU | +7.9% | +6.5% | +8.6% | +6.4% |
| Magika 2 CPU ML | +26.3% | +21.9% | +11.4% | +4.7% |
| Magika 2 CPU rules + ML | +64.4% | +35.6% | +12.9% | +5.2% |
| Magika 2 GPU ML | +19.3% | +13.7% | +9.9% | +10.1% |
| Magika 2 GPU rules + ML | +48.8% | +13.2% | +4.1% | +6.8% |
| Magika 2 rules-only CPU | +62.8% | +68.4% | +48.8% | +27.8% |
| libmagic | +17.4% | +14.8% | +6.7% | +17.8% |
| TrID | +13.6% | +9.6% | +5.0% | +5.7% |

Full precision, raw ranges, historical ratios, quality-count changes and compatibility checks are stored in `overview.json`. The strict historical compatibility check flags the changed reference/configuration and loader environment; the overview reports historical arithmetic rather than a controlled causal speedup. The corpus, labels, mapping and ordered file lists are verified identical. Magika 2 uses the deferred CPU/Metal distribution and the mapped rule pack. GPU means Metal; rules-only runs CPU signatures without model initialization.
