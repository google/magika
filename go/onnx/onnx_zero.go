//go:build !(cgo && onnxruntime)

package onnx

// NewOnnx returns a nil Onnx runtime.
// This allows for building and unit testing in a non-cgo context.
func NewOnnx(string, int) (Onnx, error) {
	return nil, nil
}
