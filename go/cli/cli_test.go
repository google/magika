//go:build cgo && onnxruntime

package main

import (
	"bytes"
	"encoding/json"
	"io"
	"path"
	"strings"
	"testing"

	"github.com/google/go-cmp/cmp"
)

const basicDir = "../../tests_data/basic"

func run(t *testing.T, stdin io.Reader, args ...string) (string, string, error) {
	t.Helper()
	var stdout, stderr bytes.Buffer
	err := cli(&stdout, &stderr, stdin, args...)
	return stdout.String(), stderr.String(), err
}

func TestCLIDefault(t *testing.T) {
	files := []string{
		path.Join(basicDir, "python/code.py"),
		path.Join(basicDir, "zip/magika_test.zip"),
	}
	stdout, _, err := run(t, nil, append([]string{"-l"}, files...)...)
	if err != nil {
		t.Fatal(err)
	}
	want := strings.Join([]string{
		files[0] + ": python",
		files[1] + ": zip",
	}, "\n")
	if d := cmp.Diff(want, strings.TrimSpace(stdout)); d != "" {
		t.Errorf("mismatch (-want +got):\n%s", d)
	}
}

func TestCLIJSON(t *testing.T) {
	file := path.Join(basicDir, "python/code.py")
	stdout, _, err := run(t, nil, "--json", file)
	if err != nil {
		t.Fatal(err)
	}
	var decoded []struct {
		Path   string `json:"path"`
		Result struct {
			Status string `json:"status"`
			Value  struct {
				Output struct {
					Label    string `json:"label"`
					MimeType string `json:"mime_type"`
				} `json:"output"`
				Score float64 `json:"score"`
			} `json:"value"`
		} `json:"result"`
	}
	if err := json.Unmarshal([]byte(stdout), &decoded); err != nil {
		t.Fatalf("unmarshal: %v\nstdout=%s", err, stdout)
	}
	if len(decoded) != 1 {
		t.Fatalf("expected 1 entry, got %d", len(decoded))
	}
	if decoded[0].Path != file {
		t.Errorf("path: got %q want %q", decoded[0].Path, file)
	}
	if decoded[0].Result.Status != "ok" {
		t.Errorf("status: got %q want ok", decoded[0].Result.Status)
	}
	if decoded[0].Result.Value.Output.Label != "python" {
		t.Errorf("label: got %q want python", decoded[0].Result.Value.Output.Label)
	}
}

func TestCLIJSONL(t *testing.T) {
	file := path.Join(basicDir, "python/code.py")
	stdout, _, err := run(t, nil, "--jsonl", file)
	if err != nil {
		t.Fatal(err)
	}
	lines := strings.Split(strings.TrimRight(stdout, "\n"), "\n")
	if len(lines) != 1 {
		t.Fatalf("expected 1 line, got %d: %q", len(lines), stdout)
	}
	var entry map[string]any
	if err := json.Unmarshal([]byte(lines[0]), &entry); err != nil {
		t.Fatalf("unmarshal: %v", err)
	}
	if entry["path"] != file {
		t.Errorf("path: got %v want %q", entry["path"], file)
	}
}

func TestCLIMime(t *testing.T) {
	file := path.Join(basicDir, "python/code.py")
	stdout, _, err := run(t, nil, "-i", file)
	if err != nil {
		t.Fatal(err)
	}
	want := file + ": text/x-python\n"
	if stdout != want {
		t.Errorf("got %q want %q", stdout, want)
	}
}

func TestCLIStdin(t *testing.T) {
	stdout, _, err := run(t, strings.NewReader("print('hi')\n"), "-l", "-")
	if err != nil {
		t.Fatal(err)
	}
	if !strings.HasPrefix(stdout, "-: ") {
		t.Errorf("unexpected output: %q", stdout)
	}
}

func TestCLIMissingFile(t *testing.T) {
	stdout, _, err := run(t, nil, "-l", "does-not-exist")
	if err != errHadErrors {
		t.Fatalf("expected errHadErrors, got %v", err)
	}
	if !strings.HasPrefix(stdout, "does-not-exist: ") {
		t.Errorf("unexpected output: %q", stdout)
	}
}

func TestCLIRecursive(t *testing.T) {
	dir := path.Join(basicDir, "python")
	stdout, _, err := run(t, nil, "-r", "-l", dir)
	if err != nil {
		t.Fatal(err)
	}
	if !strings.Contains(stdout, ": python") {
		t.Errorf("expected python entry, got %q", stdout)
	}
}
