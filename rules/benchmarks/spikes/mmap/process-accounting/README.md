# Account for the full CLI process

[Generated accounting table](results/report.md) reconciles the same Hyperfine run's
parent start/end and Magika main-entry/main-exit timestamps. Each of the twenty
traced iterations sums exactly; the displayed arithmetic means therefore sum as
well. Parent wall-clock intervals also agree with Hyperfine's exported monotonic
elapsed times within 0.1 ms. Both traced/untraced modes emit identical decisions.

The earlier 4.640 ms result is an untraced whole-process median. The earlier internal
spans came from separate Python-launched traced runs and could not account for that
whole number. This measurement finds roughly 3.09 ms before main, 1.51 ms inside it
and 0.65 ms for trace emission/exit/wakeup: 5.24 ms total traced mean. The untraced
median is 4.55 ms and the traced median is 5.26 ms. Do not assign the traced overhead
to ordinary CLI work or subtract unrelated medians to invent isolated loader time.

Before-main includes process creation, dynamic loading, runtime initialization and
scheduling. It is not pure dyld time. The unchanged 22 MiB binary links Metal,
CoreGraphics and CoreFoundation even for rules-only execution; their actual marginal
cost is not isolated here. Splitting ML/framework dependencies out of a rules-only
executable is a candidate for a subsequent measurement, not an established saving.
`DYLD_PRINT_STATISTICS=1` emitted no loader statistics in the exploratory run.

The small patch adds two wall-clock reads around Hyperfine's existing timer and
emits its JSON only after timing stops. Child tracing is measured both on and off.
The standalone exploratory launchers did not match Hyperfine timings; they are
retained separately and are not used in the final table.

Reproduce with Hyperfine tag v1.20.0:

```sh
git clone --depth 1 --branch v1.20.0 https://github.com/sharkdp/hyperfine.git HYPERFINE_SOURCE
git -C HYPERFINE_SOURCE apply /absolute/path/to/hyperfine-boundaries.patch
cargo build --release --locked --manifest-path HYPERFINE_SOURCE/Cargo.toml
python measure.py HYPERFINE_SOURCE/target/release/hyperfine \
  ../checksum-removal/none/direct/summary.json ORIGINAL_WORKING_RUN NEW_RESULTS
```

The reference command and environment are saved in the summary. Original working
run inputs are the same one-file natural workload used in the earlier benchmarks.
Only this focused startup workload is rerun. This is process accounting, not a new
accuracy or corpus evaluation.

Upstream source: [Hyperfine timer](https://github.com/sharkdp/hyperfine/blob/v1.20.0/src/timer/mod.rs)
and [raw executor](https://github.com/sharkdp/hyperfine/blob/v1.20.0/src/benchmark/executor.rs).
