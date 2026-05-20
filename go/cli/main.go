/*
CLI is a command-line interface for magika.

It takes a list of files as argument and prints the inferred content type of
each file. See the README for a description of the flags and output formats.

The primary intent is to illustrate how the magika go library can be used and
compiled, using cgo and the ONNX Runtime library.
*/
package main

import (
	"errors"
	"fmt"
	"os"
)

func main() {
	err := cli(os.Stdout, os.Stderr, os.Stdin, os.Args[1:]...)
	if err == nil {
		return
	}
	if errors.Is(err, errHadErrors) {
		os.Exit(1)
	}
	fmt.Fprintf(os.Stderr, "Error: %v\n", err)
	os.Exit(1)
}
