//go:build cgo && onnxruntime

package magika

import (
	"bytes"
	"encoding/json"
	"fmt"
	"os"
	"path"
	"testing"

	"github.com/google/go-cmp/cmp"
	"github.com/google/go-cmp/cmp/cmpopts"
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
				b, err := os.ReadFile(p)
				if err != nil {
					t.Fatalf("read %s: %v", p, err)
				}
				ct, err := s.Scan(bytes.NewReader(b), len(b))
				if err != nil {
					t.Fatalf("scan %s: %v", p, err)
				}
				if d := cmp.Diff(e.Name(), ct.Label); d != "" {
					t.Errorf("unexpected content type for %s (-want +got):\n%s", ee.Name(), d)
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
			if d := cmp.Diff(s.ckb[c.want], ct); d != "" {
				t.Errorf("unexpected content type (-want +got):\n%s", d)
			}
		})
	}
}

func TestScannerReference(t *testing.T) {
	type prediction struct {
		Dl              string  `json:"dl"`
		Output          string  `json:"output"`
		Score           float32 `json:"score"`
		OverwriteReason string  `json:"overwrite_reason"`
	}
	type tcase struct {
		PredictionMode string     `json:"prediction_mode"`
		Path           string     `json:"path"`
		Content        []byte     `json:"content_base64"`
		Status         string     `json:"status"`
		Prediction     prediction `json:"prediction"`
	}

	for _, artifacts := range []string{
		"standard_v3_3-inference_examples_by_content.json.gz",
		"standard_v3_3-inference_examples_by_path.json.gz",
	} {
		b, err := loadArtifacts(t, path.Join("../../tests_data/reference", artifacts))
		if err != nil {
			t.Fatalf("load artifacts: %v", err)
		}

		var tcases []*tcase
		if err := json.Unmarshal(b, &tcases); err != nil {
			t.Fatalf("unmarshal: %s", err)
		}

		s := newTestScanner(t)
		for _, pm := range []string{"high_confidence"} {
			t.Run(fmt.Sprintf("%s-%s", artifacts, pm), func(t *testing.T) {
				var count int
				for i, c := range tcases {
					if c.PredictionMode != pm {
						continue
					}
					count++
					if c.Path != "" {
						p := path.Join("../..", c.Path)
						b, err := os.ReadFile(p)
						if err != nil {
							t.Errorf("read %s [%d]: %v", p, i, err)
							continue
						}
						c.Content = b
					}
					ct, score, err := s.scanScore(bytes.NewReader(c.Content), len(c.Content))
					if err != nil {
						t.Errorf("scan [%d]: %v", i, err)
						continue
					}
					got := prediction{
						Output: ct.Label,
						Score:  score,
					}
					if d := cmp.Diff(c.Prediction, got,
						cmpopts.EquateApprox(0, 1e-5),
						cmpopts.IgnoreFields(prediction{}, "Dl", "OverwriteReason"),
					); d != "" {
						t.Errorf("unexpected score [%d] (-want +got):\n%s", i, d)
					}
				}
				if count == 0 {
					t.Errorf("no test cases found")
				}
			})
		}
	}
}

func newTestScanner(t *testing.T) *Scanner {
	t.Helper()
	const (
		assetsDir = "../../assets"
		modelName = "standard_v3_3"
	)
	s, err := NewScanner(assetsDir, modelName)
	if err != nil {
		t.Fatalf("new scanner: %v", err)
	}
	return s
}
