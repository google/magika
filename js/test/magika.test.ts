import {jest} from '@jest/globals';
import * as fs from 'fs';
import * as path from 'path';
import {mkdtemp, rm, readFile} from 'fs/promises';
import * as os from 'os';
import {Readable} from 'stream';
import {finished} from 'stream/promises';
import {ReadableStream} from 'stream/web';
import {Magika} from '../magika';

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
            .filter((weights: {paths?: string[]}) => (weights?.paths != null))
            .map((weights: {paths: string[]}) => {
                return weights.paths.map((path) => {
                    return {
                        name: path,
                        url: Magika.MODEL_URL.replace(/model\.json$/, path)
                    }
                });
            })
            .flat();
        await Promise.all(weights.map(async (weight: {name: string, url: string}) => {
            const config = Readable.fromWeb((await fetch(weight.url)).body as ReadableStream<any>);
            await finished(config.pipe(fs.createWriteStream(path.join(workdir.root, weight.name))))
        }));
    });

    afterAll(async () => {
        if (workdir.root) {
            await rm(workdir.root, {recursive: true, force: true});
        }
    });

	it('should load default model from url', async () => {
        const magika = new Magika();
        await magika.load();
        expect(magika.model.model).toBeDefined();
        expect(magika.config.labels.length).toBeGreaterThan(0);
    });
    
	it('should load model from file path', async () => {
        const magika = new Magika();
        await magika.load({configPath: workdir.config, modelPath: workdir.model});
        expect(magika.model.model).toBeDefined();
        expect(magika.config.labels.length).toBeGreaterThan(0);
    });

    const extMap: Record<string, string> = {
        js: 'javascript',
        py: 'python',
        rs: 'rust',
        pub: 'pem',
        htm: 'txt',
        '7z': 'sevenzip',
        md: 'markdown',
        bz2: 'bzip',
        gz: 'gzip',
        class: 'javabytecode',
        jpg: 'jpeg',
        bplist: 'appleplist',
        plist: 'appleplist',
        pcapng: 'pythonbytecode', // this is wrong I think
        exe: 'pebin',
        mov: 'mp4',
        tif: 'tiff'
    };

    it.each([
        ...fs.readdirSync('../tests_data/basic').map((file) => {
            return {
                name: file,
                filePath: path.join('../tests_data/basic', file)
            };
        }),
        ...fs.readdirSync('../tests_data/mitra').map((file) => {
            return {
                name: file,
                filePath: path.join('../tests_data/mitra', file)
            };
        })
    ])('stream and byte should return the same features for "$name"', async ({name, filePath}) => {
        const magika = new Magika();
        await magika.load({configPath: workdir.config, modelPath: workdir.model});
        const featuresMock = jest.spyOn(magika.model, 'predict');

        const streamResult = await magika.identifyStream(
            fs.createReadStream(filePath),
            (await fs.promises.stat(filePath)).size
        );

        const input = await fs.promises.readFile(filePath);
        const byteResult = await magika.identifyBytes(input);
        expect(streamResult.label).toBe(byteResult.label);
        expect(featuresMock.mock.calls[0][0]).toStrictEqual(featuresMock.mock.calls[1][0]);
        const ext = name.split('.')[1];
        expect(streamResult.label).toBe(extMap[ext] || ext);
    });

});