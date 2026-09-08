#ifndef MAGIKA_H
#define MAGIKA_H

// DO NOT EDIT, see link below for more information:
// https://github.com/google/magika/tree/main/rust/gen

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * Status and error codes returned by Magika C API functions.
 */
typedef enum MagikaStatus {
  /**
   * Operation completed successfully.
   */
  MAGIKA_STATUS_OK = 0,
  /**
   * An argument was invalid or a required pointer was null.
   */
  MAGIKA_STATUS_INVALID_ARGUMENT = -1,
  /**
   * An I/O error occurred while reading the file.
   */
  MAGIKA_STATUS_IO_ERROR = -2,
  /**
   * An error occurred during neural network inference.
   */
  MAGIKA_STATUS_INFERENCE_ERROR = -3,
  /**
   * An internal panic was caught across the FFI boundary.
   */
  MAGIKA_STATUS_PANIC = -4,
} MagikaStatus;

/**
 * Hardware backend to use for neural network inference.
 */
typedef enum MagikaBackend {
  /**
   * Automatically select the best available backend.
   */
  MAGIKA_BACKEND_AUTO = 0,
  /**
   * Use CPU inference.
   */
  MAGIKA_BACKEND_CPU = 1,
  /**
   * Use GPU inference.
   */
  MAGIKA_BACKEND_GPU = 2,
} MagikaBackend;

/**
 * The kind of identified file type.
 */
typedef enum MagikaFileTypeKind {
  /**
   * The file is a directory.
   */
  MAGIKA_FILE_TYPE_KIND_DIRECTORY = 0,
  /**
   * The file is a symbolic link.
   */
  MAGIKA_FILE_TYPE_KIND_SYMLINK = 1,
  /**
   * The file is a regular file and was identified using deep learning inference.
   */
  MAGIKA_FILE_TYPE_KIND_INFERRED = 2,
  /**
   * The file is a regular file and was identified using rules.
   */
  MAGIKA_FILE_TYPE_KIND_RULED = 3,
} MagikaFileTypeKind;

/**
 * Reason why an inferred content type was overwritten.
 */
typedef enum MagikaOverwriteReason {
  /**
   * The inferred content type was not overwritten.
   */
  MAGIKA_OVERWRITE_REASON_NONE = 0,
  /**
   * The model score was below the confidence threshold for the inferred type.
   */
  MAGIKA_OVERWRITE_REASON_LOW_CONFIDENCE = 1,
  /**
   * The inferred type was mapped to another canonical type.
   */
  MAGIKA_OVERWRITE_REASON_OVERWRITE_MAP = 2,
} MagikaOverwriteReason;

/**
 * Features extracted from a file or buffer for neural network inference.
 */
typedef struct MagikaFeatures MagikaFeatures;

/**
 * Shared Magika inference runtime (thread-safe).
 */
typedef struct MagikaRuntime MagikaRuntime;

/**
 * Magika identification session (not thread-safe, one per thread).
 */
typedef struct MagikaSession MagikaSession;

/**
 * Options for configuring a Magika runtime.
 */
typedef struct MagikaRuntimeOptions {
  /**
   * The backend to use for inference.
   */
  enum MagikaBackend backend;
  /**
   * The maximum batch size to optimize for (0 for runtime default).
   */
  uintptr_t max_batch;
} MagikaRuntimeOptions;

/**
 * Content type information.
 */
typedef struct MagikaTypeInfo {
  /**
   * The unique label identifying this file type (null-terminated UTF-8 string).
   */
  const char *label;
  /**
   * The MIME type of the file type (null-terminated UTF-8 string).
   */
  const char *mime_type;
  /**
   * The group of the file type (null-terminated UTF-8 string).
   */
  const char *group;
  /**
   * The human-readable description of the file type (null-terminated UTF-8 string).
   */
  const char *description;
  /**
   * Null-terminated array of null-terminated extension strings.
   */
  const char *const *extensions;
  /**
   * Whether the file type is text.
   */
  bool is_text;
} MagikaTypeInfo;

/**
 * Result of a file identification.
 *
 * All pointers are to static memory, so no cleanup function is needed.
 */
typedef struct MagikaResult {
  /**
   * The kind of identified file type.
   */
  enum MagikaFileTypeKind kind;
  /**
   * Resolved content type information (never null, points to static storage).
   */
  const struct MagikaTypeInfo *info;
  /**
   * Confidence score between 0.0 and 1.0 (1.0 for directory, symlink, or ruled).
   */
  float score;
  /**
   * Raw model output before overwrite rules were applied (null if not inferred).
   */
  const struct MagikaTypeInfo *inferred_info;
  /**
   * Reason why the inferred type was overwritten (None if not overwritten).
   */
  enum MagikaOverwriteReason overwrite_reason;
} MagikaResult;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

/**
 * Initializes the shared Magika inference runtime (loads models and plans).
 *
 * If `options` is NULL, the default configuration is used.
 *
 * # Safety
 *
 * `out_runtime` must point to a valid, writable pointer to `MagikaRuntime`.
 */
enum MagikaStatus magika_runtime_new(const struct MagikaRuntimeOptions *options,
                                     struct MagikaRuntime **out_runtime);

