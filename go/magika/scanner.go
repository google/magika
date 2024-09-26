package magika

import (
	"errors"
	"fmt"
	"io"
	"path"

	"github.com/google/magika/onnx"
)

const modelFile = "model.onnx"

// Scanner represents a Magika scanner that returns the content type
// of the scanned content running the Magika model using an ONNX Runtime.
// This is a similar scanner interface to licensecheck, that scans
// content to identify licenses.
type Scanner struct {
	onnx onnx.Onnx
	cfg  Config
}

func NewScanner(assetDir string, cfg Config) (*Scanner, error) {
	ob, err := onnx.NewOnnx(path.Join(assetDir, modelFile), len(cfg.TargetLabelsSpace))
	if err != nil {
		return nil, fmt.Errorf("new onnx: %w", err)
	}
	if ob == nil {
		return nil, errors.New("new onnx: nil onnx object")
	}
	return &Scanner{
		onnx: ob,
		cfg:  cfg,
	}, nil
}

// Scan scans the given reader containing the given size of bytes, and
// returns the content type identification using Magika.
// It is safe for concurrent use.
func (s *Scanner) Scan(r io.ReaderAt, size int) (string, error) {
	ft, err := ExtractFeatures(s.cfg, r, size)
	if err != nil {
		return "", fmt.Errorf("extract features: %w", err)
	}
	res, err := s.onnx.Run(ft.Flatten())
	if err != nil {
		return "", fmt.Errorf("run onnx: %w", err)
	}
	if len(res) == 0 {
		return "", errors.New("run onnx: empty result")
	}
	best := 0
	for i, v := range res {
		if v > res[best] {
			best = i
		}
	}
	return s.cfg.TargetLabelsSpace[best], nil
}
