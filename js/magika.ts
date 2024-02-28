import {ContentType} from './src/contentType';
import {Config} from './src/config';
import {Model} from './src/model';
import {ModelFeatures} from './src/moduleFeatures';
import {ModelResult, ModelResultLabels, ModelResultScores} from './src/model';

class MagikaOptions {

    modelURL?: string;
    configURL?: string

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

    config: Config;
    model: Model;

    static async create(options?: MagikaOptions): Promise<Magika> {
        const magika = new Magika();
        await magika.load(options);
        return magika;
    }

    /** Loads the Magika model and config from URLs.
     *
     * @param {string} modelURL The URL where the model is stored.
     * @param {string} configURL The URL where the config is stored.
     *
     * Both parameters are optional. If not provided, the model will be loaded from Github.
     *
     */
    async load(options?: MagikaOptions): Promise<void> {
        this.config = new Config();
        this.model = new Model(this.config);
        await Promise.all([
            this.config.load(options?.configURL || 'https://google.github.io/magika/model/config.json'),
            this.model.load(options?.modelURL || 'https://google.github.io/magika/model/model.json')
        ]);
    }

    async identifyStream(stream, length: number): Promise<ModelResultScores> {
        const features = new ModelFeatures(this.config);

        const halfMid = Math.round(this.config.midBytes / 2);
        const halfpoint = Math.round(length / 2) - halfMid;
        let lastChunk = null;
        stream.on('data', (data) => {
            if ((stream.bytesRead - data.length) == 0) {
                features.withStart(data.slice(0, this.config.begBytes), 0);
            }

            const start = stream.bytesRead - (data.length + (lastChunk?.length || 0));
            if (stream.bytesRead > (halfpoint + this.config.midBytes) && start < (halfpoint - halfMid)) {
                const chunk = (lastChunk != null) ? Buffer.concat([lastChunk, data]) : data;
                const halfStart = Math.max(0, halfpoint - start);
                const halfChunk = chunk.slice(halfStart, halfStart + this.config.midBytes);
                features.withMiddle(halfChunk, this.config.midBytes / 2 - halfChunk.length / 2);
            }
            
            if (stream.bytesRead == length) {
                const chunk = (lastChunk != null) ? Buffer.concat([lastChunk, data]) : data;
                const endChunk = chunk.slice(Math.max(0, chunk.length - this.config.endBytes));
                const endOffset = Math.max(0, this.config.endBytes - endChunk.length);
                features.withEnd(endChunk, endOffset);
            }
            lastChunk = data;
        });
        await new Promise<void>((resolve) => stream.on('end', resolve));
        return this.model.generateResultFromPrediction(this.model.predict(features.toArray()));
    }

    async identifyStreamFull(stream, length: number): Promise<ModelResultLabels> {
        const result = await this.identifyStream(stream, length);
        return this.getLabelsResult(result);
    }

    /** Identifies the content type of a byte stream, returning all probabilities instead of just the top one.
     *
     * @param {*} fileBytes  a Buffer object (a fixed-length sequence of bytes)
     * @returns A dictionary containing the top label, its score, and a list of content types and their scores.
     */
    async identifyBytesFull(fileBytes: Uint16Array | Buffer): Promise<ModelResultLabels> {
        const result = await this.identifyWithBytes(fileBytes);
        return this.getLabelsResult(result);
    }

    /** Identifies the content type of a byte stream.
     *
     * @param {*} fileBytes  a Buffer object (a fixed-length sequence of bytes)
     * @returns A dictionary containing the top label and its score,
     */
    async identifyBytes(fileBytes: Uint16Array | Buffer): Promise<ModelResult> {
        const result = await this.identifyWithBytes(fileBytes);
        return {label: result.label, score: result.score};
    }

    getLabelsResult(result: ModelResultScores): ModelResultLabels {
        const labels = [
            ...Object.values(this.config.labels).map((label) => label.name),
            ...Object.values(ContentType),
        ].map((label, i) => [label, (label == result.label) ? result.score : (result.scores[i] || 0)]);
        return {label: result.label, score: result.score, lables: Object.fromEntries(labels)};
    }

    getResultForAFewBytes(fileBytes: Uint16Array | Buffer): ModelResultScores {
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

    async identifyWithBytes(fileBytes: Uint16Array | Buffer): Promise<ModelResultScores> {
        if (fileBytes.length <= this.config.minFileSizeForDl) {
            return this.getResultForAFewBytes(fileBytes);
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
