import { jest } from '@jest/globals';
import { TfnMock } from './tfnHook';
import * as fs from 'fs';
import * as path from 'path';
import { mkdtemp, rm, readFile } from 'fs/promises';
import * as os from 'os';
import { Readable } from 'stream';
import { finished } from 'stream/promises';
import { ReadableStream } from 'stream/web';
import { MagikaNode as Magika } from '../magika_node';
import * as fc from 'fast-check';
import { readdirSync, Dirent } from 'fs';
import { expect, describe, it, beforeAll, afterAll, beforeEach } from '@jest/globals';


/**
 * Returns a list of test files and their correct labels.
 * 
 * @param directory the directory to recursively scan for test files. 
 * @returns the list of file paths and labels.
 */
const getTestFilesWithLabels = (directory: string): Array<[string, Dirent]> => readdirSync(
    directory,
    { recursive: true, withFileTypes: true })
    .filter(dirent => dirent.isFile())
    .map<[string, Dirent]>((dirent) => [dirent.parentPath.split('/').pop() || 'UNKNOWN', dirent])

/** 
 * Array of all our test files and their labels.
 */
const TEST_FILES: Array<[string, Dirent]> = [
    ...(getTestFilesWithLabels('../tests_data/basic')),
    ...(getTestFilesWithLabels('../tests_data/mitra'))
];

/**
 * File types for Magika V2. Skip them in the tests for now.
 */
const SKIP_FUTURE_CONTENT_TYPES = new Set(['dockerfile', 'toml', 'typescript', 'yara'])

