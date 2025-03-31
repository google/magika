import { assert } from "console";
import { Config } from "./config.js";

export class ModelFeatures {
  beg_ints: Uint16Array;
  end_ints: Uint16Array;
  locked: { beg: boolean; end: boolean };

  constructor(public config: Config) {
    assert(this.config.mid_size == 0);

    this.beg_ints = new Uint16Array(this.config.beg_size).fill(
      this.config.padding_token,
    );
    this.end_ints = new Uint16Array(this.config.end_size).fill(
      this.config.padding_token,
    );
    this.locked = { beg: false, end: false };
  }

  withStart(data: Uint8Array, offset: number): this {
    if (!this.locked.beg) {
      this.locked.beg = true;
      this.beg_ints.set(data, offset);
    }
    return this;
  }

  withEnd(data: Uint8Array, offset: number): this {
    if (!this.locked.end) {
      this.locked.end = true;
      this.end_ints.set(data, offset);
    }
    return this;
  }

  toArray(): number[] {
    return [...this.beg_ints, ...this.end_ints];
  }
}
