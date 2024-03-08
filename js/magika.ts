
import {ContentType} from './src/contentType.js';
import {Config} from './src/config.js';
import {Model} from './src/model.js';
import {ModelFeatures} from './src/moduleFeatures.js';
import {ModelResult, ModelResultLabels, ModelResultScores} from './src/model.js';
import {MagikaOptions} from './src/magikaOptions.js';

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

    constructor() {
        this.config = new Config();
        this.model = new Model(this.config);
    }

    static CONFIG_URL = 'https://google.github.io/magika/model/config.json';
    static MODEL_URL = 'https://google.github.io/magika/model/model.json';

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
            (this.config.loadUrl(options?.configURL || Magika.CONFIG_URL)),
            (this.model.loadUrl(options?.modelURL || Magika.MODEL_URL))
        ]);
    }

    /** Identifies the content type of a byte array, returning all probabilities instead of just the top one.
     *
     * @param {*} fileBytes a Buffer object (a fixed-length sequence of bytes)
     * @returns A dictionary containing the top label, its score, and a list of content types and their scores.
     */
    async identifyBytesFull(fileBytes: Uint16Array | Uint8Array): Promise<ModelResultLabels> {
        const result = await this._identifyFromBytes(fileBytes);
        return this._getLabelsResult(result);
    }

    /** Identifies the content type of a byte array.
     *
     * @param {*} fileBytes a Buffer object (a fixed-length sequence of bytes)
     * @returns A dictionary containing the top label and its score
     */
    async identifyBytes(fileBytes: Uint16Array | Uint8Array): Promise<ModelResult> {
        const result = await this._identifyFromBytes(fileBytes);
        return {label: result.label, score: result.score};
    }

    _getLabelsResult(result: ModelResultScores): ModelResultLabels {
        const labels = [
            ...Object.values(this.config.labels).map((label) => label.name),
            ...Object.values(ContentType),
        ].map((label, i) => [label, (label == result.label) ? result.score : (result.scores[i] || 0)]);
        return {label: result.label, score: result.score, labels: Object.fromEntries(labels)};
    }

    _getResultForAFewBytes(fileBytes: Uint16Array | Uint8Array): ModelResultScores {
        if (fileBytes.length === 0) {
            return {score: 1.0, label: ContentType.EMPTY, scores: new Uint8Array()};
        }
        const decoder = new TextDecoder('utf-8', {fatal: true});
        try {
            decoder.decode(fileBytes);
            return {score: 1.0, label: ContentType.GENERIC_TEXT, scores: new Uint8Array()};
        } catch (error) {
            return {score: 1.0, label: ContentType.UNKNOWN, scores: new Uint8Array()};
        }
    }

    async _identifyFromBytes(fileBytes: Uint16Array | Uint8Array): Promise<ModelResultScores> {
        if (fileBytes.length <= this.config.minFileSizeForDl) {
            return this._getResultForAFewBytes(fileBytes);
        }
        const fileArray = new Uint16Array(fileBytes);

        // Middle chunk. Padding on either side.
        const halfpoint = Math.round(fileArray.length / 2);
        const startHalf = Math.max(0, halfpoint - this.config.midBytes / 2);
        const halfChunk = fileArray.slice(startHalf, startHalf + this.config.midBytes);

        // End chunk. It should end with the file, and padding at the beginning.
        const endChunk = fileArray.slice(Math.max(0, fileArray.length - this.config.endBytes));
        const endOffset = Math.max(0, this.config.endBytes - endChunk.length);

        const features = new ModelFeatures(this.config)
            .withStart(fileArray.slice(0, this.config.begBytes), 0)  // Beginning chunk. It should start with the file, and padding at the end.
            .withMiddle(halfChunk, this.config.midBytes / 2 - halfChunk.length / 2)
            .withEnd(endChunk, endOffset);

        return this.model.generateResultFromPrediction(this.model.predict(features.toArray()));
    }

}
