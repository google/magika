import assert from "assert";
import { ModelConfig } from "./src/model-config";
import { ContentTypeInfo } from "./src/content-type-info";
import { ContentTypeLabel } from "./src/content-type-label";
import { ContentTypesInfos } from "./src/content-types-infos";
import { MagikaOptions } from "./src/magika-options";
import { MagikaResult } from "./src/magika-result";
import { ModelPrediction } from "./src/model-prediction";
import { Model } from "./src/model";
import { ModelFeatures } from "./src/model-features";
import { OverwriteReason } from "./src/overwrite-reason";
import { Status } from "./src/status";

/**
 * The main Magika object for client-side use.
 *
 * Example usage:
 * ```js
 * const file = new File(["# Hello I am a markdown file"], "hello.md");
 * const fileBytes = new Uint8Array(await file.arrayBuffer());
 * const magika = new Magika();
 * await magika.load();
 * const result = await magika.identifyBytes(fileBytes);
 * console.log(result.prediction.output.label);
 * ```
 * For a Node implementation, please import `MagikaNode` instead.
 *
 * Demos:
 * - Node: `<MAGIKA_REPO>/js/index.js`, which you can run with `yarn run bin -h`.
 * - Client-side: see `<MAGIKA_REPO>/website/src/components/FileClassifierDemo.vue`
 */
export class Magika {
  model_config: ModelConfig;
  model: Model;
  model_version: string;
  cts_infos: ContentTypesInfos;

  constructor() {
    this.model_config = new ModelConfig();
    this.model = new Model(this.model_config);
    this.model_version = "unknown";
    this.cts_infos = ContentTypesInfos.get();
  }

  static MODEL_VERSION = "standard_v3_2";
  static MODEL_CONFIG_URL = `https://google.github.io/magika/models/${this.MODEL_VERSION}/config.min.json`;
  static MODEL_URL = `https://google.github.io/magika/models/${this.MODEL_VERSION}/model.json`;
  static WHITESPACE_CHARS = [..." \t\n\r\v\f"].map((c) => c.charCodeAt(0));

  static async create(options?: MagikaOptions): Promise<Magika> {
    const magika = new Magika();
    await magika.load(options);
    return magika;
  }

  /**
   * Loads the Magika model and config from URLs.
   *
   * @param {MagikaOptions} options The urls or file paths where the model and
   * its config are stored.
   *
   * Parameters are optional. If not provided, the model will be loaded from GitHub.
   */
  async load(options?: MagikaOptions): Promise<void> {
    this.model_version = options?.modelVersion || Magika.MODEL_VERSION;
    await Promise.all([
      this.model_config.loadUrl(
        options?.modelConfigURL || Magika.MODEL_CONFIG_URL,
      ),
      this.model.loadUrl(options?.modelURL || Magika.MODEL_URL),
    ]);
  }

  /**
   * Identifies the content type of a byte array.
   *
   * @param {Uint8Array} fileBytes A fixed-length sequence of bytes.
   * @returns {MagikaResult} An object containing the result of the content type
   * prediction.
   */
  async identifyBytes(fileBytes: Uint8Array): Promise<MagikaResult> {
    const result = await this._identifyFromBytes(fileBytes);
    return result;
  }

  getModelVersion(): string {
    return this.model_version;
  }

  _get_result_for_a_few_bytes(
    fileBytes: Uint8Array,
    path: string = "-",
  ): MagikaResult {
    assert(fileBytes.length <= 4 * this.model_config.block_size);
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

  static _lstrip(fileBytes: Uint8Array): Uint8Array {
    let startIndex = 0;
    while (
      startIndex < fileBytes.length &&
      Magika.WHITESPACE_CHARS.includes(fileBytes[startIndex])
    ) {
      startIndex++;
    }
    return fileBytes.subarray(startIndex);
  }

  static _rstrip(fileBytes: Uint8Array): Uint8Array {
    let endIndex = fileBytes.length - 1;
    while (
      endIndex >= 0 &&
      Magika.WHITESPACE_CHARS.includes(fileBytes[endIndex])
    ) {
      endIndex--;
    }
    return fileBytes.subarray(0, endIndex + 1);
  }

  async _identifyFromBytes(fileBytes: Uint8Array): Promise<MagikaResult> {
    if (fileBytes.length === 0) {
      return this._get_result_from_labels_and_score(
        "-",
        Status.OK,
        ContentTypeLabel.UNDEFINED,
        ContentTypeLabel.EMPTY,
        1.0,
      );
    }

    if (fileBytes.length < this.model_config.min_file_size_for_dl) {
      return this._get_result_for_a_few_bytes(fileBytes);
    }

    const features = Magika._extract_features_from_bytes(
      fileBytes,
      this.model_config.beg_size,
      this.model_config.mid_size,
      this.model_config.end_size,
      this.model_config.padding_token,
      this.model_config.block_size,
      this.model_config.use_inputs_at_offsets,
    );
    return await this._get_result_from_features(features);
  }

  _get_output_label_from_model_prediction(
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

  static _extract_features_from_bytes(
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
    scores_map?: Partial<Record<ContentTypeLabel, number>>,
  ): MagikaResult {
    return {
      path: path,
      status: status,
      prediction: {
        dl: this._get_ct_info(dl_label),
        output: this._get_ct_info(output),
        score: score,
        overwrite_reason: overwrite_reason,
        scores_map: scores_map,
      },
    };
  }

  async _get_result_from_features(
    features: ModelFeatures,
  ): Promise<MagikaResult> {
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
      model_prediction.scores_map,
    );
  }
}
