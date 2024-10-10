package onnx

// Onnx represents something that can run inferences on features.
type Onnx interface {
	// Run returns the result of the inference on the given features.
	Run(features []int32) ([]float32, error)
}
