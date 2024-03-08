import { ReadStream } from 'fs';
import { finished } from 'stream/promises';

import { Magika } from './magika.js';
import { ModelNode } from './src/model_node.js';
import { ModelFeatures } from './src/moduleFeatures.js';
import { ModelResult, ModelResultLabels, ModelResultScores } from './src/model.js';
import { MagikaOptions } from './src/magikaOptions.js';

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
    async identifyStream(stream: ReadStream, length: number): Promise<ModelResult> {
        const result = await this._identifyFromStream(stream, length);
        return { label: result.label, score: result.score };
    }

    /** Identifies the content type from a read stream
     * 
     * @param stream A read stream
     * @param length Total length of stream data (this is needed to find the middle without keep the file in memory)
     * @returns A dictionary containing the top label, its score, and a list of content types and their scores.
     */
    async identifyStreamFull(stream: ReadStream, length: number): Promise<ModelResultLabels> {
        const result = await this._identifyFromStream(stream, length);
        return this._getLabelsResult(result);
    }

    /** Identifies the content type of a byte array, returning all probabilities instead of just the top one.
     *
     * @param {*} fileBytes a Buffer object (a fixed-length sequence of bytes)
     * @returns A dictionary containing the top label, its score, and a list of content types and their scores.
     */
    async identifyBytesFull(fileBytes: Uint16Array | Uint8Array | Buffer): Promise<ModelResultLabels> {

        const result = await this._identifyFromBytes(new Uint16Array(fileBytes));
        return this._getLabelsResult(result);
    }

    /** Identifies the content type of a byte array.
     *
     * @param {*} fileBytes a Buffer object (a fixed-length sequence of bytes)
     * @returns A dictionary containing the top label and its score
     */
    async identifyBytes(fileBytes: Uint16Array | Uint8Array | Buffer): Promise<ModelResult> {
        const result = await this._identifyFromBytes(new Uint16Array(fileBytes));
        return { label: result.label, score: result.score };
    }

    async _identifyFromStream(stream: ReadStream, length: number): Promise<ModelResultScores> {
        const features = new ModelFeatures(this.config);

        const halfpoint = Math.max(0, Math.round(length / 2) - Math.round(this.config.midBytes / 2));
        const halfpointCap = Math.min(length, (halfpoint + this.config.midBytes));
        let lastChunk: Buffer | null = null;
        stream.on('data', (data: Buffer) => {
            if ((stream.bytesRead - data.length) == 0) {
                features.withStart(data.slice(0, this.config.begBytes), 0);
            }

            const start = stream.bytesRead - (data.length + (lastChunk?.length || 0));
            if (stream.bytesRead >= halfpointCap && start <= halfpoint) {
                const chunk = (lastChunk != null) ? Buffer.concat([lastChunk, data]) : data;
                const halfStart = Math.max(0, halfpoint - start);
                const halfChunk = chunk.subarray(halfStart, halfStart + this.config.midBytes);
                features.withMiddle(halfChunk, this.config.midBytes / 2 - halfChunk.length / 2);
            }

            if (stream.bytesRead == length) {
                const chunk = (lastChunk != null) ? Buffer.concat([lastChunk, data]) : data;
                const endChunk = chunk.subarray(Math.max(0, chunk.length - this.config.endBytes));
                const endOffset = Math.max(0, this.config.endBytes - endChunk.length);
                features.withEnd(endChunk, endOffset);
            }
            lastChunk = data;
        });
        await finished(stream);
        return this.model.generateResultFromPrediction(this.model.predict(features.toArray()));
    }

}