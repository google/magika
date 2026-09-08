# BLAKE3 + Rayon (4 threads) versus accelerated SHA-256

Mapped rules on the recorded ARM64 macOS host. Stage medians from ten fresh-process traces on the same one-file rule hit. Inclusive spans overlap; do not add them.

| Stage | ARM SHA-256 ms | BLAKE3 + Rayon (4 threads) ms |
|---|---:|---:|
| main_elapsed_ns | 2.1191 | 2.1193 |
| rule_pack_load_total | 1.1560 | 1.1169 |
| rules_payload_checksum | 0.5516 | 0.4737 |
| rules_cache_identity | 0.0900 | 0.1356 |
| native_scratch_allocate | 0.5381 | 0.5553 |
| native_library_dlopen | 0.3144 | 0.3383 |
| native_scan | 0.0750 | 0.0768 |

Trace-disabled whole-process Hyperfine timings: twenty runs after one warmup, same saved natural workloads, warm OS caches. Ranges expose shared-host variability.

| Files | ARM SHA-256 median ms (min–max) | BLAKE3 + Rayon (4 threads) median ms (min–max) |
|---:|---:|---:|
| 1 | 5.449 (5.141–5.612) | 5.228 (4.681–5.929) |
| 10 | 5.952 (5.731–6.263) | 5.845 (5.436–6.592) |
| 1000 | 40.693 (38.818–56.402) | 38.851 (38.176–40.090) |

Both builds retain full payload/manifest hashing and native validation. They use separate pack versions and cache identities. Timed workloads and all traced classifications are checked against the saved corpus observations. This is a focused hash/startup comparison, not a new full-corpus replay. Measurements include serialized loaders and misses in the raw JSON as well.

[SHA-256 raw traces](sha256/startup/traces.json) · [BLAKE3 raw traces](blake3/startup/traces.json) · [SHA-256 process measurements](sha256/direct/summary.json) · [BLAKE3 process measurements](blake3/direct/summary.json)
