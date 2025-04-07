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

import { ReadStream } from "fs";
import { finished } from "stream/promises";
import { Magika } from "./magika.js";
import { MagikaOptions } from "./src/magika-options.js";
import { MagikaResult } from "./src/magika-result.js";
import { ModelNode } from "./src/model-node.js";
import { ModelConfigNode } from "./src/model-config-node.js";

/**
 * The main Magika object for Node use.
 *
 * Example usage:
 * ```js
 * import { readFile } from "fs/promises";
 * import { MagikaNode as Magika } from "magika";
 * const data = await readFile("some file");
 * const magika = await Magika.create();
 * const result = await magika.identifyBytes(data);
 * console.log(result.prediction.output.label);
 * ```
 * For a client-side implementation, please import `Magika` instead.
 *
 * Demos:
 * - Node: `<MAGIKA_REPO>/js/index.js`, which you can run with `yarn run bin -h`.
 * - Client-side: see `<MAGIKA_REPO>/website/src/components/FileClassifierDemo.vue`
 */
export class MagikaNode extends Magika {
  model_config: ModelConfigNode;
  model: ModelNode;

  protected constructor() {
    super();
    // We load the version of the model that uses tfjs/node.
    this.model_config = new ModelConfigNode();
    this.model = new ModelNode(this.model_config);
  }

  /**
   * Factory method to create a Magika instance.
   *
   * @param {MagikaOptions} options The urls or file paths where the model and
   * its config are stored.
   *
   * Parameters are optional. If not provided, the model will be loaded from GitHub.
   */
  public static async create(options?: MagikaOptions): Promise<MagikaNode> {
    const magika = new MagikaNode();
    await magika.load(options);
    return magika;
  }

  protected async load(options?: MagikaOptions): Promise<void> {
    const promises: Promise<void>[] = [];
    if (options?.modelConfigPath != null) {
      promises.push(this.model_config.loadFile(options?.modelConfigPath));
    } else {
      promises.push(
        this.model_config.loadUrl(
          options?.modelConfigURL || Magika.MODEL_CONFIG_URL,
        ),
      );
    }
    if (options?.modelPath != null) {
      promises.push(this.model.loadFile(options?.modelPath));
    } else {
      promises.push(this.model.loadUrl(options?.modelURL || Magika.MODEL_URL));
    }
    await Promise.all(promises);
  }

  /**
   * Identifies the content type from a read stream
   *
   * @param {ReadStream} stream A read stream.
   * @param {number} length Total length of stream data.
   * @returns {MagikaResult} An object containing the result of the content type
   * prediction.
   */
  public async identifyStream(
    stream: ReadStream,
    length: number,
  ): Promise<MagikaResult> {
    let result = await this._identifyFromStream(stream, length);
    return result;
  }

  /**
   * Identifies the content type of a byte array.
   *
   * @param {Uint8Array | Buffer} fileBytes A fixed-length sequence of bytes.
   * @returns {MagikaResult} An object containing the result of the content type
   * prediction.
   *
   * This extends the existing Magika's fileBytes method to add support
   * prediction from a Buffer object as well.
   */
  public async identifyBytes(
    fileBytes: Uint8Array | Buffer,
  ): Promise<MagikaResult> {
    const result = await this._identifyFromBytes(new Uint8Array(fileBytes));
    return result;
  }

  private async _identifyFromStream(
    stream: ReadStream,
    length: number,
  ): Promise<MagikaResult> {
    let accData: Buffer = Buffer.from("");
    let fileData: Buffer = Buffer.from("");
    stream.on("data", (data: string | Buffer) => {
      if (typeof data === "string") {
        throw new Error("Stream data should be a Buffer, not a string");
      }

      // ReadStream allows us to read a file chunk by chunk, sequentially.
      // It does not allow to seek around. So, the optimization we do here
      // is to avoid to store the full file in memory; but we are indeed
      // traversing the full file.

      // Here we collect the file bytes. For small files, we collect the full
      // stream of bytes. For large files, we collect only the first and last
      // `block_size` bytes.

      if (length <= 4 * this.model_config.block_size) {
        // The file is small; we read the full file in memory.
        fileData = Buffer.concat([fileData, data]);
      } else {
        accData = Buffer.concat([accData, data]);
        if (fileData.length === 0) {
          if (accData.length >= this.model_config.block_size) {
            // We have at least block_size bytes, let's keep them as the first
            // block.
            fileData = Buffer.concat([
              fileData,
              accData.subarray(0, this.model_config.block_size),
            ]);
          }
        }
        if (fileData.length > 0) {
          // If we are here, it means we have already collected block_size bytes
          // and kept it as the "beg block". Now, we keep processing bytes, and
          // we just store the last block_size bytes. Then, once we are at the
          // very end of the stream, we take these last block_size bytes as the
          // "end block".
          accData = accData.subarray(
            accData.length - this.model_config.block_size,
          );
          if (stream.bytesRead === length) {
            // We have just read the last chunk. We now these last block_size
            // bytes as "the end block", which together with the "beg block"
            // form the file's bytes that we can pass to the features
            // extraction.
            fileData = Buffer.concat([fileData, accData]);
          }
        }
      }
    });
    await finished(stream);
    return await this._identifyFromBytes(fileData);
  }
}
