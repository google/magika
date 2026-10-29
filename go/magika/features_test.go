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

package magika

import (
	"bytes"
	"compress/gzip"
	"encoding/json"
	"io"
	"os"
	"testing"

	"github.com/google/go-cmp/cmp"
	"github.com/google/go-cmp/cmp/cmpopts"
)

func TestExtractFeatures(t *testing.T) {
	const artifacts = "../../tests_data/features_extraction/reference.json.gz"
	b, err := loadArtifacts(t, artifacts)
	if err != nil {
		t.Fatalf("load artifacts: %s", err)
	}

	var cases []struct {
		TestInfo   Config   `json:"test_info"`
		Content    []byte   `json:"content"`
		FeaturesV2 Features `json:"features_v2"`
	}
	if err := json.Unmarshal(b, &cases); err != nil {
		t.Fatal(err)
	}
	for i, c := range cases {
		ft, err := ExtractFeatures(c.TestInfo, bytes.NewReader(c.Content), len(c.Content))
		if err != nil {
			t.Fatal(err)
		}
		if d := cmp.Diff(c.FeaturesV2, ft, cmpopts.IgnoreUnexported(Features{})); d != "" {
			t.Errorf("Feature [%d]: mismatch (-want +got):\n%s", i, d)
		}
	}
}

func TestReferenceExtractFeatures(t *testing.T) {
	const artifacts = "../../tests_data/reference/features_extraction_examples.json.gz"
	b, err := loadArtifacts(t, artifacts)
	if err != nil {
		t.Fatalf("load artifacts: %s", err)
	}

	var cases []struct {
		TestInfo   Config   `json:"args"`
		Content    []byte   `json:"content_base64"`
		FeaturesV2 Features `json:"features"`
	}
	if err := json.Unmarshal(b, &cases); err != nil {
		t.Fatal(err)
	}
	for i, c := range cases {
		ft, err := ExtractFeatures(c.TestInfo, bytes.NewReader(c.Content), len(c.Content))
		if err != nil {
			t.Fatal(err)
		}
		if d := cmp.Diff(c.FeaturesV2, ft,
			cmpopts.IgnoreUnexported(Features{}),
			cmpopts.IgnoreFields(Features{}, "Offset8000", "Offset8800", "Offset9000", "Offset9800"),
		); d != "" {
			t.Errorf("Feature [%d]: mismatch (-want +got):\n%s", i, d)
		}
	}
}

func loadArtifacts(t *testing.T, path string) ([]byte, error) {
	t.Helper()
	f, err := os.Open(path)
	if err != nil {
		t.Fatalf("Open %s: %v", path, err)
	}
	r, err := gzip.NewReader(f)
	if err != nil {
		t.Fatalf("could not uncompress test data: %s", err)
	}
	b, err := io.ReadAll(r)
	if err != nil {
		t.Fatalf("could not read uncompress test data: %s", err)
	}
	return b, nil
}
