
import * as fs from 'fs';
import {Magika} from '../magika';

describe('Magika class', () => {

	it('should load model', async () => {
        const magika = new Magika();
        await magika.load();
        const featuresMock = jest.spyOn(magika.model, 'predict');

        const streamResult = await magika.identifyStream(
            fs.createReadStream('./package.json'),
            (await fs.promises.stat('./package.json')).size
        );

        const input = await fs.promises.readFile('./package.json');
        const byteResult = await magika.identifyBytes(input);
        expect(streamResult.label).toBe(byteResult.label);
        expect(featuresMock.mock.calls[0][0]).toStrictEqual(featuresMock.mock.calls[1][0]);
    });

});