# Performance handoff

The [original handoff](perf-handoff-2026-09-08.md) and its
[Auto-mode addendum](perf-handoff-2026-09-08-with-auto-addendum.md) are preserved.
Current measurements are in the [benchmark catalogue](../CATALOG.md).

Accepted changes: CPU batching for two/three files, small-batch convolution
fusion, ARM CRC32C, and CPU inference during asynchronous GPU preparation.
Measurements did not justify early process exit or fewer default CPU workers.
Pre-packed model data remains deferred; the 800 µs preparation and 100 µs scratch
allocation targets were not reached.

The [component measurements and decisions](https://github.com/google/magika/tree/5bc2f250ce67153589c00bb546d0ad9ebfc4fc77/rules/benchmarks/reports/2026-09-09-handoff-validation),
[spike archive](https://github.com/google/magika/tree/5bc2f250ce67153589c00bb546d0ad9ebfc4fc77/rules/benchmarks/spikes),
and [superseded CPU/GPU candidate](https://github.com/google/magika/tree/5bc2f250ce67153589c00bb546d0ad9ebfc4fc77/rules/benchmarks/reports/2026-09-09-a8429b03-cpu-first-combined)
remain in Git history. Temporary build logs and experiment programs are not part
of the maintained implementation.
