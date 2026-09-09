# Handoff decisions

The [measurements](measurements.md) and raw JSON are the evidence for these decisions.
The [original handoff](../../handoffs/perf-handoff-2026-09-08.md) is unchanged.

| Recommendation | Disposition | Evidence and limit |
|---|---|---|
| Cap known small-file batches | Accepted for explicit CPU runs with two or three paths | The earlier broad cap regressed five-file runs and was narrowed. Both experiments remain in the catalogue. |
| ARM hardware CRC32C | Accepted for supported ARM targets | Scratch allocation fell from 579 to 189 µs; 65,696 oracle cases and three mapped-database regressions pass. The 100 µs target was **not** reached. Other targets retain their existing instruction baseline. |
| Fuse convolution for small model batches | Accepted | Batch-one inference exceeded 1,000 files/s; measured probe scores were bit-identical. All 25 runtime regressions and all 25,421 normalized corpus decisions pass. Weights and GPU graphs are byte-for-byte unchanged. The initial and confirmation whole-process timings are both retained; this is not a claim of a sustained-throughput improvement. |
| Exit immediately after output | Rejected | Session destruction took 0.46 µs and runtime destruction 17.04 µs. They execute in inference workers before the main thread finishes joining them. An early process exit is not justified by that measured cost. |
| Ship pre-packed binary model data now | Deferred; current format retained | Fresh-process preparation still takes 1.95 ms, so the 800 µs target remains open. The warm component profile puts JSON parsing/validation around 0.16 ms. Packing is selected from the host's tract kernel formats, and the x86 eager path also reorders kernel weights. A portable binary-JSON substitution alone has not demonstrated the requested improvement; no unvalidated host-specific representation was shipped. |
| Reduce the macOS thread default | Rejected | Four threads reduced user CPU time but increased 1,000-file wall time from 268 to 366 ms. The default retains the faster policy; callers can request fewer threads. |

The corpus comparison used the named **adjudicated combined snapshot**, with 25,421 files.
It does not relabel that snapshot as the full V56 release or Sembiance. Their earlier
independent, versioned reports remain in the [catalogue](../../CATALOG.md).

To reproduce the component fusion experiment, use the preserved
`unfused-model.graph.json` as `MAGIKA_PROFILE_BASELINE_GRAPH`, then run:

```sh
cargo test --release --manifest-path rust/tract-runtime/Cargo.toml --no-default-features \
  profile_small_batch_preparation_and_fusion -- --ignored --nocapture --test-threads=1
```

The runtime preparation probe is `prepare.py LIBRARY`, run in a fresh process for
each observation. The whole-process JSON records exact Hyperfine commands and
file lists. Run timings on an idle host without concurrent compilation.
