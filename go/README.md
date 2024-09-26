# Go library

This directory contains the Go library for Magika.

The inference relies on the [ONNX Runtime](https://onnxruntime.ai/), and it
requires [cgo](https://go.dev/blog/cgo) for interfacing with the ONNX Runtime
[C API](https://onnxruntime.ai/docs/api/c/).

- [`docker`](./docker) contains a sample docker file that builds a
container image that ties together a Magika CLI, an ONNX Runtime,
and a [model](../assets/models/standard_v2_1).
- [`cli`](./cli) contains a basic CLI that illustrates how to
the Magika go library may be called from within an application.
- [`magika`](./magika) contains the library, that extracts
features from a sequence of bytes.
- [`onnx`](./onnx) wraps the C API of the ONNX Runtime to
provide an inference engine.