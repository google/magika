package main

import (
	"bytes"
	"fmt"
	"io"
	"os"

	"github.com/google/magika/magika"
)

const (
	assetsDirEnv = "MAGIKA_ASSETS_DIR"
	modelNameEnv = "MAGIKA_MODEL"
)

// cli is a basic CLI that infers the content type of the files listed on
// the command line. The assets dir and the model name are given via the
// environment variable MAGIKA_ASSETS_DIR and MAGIKA_MODEL respectively.
func cli(w io.Writer, args ...string) error {
	assetsDir := os.Getenv(assetsDirEnv)
	if assetsDir == "" {
		return fmt.Errorf("%s environment variable not set or empty", assetsDirEnv)
	}
	modelName := os.Getenv(modelNameEnv)
	if modelName == "" {
		return fmt.Errorf("%s environment variable not set or empty", modelNameEnv)
	}
	s, err := magika.NewScanner(assetsDir, modelName)
	if err != nil {
		return fmt.Errorf("create scanner: %w", err)
	}

	// For each filename given as argument, read the file and scan its content.
	for _, a := range args {
		fmt.Fprintf(w, "%s: ", a)
		b, err := os.ReadFile(a)
		if err != nil {
			fmt.Fprintf(w, "%v\n", err)
			continue
		}
		ct, err := s.Scan(bytes.NewReader(b), len(b))
		if err != nil {
			fmt.Fprintf(w, "scan: %v\n", err)
			continue
		}
		fmt.Fprintf(w, "%s\n", ct.Label)
	}
	return nil
}
