# Mapped rules startup diagnostic

Same binary and inputs; mode selected in the process environment, with no env executable wrapper. Twenty measured fresh-process runs after one warmup; warm OS caches. Times include startup and shutdown.

| Path | Files | Median ms | Min ms | Max ms |
|---|---:|---:|---:|---:|
| cli-before-rules | 0 | 3.134 | 2.784 | 3.662 |
| rules-mapped-files-1-hits-natural | 1 | 4.640 | 4.281 | 4.914 |
| rules-serialized-files-1-hits-natural | 1 | 5.728 | 5.312 | 6.381 |
| rules-mapped-files-10-hits-natural | 10 | 5.406 | 4.699 | 5.905 |
| rules-serialized-files-10-hits-natural | 10 | 6.016 | 5.777 | 6.380 |
| rules-mapped-files-1000-hits-natural | 1000 | 38.318 | 37.816 | 39.251 |
| rules-serialized-files-1000-hits-natural | 1000 | 39.005 | 38.557 | 39.582 |

The zero-file diagnostic exits before rules loading; it is not isolated OS loader timing. It is not subtracted from the classification timings. Full corpus parity was established separately.
