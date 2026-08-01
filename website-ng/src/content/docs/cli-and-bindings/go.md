---
title: "Go `magika` library"
---

The Go binding lives in the [`go/`](https://github.com/google/magika/tree/main/go) subdirectory of the Magika repository. It wraps the ONNX Runtime C API and requires [cgo](https://go.dev/blog/cgo).

:::caution
The Go binding is experimental. It is not yet published as a tagged module and its API may change.
:::

## Install

Vendor the module directly from the source tree:

```bash
go get github.com/google/magika/go
```

You will also need the ONNX Runtime shared library and the Magika asset directory available at build or run time. See the [`go/README.md`](https://github.com/google/magika/blob/main/go/README.md) for setup details, and the [`go/docker/Dockerfile`](https://github.com/google/magika/blob/main/go/docker/Dockerfile) for a complete example.

## Usage

The entry point is `magika.NewScanner`, which loads a model from an asset directory and returns a reusable scanner:

```go
//go:build cgo && onnxruntime

package main

import (
	"fmt"
	"log"
	"strings"

	"github.com/google/magika/go/magika"
)

func main() {
	scanner, err := magika.NewScanner("/opt/magika/assets", "standard_v3_3")
	if err != nil {
		log.Fatalf("NewScanner failed: %v", err)
	}

	content := "Hello world"
	ct, err := scanner.Scan(strings.NewReader(content), len(content))
	if err != nil {
		log.Fatalf("Scan failed: %v", err)
	}

	fmt.Printf("%+v\n", ct)
}
```

Build and run it with the `onnxruntime` tag and a link flag pointing at the ONNX Runtime installation:

```bash
go run -tags onnxruntime -ldflags="-linkmode=external -extldflags=-L/opt/onnxruntime/lib" .
```

A working end-to-end example is at [`go/example/main.go`](https://github.com/google/magika/blob/main/go/example/main.go), and a more elaborate CLI is at [`go/cli/cli.go`](https://github.com/google/magika/blob/main/go/cli/cli.go).
