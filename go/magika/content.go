package magika

import (
	"encoding/json"
	"fmt"
	"os"
)

const (
	contentTypeLabelEmpty   = "empty"
	contentTypeLabelTxt     = "txt"
	contentTypeLabelUnknown = "unknown"
)

// ContentType holds the definition of a content type.
type ContentType struct {
	Label       string   // As keyed in the content types KB.
	MimeType    string   `json:"mime_type"`
	Group       string   `json:"group"`
	Description string   `json:"description"`
	Extensions  []string `json:"extensions"`
	IsText      bool     `json:"is_text"`
}

// readContentTypesKB is a helper that reads and unmarshal a content types KB,
// given the assets dir.
// It returns a dictionary that maps a label as defined in the model config
// target label space to a content type.
func readContentTypesKB(assetsDir string) (map[string]ContentType, error) {
	var ckb map[string]ContentType
	p := contentTypesKBPath(assetsDir)
	b, err := os.ReadFile(p)
	if err != nil {
		return nil, fmt.Errorf("read %q: %w", p, err)
	}
	if err := json.Unmarshal(b, &ckb); err != nil {
		return nil, fmt.Errorf("unmarshal: %w", err)
	}
	for label, ct := range ckb {
		ct.Label = label
		ckb[label] = ct
	}
	return ckb, nil
}