/**
 * Frees a Magika runtime. Passing NULL is a safe no-op.
 *
 * # Safety
 *
 * If non-null, `runtime` must have been returned by `magika_runtime_new` and not previously freed.
 */
void magika_runtime_free(struct MagikaRuntime *runtime);

/**
 * Spawns a new identification session from the runtime.
 *
 * # Safety
 *
 * - `runtime` must point to a valid `MagikaRuntime`.
 * - `out_session` must point to a valid, writable pointer to `MagikaSession`.
 */
enum MagikaStatus magika_session_new(const struct MagikaRuntime *runtime,
                                     struct MagikaSession **out_session);

/**
 * Frees a Magika session. Passing NULL is a safe no-op.
 *
 * # Safety
 *
 * If non-null, `session` must have been returned by `magika_session_new` and not previously freed.
 */
void magika_session_free(struct MagikaSession *session);

/**
 * Frees Magika features. Passing NULL is a safe no-op.
 *
 * # Safety
 *
 * If non-null, `features` must have been returned by a Magika features extraction
 * function and not previously freed.
 */
void magika_features_free(struct MagikaFeatures *features);

/**
 * Identifies the content type of a file on disk.
 *
 * # Safety
 *
 * - `session` must point to a valid `MagikaSession`.
 * - `path` must point to a null-terminated C string.
 * - `out_result` must point to a valid, writable `MagikaResult` struct.
 */
enum MagikaStatus magika_identify_file(struct MagikaSession *session,
                                       const char *path,
                                       struct MagikaResult *out_result);

/**
 * Identifies the content type of an in-memory buffer.
 *
 * # Safety
 *
 * - `session` must point to a valid `MagikaSession`.
 * - `data` must point to at least `len` readable bytes (may be NULL only if `len == 0`).
 * - `out_result` must point to a valid, writable `MagikaResult` struct.
 */
enum MagikaStatus magika_identify_content(struct MagikaSession *session,
                                          const uint8_t *data,
                                          uintptr_t len,
                                          struct MagikaResult *out_result);

/**
 * Extracts features from a file on disk for neural network inference.
 *
 * If the file does not require neural network inference (for example, if it is empty
 * or identified by rules):
 * - `*out_features` is set to NULL.
 * - If `out_result` is non-null, `*out_result` is populated with the identification result.
 *
 * If the file requires neural network inference:
 * - `*out_features` is set to a newly allocated `MagikaFeatures` (which must be freed
 *   with `magika_features_free`).
 * - `out_result` is left unchanged.
 *
 * # Safety
 *
 * - `path` must point to a null-terminated C string.
 * - `out_features` must point to a valid, writable pointer to `MagikaFeatures`.
 * - `out_result` may be NULL, or must point to a valid, writable `MagikaResult` struct.
 */
enum MagikaStatus magika_features_extract_file(const char *path,
                                               struct MagikaFeatures **out_features,
                                               struct MagikaResult *out_result);

/**
 * Extracts features from an in-memory buffer for neural network inference.
 *
 * If the buffer does not require neural network inference (for example, if it is empty
 * or identified by rules):
 * - `*out_features` is set to NULL.
 * - If `out_result` is non-null, `*out_result` is populated with the identification result.
 *
 * If the buffer requires neural network inference:
 * - `*out_features` is set to a newly allocated `MagikaFeatures` (which must be freed
 *   with `magika_features_free`).
 * - `out_result` is left unchanged.
 *
 * # Safety
 *
 * - `data` must point to at least `len` readable bytes (may be NULL only if `len == 0`).
 * - `out_features` must point to a valid, writable pointer to `MagikaFeatures`.
 * - `out_result` may be NULL, or must point to a valid, writable `MagikaResult` struct.
 */
enum MagikaStatus magika_features_extract_content(const uint8_t *data,
                                                  uintptr_t len,
                                                  struct MagikaFeatures **out_features,
                                                  struct MagikaResult *out_result);

/**
 * Identifies the content type of a file from its extracted features.
 *
 * # Safety
 *
 * - `session` must point to a valid `MagikaSession`.
 * - `features` must point to a valid `MagikaFeatures`.
 * - `out_result` must point to a valid, writable `MagikaResult` struct.
 */
enum MagikaStatus magika_identify_features(struct MagikaSession *session,
                                           const struct MagikaFeatures *features,
                                           struct MagikaResult *out_result);

/**
 * Identifies the content types of multiple files from their extracted features in a batch.
 *
 * # Safety
 *
 * Unless `count` is 0:
 * - `session` must point to a valid `MagikaSession`.
 * - `features` must point to an array of at least `count` valid, non-null `MagikaFeatures`
 *   pointers.
 * - `out_results` must point to an array of at least `count` writable `MagikaResult` structs.
 */
enum MagikaStatus magika_identify_features_batch(struct MagikaSession *session,
                                                 const struct MagikaFeatures *const *features,
                                                 uintptr_t count,
                                                 struct MagikaResult *out_results);

#ifdef __cplusplus
}  // extern "C"
#endif  // __cplusplus

#endif  /* MAGIKA_H */
