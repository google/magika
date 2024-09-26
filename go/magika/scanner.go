package magika

import (
	"errors"
	"fmt"
	"io"
	"unicode/utf8"

	"github.com/google/magika/onnx"
)

// Scanner represents a Magika scanner that returns the content type
// of the scanned content running the Magika model using an ONNX Runtime.
// This is a similar scanner interface to licensecheck, that scans
// content to identify licenses.
type Scanner struct {
	onnx onnx.Onnx
	cfg  Config
	ckb  map[string]ContentType
}

// NewScanner returns a scanner based on the model of the given name defined
// in the given the assets dir.
func NewScanner(assetsDir, name string) (*Scanner, error) {
	cfg, err := ReadConfig(assetsDir, name)
	if err != nil {
		return nil, fmt.Errorf("read config: %w", err)
	}
	p := modelPath(assetsDir, name)
	ob, err := onnx.NewOnnx(p, len(cfg.TargetLabelsSpace))
	if err != nil {
		return nil, fmt.Errorf("new onnx: %w", err)
	}
	if ob == nil {
		return nil, errors.New("new onnx: nil onnx object")
	}
	ckb, err := readContentTypesKB(assetsDir)
	if err != nil {
		return nil, fmt.Errorf("read content types KB: %w", err)
	}
	return &Scanner{
		onnx: ob,
		cfg:  cfg,
		ckb:  ckb,
	}, nil
}

// Scan scans the given reader containing the given size of bytes, and
// returns the inferred content type.
// It is safe for concurrent use.
func (s *Scanner) Scan(r io.ReaderAt, size int) (ContentType, error) {
	if size == 0 {
		return s.ckb[contentTypeLabelEmpty], nil
	}
	ft, err := ExtractFeatures(s.cfg, r, size)
	if err != nil {
		return ContentType{}, fmt.Errorf("extract features: %w", err)
	}
	// Do not use the model for small files.
	if ft.Beg[s.cfg.MinFileSizeForDl-1] == int32(s.cfg.PaddingToken) {
		if utf8.Valid(ft.firstBlock) {
			return s.ckb[contentTypeLabelTxt], nil
		} else {
			return s.ckb[contentTypeLabelUnknown], nil
		}
	}
	scores, err := s.onnx.Run(ft.Flatten())
	if err != nil {
		return ContentType{}, fmt.Errorf("run onnx: %w", err)
	}
	if len(scores) == 0 {
		return ContentType{}, errors.New("run onnx: empty result")
	}
	best := 0
	for i, v := range scores {
		if v > scores[best] {
			best = i
		}
	}
	return s.contentType(best, scores[best])
}

func (s *Scanner) contentType(best int, score float32) (ContentType, error) {
	l := s.cfg.TargetLabelsSpace[best]
	ct, ok := s.ckb[l]
	if !ok {
		return ContentType{}, fmt.Errorf("no content type found for %q", l)
	}
	th := s.cfg.MediumConfidenceThreshold
	if t, ok := s.cfg.Thresholds[l]; ok {
		th = t
	}
	// Return the inferred content type if the threshold is met, otherwise
	// falls back to a relevant default.
	switch {
	case score >= th:
		return ct, nil
	case ct.IsText:
		return s.ckb[contentTypeLabelTxt], nil
	default:
		return s.ckb[contentTypeLabelUnknown], nil
	}
}
