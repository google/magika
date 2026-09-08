// Copyright 2026 Google LLC
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

#include <assert.h>
#include <stdio.h>
#include <string.h>

#include "magika.h"

int main(void) {
    // Test runtime creation with explicit options
    MagikaRuntimeOptions options = {
        .backend = MAGIKA_BACKEND_CPU,
        .max_batch = 0,
    };
    MagikaRuntime* runtime_opt = NULL;
    MagikaStatus status_opt = magika_runtime_new(&options, &runtime_opt);
    assert(status_opt == MAGIKA_STATUS_OK);
    assert(runtime_opt != NULL);
    magika_runtime_free(runtime_opt);

    // Test runtime creation with default options
    MagikaRuntime* runtime = NULL;
    MagikaStatus status = magika_runtime_new(NULL, &runtime);
    assert(status == MAGIKA_STATUS_OK);
    assert(runtime != NULL);

    // Test session creation from runtime
    MagikaSession* session = NULL;
    status = magika_session_new(runtime, &session);
    assert(status == MAGIKA_STATUS_OK);
    assert(session != NULL);

    // Test identifying a known file by path
    MagikaResult result;
    const char* test_path = "src/lib.rs";
    status = magika_identify_file(session, test_path, &result);
    assert(status == MAGIKA_STATUS_OK);
    assert(result.info != NULL);
    assert(strcmp(result.info->label, "rust") == 0);
    assert(strcmp(result.info->mime_type, "application/x-rust") == 0);
    assert(strcmp(result.info->group, "code") == 0);
    assert(result.info->is_text == true);
    assert(result.score > 0.5f);
    assert(result.kind == MAGIKA_FILE_TYPE_KIND_INFERRED);
    assert(result.inferred_info != NULL);
    assert(strcmp(result.inferred_info->label, "rust") == 0);
    assert(result.overwrite_reason == MAGIKA_OVERWRITE_REASON_NONE);

    // Test extensions list
    assert(result.info->extensions != NULL);
    bool found_rs_ext = false;
    for (const char* const* ext = result.info->extensions; *ext != NULL; ++ext) {
        if (strcmp(*ext, "rs") == 0) {
            found_rs_ext = true;
        }
    }
    assert(found_rs_ext);

    // Test identifying content in memory
    const char shell_content[] = "#!/bin/sh\necho hello\n";
    const uint8_t* data_ptr = (const uint8_t*)shell_content;
    size_t data_len = strlen(shell_content);
    status = magika_identify_content(session, data_ptr, data_len, &result);
    assert(status == MAGIKA_STATUS_OK);
    assert(result.info != NULL);
    assert(strcmp(result.info->label, "shell") == 0);

    // Test identifying empty content
    status = magika_identify_content(session, NULL, 0, &result);
    assert(status == MAGIKA_STATUS_OK);
    assert(result.info != NULL);
    assert(strcmp(result.info->label, "empty") == 0);

    // Test error on non-existent file
    const char* nonexistent_path = "this_file_does_not_exist_12345.xyz";
    status = magika_identify_file(session, nonexistent_path, &result);
    assert(status == MAGIKA_STATUS_IO_ERROR);

    // Test invalid argument on NULL pointers
    status = magika_identify_file(NULL, test_path, &result);
    assert(status == MAGIKA_STATUS_INVALID_ARGUMENT);

    status = magika_identify_file(session, NULL, &result);
    assert(status == MAGIKA_STATUS_INVALID_ARGUMENT);

    status = magika_identify_file(session, test_path, NULL);
    assert(status == MAGIKA_STATUS_INVALID_ARGUMENT);

    status = magika_session_new(NULL, &session);
    assert(status == MAGIKA_STATUS_INVALID_ARGUMENT);

    // Test feature extraction from file
    MagikaFeatures* feat_file = NULL;
    status = magika_features_extract_file(test_path, &feat_file, &result);
    assert(status == MAGIKA_STATUS_OK);
    assert(feat_file != NULL);

    // Test single feature identification
    MagikaResult feat_result;
    status = magika_identify_features(session, feat_file, &feat_result);
    assert(status == MAGIKA_STATUS_OK);
    assert(feat_result.info != NULL);
    assert(strcmp(feat_result.info->label, "rust") == 0);
    assert(feat_result.kind == MAGIKA_FILE_TYPE_KIND_INFERRED);
    assert(feat_result.score > 0.5f);

    // Test feature extraction from memory content
    MagikaFeatures* feat_content = NULL;
    status = magika_features_extract_content(data_ptr, data_len, &feat_content, &result);
    assert(status == MAGIKA_STATUS_OK);
    assert(feat_content != NULL);

    // Test batch feature identification
    const MagikaFeatures* batch[2] = { feat_file, feat_content };
    MagikaResult batch_results[2];
    status = magika_identify_features_batch(session, batch, 2, batch_results);
    assert(status == MAGIKA_STATUS_OK);
    assert(batch_results[0].info != NULL);
    assert(strcmp(batch_results[0].info->label, "rust") == 0);
    assert(batch_results[1].info != NULL);
    assert(strcmp(batch_results[1].info->label, "shell") == 0);

    // Test batch with count == 0
    status = magika_identify_features_batch(NULL, NULL, 0, NULL);
    assert(status == MAGIKA_STATUS_OK);

    // Test feature extraction on empty content (ruled, out_features set to NULL)
    MagikaFeatures* empty_feat = (MagikaFeatures*)0x1234;
    status = magika_features_extract_content(NULL, 0, &empty_feat, &result);
    assert(status == MAGIKA_STATUS_OK);
    assert(empty_feat == NULL);
    assert(result.info != NULL);
    assert(strcmp(result.info->label, "empty") == 0);
    assert(result.kind == MAGIKA_FILE_TYPE_KIND_RULED);

    // Test error on extracting features from non-existent file
    MagikaFeatures* missing_feat = NULL;
    status = magika_features_extract_file(nonexistent_path, &missing_feat, &result);
    assert(status == MAGIKA_STATUS_IO_ERROR);
    assert(missing_feat == NULL);

    // Test invalid argument on NULL pointers for feature extraction
    MagikaFeatures* dummy_feat = NULL;
    status = magika_features_extract_file(NULL, &dummy_feat, &result);
    assert(status == MAGIKA_STATUS_INVALID_ARGUMENT);

    status = magika_features_extract_file(test_path, NULL, &result);
    assert(status == MAGIKA_STATUS_INVALID_ARGUMENT);

    status = magika_features_extract_content(NULL, 10, &dummy_feat, &result);
    assert(status == MAGIKA_STATUS_INVALID_ARGUMENT);

    status = magika_features_extract_content(data_ptr, data_len, NULL, &result);
    assert(status == MAGIKA_STATUS_INVALID_ARGUMENT);

    // Test invalid argument on NULL pointers for identify_features
    status = magika_identify_features(NULL, feat_file, &feat_result);
    assert(status == MAGIKA_STATUS_INVALID_ARGUMENT);

    status = magika_identify_features(session, NULL, &feat_result);
    assert(status == MAGIKA_STATUS_INVALID_ARGUMENT);

    status = magika_identify_features(session, feat_file, NULL);
    assert(status == MAGIKA_STATUS_INVALID_ARGUMENT);

    // Test invalid argument on NULL pointers for identify_features_batch
    status = magika_identify_features_batch(NULL, batch, 2, batch_results);
    assert(status == MAGIKA_STATUS_INVALID_ARGUMENT);

    status = magika_identify_features_batch(session, NULL, 2, batch_results);
    assert(status == MAGIKA_STATUS_INVALID_ARGUMENT);

    status = magika_identify_features_batch(session, batch, 2, NULL);
    assert(status == MAGIKA_STATUS_INVALID_ARGUMENT);

    // Test null element inside batch
    const MagikaFeatures* null_batch[2] = { feat_file, NULL };
    status = magika_identify_features_batch(session, null_batch, 2, batch_results);
    assert(status == MAGIKA_STATUS_INVALID_ARGUMENT);

    // Test freeing features
    magika_features_free(feat_file);
    magika_features_free(feat_content);
    magika_features_free(NULL); // Safe no-op

    // Test cleanup
    magika_session_free(session);
    magika_session_free(NULL); // Safe no-op

    magika_runtime_free(runtime);
    magika_runtime_free(NULL); // Safe no-op

    return 0;
}
