# Preserved startup experiments

The startup spike's source, measurements and diagnostic logs are preserved in
[the archive](../startup-spikes-2026-09-08.tar.gz), with a
[per-file receipt](../startup-spikes-2026-09-08.json). Its 363 files are historical
evidence from their recorded revisions; they are not the current build workflow.

Extract into an empty directory with `tar -xzf startup-spikes-2026-09-08.tar.gz`.
The receipt verifies every extracted file; internal relative paths are preserved.
Current builds use [runtime packaging](../../../../rust/runtime/README.md), and
current measurements remain in the [benchmark catalogue](../../CATALOG.md).
The [performance handoff](../../handoffs/perf-handoff-2026-09-08.md) is preserved
unchanged; its remaining improvement targets are tracked separately.
