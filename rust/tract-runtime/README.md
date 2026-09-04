# Magika tract runtime

This crate is the inference layer shared by the Rust `magika` library, CLI, and runtime benchmark.
It loads the checked NNEF release artifact, binds fixed batch classes `1, 4, 8, 16, 32, 64`, and
prepares their target-specific tract plans once. Each inference thread then spawns private mutable
state from those shared plans.

The public device choice is intentionally generic: automatic, CPU, or GPU. On macOS the compiled
GPU implementation is Metal. CUDA can be compiled on supported systems with the `cuda` feature.
Callers do not select Metal or CUDA directly; the resolved implementation is available through
`BackendInfo` for verbose diagnostics.

Inference is synchronous. Async file reading and batch accumulation belong above this crate, so
CPU- or GPU-bound model execution never occupies an async executor thread.
