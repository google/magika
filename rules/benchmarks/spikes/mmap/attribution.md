# Startup attribution

Mapped loader, one signature hit, ten traces per build. Inclusive stage medians in milliseconds. Nested intervals overlap; do not sum this table. The accelerated build enables sha2 0.10.9 asm support on ARM; cache/image validation is retained.

| Stage | Software SHA | Accelerated SHA |
|---|---:|---:|
| main_elapsed_ns | 4.3141 | 1.7880 |
| rule_pack_load_total | 3.4839 | 0.9454 |
| rules_payload_checksum | 2.6700 | 0.5190 |
| rules_cache_identity | 0.4383 | 0.0844 |
| native_scratch_allocate | 0.5162 | 0.5085 |
| native_library_dlopen | 0.2364 | 0.2275 |
| pipeline_thread_launch | 0.0306 | 0.0298 |
| native_scan | 0.0625 | 0.0613 |
| output_format_and_write | 0.0217 | 0.0225 |

Main elapsed time excludes launch-to-main and process teardown. Parent-observed boundaries are retained in the raw traces; they include launcher/scheduler overhead. Trace JSON emission is outside main elapsed time. These spans are diagnostic, not replacements for Hyperfine measurements.

[Software SHA direct-execution timings](direct-exec/report.md) · [Accelerated SHA direct-execution timings](direct-exec-accelerated/report.md)

The later direct-execution run has wider timing ranges on the shared host. Retain its raw ranges rather than treating every end-to-end delta as a stable speedup.
