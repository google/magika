# Mapped rules startup diagnostic

Same binary and inputs; mode selected in the process environment, with no env executable wrapper. Twenty measured fresh-process runs after one warmup; warm OS caches. Times include startup and shutdown.

| Path | Files | Median ms | Min ms | Max ms |
|---|---:|---:|---:|---:|
| cli-before-rules | 0 | 2.402 | 2.224 | 2.970 |
| rules-mapped-files-1-hits-natural | 1 | 5.662 | 5.081 | 7.704 |
| rules-serialized-files-1-hits-natural | 1 | 5.834 | 5.193 | 7.658 |
| rules-mapped-files-10-hits-natural | 10 | 7.020 | 6.044 | 9.403 |
| rules-serialized-files-10-hits-natural | 10 | 5.428 | 5.263 | 5.961 |
| rules-mapped-files-1000-hits-natural | 1000 | 36.992 | 36.468 | 40.128 |
| rules-serialized-files-1000-hits-natural | 1000 | 42.502 | 37.540 | 46.375 |

The zero-file diagnostic exits before rules loading; it is not isolated OS loader timing. It is not subtracted from the classification timings. Full corpus parity was established separately.
