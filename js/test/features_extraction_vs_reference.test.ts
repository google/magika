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

import { beforeAll, describe, expect, it } from "@jest/globals";
import { MagikaNode as Magika } from "../magika-node.js";
import * as utils from "./utils.js";
import { ModelFeatures } from "../src/model-features.js";

const FEATURES_EXTRACTION_EXAMPLES: FeaturesExtractionExamples = [
  ...parseGzippedFeaturesExtractionExamples(),
];

class TestableMagika extends Magika {
  public static extractFeaturesFromBytes(
    fileBytes: Uint8Array,
    beg_size: number,
    mid_size: number,
    end_size: number,
    padding_token: number,
    block_size: number,
    use_inputs_at_offsets: boolean,
  ): ModelFeatures {
    return Magika._extractFeaturesFromBytes(
      fileBytes,
      beg_size,
      mid_size,
      end_size,
      padding_token,
      block_size,
      use_inputs_at_offsets,
    );
  }
}

describe("Magika -- features extraction vs. reference", () => {
  let magika: Magika;
  const repoRootDir = "../";

  beforeAll(async () => {
    magika = await Magika.create();
  });

  it.each(FEATURES_EXTRACTION_EXAMPLES)(
    "check features extraction vs. reference",
    async (example) => {
      if (example.args.mid_size != 0 || example.args.use_inputs_at_offsets) {
        // We do not support these settings at the moment.
        return;
      }

      const fileBytes = Buffer.from(example.content_base64, "base64");
      const features = TestableMagika.extractFeaturesFromBytes(
        fileBytes,
        example.args.beg_size,
        example.args.mid_size,
        example.args.end_size,
        example.args.padding_token,
        example.args.block_size,
        example.args.use_inputs_at_offsets,
      );

      expect(features.beg_ints).toEqual(new Uint16Array(example.features.beg));
      expect(features.end_ints).toEqual(new Uint16Array(example.features.end));
    },
  );
});

interface FeaturesExtractionExample {
  args: FeaturesExtractionExampleArgs;
  metadata: FeaturesExtractionExampleMetadata;
  content_base64: string;
  features: ExampleModelFeatures;
}

interface FeaturesExtractionExampleArgs {
  beg_size: number;
  mid_size: number;
  end_size: number;
  block_size: number;
  padding_token: number;
  use_inputs_at_offsets: boolean;
}

interface FeaturesExtractionExampleMetadata {
  core_content_size: number;
  left_ws_num: number;
  right_ws_num: number;
}

interface ExampleModelFeatures {
  beg: number[];
  mid: number[];
  end: number[];
  offset_0x8000_0x8007: number[];
  offset_0x8800_0x8807: number[];
  offset_0x9000_0x9007: number[];
  offset_0x9800_0x9807: number[];
}

type FeaturesExtractionExamples = FeaturesExtractionExample[];

function parseGzippedFeaturesExtractionExamples(): FeaturesExtractionExamples {
  const parsedData = utils.parseGzippedJSON(
    "../tests_data/reference/features_extraction_examples.json.gz",
  );
  const featuresExtractionExamples = parsedData as FeaturesExtractionExamples;
  for (const example of featuresExtractionExamples) {
    if (
      example.features.beg.length != example.args.beg_size ||
      example.features.end.length != example.args.end_size
    ) {
      const error_msg = `Error parsing: ${JSON.stringify(example)}`;
      throw new Error(error_msg);
    }
  }
  return featuresExtractionExamples;
}
