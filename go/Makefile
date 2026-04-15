# Makefile for the Magika Go CLI and library.
#
# Build artifacts land in $(BUILD_DIR). Override any variable on the command
# line, e.g.:
#   make build ONNXRUNTIME_PREFIX=/opt/onnxruntime
#   make run ARGS="-l path/to/file"
#   make test MAGIKA_MODEL=standard_v3_3

UNAME_S := $(shell uname -s)

# Default ONNX Runtime prefix by OS. Override with ONNXRUNTIME_PREFIX=...
ifeq ($(UNAME_S),Darwin)
  ONNXRUNTIME_PREFIX ?= /opt/homebrew/opt/onnxruntime
  RUNTIME_LIB_VAR    := DYLD_LIBRARY_PATH
else
  ONNXRUNTIME_PREFIX ?= /opt/onnxruntime
  RUNTIME_LIB_VAR    := LD_LIBRARY_PATH
endif

# Homebrew ships headers under include/onnxruntime/; others use include/.
ifeq ($(UNAME_S),Darwin)
  ONNX_INCLUDE ?= $(ONNXRUNTIME_PREFIX)/include/onnxruntime
else
  ONNX_INCLUDE ?= $(ONNXRUNTIME_PREFIX)/include
endif
ONNX_LIB ?= $(ONNXRUNTIME_PREFIX)/lib

# Model location. Flags on the CLI ultimately read these at runtime.
MAGIKA_ASSETS_DIR ?= $(CURDIR)/../assets
MAGIKA_MODEL      ?= standard_v3_3

BUILD_DIR ?= build
BINARY    := $(BUILD_DIR)/magika
GO_TAGS   := onnxruntime

# cgo and runtime env baked into every recipe that touches the scanner.
export CGO_CFLAGS  := -I$(ONNX_INCLUDE)
export CGO_LDFLAGS := -L$(ONNX_LIB)
export $(RUNTIME_LIB_VAR) := $(ONNX_LIB):$($(RUNTIME_LIB_VAR))
export MAGIKA_ASSETS_DIR
export MAGIKA_MODEL

.PHONY: help build run test bench vet fmt clean

help:
	@echo "Targets:"
	@echo "  build  - compile CLI into $(BINARY)"
	@echo "  run    - go run ./cli -- pass args via ARGS=..."
	@echo "  test   - run full test suite (cli + magika)"
	@echo "  bench  - run Benchmark* entries only"
	@echo "  vet    - go vet with onnxruntime tag"
	@echo "  fmt    - gofmt -w on the module"
	@echo "  clean  - remove $(BUILD_DIR)/*"
	@echo ""
	@echo "Key variables (current values):"
	@echo "  ONNXRUNTIME_PREFIX = $(ONNXRUNTIME_PREFIX)"
	@echo "  MAGIKA_ASSETS_DIR  = $(MAGIKA_ASSETS_DIR)"
	@echo "  MAGIKA_MODEL       = $(MAGIKA_MODEL)"

build:
	@mkdir -p $(BUILD_DIR)
	go build -tags $(GO_TAGS) -o $(BINARY) ./cli

run:
	go run -tags $(GO_TAGS) ./cli $(ARGS)

test:
	go test -tags $(GO_TAGS) ./...

bench:
	go test -tags $(GO_TAGS) -run=^$$ -bench=. -benchmem ./cli/...

vet:
	go vet -tags $(GO_TAGS) ./...

fmt:
	gofmt -w .

clean:
	rm -rf $(BUILD_DIR)/*
