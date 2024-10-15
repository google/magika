package magika

import (
	"encoding/json"
	"fmt"
	"os"
	"path"
)

const configFile = "config.min.json"

// Config holds the portion of Magika's model configuration that is relevant
// for inference.
type Config struct {
	BegSize                   int                `json:"beg_size"`
	MidSize                   int                `json:"mid_size"`
	EndSize                   int                `json:"end_size"`
	UseInputsAtOffsets        bool               `json:"use_inputs_at_offsets"`
	MediumConfidenceThreshold float32            `json:"medium_confidence_threshold"`
	MinFileSizeForDl          int64              `json:"min_file_size_for_dl"`
	PaddingToken              int                `json:"padding_token"`
	BlockSize                 int                `json:"block_size"`
	TargetLabelsSpace         []string           `json:"target_labels_space"`
	Thresholds                map[string]float32 `json:"thresholds"`
}

// ReadConfig is a helper that reads and unmarshal a Config, given a model
// directory path.
func ReadConfig(modelDir string) (Config, error) {
	var cfg Config
	p := path.Join(modelDir, configFile)
	b, err := os.ReadFile(p)
	if err != nil {
		return Config{}, fmt.Errorf("read %q: %w", p, err)
	}
	if err := json.Unmarshal(b, &cfg); err != nil {
		return Config{}, fmt.Errorf("unmarshal: %w", err)
	}
	return cfg, nil
}
