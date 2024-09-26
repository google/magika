package magika

import (
	"bytes"
	"compress/gzip"
	"encoding/json"
	"fmt"
	"io"
	"os"
	"testing"

	"github.com/google/go-cmp/cmp"
	"github.com/google/go-cmp/cmp/cmpopts"
)

func TestExtractFeatures(t *testing.T) {
	f, err := os.Open("../../tests_data/features_extraction/reference.json.gz")
	if err != nil {
		t.Fatal(err)
	}
	r, err := gzip.NewReader(f)
	if err != nil {
		t.Fatalf("could not uncompress test data: %s", err)
	}
	b, err := io.ReadAll(r)
	if err != nil {
		t.Fatalf("could not read uncompress test data: %s", err)
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
		t.Run(fmt.Sprintf("%d", i), func(t *testing.T) {
			ft, err := ExtractFeatures(c.TestInfo, bytes.NewReader(c.Content), len(c.Content))
			if err != nil {
				t.Fatal(err)
			}
			if d := cmp.Diff(ft, c.FeaturesV2, cmpopts.IgnoreUnexported(Features{})); d != "" {
				t.Errorf("mismatch (-got +want):\n%s", d)
			}
		})
	}
}
