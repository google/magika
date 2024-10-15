//go:build cgo && onnxruntime

package magika

import (
	"os"
	"path"
	"testing"

	"github.com/google/go-cmp/cmp"
)

func TestScanner(t *testing.T) {
	const (
		basicDir = "../../tests_data/basic"
		ckbPath  = "../../assets/content_types_kb.min.json"
		modelDir = "../../assets/models/standard_v2_1"
	)
	es, err := os.ReadDir(basicDir)
	if err != nil {
		t.Fatalf("read tests data: %v", err)
	}
	ckb, err := ReadContentTypesKB(ckbPath)
	if err != nil {
		t.Fatalf("read content types KB: %v", err)
	}
	s, err := NewScanner(modelDir, ckb)
	if err != nil {
		t.Fatalf("new scanner: %v", err)
	}
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
				if d := cmp.Diff(ct, e.Name()); d != "" {
					t.Logf("unexpected content type for %s (-got +want):\n%s", ee.Name(), d)
				}
			}
		})
	}
}
