
import {jest} from '@jest/globals';

export class TfnMock {

    static accessed: Record<string, number> = {};

    static mock = jest.mock('@tensorflow/tfjs-node', () => {
        const hook = {};
        const original = jest.requireActual('@tensorflow/tfjs-node') as any;
        Object.keys(original as object).forEach((key) => {
            TfnMock.accessed[key] = 0;
            Object.defineProperty(hook, key, {
                configurable: true, // allow spyOn to work
                enumerable: true, // so the key shows up
                get(): any {
                    TfnMock.accessed[key] = (TfnMock.accessed[key] || 0) + 1;
                    return original[key];
                }
            });
        });
        return hook;
    }, {virtual: true});

    static reset() {
        for (const i in TfnMock.accessed) {
            TfnMock.accessed[i] = 0;
        }
    }

}