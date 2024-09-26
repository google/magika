package main

import (
	"bytes"
	"fmt"
	"os"

	"github.com/google/magika/magika"
)

const modelPathEnv = "MAGIKA_MODEL_PATH"

// cli is a basic CLI that infers the content type of the files listed on
// the command line.
func cli() error {
	dir := os.Getenv(modelPathEnv)
	if dir == "" {
		return fmt.Errorf("%s environment variable not set or empty", modelPathEnv)
	}
	cfg, err := magika.ReadConfig(dir)
	if err != nil {
		return fmt.Errorf("read config: %w", err)
	}
	s, err := magika.NewScanner(dir, cfg)
	if err != nil {
		return fmt.Errorf("create scanner: %w", err)
	}

	// For each filename given as argument, read the file and scan its content.
	for _, a := range os.Args[1:] {
		fmt.Printf("%s: ", a)
		b, err := os.ReadFile(a)
		if err != nil {
			fmt.Printf("%v\n", err)
			continue
		}
		ct, err := s.Scan(bytes.NewReader(b), len(b))
		if err != nil {
			fmt.Printf("scan: %v\n", err)
			continue
		}
		fmt.Printf("%s\n", ct)
	}
	return nil
}
