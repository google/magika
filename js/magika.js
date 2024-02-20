import * as tf from "@tensorflow/tfjs";

// Default content types that aren't emitted by the model directly.
const ContentType = Object.freeze({
  EMPTY: "empty",
  GENERIC_TEXT: "txt",
  UNKNOWN: "unknown",
});

// Holds the magika config
class Config {
  async load(configURL) {
    if (this.labels) return;
    const config = await (await fetch(configURL)).json();
    this.minFileSizeForDl = config["min_file_size_for_dl"];
    this.paddingToken = config["padding_token"];
    this.labels = config["labels"];
    this.begBytes = config["input_size_beg"];
    this.midBytes = config["input_size_beg"];
    this.endBytes = config["input_size_beg"];
    this.extractSize =
      this.begBytes > 0
        ? this.begBytes
        : this.midBytes > 0
          ? this.midBytes
          : this.endBytes;
  }
}

class Model {
  async load(modelURL) {
    if (this.model) return;
    this.model = await tf.loadGraphModel(modelURL);
  }

  predict(features) {
    const modelInput = tf.tensor([features]);
    const modelOutput = tf.squeeze(this.model.predict(modelInput));
    const maxProbability = tf.argMax(modelOutput);
    const labelIndex = maxProbability.dataSync()[0];
    const labelProbabilities = modelOutput.dataSync();
    maxProbability.dispose();
    modelInput.dispose();
    modelOutput.dispose();
    return [labelIndex, labelProbabilities];
  }
}

/**
 * The Magika object.
 *
 * Example usage:
 * ```js
 *   const data = await readFile('some file');
 *   const magika = new Magika();
 *   await magika.load();
 *   const prediction = await magika.identifyBytes(data);
 *   console.log(prediction);
 * ```
 * For a Node implementation, see `<MAGIKA_REPO>/js/index.js`, which you can run with `yarn run bin -h`.
 *
 * For a web version, see `<MAGIKA_REPO>/docs/src/components/FileClassifierDemo.vue`,
 */
export class Magika {
  /** Loads the Magika model and config from URLs.
   *
   * @param {string} modelURL The URL where the model is stored.
   * @param {string} configURL The URL where the config is stored.
   *
   * Both parameters are optional. If not provided, the model will be loaded from Github.
   *
   */
  async load({ modelURL, configURL } = {}) {
    modelURL = modelURL || "https://google.github.io/magika/model/model.json";
    configURL = configURL || "https://google.github.io/magika/model/config.json";
    this.config = new Config();
    this.model = new Model();
    await Promise.all([this.config.load(configURL), this.model.load(modelURL)]);
  }

  /** Identifies the content type of a byte stream.
   *
   * @param {*} fileBytes  a Buffer object (a fixed-length sequence of bytes)
   * @returns A dictionary containing the top label and its score,
   */
  async identifyBytes(fileBytes) {
    return this._identifyBytes(fileBytes, (args) => this._generateResult(args));
  }

  /** Identifies the content type of a byte stream, returning all probabilities instead of just the top one.
   *
   * @param {*} fileBytes  a Buffer object (a fixed-length sequence of bytes)
   * @returns A dictionary containing the top label, its score, and a list of content types and their scores.
   */
  async identifyBytesFull(fileBytes) {
    return this._identifyBytes(fileBytes, (args) =>
      this._generateResultFull(args),
    );
  }

  _generateResult({ label, score }) {
    return { label, score };
  }

  _generateResultFull({ label, score, scores }) {
    const labels = [
      ...Object.values(this.config.labels).map((l) => l.name),
      ...Object.values(ContentType),
    ];
    if (!scores) {
      scores = labels.map((l) => (l === label ? score : 0));
    }
    return {
      label,
      score,
      labels: Object.fromEntries(labels.map((l, i) => [l, scores[i] || 0])),
    };
  }

  _getResultForAFewBytes(fileBytes, generateResult) {
    const decoder = new TextDecoder("utf-8", { fatal: true });
    try {
      decoder.decode(fileBytes);
      return generateResult({
        score: 1.0,
        label: ContentType.GENERIC_TEXT,
      });
    } catch (error) {
      return generateResult({ score: 1.0, label: ContentType.UNKNOWN });
    }
  }

  async _identifyBytes(fileBytes, generateResult) {
    if (fileBytes.length === 0)
      return generateResult({ score: 1.0, label: ContentType.EMPTY });
    if (fileBytes.length <= this.config.minFileSizeForDl)
      return this._getResultForAFewBytes(fileBytes, generateResult);
    const [extractionResult, features] = await this._extractFeaturesFromBytes(
      fileBytes,
      generateResult,
    );
    if (extractionResult) return extractionResult;
    // End of special cases, now we can do deep learning!
    return this._generateResultFromPrediction(
      this.model.predict(features),
      generateResult,
    );
  }

  _generateResultFromPrediction([labelIndex, scores], generateResult) {
    const score = scores[labelIndex];
    const labelConfig = this.config["labels"][labelIndex];
    const { name, threshold } = labelConfig;
    if (score >= threshold)
      return generateResult({ score, label: name, scores });
    if (labelConfig["is_text"])
      return generateResult({ score, label: ContentType.GENERIC_TEXT, scores });
    return generateResult({ score, label: ContentType.UNKNOWN, scores });
  }

  async _extractFeaturesFromBytes(fileBytes, generateResult) {
    const fileArray = new Uint16Array(fileBytes);
    if (fileArray.length <= this.config.minFileSizeForDl)
      return [this._getResultForAFewBytes(fileBytes, generateResult), null];

    let beg = new Uint16Array(this.config.begBytes).fill(
      this.config.paddingToken,
    );
    let mid = new Uint16Array(this.config.midBytes).fill(
      this.config.paddingToken,
    );
    let end = new Uint16Array(this.config.endBytes).fill(
      this.config.paddingToken,
    );

    // Beginning chunk. It should start with the file, and padding at the end.
    beg.set(fileArray.slice(0, this.config.begBytes), 0);

    // Middle chunk. Padding on either side.
    const halfpoint = 2 * Math.round(fileArray.length / 2);
    const startHalf = Math.max(0, halfpoint - this.config.midBytes / 2);
    const halfChunk = fileArray.slice(
      startHalf,
      startHalf + this.config.midBytes,
    );
    mid.set(halfChunk, this.config.midBytes / 2 - halfChunk.length / 2);

    // End chunk. It should end with the file, and padding at the beginning.
    const endChunk = fileArray.slice(
      Math.max(0, fileArray.length - this.config.endBytes),
    );
    const endOffset = Math.max(0, this.config.endBytes - endChunk.length);
    end.set(endChunk, endOffset);

    return [null, [...beg, ...mid, ...end]];
  }
}
