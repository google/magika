# Magika tract runtime

This crate is the inference layer shared by the Rust `magika` library, CLI, and runtime benchmark.
It loads the checked NNEF release artifact, binds fixed batch classes `1, 4, 8, 16, 32, 64`, and
prepares their target-specific tract plans once. Each inference thread then spawns private mutable
state from those shared plans. When a CPU caller declares a maximum batch, only its largest
reachable plan is prepared; smaller requests pad through that plan and discard padding outputs.
This keeps partial batches from allocating additional unfused execution states.

The public device choice is intentionally generic: automatic, CPU, or GPU. On macOS the compiled
GPU implementation is Metal. CUDA can be compiled on supported systems with the `cuda` feature.
Callers do not select Metal or CUDA directly; the resolved implementation is available through
`BackendInfo` for verbose diagnostics.

Inference is synchronous. Async file reading and batch accumulation belong above this crate, so
CPU- or GPU-bound model execution never occupies an async executor thread.

GPU preparation retains the model's original normalization variance expression,
`max(E[x*x] - E[x]*E[x], 0)`. Replacing it with the mean squared centered input changes
floating-point behavior, especially for nearly constant activations. CPU and GPU kernels can
still accumulate in different orders. The startup probe checks every row of every resident GPU batch plan using a stored CPU
reference for one repeated input;
it does not establish score agreement for every file or batch class.

Confidence scores can vary with the backend, batch class and position within a batch because
floating-point reductions can run in different orders. Regression qualification checks final
classification and overwrite decisions across varied reference files and every batch class.

The existing `MAGIKA_DIRECT_TILE_COLUMNS` and `MAGIKA_DIRECT_TILE_BATCHES` environment variables
control CPU convolution tiling when plans are prepared. Both accept positive integers, clamped
to the graph's columns or batch count. `TILE_COLUMNS` takes precedence for the column tile;
setting `TILE_BATCHES` selects batch-based tiling instead of the x86 default. They can change
kernel selection, speed and floating-point rounding. See the [existing benchmark](../tract-bench/README.md#retuning-the-x86_64-convolution-tile)
for measuring a tuning change on the target machine. These controls do not change GPU tiling.

GPU convolution expands an intermediate tensor to `[batch, 508, 1280]` f32 values: 83,230,720
bytes at batch 32 and 166,461,440 bytes at batch 64. Those are individual tensor sizes, not
measurements of peak memory; weights, other intermediates and concurrently active session
states add to memory use. `with_max_batch` limits the prepared GPU batch classes, and session
execution states are created when a class first runs. CPU plans explicitly use tract's
single-thread executor; GPU plans use the GPU runtime's executor settings. In an embedding
application, changing tract's global executor can affect CPU operations within GPU plans.
