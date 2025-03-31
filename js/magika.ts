import assert from "assert";
import { Config } from "./src/config.js";
import { ContentTypeInfo } from "./src/content-type-info.js";
import { ContentTypeLabel } from "./src/content-type-label.js";
import { ContentTypesInfos } from "./src/content-types-infos.js";
import { MagikaOptions } from "./src/magika-options.js";
import { MagikaResult } from "./src/magika-result.js";
import { Model, ModelPrediction } from "./src/model.js";
import { ModelFeatures } from "./src/module-features.js";
import { OverwriteReason } from "./src/overwrite_reason.js";
import { Status } from "./src/status.js";

/**
 * The main Magika object for client-side use.
 *
 * Example usage:
 * ```js
 * const file = new File(["# Hello I am a markdown file"], "hello.md");
 * const fileBytes = new Uint8Array(await file.arrayBuffer());
 * const magika = new Magika();
 * await magika.load();
 * const prediction = await magika.identifyBytes(fileBytes);
 * console.log(prediction);
 * ```
 * For a Node implementation, please import `MagikaNode` instead.
 *
 * Demos:
 * - Node: `<MAGIKA_REPO>/js/index.js`, which you can run with `yarn run bin -h`.
 * - Client-side: see `<MAGIKA_REPO>/website/src/components/FileClassifierDemo.vue`
 */
export class Magika {
  config: Config;
  model: Model;
  cts_infos: ContentTypesInfos;

  constructor() {
    this.cts_infos = ContentTypesInfos.get();
    this.config = new Config();
    this.model = new Model(this.config);
  }

  static CONFIG_URL =
    "https://google.github.io/magika/models/standard_v3_2/config.min.json";
  static MODEL_URL =
    "https://google.github.io/magika/models/standard_v3_2/model.json";

  static async create(options?: MagikaOptions): Promise<Magika> {
    const magika = new Magika();
    await magika.load(options);
    return magika;
  }

  /** Loads the Magika model and config from URLs.
   *
   * @param {MagikaOptions} options The urls where the model and its config are stored.
   *
   * Parameters are optional. If not provided, the model will be loaded from GitHub.
   */
  async load(options?: MagikaOptions): Promise<void> {
    await Promise.all([
      this.config.loadUrl(options?.configURL || Magika.CONFIG_URL),
      this.model.loadUrl(options?.modelURL || Magika.MODEL_URL),
    ]);
  }

  /** Identifies the content type of a byte array, returning all probabilities instead of just the top one.
   *
   * @param {*} fileBytes a Buffer object (a fixed-length sequence of bytes)
   * @returns A dictionary containing the top label, its score, and a list of content types and their scores.
   */
  // async identifyBytesFull(fileBytes: Uint8Array): Promise<ModelResultLabels> {
  //   const result = await this._identifyFromBytes(fileBytes);
  //   return this._getLabelsResult(result);
  // }

  /** Identifies the content type of a byte array.
   *
   * @param {*} fileBytes a Buffer object (a fixed-length sequence of bytes)
   * @returns A dictionary containing the top label and its score
   */
  async identifyBytes(fileBytes: Uint8Array): Promise<MagikaResult> {
    const result = await this._identifyFromBytes(fileBytes);
    return result;
  }

  // _getLabelsResult(result: ModelResultScores): ModelResultLabels {
  //   const labels = [
  //     ...Object.values(this.config.target_labels_space).map(
  //       (label) => label.name,
  //     ),
  //     ...Object.values(ContentTypeLabel),
  //   ].map((label, i) => [
  //     label,
  //     label == result.label ? result.score : result.scores[i] || 0,
  //   ]);
  //   return {
  //     label: result.label,
  //     score: result.score,
  //     labels: Object.fromEntries(labels),
  //   };
  // }

  _getResultForAFewBytes(
    fileBytes: Uint8Array,
    path: string = "-",
  ): MagikaResult {
    assert(fileBytes.length <= 4 * this.config.block_size);
    const decoder = new TextDecoder("utf-8", { fatal: true });
    try {
      decoder.decode(fileBytes);

      return this._get_result_from_labels_and_score(
        "-",
        Status.OK,
        ContentTypeLabel.UNDEFINED,
        ContentTypeLabel.TXT,
        1.0,
      );
    } catch (error) {
      return this._get_result_from_labels_and_score(
        "-",
        Status.OK,
        ContentTypeLabel.UNDEFINED,
        ContentTypeLabel.UNKNOWN,
        1.0,
      );
    }
  }

