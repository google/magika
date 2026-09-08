# Mapped rules startup diagnostic

Same binary and inputs; mode selected in the process environment, with no env executable wrapper. Twenty measured fresh-process runs after one warmup; warm OS caches. Times include startup and shutdown.

| Path | Files | Median ms | Min ms | Max ms |
|---|---:|---:|---:|---:|
| cli-before-rules | 0 | 3.507 | 3.075 | 4.041 |
| rules-mapped-files-1-hits-natural | 1 | 5.228 | 4.681 | 5.929 |
| rules-serialized-files-1-hits-natural | 1 | 6.159 | 5.963 | 6.444 |
| rules-mapped-files-10-hits-natural | 10 | 5.845 | 5.436 | 6.592 |
| rules-serialized-files-10-hits-natural | 10 | 6.791 | 6.462 | 7.233 |
| rules-mapped-files-1000-hits-natural | 1000 | 38.851 | 38.176 | 40.090 |
| rules-serialized-files-1000-hits-natural | 1000 | 39.620 | 39.024 | 40.457 |

The zero-file diagnostic exits before rules loading; it is not isolated OS loader timing. It is not subtracted from the classification timings. Full corpus parity was established separately.
