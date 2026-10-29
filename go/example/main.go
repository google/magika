// Copyright 2025 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