  _lstrip(fileBytes: Uint8Array): Uint8Array {
    const whitespaceChars = [32, 9, 10, 13, 11, 12]; // ASCII values for ' ', '\t', '\n', '\r', '\v', '\f'
    let startIndex = 0;

    while (
      startIndex < fileBytes.length &&
      whitespaceChars.includes(fileBytes[startIndex])
    ) {
      startIndex++;
    }
    return fileBytes.subarray(startIndex);
  }

  _rstrip(fileBytes: Uint8Array): Uint8Array {
    const whitespaceChars = [32, 9, 10, 13, 11, 12]; // ASCII values for ' ', '\t', '\n', '\r', '\v', '\f'
    let endIndex = fileBytes.length - 1;

    while (endIndex >= 0 && whitespaceChars.includes(fileBytes[endIndex])) {
      endIndex--;
    }
    return fileBytes.subarray(0, endIndex + 1);
  }

  async _identifyFromBytes(fileBytes: Uint8Array): Promise<MagikaResult> {
    if (fileBytes.length == 0) {
      return this._get_result_from_labels_and_score(
        "-",
        Status.OK,
        ContentTypeLabel.UNDEFINED,
        ContentTypeLabel.EMPTY,
        1.0,
      );
    }

    if (fileBytes.length < this.config.min_file_size_for_dl) {
      return this._getResultForAFewBytes(fileBytes);
    }

    let features = this._extract_features_from_bytes(fileBytes);
    let model_prediction = await this.model.predict(features);
    let [output_label, overwrite_reason] =
      this._get_output_label_from_model_prediction(model_prediction);
    return this._get_result_from_labels_and_score(
      "-",
      Status.OK,
      model_prediction.label,
      output_label,
      model_prediction.score,
      overwrite_reason,
    );
  }

  _get_output_label_from_model_prediction(
    model_prediction: ModelPrediction,
  ): [ContentTypeLabel, OverwriteReason] {
    let overwrite_reason = OverwriteReason.NONE;

    // Overwrite model_prediction.label if specified in the overwrite_map.
    let output_label =
      this.config.overwrite_map[model_prediction.label] ??
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
      (this.config.thresholds[model_prediction.label] ??
        this.config.medium_confidence_threshold)
    ) {
      // overwrite_reason = OverwriteReason.LOW_CONFIDENCE
      if (this.cts_infos[model_prediction.label].is_text) {
        output_label = ContentTypeLabel.TXT;
      } else {
        output_label = ContentTypeLabel.UNKNOWN;
      }
      if (model_prediction.label == output_label) {
        // overwrite_reason is useful to convey to clients why the output
        // predicted is different than the model predicted type; if those two
        // are the same, the model predicted type has not actually been
        // overwritten, so we set this to NONE.
        overwrite_reason = OverwriteReason.NONE;
      }
    }

    return [output_label, overwrite_reason];
  }

  _extract_features_from_bytes(fileBytes: Uint8Array): ModelFeatures {
    const begChunk = this._lstrip(
      fileBytes.slice(0, Math.min(this.config.block_size, fileBytes.length)),
    );
    const begBytes = begChunk.slice(
      0,
      Math.min(begChunk.length, this.config.beg_size),
    );

    const endChunk = this._rstrip(
      fileBytes.slice(Math.max(0, fileBytes.length - this.config.block_size)),
    );
    const endBytes = endChunk.slice(
      Math.max(0, endChunk.length - this.config.end_size),
    );
    const endOffset = Math.max(0, this.config.end_size - endBytes.length);

    return new ModelFeatures(this.config)
      .withStart(begBytes, 0)
      .withEnd(endBytes, endOffset);
  }

  _get_ct_info(label: ContentTypeLabel): ContentTypeInfo {
    return this.cts_infos[label];
  }

  _get_result_from_labels_and_score(
    path: string,
    status: Status = Status.OK,
    dl_label: ContentTypeLabel,
    output: ContentTypeLabel,
    score: number,
    overwrite_reason: OverwriteReason = OverwriteReason.NONE,
  ): MagikaResult {
    return {
      path: path,
      status: status,
      prediction: {
        dl: this._get_ct_info(dl_label),
        output: this._get_ct_info(output),
        score: score,
        overwrite_reason: overwrite_reason,
      },
    };
  }
}
