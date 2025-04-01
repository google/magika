import { ModelConfig } from "./model-config.js";

export class ModelFeatures {
  beg_ints: Uint16Array;
  end_ints: Uint16Array;
  locked: { beg: boolean; end: boolean };

  constructor(public model_config: ModelConfig) {
    if (this.model_config.mid_size != 0) {
      throw new Error(
        `Assertion failed: This implementation does not support mid_size (${this.model_config.mid_size}) != 0 model config.`,
      );
    }

    this.beg_ints = new Uint16Array(this.model_config.beg_size).fill(
      this.model_config.padding_token,
    );
    this.end_ints = new Uint16Array(this.model_config.end_size).fill(
      this.model_config.padding_token,
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
