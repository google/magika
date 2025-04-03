import { ModelConfig } from "./model-config.js";

export class ModelFeatures {
  beg_ints: Uint16Array;
  end_ints: Uint16Array;
  locked: { beg: boolean; end: boolean };

  constructor(
    beg_size: number,
    mid_size: number,
    end_size: number,
    padding_token: number,
    use_inputs_at_offsets: boolean,
  ) {
    if (mid_size != 0) {
      throw new Error(
        `Assertion failed: This implementation does not support mid_size (${mid_size}) != 0 model config.`,
      );
    }
    if (use_inputs_at_offsets) {
      throw new Error(
        `Assertion failed: This implementation does not support use_inputs_at_offsets = true model config.`,
      );
    }

    this.beg_ints = new Uint16Array(beg_size).fill(padding_token);
    this.end_ints = new Uint16Array(end_size).fill(padding_token);
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
