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

package main

import (
	"bytes"
	"fmt"
	"io"
	"os"

	"github.com/google/magika/go/magika"
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
