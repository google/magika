# Go library

This directory contains the Go library for Magika.

The inference relies on the [ONNX Runtime](https://onnxruntime.ai/), and it
requires [cgo](https://go.dev/blog/cgo) for interfacing with the ONNX Runtime
[C API](https://onnxruntime.ai/docs/api/c/).

## Usage
As illustrated in [`example/main.go`](./example/main.go), calling magika from go boils down
to creating a scanner associated with a given model, and scanning the content.

```golang
//go:build cgo && onnxruntime

// This package illustrates the usage of the Magika go binding.
//
// It requires the onnxruntime and the Magika assets to be accessible.
// onnxruntime is available on https://github.com/microsoft/onnxruntime/releases
// Magika asserts are available on https://github.com/google/magika/tree/main/assets
//
// Tag and link directives must be provided a build or run time:
// go run -tags onnxruntime -ldflags="-linkmode=external -extldflags=-L/opt/onnxruntime/lib" .

package main

import (
	"fmt"
	"strings"

	"github.com/google/magika/go/magika"
)

const (
	// assetsDir holds where the Magika assets have been installed.
	assetsDir = "/opt/magika/assets"
	// modelName holds the Magika model to use.
	modelName = "standard_v3_3"
)

func main() {
	// Create a scanner.
	s, err := magika.NewScanner(assetsDir, modelName)
	if err != nil {
		log.Fatalf("NewScanner failed: %v", err)
	}
	// Scan
	ct, err := s.Scan(strings.NewReader("Hello world"), 11)
	if err != nil {
		log.Fatalf("Scan failed: %v", err)
	}
	fmt.Printf("%+v\n", ct)
}
```

Inspiration on how to download and install onnxruntime and magika assets can be found in [`docker/Dockerfile`](docker/Dockerfile),
and [`cli/cli.go`](cli/cli.go) provides a somewhat more elaborate usage of the go binding.

## Content
- [`docker`](./docker) contains a sample docker file that builds a
container image that ties together a Magika CLI, an ONNX Runtime,
and a [model](../assets/models/standard_v3_3).
- [`cli`](./cli) contains a basic CLI that illustrates how 
the Magika go library may be called from within an application.
- [`magika`](./magika) contains the library, that extracts
features from a sequence of bytes, and provides a scanner to infer
content types.
- [`onnx`](./onnx) wraps the C API of the ONNX Runtime to
provide an inference engine.
- [`example`](./example) contains a rudimentary example for creating
and using a content type scanner.
