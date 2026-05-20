//go:build cgo && onnxruntime

package main

import (
	"bytes"
	"io"
	"os"
	"path"
	"testing"

	"github.com/google/magika/go/magika"
)

// benchAssetsDir locates the magika assets. Benchmarks skip when both the
// --assets-dir equivalents are unset, so they stay green in environments
// without onnxruntime configured.
func benchAssetsDir(b *testing.B) string {
	b.Helper()
	dir := os.Getenv(assetsDirEnv)
	if dir == "" {
		b.Skip(assetsDirEnv + " not set; skipping scanner-backed benchmark")
	}
	return dir
}

func benchModelName(b *testing.B) string {
	b.Helper()
	name := os.Getenv(modelNameEnv)
	if name == "" {
		b.Skip(modelNameEnv + " not set; skipping scanner-backed benchmark")
	}
	return name
}

// newBenchScanner builds a scanner once per benchmark. Creation is expensive
// (loads the ONNX model), so callers should keep the returned value outside
// the b.N loop.
func newBenchScanner(b *testing.B) *magika.Scanner {
	b.Helper()
	s, err := magika.NewScanner(benchAssetsDir(b), benchModelName(b))
	if err != nil {
		b.Fatalf("NewScanner: %v", err)
	}
	return s
}

// BenchmarkScan measures the per-call cost of a single Scan on a small
// Python source file using a shared scanner (model-load excluded).
func BenchmarkScan(b *testing.B) {
	s := newBenchScanner(b)
	data, err := os.ReadFile(path.Join(basicDir, "python/code.py"))
	if err != nil {
		b.Fatal(err)
	}
	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		if _, _, err := s.ScanScore(bytes.NewReader(data), len(data)); err != nil {
			b.Fatal(err)
		}
	}
}

// BenchmarkScanParallel runs ScanScore concurrently to exercise the scanner's
// documented concurrent-safety and surface any contention overhead.
func BenchmarkScanParallel(b *testing.B) {
	s := newBenchScanner(b)
	data, err := os.ReadFile(path.Join(basicDir, "python/code.py"))
	if err != nil {
		b.Fatal(err)
	}
	b.ReportAllocs()
	b.ResetTimer()
	b.RunParallel(func(pb *testing.PB) {
		for pb.Next() {
			if _, _, err := s.ScanScore(bytes.NewReader(data), len(data)); err != nil {
				b.Fatal(err)
			}
		}
	})
}

// BenchmarkCLIDefault measures the full CLI pipeline: flag parsing, scanner
// creation, classification, and text rendering. Scanner creation dominates,
// so this benchmark is most useful for tracking startup regressions.
func BenchmarkCLIDefault(b *testing.B) {
	benchAssetsDir(b)
	benchModelName(b)
	file := path.Join(basicDir, "python/code.py")
	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		if err := cli(io.Discard, io.Discard, nil, "-l", file); err != nil {
			b.Fatal(err)
		}
	}
}

// BenchmarkCLIJSON measures the JSON pipeline end-to-end, including the
// hand-rolled array layout.
func BenchmarkCLIJSON(b *testing.B) {
	benchAssetsDir(b)
	benchModelName(b)
	file := path.Join(basicDir, "python/code.py")
	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		if err := cli(io.Discard, io.Discard, nil, "--json", file); err != nil {
			b.Fatal(err)
		}
	}
}

// sampleOutcome is a fixed outcome used by pure-formatter benchmarks so they
// exercise the rendering path without touching the scanner.
var sampleOutcome = &outcome{
	path: "tests_data/basic/python/code.py",
	ct: magika.ContentType{
		Label:       "python",
		MimeType:    "text/x-python",
		Group:       "code",
		Description: "Python source",
		Extensions:  []string{"py", "pyi"},
		IsText:      true,
	},
	score: 0.997,
}

// BenchmarkExpandFormat measures placeholder expansion on a representative
// format string.
func BenchmarkExpandFormat(b *testing.B) {
	const format = "%p: %l (%g) %m %e %s %S %%"
	b.ReportAllocs()
	for i := 0; i < b.N; i++ {
		_ = expandFormat(format, sampleOutcome)
	}
}

// BenchmarkRenderText measures the default text rendering path (format
// composition + expansion, no colors).
func BenchmarkRenderText(b *testing.B) {
	f := cliFlags{label: true, outputScore: true}
	b.ReportAllocs()
	for i := 0; i < b.N; i++ {
		_ = renderText(sampleOutcome, f, false)
	}
}

// BenchmarkColorize measures the ANSI-wrapping cost applied on top of a
// pre-rendered line.
func BenchmarkColorize(b *testing.B) {
	text := "tests_data/basic/python/code.py: python"
	b.ReportAllocs()
	for i := 0; i < b.N; i++ {
		_ = colorize(sampleOutcome, text)
	}
}

// BenchmarkJSONEntry measures converting one outcome into its JSON struct.
func BenchmarkJSONEntry(b *testing.B) {
	b.ReportAllocs()
	for i := 0; i < b.N; i++ {
		_ = jsonEntry(sampleOutcome)
	}
}

// BenchmarkEmitJSONArray measures the pretty-printed --json layout, which is
// the most expensive JSON path because of the manual indentation.
func BenchmarkEmitJSONArray(b *testing.B) {
	outcomes := []*outcome{sampleOutcome, sampleOutcome, sampleOutcome}
	var buf bytes.Buffer
	b.ReportAllocs()
	for i := 0; i < b.N; i++ {
		buf.Reset()
		if err := emitJSONArray(&buf, outcomes); err != nil {
			b.Fatal(err)
		}
	}
}
