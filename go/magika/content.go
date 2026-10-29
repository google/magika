// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
