import { beforeAll, describe, expect, it } from "@jest/globals";
import { MagikaNode as Magika } from "../magika-node";
import * as utils from "./utils";

const FEATURES_EXTRACTION_EXAMPLES: FeaturesExtractionExamples = [
  ...parseGzippedFeaturesExtractionExamples(),
];

describe("Magika -- features extraction vs. reference", () => {
  let magika: Magika;
  const repoRootDir = "../";

  beforeAll(async () => {
    magika = new Magika();
    await magika.load();
  });

  it.each(FEATURES_EXTRACTION_EXAMPLES)(
    "check features extraction vs. reference",
    async (example) => {
      if (example.args.mid_size != 0 || example.args.use_inputs_at_offsets) {
        // We do not support these settings at the moment.
        return;
      }

      const fileBytes = Buffer.from(example.content_base64, "base64");
      const features = Magika._extract_features_from_bytes(
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
  features: ModelFeatures;
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

interface ModelFeatures {
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
