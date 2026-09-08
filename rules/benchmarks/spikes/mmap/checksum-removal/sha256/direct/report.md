# Mapped rules startup diagnostic

Same binary and inputs; mode selected in the process environment, with no env executable wrapper. Twenty measured fresh-process runs after one warmup; warm OS caches. Times include startup and shutdown.

| Path | Files | Median ms | Min ms | Max ms |
|---|---:|---:|---:|---:|
| cli-before-rules | 0 | 3.065 | 2.711 | 3.717 |
| rules-mapped-files-1-hits-natural | 1 | 5.306 | 4.977 | 5.654 |
| rules-serialized-files-1-hits-natural | 1 | 6.165 | 5.908 | 6.459 |
| rules-mapped-files-10-hits-natural | 10 | 5.677 | 5.339 | 6.177 |
| rules-serialized-files-10-hits-natural | 10 | 6.640 | 6.342 | 6.992 |
| rules-mapped-files-1000-hits-natural | 1000 | 39.095 | 38.279 | 40.116 |
| rules-serialized-files-1000-hits-natural | 1000 | 40.124 | 39.103 | 44.290 |

The zero-file diagnostic exits before rules loading; it is not isolated OS loader timing. It is not subtracted from the classification timings. Full corpus parity was established separately.
