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

import { jest } from "@jest/globals";

export class TfnMock {
  static accessed: Record<string, number> = {};

  static mock = jest.mock(
    "@tensorflow/tfjs-node",
    () => {
      const hook = {};
      const original = jest.requireActual("@tensorflow/tfjs-node") as any;
      Object.keys(original as object).forEach((key) => {
        TfnMock.accessed[key] = 0;
        Object.defineProperty(hook, key, {
          configurable: true, // allow spyOn to work
          enumerable: true, // so the key shows up
          get(): any {
            TfnMock.accessed[key] = (TfnMock.accessed[key] || 0) + 1;
            return original[key];
          },
        });
      });
      return hook;
    },
    { virtual: true },
  );

  static reset() {
    for (const i in TfnMock.accessed) {
      TfnMock.accessed[i] = 0;
    }
  }
}
