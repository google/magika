# BLAKE3 versus accelerated SHA-256

Mapped rules on the recorded ARM64 macOS host. Stage medians from ten fresh-process traces on the same one-file rule hit. Inclusive spans overlap; do not add them.

| Stage | ARM SHA-256 ms | BLAKE3 ms |
|---|---:|---:|
| main_elapsed_ns | 2.0241 | 2.3060 |
| rule_pack_load_total | 1.0930 | 1.3658 |
| rules_payload_checksum | 0.5460 | 0.7593 |
| rules_cache_identity | 0.0879 | 0.1350 |
| native_scratch_allocate | 0.5429 | 0.5227 |
| native_library_dlopen | 0.3105 | 0.2967 |
| native_scan | 0.0730 | 0.0674 |

Trace-disabled whole-process Hyperfine timings: twenty runs after one warmup, same saved natural workloads, warm OS caches. Ranges expose shared-host variability.

| Files | ARM SHA-256 median ms (min–max) | BLAKE3 median ms (min–max) |
|---:|---:|---:|
| 1 | 5.526 (5.022–5.820) | 5.507 (5.189–5.785) |
| 10 | 5.894 (5.492–6.312) | 5.938 (5.642–6.201) |
| 1000 | 39.962 (38.593–41.917) | 38.746 (38.022–39.726) |

Both builds retain full payload/manifest hashing and native validation. They use separate pack versions and cache identities. Timed workloads and all traced classifications are checked against the saved corpus observations. This is a focused hash/startup comparison, not a new full-corpus replay. Measurements include serialized loaders and misses in the raw JSON as well.

[SHA-256 raw traces](sha256/startup/traces.json) · [BLAKE3 raw traces](blake3/startup/traces.json) · [SHA-256 process measurements](sha256/direct/summary.json) · [BLAKE3 process measurements](blake3/direct/summary.json)
