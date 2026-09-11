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

#include <stdio.h>
#include <magika.h>

int main(void) {
    // Initialize runtime with default options (thread-safe, shareable)
    MagikaRuntime* runtime = NULL;
    if (magika_runtime_new(NULL, &runtime) != MAGIKA_STATUS_OK) {
        fprintf(stderr, "Failed to initialize Magika runtime\n");
        return 1;
    }

    // Create a session for detection (single-threaded)
    MagikaSession* session = NULL;
    if (magika_session_new(runtime, &session) != MAGIKA_STATUS_OK) {
        fprintf(stderr, "Failed to create Magika session\n");
        magika_runtime_free(runtime);
        return 1;
    }

    // Identify a file on disk
    MagikaResult result;
    if (magika_identify_file(session, "README.md", &result) == MAGIKA_STATUS_OK) {
        printf("Label:       %s\n", result.info->label);
        printf("MIME type:   %s\n", result.info->mime_type);
        printf("Group:       %s\n", result.info->group);
        printf("Description: %s\n", result.info->description);
        printf("Score:       %.2f\n", result.score);
    }

    // Identify from an in-memory buffer
    const char data[] = "#!/bin/sh\necho hello\n";
    if (!magika_identify_content(session, (const uint8_t*)data, sizeof(data) - 1, &result)) {
        printf("Buffer label: %s\n", result.info->label);
    }

    // Clean up
    magika_session_free(session);
    magika_runtime_free(runtime);
    return 0;
}
