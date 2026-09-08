# Mapped rules startup diagnostic

Same binary and inputs; mode selected in the process environment, with no env executable wrapper. Twenty measured fresh-process runs after one warmup; warm OS caches. Times include startup and shutdown.

| Path | Files | Median ms | Min ms | Max ms |
|---|---:|---:|---:|---:|
| cli-before-rules | 0 | 2.934 | 2.761 | 3.536 |
| rules-mapped-files-1-hits-natural | 1 | 5.526 | 5.022 | 5.820 |
| rules-serialized-files-1-hits-natural | 1 | 6.262 | 6.002 | 6.875 |
| rules-mapped-files-10-hits-natural | 10 | 5.894 | 5.492 | 6.312 |
| rules-serialized-files-10-hits-natural | 10 | 6.671 | 6.394 | 7.026 |
| rules-mapped-files-1000-hits-natural | 1000 | 39.962 | 38.593 | 41.917 |
| rules-serialized-files-1000-hits-natural | 1000 | 39.264 | 38.787 | 40.328 |

The zero-file diagnostic exits before rules loading; it is not isolated OS loader timing. It is not subtracted from the classification timings. Full corpus parity was established separately.
