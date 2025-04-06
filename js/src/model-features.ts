// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
