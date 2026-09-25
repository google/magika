# Magika tract runtime

This crate is the inference layer shared by the Rust `magika` library, CLI, and runtime benchmark.
It loads the release model for fixed batch classes `1, 4, 8, 16, 32, 64` and prepares their
target-specific tract plans once. Each inference thread then spawns private mutable state from those
shared plans.

The model is embedded as the graph that parsing the checked NNEF archive produces
(`models/model.graph.json` and `models/model.weights`), which loads in about 3 ms instead of the
14 ms parsing takes. `rust/sync.sh` writes both with `tract-bench`'s `convert-model`, and `cargo test` checks that they agree.

The public device choice is intentionally generic: automatic, CPU, or GPU. On macOS the compiled
GPU implementation is Metal. CUDA can be compiled on supported systems with the `cuda` feature.
Callers do not select Metal or CUDA directly; the resolved implementation is available through
`BackendInfo` for verbose diagnostics.

Inference is synchronous. Async file reading and batch accumulation belong above this crate, so
CPU- or GPU-bound model execution never occupies an async executor thread.

At startup on a GPU, every resident batch plan runs repeated copies of one input and every
output row is checked against a stored CPU reference. The probe is a device health check: it
does not establish score agreement for every file.

CPU and GPU confidence scores are backend dependent. GPU release qualification checks final
classification and overwrite decisions across varied reference files and every batch class.
