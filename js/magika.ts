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

import type { ContentTypeInfo } from "./src/content-type-info.js";
import { ContentTypeLabel } from "./src/content-type-label.js";
import { ContentTypesInfos } from "./src/content-types-infos.js";
import { MagikaOptions } from "./src/magika-options.js";
import { MagikaResult } from "./src/magika-result.js";
import { ModelConfig } from "./src/model-config.js";
import { ModelFeatures } from "./src/model-features.js";
import { ModelPrediction } from "./src/model-prediction.js";
import { Model } from "./src/model.js";
import { OverwriteReason } from "./src/overwrite-reason.js";
import { Status } from "./src/status.js";

export type { ContentTypeInfo };
export { ContentTypeLabel };

/**
 * The main Magika object for client-side use.
 *
 * Example usage:
 * ```js
 * const file = new File(["# Hello I am a markdown file"], "hello.md");
 * const fileBytes = new Uint8Array(await file.arrayBuffer());
 * const magika = await Magika.create();
 * const result = await magika.identifyBytes(fileBytes);
 * console.log(result.prediction.output.label);
 * ```
 * For a Node implementation, please import `MagikaNode` instead.
 *
 * Demos:
 * - Node: `<MAGIKA_REPO>/js/magika-cli.js`, which you can run with `yarn run bin -h`.
 * - Client-side: see `<MAGIKA_REPO>/website-ng/src/components/MagikaDemo.svelte`
 */
export class Magika {
  model_config: ModelConfig;
  model: Model;
  model_name: string;
  cts_infos: ContentTypesInfos;

  static MODEL_VERSION = "standard_v3_3";
  static MODEL_CONFIG_URL = `https://google.github.io/magika/models/${this.MODEL_VERSION}/config.min.json`;
  static MODEL_URL = `https://google.github.io/magika/models/${this.MODEL_VERSION}/model.json`;
  static WHITESPACE_CHARS = [..." \t\n\r\v\f"].map((c) => c.charCodeAt(0));

  protected constructor() {
    this.model_config = new ModelConfig();
    this.model = new Model(this.model_config);
    this.model_name = "unknown";
    this.cts_infos = ContentTypesInfos.get();
  }

  /**
   * Factory method to create a Magika instance.
   *
   * @param {MagikaOptions} options The urls or file paths where the model and
   * its config are stored.
   *
   * Parameters are optional. If not provided, the model will be loaded from GitHub.
   */
  public static async create(options?: MagikaOptions): Promise<Magika> {
    const magika = new Magika();
    await magika.load(options);
    return magika;
  }

  /**
   * Returns knowledge-base metadata and detection-stage coverage for an
   * exposed Magika content type label.
   *
   * Note: `ContentTypeLabel` covers all exposed content types across Magika's
   * heuristics, native Rust format resolvers, and ML model. Because the
   * JavaScript binding runs byte/stream heuristics and the TensorFlow.js ML
   * model without the native Rust format resolvers, labels covered only by
   * resolvers (`in_resolvers && !in_ml_model`) are not emitted by
   * `identifyBytes` / `identifyStream`; call `magika.getOutputContentTypes()`
   * on a loaded instance to list the labels this runtime can emit.
   *
   * @param {ContentTypeLabel | string} label An exposed `ContentTypeLabel` or
   * string label (e.g., `"python"`).
   * @returns {ContentTypeInfo} Metadata and stage coverage for `label`.
   */
  public static getContentTypeInfo(
    label: ContentTypeLabel | string,
  ): ContentTypeInfo {
    if (typeof label !== "string") {
      throw new TypeError(
        `Expected ContentTypeLabel or string, got ${typeof label}.`,
      );
    }
    const info = ContentTypesInfos.getInfo(label);
    if (info === undefined) {
      throw new Error(`Unknown content type label: ${JSON.stringify(label)}`);
    }
    return info;
  }

  protected async load(options?: MagikaOptions): Promise<void> {
    const modelURL = options?.modelURL || Magika.MODEL_URL;
    const modelConfigURL = options?.modelConfigURL || Magika.MODEL_CONFIG_URL;
    this.model_name = this._getModelName(modelURL);
    await Promise.all([
      this.model.loadUrl(modelURL),
      this.model_config.loadUrl(modelConfigURL),
    ]);
  }

  /**
   * Identifies the content type of a byte array.
   *
   * @param {Uint8Array} fileBytes A fixed-length sequence of bytes.
   * @returns {MagikaResult} An object containing the result of the content type
   * prediction.
   */
  public async identifyBytes(fileBytes: Uint8Array): Promise<MagikaResult> {
    const result = await this._identifyFromBytes(fileBytes);
    return result;
  }

  public getModelName(): string {
    return this.model_name;
  }

