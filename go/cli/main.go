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
