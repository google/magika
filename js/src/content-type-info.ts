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

import { ContentTypeLabel } from "./content-type-label.js";

/**
 * Metadata and detection-stage coverage for an exposed Magika content type.
 */
export interface ContentTypeInfo {
  /** Unique content type label. */
  label: ContentTypeLabel;
  /** MIME type associated with the content type. */
  mime_type: string;
  /** High-level category (e.g., `"code"`, `"document"`). */
  group: string;
  /** Human-readable description of the content type. */
  description: string;
  /** Common file extensions (without leading dots). */
  extensions: string[];
  /** Whether the content type is text-based. */
  is_text: boolean;
  /** Whether this content type is supported by built-in heuristics. */
  in_heuristics: boolean;
  /**
   * Whether this content type is supported by the native Rust format resolvers
   * (not executed by the JavaScript TF.js pipeline).
   */
  in_resolvers: boolean;
  /** Whether this content type is supported by the default active ML model. */
  in_ml_model: boolean;
  /**
   * Whether ML model predictions of this content type are vetoed when no
   * native Rust format resolver matched.
   */
  vetoed_by_resolvers: boolean;
}