  /**
   * Returns the sorted list of all possible content type labels that this
   * JavaScript `Magika` instance can return in
   * `MagikaResult.prediction.output.label`.
   *
   * This includes the byte/stream heuristic labels (`empty`, `txt`, `unknown`)
   * and the loaded ML model's output labels after applying `overwrite_map`.
   * Unlike the native Rust/Python pipeline, this excludes filesystem-only
   * labels (`directory`, `symlink`, `unsupported`) and labels detected
   * exclusively by the native Rust format resolvers (`in_resolvers &&
   * !in_ml_model`). To inspect all exposed KB labels or filter by Rust
   * pipeline stage coverage, iterate `Object.values(ContentTypeLabel)` with
   * `Magika.getContentTypeInfo(label)`.
   *
   * @returns {ContentTypeLabel[]} Sorted list of possible output labels.
   */
  public getOutputContentTypes(): ContentTypeLabel[] {
    if (!this.model_config.loaded) {
      throw new Error("model config has not been loaded");
    }
    const outputContentTypes = new Set<ContentTypeLabel>([
      ContentTypeLabel.EMPTY,
      ContentTypeLabel.TXT,
      ContentTypeLabel.UNKNOWN,
    ]);
    for (const label of this.model_config.target_labels_space) {
      outputContentTypes.add(this.model_config.overwrite_map[label] ?? label);
    }
    return Array.from(outputContentTypes).sort();
  }

  /**
   * Returns the sorted list of content type labels supported by the loaded ML
   * model (excluding labels remapped by `overwrite_map`).
   *
   * Most callers should use `getOutputContentTypes()` instead.
   *
   * @returns {ContentTypeLabel[]} Sorted list of ML model content type labels.
   */
  public getModelContentTypes(): ContentTypeLabel[] {
    if (!this.model_config.loaded) {
      throw new Error("model config has not been loaded");
    }
    const modelContentTypes = new Set<ContentTypeLabel>();
    for (const label of this.model_config.target_labels_space) {
      if (
        !Object.prototype.hasOwnProperty.call(
          this.model_config.overwrite_map,
          label,
        )
      ) {
        modelContentTypes.add(label);
      }
    }
    return Array.from(modelContentTypes).sort();
  }

  private _getResultFromFewBytes(
    fileBytes: Uint8Array,
    path: string = "-",
  ): MagikaResult {
    if (fileBytes.length > 4 * this.model_config.block_size) {
      throw new Error("fileBytes is unexpectedly long for this function.");
    }
    const decoder = new TextDecoder("utf-8", { fatal: true });
    try {
      decoder.decode(fileBytes);

      return this._getResultFromLabelsAndScore(
        path,
        Status.OK,
        ContentTypeLabel.UNDEFINED,
        ContentTypeLabel.TXT,
        1.0,
      );
    } catch (error) {
      return this._getResultFromLabelsAndScore(
        path,
        Status.OK,
        ContentTypeLabel.UNDEFINED,
        ContentTypeLabel.UNKNOWN,
        1.0,
      );
    }
  }

  private static _lstrip(fileBytes: Uint8Array): Uint8Array {
    let startIndex = 0;
    while (
      startIndex < fileBytes.length &&
      Magika.WHITESPACE_CHARS.includes(fileBytes[startIndex])
    ) {
      startIndex++;
    }
    return fileBytes.subarray(startIndex);
  }

  private static _rstrip(fileBytes: Uint8Array): Uint8Array {
    let endIndex = fileBytes.length - 1;
    while (
      endIndex >= 0 &&
      Magika.WHITESPACE_CHARS.includes(fileBytes[endIndex])
    ) {
      endIndex--;
    }
    return fileBytes.subarray(0, endIndex + 1);
  }

  protected async _identifyFromBytes(
    fileBytes: Uint8Array,
  ): Promise<MagikaResult> {
    if (fileBytes.length === 0) {
      return this._getResultFromLabelsAndScore(
        "-",
        Status.OK,
        ContentTypeLabel.UNDEFINED,
        ContentTypeLabel.EMPTY,
        1.0,
      );
    }

    if (fileBytes.length < this.model_config.min_file_size_for_dl) {
      return this._getResultFromFewBytes(fileBytes);
    }

    const features = Magika._extractFeaturesFromBytes(
      fileBytes,
      this.model_config.beg_size,
      this.model_config.mid_size,
      this.model_config.end_size,
      this.model_config.padding_token,
      this.model_config.block_size,
      this.model_config.use_inputs_at_offsets,
    );
    if (
      features.beg_ints[this.model_config.min_file_size_for_dl - 1] ===
      this.model_config.padding_token
    ) {
      // After stripping the leading whitespace, there are fewer than
      // min_file_size_for_dl meaningful bytes left, so the model input would
      // be all padding. Fall back to the UTF-8 rule instead of running the
      // model, like the reference implementations do.
      const fewBytes = fileBytes.subarray(
        0,
        Math.min(fileBytes.length, this.model_config.block_size),
      );
      return this._getResultFromFewBytes(fewBytes);
    }
    return await this._getResultFromFeatures(features);
  }

