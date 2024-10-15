package magika

import (
	"encoding/json"
	"fmt"
	"os"
)

const (
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

// ReadContentTypesKB is a helper that reads and unmarshal a content types KB,
// given the kb path.
// It returns a dictionary that maps a label as defined in the model config
// target label space to a content type.
func ReadContentTypesKB(path string) (map[string]ContentType, error) {
	var ckb map[string]ContentType
	b, err := os.ReadFile(path)
	if err != nil {
		return nil, fmt.Errorf("read %q: %w", path, err)
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
