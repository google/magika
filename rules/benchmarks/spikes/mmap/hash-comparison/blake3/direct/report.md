# Mapped rules startup diagnostic

Same binary and inputs; mode selected in the process environment, with no env executable wrapper. Twenty measured fresh-process runs after one warmup; warm OS caches. Times include startup and shutdown.

| Path | Files | Median ms | Min ms | Max ms |
|---|---:|---:|---:|---:|
| cli-before-rules | 0 | 3.351 | 2.862 | 3.861 |
| rules-mapped-files-1-hits-natural | 1 | 5.507 | 5.189 | 5.785 |
| rules-serialized-files-1-hits-natural | 1 | 6.614 | 6.225 | 7.076 |
| rules-mapped-files-10-hits-natural | 10 | 5.938 | 5.642 | 6.201 |
| rules-serialized-files-10-hits-natural | 10 | 7.328 | 6.741 | 7.747 |
| rules-mapped-files-1000-hits-natural | 1000 | 38.746 | 38.022 | 39.726 |
| rules-serialized-files-1000-hits-natural | 1000 | 39.726 | 39.449 | 40.895 |

The zero-file diagnostic exits before rules loading; it is not isolated OS loader timing. It is not subtracted from the classification timings. Full corpus parity was established separately.