  private _getOutputLabelFromModelPrediction(
    model_prediction: ModelPrediction,
  ): [ContentTypeLabel, OverwriteReason] {
    let overwrite_reason = OverwriteReason.NONE;

    // Overwrite model_prediction.label if specified in the overwrite_map.
    let output_label =
      this.model_config.overwrite_map[model_prediction.label] ??
      model_prediction.label;
    if (output_label != model_prediction.label) {
      overwrite_reason = OverwriteReason.OVERWRITE_MAP;
    }

    // The following code checks whether the score is "high enough" according to
    // HIGH_CONFIDENCE prediction mode (the only one we currently support in
    // this implementation). If it's not, it means we can't trust the model, and
    // we return a generic content type.
    if (
      model_prediction.score <
      (this.model_config.thresholds[model_prediction.label] ??
        this.model_config.medium_confidence_threshold)
    ) {
      overwrite_reason = OverwriteReason.LOW_CONFIDENCE;
      if (this.cts_infos[model_prediction.label].is_text) {
        output_label = ContentTypeLabel.TXT;
      } else {
        output_label = ContentTypeLabel.UNKNOWN;
      }
      if (model_prediction.label === output_label) {
        // overwrite_reason is useful to convey to clients why the output
        // predicted is different than the model predicted type; if those two
        // are the same, the model predicted type has not actually been
        // overwritten, so we set this to NONE.
        overwrite_reason = OverwriteReason.NONE;
      }
    }

    return [output_label, overwrite_reason];
  }

  protected static _extractFeaturesFromBytes(
    fileBytes: Uint8Array,
    beg_size: number,
    mid_size: number,
    end_size: number,
    padding_token: number,
    block_size: number,
    use_inputs_at_offsets: boolean,
  ): ModelFeatures {
    const begChunk = this._lstrip(
      fileBytes.slice(0, Math.min(block_size, fileBytes.length)),
    );
    const begBytes = begChunk.slice(0, Math.min(begChunk.length, beg_size));

    const endChunk = this._rstrip(
      fileBytes.slice(Math.max(0, fileBytes.length - block_size)),
    );
    const endBytes = endChunk.slice(Math.max(0, endChunk.length - end_size));
    const endOffset = Math.max(0, end_size - endBytes.length);

    return new ModelFeatures(
      beg_size,
      mid_size,
      end_size,
      padding_token,
      use_inputs_at_offsets,
    )
      .withStart(begBytes, 0)
      .withEnd(endBytes, endOffset);
  }

  private _getContentTypeInfo(label: ContentTypeLabel): ContentTypeInfo {
    const info = this.cts_infos[label];
    return { ...info, extensions: [...info.extensions] };
  }

  private _getResultFromLabelsAndScore(
    path: string,
    status: Status = Status.OK,
    dl_label: ContentTypeLabel,
    output: ContentTypeLabel,
    score: number,
    overwrite_reason: OverwriteReason = OverwriteReason.NONE,
    scores_map?: Partial<Record<ContentTypeLabel, number>>,
  ): MagikaResult {
    return {
      path: path,
      status: status,
      prediction: {
        dl: this._getContentTypeInfo(dl_label),
        output: this._getContentTypeInfo(output),
        score: score,
        overwrite_reason: overwrite_reason,
        scores_map: scores_map,
      },
    };
  }

  private async _getResultFromFeatures(
    features: ModelFeatures,
  ): Promise<MagikaResult> {
    let model_prediction = await this.model.predict(features);
    let [output_label, overwrite_reason] =
      this._getOutputLabelFromModelPrediction(model_prediction);
    return this._getResultFromLabelsAndScore(
      "-",
      Status.OK,
      model_prediction.label,
      output_label,
      model_prediction.score,
      overwrite_reason,
      model_prediction.scores_map,
    );
  }

  protected _getModelName(pathOrUrl: string): string {
    const UNKNOWN_MODEL_NAME = "unknown";
    try {
      const parts = pathOrUrl.split("/");
      // Filter out empty strings that can occur due to leading/trailing slashes
      const nonEmptyParts = parts.filter((part) => part !== "");

      if (nonEmptyParts.length >= 2) {
        return nonEmptyParts[nonEmptyParts.length - 2];
      } else {
        return UNKNOWN_MODEL_NAME;
      }
    } catch (error) {
      console.error("Error processing path or URL to get model name:", error);
      return UNKNOWN_MODEL_NAME;
    }
  }
}
