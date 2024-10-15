package main

import (
	"bytes"
	"fmt"
	"io"
	"os"

	"github.com/google/magika/magika"
)

const (
	contentTypesKBPathEnv = "MAGIKA_CONTENT_TYPES_KB_PATH"
	modelDirEnv           = "MAGIKA_MODEL_DIR"
)

// cli is a basic CLI that infers the content type of the files listed on
// the command line.
func cli(w io.Writer, args ...string) error {
	ckbPath := os.Getenv(contentTypesKBPathEnv)
	if ckbPath == "" {
		return fmt.Errorf("%s environment variable not set or empty", contentTypesKBPathEnv)
	}
	modelDir := os.Getenv(modelDirEnv)
	if modelDir == "" {
		return fmt.Errorf("%s environment variable not set or empty", modelDirEnv)
	}
	ckb, err := magika.ReadContentTypesKB(ckbPath)
	if err != nil {
		return fmt.Errorf("read content types KB: %w", err)
	}
	s, err := magika.NewScanner(modelDir, ckb)
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
