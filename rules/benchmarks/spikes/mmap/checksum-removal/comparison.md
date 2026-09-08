# Startup after removing the outer payload checksum

Mapped rules, same saved inputs and native image. Ten fresh-process traces per build/hit/miss case. Inclusive stage medians overlap; do not sum them. The removed checksum event is absent from every new trace. Source/compiler cache keys still hash their inputs; structural and native CRC checks remain.

| Stage (one rule hit) | With ARM SHA-256 ms | No outer checksum ms |
|---|---:|---:|
| main_elapsed_ns | 2.0440 | 1.5223 |
| rule_pack_load_total | 1.0994 | 0.5244 |
| rules_payload_checksum | 0.5506 | absent |
| rules_cache_identity | 0.0898 | 0.0861 |
| native_scratch_allocate | 0.5326 | 0.5924 |
| native_library_dlopen | 0.3004 | 0.2787 |

Trace-disabled Hyperfine: twenty runs after one warmup, warm OS caches, same saved natural file sets. Whole-process timings include startup and shutdown.

| Files | With checksum median ms (min–max) | Without checksum median ms (min–max) |
|---:|---:|---:|
| 1 | 5.306 (4.977–5.654) | 4.640 (4.281–4.914) |
| 10 | 5.677 (5.339–6.177) | 5.406 (4.699–5.905) |
| 1000 | 39.095 (38.279–40.116) | 38.318 (37.816–39.251) |

All traced classifications and each timed workload match the saved full-corpus observations. This focused loader change does not claim a new full-corpus replay. Raw JSON also includes serialized mode, misses and individual timing ranges.
