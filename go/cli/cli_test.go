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
	if d := cmp.Diff(strings.TrimSpace(b.String()), strings.Join([]string{
		"../../tests_data/basic/python/code.py: python",
		"../../tests_data/basic/zip/magika_test.zip: zip",
	}, "\n")); d != "" {
		t.Errorf("mismatch (-want +got):\n%s", d)
	}
}
