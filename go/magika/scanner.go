package magika

import (
	"errors"
	"fmt"
	"io"
	"unicode/utf8"

	"github.com/google/magika/go/onnx"
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
	_, out, _, err := s.ScanDetails(r, size)
	return out, err
}

// ScanScore scans the given reader containing the given size of bytes, and
// returns the inferred content type and its score (between 0 and 1).
// It is safe for concurrent use.
func (s *Scanner) ScanScore(r io.ReaderAt, size int) (ContentType, float32, error) {
	_, out, score, err := s.ScanDetails(r, size)
	return out, score, err
}

// ScanDetails returns the raw deep-learning prediction (dl), the final
// content type after low-confidence and Overwrite remaps (output), and the
// model score. For inputs that bypass the model (empty or smaller than
// MinFileSizeForDl) dl equals output.
// It is safe for concurrent use.
func (s *Scanner) ScanDetails(r io.ReaderAt, size int) (ContentType, ContentType, float32, error) {
	if size == 0 {
		ct := s.ckb[contentTypeLabelEmpty]
		return ct, ct, 1, nil
	}
	ft, err := ExtractFeatures(s.cfg, r, size)
	if err != nil {
		return ContentType{}, ContentType{}, 0, fmt.Errorf("extract features: %w", err)
	}
	// Do not use the model for small files.
	if ft.Beg[s.cfg.MinFileSizeForDl-1] == int32(s.cfg.PaddingToken) {
		var ct ContentType
		if utf8.Valid(ft.firstBlock) {
			ct = s.ckb[contentTypeLabelTxt]
		} else {
			ct = s.ckb[contentTypeLabelUnknown]
		}
		return ct, ct, 1, nil
	}
	scores, err := s.onnx.Run(ft.Flatten())
	if err != nil {
		return ContentType{}, ContentType{}, 0, fmt.Errorf("run onnx: %w", err)
	}
	if len(scores) == 0 {
		return ContentType{}, ContentType{}, 0, errors.New("run onnx: empty result")
	}
	best := 0
	for i, v := range scores {
		if v > scores[best] {
			best = i
		}
	}
	dl, output, err := s.contentType(best, scores[best])
	if err != nil {
		return ContentType{}, ContentType{}, 0, fmt.Errorf("get content type: %w", err)
	}
	return dl, output, scores[best], nil
}

// contentType resolves the model's top index into (dl, output). dl is the raw
// label the model picked; output applies the low-confidence fallback and
// cfg.Overwrite remap.
func (s *Scanner) contentType(best int, score float32) (ContentType, ContentType, error) {
	l := s.cfg.TargetLabelsSpace[best]
	dl, ok := s.ckb[l]
	if !ok {
		return ContentType{}, ContentType{}, fmt.Errorf("no content type found for %q", l)
	}
	th := s.cfg.MediumConfidenceThreshold
	if t, ok := s.cfg.Thresholds[l]; ok {
		th = t
	}
	outLabel := l
	switch {
	case score >= th:
	case dl.IsText:
		outLabel = contentTypeLabelTxt
	default:
		outLabel = contentTypeLabelUnknown
	}
	output, ok := s.ckb[outLabel]
	if !ok {
		return ContentType{}, ContentType{}, fmt.Errorf("no content type found for %q", outLabel)
	}
	if remap, ok := s.cfg.Overwrite[outLabel]; ok {
		if output, ok = s.ckb[remap]; !ok {
			return ContentType{}, ContentType{}, fmt.Errorf("no content type found for %q", remap)
		}
	}
	return dl, output, nil
}
