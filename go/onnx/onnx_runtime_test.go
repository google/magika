//go:build cgo && onnxruntime

package onnx_test

import (
	"math/rand/v2"
	"testing"

	"github.com/google/magika/magika"
	"github.com/google/magika/onnx"
)

func TestONNXRuntime(t *testing.T) {
	const (
		assetsDir = "../../assets"
		modelName = "standard_v2_1"
		modelPath = "../../assets/models/" + modelName + "/model.onnx"
	)

	cfg, err := magika.ReadConfig(assetsDir, modelName)
	if err != nil {
		t.Fatal(err)
	}

	rt, err := onnx.NewOnnx(modelPath, len(cfg.TargetLabelsSpace))
	if err != nil {
		t.Fatalf("Create onnx: %v", err)
	}

	// Initialize a random features tensor.
	features := make([]int32, cfg.BegSize+cfg.MidSize+cfg.EndSize)
	for i := range features {
		features[i] = rand.Int32()
	}

	// Get the scores and check its size.
	scores, err := rt.Run(features)
	if err != nil {
		t.Fatalf("Run onnx: %v", err)
	}
	if n, m := len(scores), len(cfg.TargetLabelsSpace); n != m {
		t.Fatalf("Unexpected scores len: got %d, want %d", n, m)
	}
}
