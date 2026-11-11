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

//go:build cgo && onnxruntime

package main

import (
	"path"
	"strings"
	"testing"

	"github.com/google/go-cmp/cmp"
)

func TestCLI(t *testing.T) {
	const basicDir = "../../tests_data/basic"
	var (
		files = []string{
			path.Join(basicDir, "python/code.py"),
			path.Join(basicDir, "zip/magika_test.zip"),
		}
		b strings.Builder
	)
	if err := cli(&b, files...); err != nil {
		t.Fatal(err)
	}
	if d := cmp.Diff(strings.Join([]string{
		"../../tests_data/basic/python/code.py: python",
		"../../tests_data/basic/zip/magika_test.zip: zip",
	}, "\n"), strings.TrimSpace(b.String())); d != "" {
		t.Errorf("mismatch (-want +got):\n%s", d)
	}
}
