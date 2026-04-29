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
// Magika assets are available on https://github.com/google/magika/tree/main/assets
//
// Tag, cgo, and link directives must be provided at build or run time, e.g.:
// CGO_CFLAGS=-I/opt/onnxruntime/include CGO_LDFLAGS=-L/opt/onnxruntime/lib \
//   go run -tags onnxruntime .

package main

import (
	"fmt"
	"log"
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

## CLI

The [`cli`](./cli) package ships a command-line tool with feature:
recursive directory traversal, JSON/JSONL output, custom format strings,
MIME-type and label modes, score reporting, stdin input via `-`, and
colored output.

### Build & run

The CLI requires cgo and the ONNX Runtime C library. Point the compiler at
your local install and load the matching assets dir and model.

#### Using the Makefile

The `Makefile` wraps the usual env-var plumbing. It auto-detects the OS and
defaults `ONNXRUNTIME_PREFIX` to `/opt/homebrew/opt/onnxruntime` on macOS and
`/opt/onnxruntime` on Linux; `MAGIKA_ASSETS_DIR` defaults to `../assets` and
`MAGIKA_MODEL` to `standard_v3_3`. Override any variable on the command line.

```shell
make help                               # list targets and current defaults
make build                              # produces build/magika
make run ARGS="-l path/to/file"         # go run ./cli with passthrough args
make run ARGS="--json path/to/file"
make test                               # full test suite
make bench                              # Benchmark* only, with -benchmem
make clean                              # remove build/*

# Override the ONNX Runtime location or model
make build ONNXRUNTIME_PREFIX=/opt/onnxruntime
make run  ARGS="-l path/to/file" MAGIKA_MODEL=standard_v3_3
```

The resulting `build/magika` binary needs the ONNX Runtime library resolvable
at runtime (`DYLD_LIBRARY_PATH` on macOS, `LD_LIBRARY_PATH` on Linux); the
`make run` target handles that for you.

#### macOS (Homebrew):

```shell
# Run directly
MAGIKA_ASSETS_DIR=../assets \
MAGIKA_MODEL=standard_v3_3 \
CGO_CFLAGS="-I/opt/homebrew/Cellar/onnxruntime/1.24.4_1/include/onnxruntime" \
CGO_LDFLAGS="-L/opt/homebrew/Cellar/onnxruntime/1.24.4_1/lib" \
go run -tags onnxruntime ./cli/... path/to/file

# Build a standalone binary
CGO_CFLAGS="-I/opt/homebrew/Cellar/onnxruntime/1.24.4_1/include/onnxruntime" \
CGO_LDFLAGS="-L/opt/homebrew/Cellar/onnxruntime/1.24.4_1/lib" \
go build -tags onnxruntime -o build/magika ./cli

# Run the binary (onnxruntime dylib must be resolvable at runtime)
DYLD_LIBRARY_PATH=/opt/homebrew/Cellar/onnxruntime/1.24.4_1/lib \
MAGIKA_ASSETS_DIR=../assets \
MAGIKA_MODEL=standard_v3_3 \
./build/magika path/to/file
```

#### Linux (e.g. onnxruntime extracted to `/opt/onnxruntime`):

```shell
# Run directly
MAGIKA_ASSETS_DIR=../assets \
MAGIKA_MODEL=standard_v3_3 \
CGO_CFLAGS="-I/opt/onnxruntime/include" \
CGO_LDFLAGS="-L/opt/onnxruntime/lib" \
LD_LIBRARY_PATH=/opt/onnxruntime/lib \
go run -tags onnxruntime ./cli/... path/to/file

# Build a standalone binary
CGO_CFLAGS="-I/opt/onnxruntime/include" \
CGO_LDFLAGS="-L/opt/onnxruntime/lib" \
go build -tags onnxruntime -o build/magika ./cli

# Run the binary (onnxruntime .so must be resolvable at runtime)
LD_LIBRARY_PATH=/opt/onnxruntime/lib \
MAGIKA_ASSETS_DIR=../assets \
MAGIKA_MODEL=standard_v3_3 \
./build/magika path/to/file
```

Equivalent invocation using CLI flags instead of environment variables:

```shell
go run -tags onnxruntime ./cli/... \
  --assets-dir ../assets \
  --model standard_v3_3 \
  path/to/file
```

Flags take precedence over `MAGIKA_ASSETS_DIR` and `MAGIKA_MODEL`.

### Flags

| Flag | Description |
|------|-------------|
| `--assets-dir <dir>` | Assets directory (overrides `MAGIKA_ASSETS_DIR`). |
| `--model <name>` | Model name under `assets/models/` (overrides `MAGIKA_MODEL`). |
| `-r`, `--recursive` | Identify files inside directories instead of the directory itself. |
| `--no-dereference` | Treat symbolic links as symlinks rather than following them. |
| `--colors` / `--no-colors` | Force or disable color output. |
| `-s`, `--output-score` | Append the prediction score to each line. |
| `-i`, `--mime-type` | Print the MIME type instead of the description. |
| `-l`, `--label` | Print the short label instead of the description. |
| `--json` | Print results as a JSON array. |
| `--jsonl` | Print results as newline-delimited JSON. |
| `--format <fmt>` | Custom format. Placeholders: `%p %l %d %g %m %e %s %S %b %%`. |

Use `-` as a path to read from standard input (at most once per invocation).
The process exits with status `1` if any path fails to scan.

### Examples

```shell
$ magika --assets-dir assets --model standard_v3_3 tests_data/basic/python/code.py tests_data/basic/zip/magika_test.zip
tests_data/basic/python/code.py: Python source (code)
tests_data/basic/zip/magika_test.zip: Zip archive data (archive)

$ magika --assets-dir assets --model standard_v3_3 -l -s tests_data/basic/python/code.py
tests_data/basic/python/code.py: python 99%

$ magika --assets-dir assets --model standard_v3_3 --json tests_data/basic/python/code.py
[
  {
    "path": "tests_data/basic/python/code.py",
    "result": {
      "status": "ok",
      "value": {
        "dl":     { "label": "python", ... },
        "output": { "label": "python", ... },
        "score":  0.997
      }
    }
  }
]

$ cat tests_data/basic/ini/doc.ini | magika --assets-dir assets --model standard_v3_3 -l -
-: ini
```

### Tests & benchmarks

Tests and benchmarks live under `./cli/...` and require the same cgo setup as
the build. The scanner-backed tests/benchmarks load the assets from
`MAGIKA_ASSETS_DIR` + `MAGIKA_MODEL`; pure-formatter benchmarks run without
them and are skipped by the `bench` command above when unset.

#### Using the Makefile

```shell
make test                                      # full test suite
make bench                                     # Benchmark* only (-benchmem)
make test ONNXRUNTIME_PREFIX=/opt/onnxruntime  # override runtime location
```

#### macOS (Homebrew):

```shell
# Run the full test suite
MAGIKA_ASSETS_DIR=../../assets \
MAGIKA_MODEL=standard_v3_3 \
CGO_CFLAGS="-I/opt/homebrew/Cellar/onnxruntime/1.24.4_1/include/onnxruntime" \
CGO_LDFLAGS="-L/opt/homebrew/Cellar/onnxruntime/1.24.4_1/lib" \
DYLD_LIBRARY_PATH=/opt/homebrew/Cellar/onnxruntime/1.24.4_1/lib \
go test -tags onnxruntime ./cli/...

# Run only the benchmarks
MAGIKA_ASSETS_DIR=../../assets \
MAGIKA_MODEL=standard_v3_3 \
CGO_CFLAGS="-I/opt/homebrew/Cellar/onnxruntime/1.24.4_1/include/onnxruntime" \
CGO_LDFLAGS="-L/opt/homebrew/Cellar/onnxruntime/1.24.4_1/lib" \
DYLD_LIBRARY_PATH=/opt/homebrew/Cellar/onnxruntime/1.24.4_1/lib \
go test -tags onnxruntime -run=^$ -bench=. ./cli/...
```

#### Linux (e.g. onnxruntime extracted to `/opt/onnxruntime`):

```shell
# Run the full test suite
MAGIKA_ASSETS_DIR=../../assets \
MAGIKA_MODEL=standard_v3_3 \
CGO_CFLAGS="-I/opt/onnxruntime/include" \
CGO_LDFLAGS="-L/opt/onnxruntime/lib" \
LD_LIBRARY_PATH=/opt/onnxruntime/lib \
go test -tags onnxruntime ./cli/...

# Run only the benchmarks
MAGIKA_ASSETS_DIR=../../assets \
MAGIKA_MODEL=standard_v3_3 \
CGO_CFLAGS="-I/opt/onnxruntime/include" \
CGO_LDFLAGS="-L/opt/onnxruntime/lib" \
LD_LIBRARY_PATH=/opt/onnxruntime/lib \
go test -tags onnxruntime -run=^$ -bench=. ./cli/...
```

`-run=^$` disables the functional tests so only `Benchmark*` entries run. Use
`-benchtime=5s` or `-benchmem` to adjust duration and enable per-op alloc
stats.

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
