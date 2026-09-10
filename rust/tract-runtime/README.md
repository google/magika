# Magika tract runtime

This crate loads the release-generated graphs and shared weights for batch
classes `1, 4, 8, 16, 32, 64`. Each inference thread owns mutable execution state.
For a CPU caller with a maximum batch, the largest reachable plan is prepared;
smaller requests pad through that plan and discard the padding outputs.

Regenerate and qualify model artifacts with the [model release tools](../tract-bench/README.md#model-release).
CPU convolution fusion applies to every batch class. Exporting a graph alone
is not qualification: the tests compare scores and decisions with the source model.

`Cpu` skips GPU initialization. `Gpu` requires successful GPU preparation and its
correctness probe. This crate's `Auto` prepares GPU synchronously and falls back
to CPU on failure. The [CLI](../cli/README.md) implements asynchronous preparation
and CPU warmup above this layer. Metal is the macOS GPU implementation; CUDA is
available through its Cargo feature on supported targets.

CUDA compilation is checked by CI, but hardware execution is not qualified.
Windows CUDA remains a release hold because its dependency DLL loading also
requires qualification. Explicit CPU avoids those loads.

GPU normalization retains `max(E[x*x] - E[x]*E[x], 0)`. Floating-point reductions
can differ by backend, batch size and row position; regression tests check final
classification and confidence-threshold decisions. The startup probe covers its
reference input, not every possible file.

Use the [runtime benchmark](../tract-bench/README.md#runtime-benchmark) for compute
measurements and the [cross-tool benchmark](../../rules/benchmarks/README.md) for
whole-process CLI timing. CPU tile controls are documented with the
[x86 tuning procedure](../tract-bench/README.md#retuning-the-x86_64-convolution-tile).
