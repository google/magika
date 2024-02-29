import {Config} from './config.js';

export class ModelFeatures {

    start: Uint16Array;
    middle: Uint16Array;
    end: Uint16Array;
    locked: {start: boolean, middle: boolean, end: boolean};

    constructor(public config: Config) {
        this.start = new Uint16Array(this.config.begBytes).fill(this.config.paddingToken);
        this.middle = new Uint16Array(this.config.midBytes).fill(this.config.paddingToken);
        this.end = new Uint16Array(this.config.endBytes).fill(this.config.paddingToken);
        this.locked = {start: false, middle: false, end: false};
    }

    withStart(data: Uint16Array | Buffer, offset: number): this {
        if (!this.locked.start) {
            this.locked.start = true;
            this.start.set(data, offset);
        }
        return this;
    }
    
    withMiddle(data: Uint16Array | Buffer, offset: number): this {
        if (!this.locked.middle) {
            this.locked.middle = true;
            this.middle.set(data, offset);
        }
        return this;
    }
    
    withEnd(data: Uint16Array | Buffer, offset: number): this {
        if (!this.locked.end) {
            this.locked.end = true;
            this.end.set(data, offset);
        }
        return this;
    }

    toArray(): number[] {
        return [...this.start, ...this.middle, ...this.end];
    }

}