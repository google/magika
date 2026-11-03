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

//go:build cgo && onnxruntime

package onnx_test

import (
	"math/rand/v2"
	"testing"

	"github.com/google/magika/go/magika"
	"github.com/google/magika/go/onnx"
)

func TestONNXRuntime(t *testing.T) {
	const (
		assetsDir = "../../assets"
		modelName = "standard_v3_3"
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
