# Mapped rules startup diagnostic

Same binary and inputs; mode selected in the process environment, with no env executable wrapper. Twenty measured fresh-process runs after one warmup; warm OS caches. Times include startup and shutdown.

| Path | Files | Median ms | Min ms | Max ms |
|---|---:|---:|---:|---:|
| cli-before-rules | 0 | 2.272 | 2.170 | 2.638 |
| rules-mapped-files-1-hits-natural | 1 | 6.553 | 6.341 | 6.965 |
| rules-serialized-files-1-hits-natural | 1 | 7.367 | 7.163 | 7.856 |
| rules-mapped-files-10-hits-natural | 10 | 7.060 | 6.852 | 7.424 |
| rules-serialized-files-10-hits-natural | 10 | 7.834 | 7.634 | 8.236 |
| rules-mapped-files-1000-hits-natural | 1000 | 39.247 | 38.623 | 40.245 |
| rules-serialized-files-1000-hits-natural | 1000 | 40.102 | 39.761 | 40.826 |

The zero-file diagnostic exits before rules loading; it is not isolated OS loader timing. It is not subtracted from the classification timings. Full corpus parity was established separately.
