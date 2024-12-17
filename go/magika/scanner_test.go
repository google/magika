//go:build cgo && onnxruntime

package magika

import (
	"bytes"
	"os"
	"path"
	"testing"

	"github.com/google/go-cmp/cmp"
)

func TestScannerBasic(t *testing.T) {
	const basicDir = "../../tests_data/basic"
	es, err := os.ReadDir(basicDir)
	if err != nil {
		t.Fatalf("read tests data: %v", err)
	}
	s := newTestScanner(t)
	for _, e := range es {
		t.Run(e.Name(), func(t *testing.T) {
			dir := path.Join(basicDir, e.Name())
			es, err := os.ReadDir(dir)
			if err != nil {
				t.Fatalf("read tests data: %v", err)
			}
			for _, ee := range es {
				p := path.Join(dir, ee.Name())
				fi, err := os.Stat(p)
				if err != nil {
					t.Fatalf("stat %s: %v", p, err)
				}
				f, err := os.Open(p)
				if err != nil {
					t.Fatalf("open %s: %v", p, err)
				}
				ct, err := s.Scan(f, int(fi.Size()))
				if err != nil {
					t.Fatalf("scan %s: %v", p, err)
				}
				if d := cmp.Diff(ct.Label, e.Name()); d != "" {
					t.Errorf("unexpected content type for %s (-got +want):\n%s", ee.Name(), d)
				}
			}
		})
	}
}

func TestScannerSmall(t *testing.T) {
	s := newTestScanner(t)
	for _, c := range []struct {
		name string
		data []byte
		want string
	}{{
		name: "empty",
		data: []byte{},
		want: contentTypeLabelEmpty,
	}, {
		name: "small txt",
		data: []byte("small"),
		want: contentTypeLabelTxt,
	}, {
		name: "small bin",
		data: []byte{0x80, 0x80, 0x80, 0x80},
		want: contentTypeLabelUnknown,
	}} {
		t.Run(c.name, func(t *testing.T) {
			ct, err := s.Scan(bytes.NewReader(c.data), len(c.data))
			if err != nil {
				t.Fatalf("scan: %v", err)
			}
			if d := cmp.Diff(ct, s.ckb[c.want]); d != "" {
				t.Errorf("unexpected content type (-got +want):\n%s", d)
			}
		})
	}
}

func newTestScanner(t *testing.T) *Scanner {
	t.Helper()
	const (
		assetsDir = "../../assets"
		modelName = "standard_v2_1"
	)
	s, err := NewScanner(assetsDir, modelName)
	if err != nil {
		t.Fatalf("new scanner: %v", err)
	}
	return s
}
