# Mapped rules startup diagnostic

Same binary and inputs; mode selected in the process environment, with no env executable wrapper. Twenty measured fresh-process runs after one warmup; warm OS caches. Times include startup and shutdown.

| Path | Files | Median ms | Min ms | Max ms |
|---|---:|---:|---:|---:|
| cli-before-rules | 0 | 3.201 | 2.725 | 3.664 |
| rules-mapped-files-1-hits-natural | 1 | 5.449 | 5.141 | 5.612 |
| rules-serialized-files-1-hits-natural | 1 | 6.360 | 6.110 | 7.329 |
| rules-mapped-files-10-hits-natural | 10 | 5.952 | 5.731 | 6.263 |
| rules-serialized-files-10-hits-natural | 10 | 7.132 | 6.857 | 7.590 |
| rules-mapped-files-1000-hits-natural | 1000 | 40.693 | 38.818 | 56.402 |
| rules-serialized-files-1000-hits-natural | 1000 | 41.910 | 39.352 | 45.817 |

The zero-file diagnostic exits before rules loading; it is not isolated OS loader timing. It is not subtracted from the classification timings. Full corpus parity was established separately.
