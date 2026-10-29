// Copyright 2024 Google LLC
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

/*
CLI is a simple command line interface for magika.

It takes a list of files as argument, and infers their types in sequence.
For example:

	$ magika test.go readme.md
	test.go: go
	readme.md: markdown

The primary intent is to illustrate how the magika go library can be used
and compiled, using cgo and the ONNX Runtime library.
*/
package main

import (
	"fmt"
	"os"
)

func main() {
	if err := cli(os.Stdout, os.Args[1:]...); err != nil {
		fmt.Printf("Error: %v\n", err)
		os.Exit(1)
	}
}
