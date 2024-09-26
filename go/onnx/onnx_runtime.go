//go:build cgo && onnxruntime

package onnx

// #cgo LDFLAGS: -lonnxruntime
// #include "onnx_runtime.h"
import "C"

import (
	"fmt"
)

// NewOnnx returns an onnx that can perform inferences using an ONNX Runtime
// (https://onnxruntime.ai/) and the given model.
// It wraps the C calls to the ONNX Runtime API https://onnxruntime.ai/docs/api/c.
func NewOnnx(modelPath string, sizeTarget int) (Onnx, error) {
	ort := &onnxRuntime{
		api:        C.GetApiBase(),
		sizeTarget: sizeTarget,
	}
	if err := C.CreateSession(ort.api, C.CString(modelPath), &ort.session, &ort.memory); err != nil {
		return nil, fmt.Errorf("create session: %v", C.GoString(C.GetErrorMessage(err)))
	}
	return ort, nil
}

// onnxRuntime implements the Onnx interface relying on a cgo call
// to a C ONNX Runtime library.
type onnxRuntime struct {
	api        *C.OrtApi
	session    *C.OrtSession
	memory     *C.OrtMemoryInfo
	sizeTarget int
}

func (ort *onnxRuntime) Run(features []int32) ([]float32, error) {
	target := make([]float32, ort.sizeTarget)
	if err := C.Run(ort.api, ort.session, ort.memory, (*C.int32_t)(&features[0]), C.int64_t(len(features)), (*C.float)(&target[0]), C.int64_t(len(target))); err != nil {
		return nil, fmt.Errorf("run: %v", C.GoString(C.GetErrorMessage(err)))
	}
	return target, nil
}
