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

#include <stdio.h>
#include <onnxruntime_c_api.h>

#define RETURN_ON_ERROR(expr) {      \
	OrtStatus* onnx_status = (expr); \
	if (onnx_status != NULL) {       \
		return onnx_status;          \
	}                                \
}

const OrtApi *GetApiBase() {
	return OrtGetApiBase()->GetApi(ORT_API_VERSION);
}

OrtStatus *CreateSession(const OrtApi *ort, const char *model, OrtSession **session, OrtMemoryInfo **memory_info) {
	OrtEnv *env;
	RETURN_ON_ERROR(ort->CreateEnv(ORT_LOGGING_LEVEL_ERROR, "onnx", &env));
	RETURN_ON_ERROR(ort->DisableTelemetryEvents(env));
	OrtSessionOptions *options;
	RETURN_ON_ERROR(ort->CreateSessionOptions(&options));
	RETURN_ON_ERROR(ort->EnableCpuMemArena(options));
	RETURN_ON_ERROR(ort->CreateSession(env, model, options, session));
	RETURN_ON_ERROR(ort->CreateCpuMemoryInfo(OrtArenaAllocator, OrtMemTypeDefault, memory_info));
	return NULL;
}

OrtStatus *Run(const OrtApi *ort, OrtSession *session, OrtMemoryInfo *memory_info, int32_t features[], int64_t sizeFeatures, float target[], int64_t sizeTarget) {
	const char *input_names[] = {"bytes"};
	const char *output_names[] = {"target_label"};
	const int64_t input_shape[] = {1, sizeFeatures};
	OrtValue *input_tensor = NULL;
	RETURN_ON_ERROR(ort->CreateTensorWithDataAsOrtValue(memory_info, features, sizeFeatures * sizeof(int32_t), input_shape, 2, ONNX_TENSOR_ELEMENT_DATA_TYPE_INT32, &input_tensor));
	OrtValue *output_tensor = NULL;
	RETURN_ON_ERROR(ort->Run(session, NULL, input_names, (const OrtValue *const *) &input_tensor, 1, output_names, 1, &output_tensor));
	float *out = NULL;
	RETURN_ON_ERROR(ort->GetTensorMutableData(output_tensor, (void **) &out));
	memcpy(target, out, sizeTarget * sizeof(float));
	ort->ReleaseValue(input_tensor);
	ort->ReleaseValue(output_tensor);
	return NULL;
}

const char *GetErrorMessage(const OrtStatus* onnx_status) {
	if (onnx_status == NULL) {
		return "";
	}
	return OrtGetApiBase()->GetApi(ORT_API_VERSION)->GetErrorMessage(onnx_status);
}