describe('Magika class', () => {

    const workdir = {
        root: '',
        config: '',
        model: ''
    };
    beforeAll(async () => {
        workdir.root = await mkdtemp(path.join(os.tmpdir(), 'magika-'));
        workdir.config = path.join(workdir.root, 'config.json');
        workdir.model = path.join(workdir.root, 'model.json');

        const config = Readable.fromWeb((await fetch(Magika.CONFIG_URL)).body as ReadableStream<any>);
        const model = Readable.fromWeb((await fetch(Magika.MODEL_URL)).body as ReadableStream<any>);
        await Promise.all([
            await finished(config.pipe(fs.createWriteStream(workdir.config))),
            await finished(model.pipe(fs.createWriteStream(workdir.model)))
        ]);
        const weights = JSON.parse((await readFile(workdir.model)).toString()).weightsManifest
            .filter((weights: { paths?: string[] }) => (weights?.paths != null))
            .map((weights: { paths: string[] }) => {
                return weights.paths.map((path) => {
                    return {
                        name: path,
                        url: Magika.MODEL_URL.replace(/model\.json$/, path)
                    }
                });
            })
            .flat();
        await Promise.all(weights.map(async (weight: { name: string, url: string }) => {
            const config = Readable.fromWeb((await fetch(weight.url)).body as ReadableStream<any>);
            await finished(config.pipe(fs.createWriteStream(path.join(workdir.root, weight.name))))
        }));

    });

    beforeEach(async () => {
        TfnMock.reset();
    });

    afterAll(async () => {
        if (workdir.root) {
            await rm(workdir.root, { recursive: true, force: true });
        }
    });

    it('should load default model from url', async () => {
        const magika = new Magika();
        await magika.load();
        expect(magika.model.model).toBeDefined();
        expect(magika.config.labels.length).toBeGreaterThan(0);
        expect(Object.values(TfnMock.accessed).reduce((a, b) => a + b, 0)).toBe(0);
    });

    it('should load model from file path', async () => {
        const magika = new Magika();
        await magika.load({ configPath: workdir.config, modelPath: workdir.model });
        expect(magika.model.model).toBeDefined();
        expect(magika.config.labels.length).toBeGreaterThan(0);
        expect(TfnMock.accessed.io).toBe(1);
        expect(Object.values(TfnMock.accessed).reduce((a, b) => a + b, 0)).toBe(1);
    });


    it('scores should be in the expected range', async () => {
        const magika = new Magika();
        await magika.load();
        fc.assert(
            fc.asyncProperty(
                fc.array(
                    fc.integer({ min: 0, max: 255 }),
                    { minLength: 0, maxLength: 10 })
                ,
                async (bytesContent) => {
                    const output = await magika.identifyBytes(Uint8Array.from(bytesContent));
                    expect(output.score).toBeGreaterThanOrEqual(0);
                    expect(output.score).toBeLessThanOrEqual(1);
                }
            ))
    });

    it('features should result in known value', async () => {
        const magika = new Magika();
        await magika.load({ configPath: workdir.config, modelPath: workdir.model });
        const featuresMock = jest.spyOn(magika.model, 'predict');

        const streamResult = await magika.identifyStream(
            fs.createReadStream('../tests_data/basic/javascript/code.js'),
            (await fs.promises.stat('../tests_data/basic/javascript/code.js')).size
        );

        const input = await fs.promises.readFile('../tests_data/basic/javascript/code.js');
        const byteResult = await magika.identifyBytes(input);
        expect(streamResult.label).toBe(byteResult.label);
        expect(featuresMock.mock.calls[0][0]).toStrictEqual(featuresMock.mock.calls[1][0]);
        const featuresChunk = [
            40, 102, 117, 110, 99, 116, 105, 111, 110, 40, 41, 123, 47, 42, 10, 10, 32, 67, 111, 112, 121, 114, 105, 103, 104, 116, 32, 84, 104, 101,
            32, 67, 108, 111, 115, 117, 114, 101, 32, 76, 105, 98, 114, 97, 114, 121, 32, 65, 117, 116, 104, 111, 114, 115, 46, 10, 32, 83, 80, 68,
            88, 45, 76, 105, 99, 101, 110, 115, 101, 45, 73, 100, 101, 110, 116, 105, 102, 105, 101, 114, 58, 32, 65, 112, 97, 99, 104, 101, 45, 50,
            46, 48, 10, 42, 47, 10, 118, 97, 114, 32, 110, 61, 116, 104, 105, 115, 124, 124, 115, 101, 108, 102, 44, 112, 61, 102, 117, 110, 99, 116,
            105, 111, 110, 40, 97, 44, 98, 41, 123, 97, 61, 97, 46, 115, 112, 108, 105, 116, 40, 34, 46, 34, 41, 59, 118, 97, 114, 32, 99, 61,
            110, 59, 97, 91, 48, 93, 105, 110, 32, 99, 124, 124, 34, 117, 110, 100, 101, 102, 105, 110, 101, 100, 34, 61, 61, 116, 121, 112, 101, 111,
            102, 32, 99, 46, 101, 120, 101, 99, 83, 99, 114, 105, 112, 116, 124, 124, 99, 46, 101, 120, 101, 99, 83, 99, 114, 105, 112, 116, 40, 34,
            118, 97, 114, 32, 34, 43, 97, 91, 48, 93, 41, 59, 102, 111, 114, 40, 118, 97, 114, 32, 100, 59, 97, 46, 108, 101, 110, 103, 116, 104,
            38, 38, 40, 100, 61, 97, 46, 115, 104, 105, 102, 116, 40, 41, 41, 59, 41, 97, 46, 108, 101, 110, 103, 116, 104, 124, 124, 118, 111, 105,
            100, 32, 48, 61, 61, 61, 98, 63, 99, 61, 99, 91, 100, 93, 38, 38, 99, 91, 100, 93, 33, 61, 61, 79, 98, 106, 101, 99, 116, 46,
            112, 114, 111, 116, 111, 116, 121, 112, 101, 91, 100, 93, 63, 99, 91, 100, 93, 58, 99, 91, 100, 93, 61, 123, 125, 58, 99, 91, 100, 93,
            61, 98, 125, 59, 102, 117, 110, 99, 116, 105, 111, 110, 32, 113, 40, 41, 123, 102, 111, 114, 40, 118, 97, 114, 32, 97, 61, 114, 44, 98,
            61, 123, 125, 44, 99, 61, 48, 59, 99, 60, 97, 46, 108, 101, 110, 103, 116, 104, 59, 43, 43, 99, 41, 98, 91, 97, 91, 99, 93, 93,
            61, 99, 59, 114, 101, 116, 117, 114, 110, 32, 98, 125, 102, 117, 110, 99, 116, 105, 111, 110, 32, 117, 40, 41, 123, 118, 97, 114, 32, 97,
            61, 34, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 34, 59,
            97, 43, 61, 97, 46, 116, 111, 76, 111, 119, 101, 114, 67, 97, 115, 101, 40, 41, 43, 34, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57,
            45, 95, 34, 59, 114, 101, 116, 117, 114, 110, 32, 97, 43, 34, 46, 34, 125, 118, 97, 114, 32, 114, 44, 118, 59, 10, 102, 117, 110, 99,
            116, 105, 110, 34, 44, 34, 100, 108, 34, 44, 34, 34, 41, 44, 108, 98, 61, 83, 40, 34, 114, 101, 102, 101, 114, 114, 101, 114, 34, 44,
            34, 100, 114, 34, 41, 44, 109, 98, 61, 83, 40, 34, 112, 97, 103, 101, 34, 44, 34, 100, 112, 34, 44, 34, 34, 41, 59, 83, 40, 34,
            104, 111, 115, 116, 110, 97, 109, 101, 34, 44, 34, 100, 104, 34, 41, 59, 10, 118, 97, 114, 32, 110, 98, 61, 83, 40, 34, 108, 97, 110,
            103, 117, 97, 103, 101, 34, 44, 34, 117, 108, 34, 41, 44, 111, 98, 61, 83, 40, 34, 101, 110, 99, 111, 100, 105, 110, 103, 34, 44, 34,
            100, 101, 34, 41, 44, 113, 102, 61, 83, 40, 34, 116, 105, 116, 108, 101, 34, 44, 34, 100, 116, 34, 44, 102, 117, 110, 99, 116, 105, 111,
            110, 40, 41, 123, 114, 101, 116, 117, 114, 110, 32, 77, 46, 116, 105, 116, 108, 101, 124, 124, 118, 111, 105, 100, 32, 48, 125, 41, 59, 99,
            98, 40, 34, 99, 111, 110, 116, 101, 110, 116, 71, 114, 111, 117, 112, 40, 91, 48, 45, 57, 93, 43, 41, 34, 44, 102, 117, 110, 99, 116,
            105, 111, 110, 40, 97, 41, 123, 114, 101, 116, 117, 114, 110, 32, 110, 101, 119, 32, 98, 98, 40, 97, 91, 48, 93, 44, 34, 99, 103, 34,
            43, 97, 91, 49, 93, 41, 125, 41, 59, 118, 97, 114, 32, 112, 98, 61, 83, 40, 34, 115, 99, 114, 101, 101, 110, 67, 111, 108, 111, 114,
            115, 34, 44, 34, 115, 100, 34, 41, 44, 113, 98, 61, 83, 40, 34, 115, 99, 114, 101, 101, 110, 82, 101, 115, 111, 108, 117, 116, 105, 111,
            110, 34, 44, 34, 115, 114, 34, 41, 44, 114, 98, 61, 83, 40, 34, 118, 105, 101, 119, 112, 111, 114, 116, 83, 105, 122, 101, 34, 44, 34,
            118, 112, 34, 41, 44, 115, 98, 61, 83, 40, 34, 106, 97, 118, 97, 69, 110, 97, 98, 108, 101, 100, 34, 44, 34, 106, 101, 34, 41, 44,
            116, 98, 61, 83, 40, 34, 102, 108, 97, 115, 104, 86, 101, 114, 115, 105, 111, 110, 34, 44, 34, 102, 108, 34, 41, 59, 83, 40, 34, 99,
            97, 109, 112, 97, 105, 103, 110, 73, 100, 34, 44, 34, 99, 105, 34, 41, 59, 83, 40, 34, 99, 97, 109, 112, 97, 105, 103, 110, 78, 97,
            109, 101, 34, 44, 34, 99, 110, 34, 41, 59, 83, 40, 34, 99, 97, 109, 112, 97, 105, 103, 110, 83, 111, 117, 114, 99, 101, 34, 44, 34,
            99, 115, 34, 41, 59, 83, 40, 34, 99, 97, 109, 112, 97, 105, 103, 110, 77, 101, 100, 105, 117, 109, 34, 44, 34, 99, 109, 34, 41, 59,
            83, 40, 34, 99, 97, 109, 112, 97, 105, 103, 110, 75, 101, 121, 119, 111, 114, 100, 34, 44, 34, 99, 107, 34, 41, 59, 83, 40, 34, 99,
            97, 109, 112, 97, 59, 88, 40, 34, 115, 101, 116, 34, 44, 98, 44, 98, 46, 115, 101, 116, 41, 59, 105, 102, 40, 34, 104, 116, 116, 112,
            115, 58, 34, 33, 61, 77, 46, 108, 111, 99, 97, 116, 105, 111, 110, 46, 112, 114, 111, 116, 111, 99, 111, 108, 38, 38, 33, 66, 97, 41,
            123, 97, 58, 123, 98, 61, 77, 46, 103, 101, 116, 69, 108, 101, 109, 101, 110, 116, 115, 66, 121, 84, 97, 103, 78, 97, 109, 101, 40, 34,
            115, 99, 114, 105, 112, 116, 34, 41, 59, 10, 102, 111, 114, 40, 99, 61, 48, 59, 99, 60, 98, 46, 108, 101, 110, 103, 116, 104, 38, 38,
            49, 48, 48, 62, 99, 59, 99, 43, 43, 41, 123, 118, 97, 114, 32, 100, 61, 98, 91, 99, 93, 46, 115, 114, 99, 59, 105, 102, 40, 100,
            38, 38, 48, 61, 61, 100, 46, 105, 110, 100, 101, 120, 79, 102, 40, 98, 100, 40, 33, 48, 41, 43, 34, 47, 97, 110, 97, 108, 121, 116,
            105, 99, 115, 34, 41, 41, 123, 98, 61, 33, 48, 59, 98, 114, 101, 97, 107, 32, 97, 125, 125, 98, 61, 33, 49, 125, 98, 38, 38, 40,
            66, 97, 61, 33, 48, 41, 125, 40, 79, 46, 103, 97, 112, 108, 117, 103, 105, 110, 115, 61, 79, 46, 103, 97, 112, 108, 117, 103, 105, 110,
            115, 124, 124, 123, 125, 41, 46, 76, 105, 110, 107, 101, 114, 61, 68, 99, 59, 98, 61, 68, 99, 46, 112, 114, 111, 116, 111, 116, 121, 112,
            101, 59, 67, 40, 34, 108, 105, 110, 107, 101, 114, 34, 44, 68, 99, 41, 59, 88, 40, 34, 100, 101, 99, 111, 114, 97, 116, 101, 34, 44,
            98, 44, 98, 46, 99, 97, 44, 50, 48, 41, 59, 88, 40, 34, 97, 117, 116, 111, 76, 105, 110, 107, 34, 44, 98, 44, 98, 46, 83, 44,
            50, 53, 41, 59, 88, 40, 34, 112, 97, 115, 115, 116, 104, 114, 111, 117, 103, 104, 34, 44, 98, 44, 98, 46, 36, 44, 50, 53, 41, 59,
            67, 40, 34, 100, 105, 115, 112, 108, 97, 121, 102, 101, 97, 116, 117, 114, 101, 115, 34, 44, 102, 100, 41, 59, 67, 40, 34, 97, 100, 102,
            101, 97, 116, 117, 114, 101, 115, 34, 44, 102, 100, 41, 59, 90, 46, 68, 46, 97, 112, 112, 108, 121, 40, 78, 44, 97, 41, 125, 125, 59,
            118, 97, 114, 32, 120, 102, 61, 78, 46, 78, 44, 121, 102, 61, 79, 91, 103, 98, 93, 59, 121, 102, 38, 38, 121, 102, 46, 114, 63, 120,
            102, 40, 41, 58, 122, 40, 120, 102, 41, 59, 122, 40, 102, 117, 110, 99, 116, 105, 111, 110, 40, 41, 123, 90, 46, 68, 40, 91, 34, 112,
            114, 111, 118, 105, 100, 101, 34, 44, 34, 114, 101, 110, 100, 101, 114, 34, 44, 117, 97, 93, 41, 125, 41, 59, 125, 41, 40, 119, 105, 110,
            100, 111, 119, 41, 59, 10
        ];
        expect(featuresMock.mock.calls[0][0]).toStrictEqual(featuresChunk);
        expect(TfnMock.accessed.io).toBe(1);
        expect(Object.values(TfnMock.accessed).reduce((a, b) => a + b, 0)).toBe(1);
    });

    it.each(TEST_FILES)('stream and byte should return the same features for "%s"', async (label, testFile) => {
        if (SKIP_FUTURE_CONTENT_TYPES.has(label)) return;
        const magika = new Magika();
        await magika.load({ configPath: workdir.config, modelPath: workdir.model });
        const featuresMock = jest.spyOn(magika.model, 'predict');
        const filePath = path.join(testFile.parentPath, testFile.name)
        const streamResult = await magika.identifyStream(
            fs.createReadStream(filePath),
            (await fs.promises.stat(filePath)).size
        );
        const input = await fs.promises.readFile(filePath);
        const byteResult = await magika.identifyBytes(input);
        expect(streamResult.label).toBe(byteResult.label);
        expect(featuresMock.mock.calls[0][0]).toStrictEqual(featuresMock.mock.calls[1][0]);
        expect(streamResult.label).toBe(label);
        expect(TfnMock.accessed.io).toBe(1);
        expect(Object.values(TfnMock.accessed).reduce((a, b) => a + b, 0)).toBe(1);
    });

    it.each(TEST_FILES)('Magika is agnostic to the format of the input bytes for "%s"', async (label, testFile) => {
        if (SKIP_FUTURE_CONTENT_TYPES.has(label)) return;
        const magika = new Magika();
        await magika.load({ configPath: workdir.config, modelPath: workdir.model });
        const featuresMock = jest.spyOn(magika.model, 'predict');
        const filePath = path.join(testFile.parentPath, testFile.name)
        const inputBuffer = await fs.promises.readFile(filePath);
        const inputUint8 = new Uint8Array(inputBuffer);
        const inputUint16 = new Uint16Array(inputBuffer);
        const resultFromBuffer = await magika.identifyBytes(inputBuffer);
        const resultFromUint8 = await magika.identifyBytes(inputUint8);
        const resultFromUint16 = await magika.identifyBytes(inputUint16);
        expect(resultFromBuffer.label).toBe(resultFromUint8.label);
        expect(resultFromBuffer.label).toBe(resultFromUint16.label);
    });
});