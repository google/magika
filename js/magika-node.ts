import { ReadStream } from "fs";
import { finished } from "stream/promises";
import { Magika } from "./magika.js";
import { MagikaOptions } from "./src/magika-options.js";
import { MagikaResult } from "./src/magika-result.js";
import { ModelNode } from "./src/model-node.js";
import { ModelFeatures } from "./src/module-features.js";
import { Status } from "./src/status.js";

/**
 * The main Magika object for Node use.
 *
 * Example usage:
 * ```js
 * import { readFile } from "fs/promises";
 * import { MagikaNode as Magika } from "magika";
 * const data = await readFile("some file");
 * const magika = new Magika();
 * await magika.load();
 * const prediction = await magika.identifyBytes(data);
 * console.log(prediction);
 * ```
 * For a client-side implementation, please import `Magika` instead.
 *
 * Demos:
 * - Node: `<MAGIKA_REPO>/js/index.js`, which you can run with `yarn run bin -h`.
 * - Client-side: see `<MAGIKA_REPO>/website/src/components/FileClassifierDemo.vue`
 */
export class MagikaNode extends Magika {
  model: ModelNode;

  constructor() {
    super();
    // We load the version of the model that uses tfjs/node.
    this.model = new ModelNode(this.config);
  }

  /** Loads the Magika model and config from URLs.
   *
   * @param {MagikaOptions} options The urls or file paths where the model and its config are stored.
   *
   * Parameters are optional. If not provided, the model will be loaded from GitHub.
   *
   */
  async load(options?: MagikaOptions): Promise<void> {
    const p: Promise<void>[] = [];
    if (options?.configPath != null) {
      p.push(this.config.loadFile(options?.configPath));
    } else {
      p.push(this.config.loadUrl(options?.configURL || Magika.CONFIG_URL));
    }
    if (options?.modelPath != null) {
      p.push(this.model.loadFile(options?.modelPath));
    } else {
      p.push(this.model.loadUrl(options?.modelURL || Magika.MODEL_URL));
    }
    await Promise.all(p);
  }

  /** Identifies the content type from a read stream
   *
   * @param stream A read stream
   * @param length Total length of stream data (this is needed to find the middle without keep the file in memory)
   * @returns A dictionary containing the top label and its score,
   */
  async identifyStream(
    stream: ReadStream,
    length: number,
  ): Promise<MagikaResult> {
    let result = await this._identifyFromStream(stream, length);
    return result;
  }

  /** Identifies the content type from a read stream
   *
   * @param stream A read stream
   * @param length Total length of stream data (this is needed to find the middle without keep the file in memory)
   * @returns A dictionary containing the top label, its score, and a list of content types and their scores.
   */
  // async identifyStreamFull(
  //   stream: ReadStream,
  //   length: number,
  // ): Promise<MagikaResult> {
  //   const result = await this._identifyFromStream(stream, length);
  //   return this._getLabelsResult(result);
  // }

  /** Identifies the content type of a byte array, returning all probabilities instead of just the top one.
   *
   * @param {*} fileBytes a Buffer object (a fixed-length sequence of bytes)
   * @returns A dictionary containing the top label, its score, and a list of content types and their scores.
   */
  // async identifyBytesFull(
  //   fileBytes: Uint16Array | Uint8Array | Buffer,
  // ): Promise<ModelResultLabels> {
  //   const result = await this._identifyFromBytes(new Uint16Array(fileBytes));
  //   return this._getLabelsResult(result);
  // }

  /** Identifies the content type of a byte array.
   *
   * @param {*} fileBytes a Buffer object (a fixed-length sequence of bytes)
   * @returns A dictionary containing the top label and its score
   */
  async identifyBytes(
    fileBytes: Uint16Array | Uint8Array | Buffer,
  ): Promise<MagikaResult> {
    const result = await this._identifyFromBytes(new Uint8Array(fileBytes));
    return result;
  }

  async _identifyFromStream(
    stream: ReadStream,
    length: number,
  ): Promise<MagikaResult> {
    let features = new ModelFeatures(this.config);

    let accData: Buffer = Buffer.from("");
    stream.on("data", (data: string | Buffer) => {
      if (typeof data === "string") {
        throw new Error("Stream data should be a Buffer, not a string");
      }

      // ReadStream allows us to read a file chunk by chunk, sequentially.
      // It does not allow to seek around. So, the optimization we do here
      // is to avoid to store the full file in memory; but we are indeed
      // traversing the full file.

      const block_size = 4096;
      let processed_beg = false;

      if (length <= 4 * block_size) {
        // The file is small, read the full file in memory and extract
        // the features. Note that this short cut should NOT change
        // which features are eventually extracted. Let's keep
        // accumulating, and let's process the full payload once we have
        // read the entire file.
        // TODO: can we remove this check?
        accData = Buffer.concat([accData, data]);
        if (stream.bytesRead == length) {
          // Ok, we have the full file in memory.
          const fileBytes = new Uint8Array(accData);
          const begChunk = this._lstrip(
            fileBytes.slice(0, Math.min(block_size, fileBytes.length)),
          );
          const begBytes = begChunk.slice(
            0,
            Math.min(begChunk.length, this.config.beg_size),
          );
          const endChunk = this._rstrip(
            fileBytes.slice(Math.max(0, fileBytes.length - block_size)),
          );
          const endBytes = endChunk.slice(
            Math.max(0, endChunk.length - this.config.end_size),
          );
          const endOffset = Math.max(0, this.config.end_size - endBytes.length);
          features.withStart(begBytes, 0).withEnd(endBytes, endOffset);
        }
      } else {
        accData = Buffer.concat([accData, data]);
        if (!processed_beg) {
          if (accData.length >= block_size) {
            // We have at least one first block_size, let's process it.
            const fileBytes = new Uint8Array(accData);
            const begChunk = this._lstrip(
              fileBytes.slice(0, Math.min(block_size, fileBytes.length)),
            );
            const begBytes = begChunk.slice(
              0,
              Math.min(begChunk.length, this.config.beg_size),
            );
            processed_beg = true;
            features.withStart(begBytes, 0);
            // We keep a buffer of block_size bytes of what we have seen so far, not more.
            accData = accData.subarray(accData.length - block_size);
          }
        }
        if (processed_beg) {
          // If we are here, it means we have already processed the
          // beginning. Let's just be on the look out for the end
          // chunk. In the meantime, we can throw away whatever data
          // we have in excess of block_size bytes.
          accData = accData.subarray(accData.length - block_size);
          if (stream.bytesRead == length) {
            // We have just read the last chunk. We now use
            // accData's content, which is the last block_size bytes
            // from the file, and we extract the end_bytes features.
            const fileBytes = new Uint8Array(accData);
            const endChunk = this._rstrip(
              fileBytes.slice(Math.max(0, fileBytes.length - block_size)),
            );
            const endBytes = endChunk.slice(
              Math.max(0, endChunk.length - this.config.end_size),
            );
            const endOffset = Math.max(
              0,
              this.config.end_size - endBytes.length,
            );
            features.withEnd(endBytes, endOffset);
          }
        }
      }
    });
    await finished(stream);

    // TODO: refactor this to avoid code duplication
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
}
