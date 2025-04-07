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

import { beforeAll, afterAll, describe, expect, it } from "@jest/globals";
import * as fs from "fs";
import { mkdtemp, rm, writeFile } from "fs/promises";
import path from "path";
import * as os from "os";
import { MagikaNode as Magika } from "../magika-node";
import { ContentTypeLabel } from "../src/content-type-label";
import { OverwriteReason } from "../src/overwrite-reason";
import { PredictionMode } from "../src/prediction-mode";
import { Status } from "../src/status";
import * as utils from "./utils";

const EXAMPLES_BY_PATH_INFO: Array<[string, ExampleByPath]> = (() => {
  const result: Array<[string, ExampleByPath]> = [];
  for (const example of parseGzippedExamplesByPath()) {
    result.push([example.path, example]);
  }
  return result;
})();

const EXAMPLES_BY_CONTENT: ExamplesByContent = [
  ...parseGzippedExamplesByContent(),
];

describe("Magika -- inference vs. reference", () => {
  let magika: Magika;
  const repoRootDir = "../";
  let workdir = "";

  beforeAll(async () => {
    magika = await Magika.create();
    workdir = await mkdtemp(path.join(os.tmpdir(), "magika-"));
  });

  afterAll(async () => {
    // Make sure we would only delete a tmp dir.
    if (workdir && workdir.startsWith("/tmp/")) {
      await rm(workdir, { recursive: true, force: true });
    }
  });

  it.each(EXAMPLES_BY_PATH_INFO)(
    'check inference vs. reference - examples_by_path for "%s"',
    async (examplePath, exampleByPath) => {
      if (exampleByPath.prediction_mode != PredictionMode.HIGH_CONFIDENCE) {
        // We only support HIGH_CONFIDENCE mode for now.
        return;
      }

      const fileBytes = fs.readFileSync(repoRootDir + exampleByPath.path);
      let tempFilePath = path.join(workdir, "file.bin");
      await writeFile(tempFilePath, fileBytes);

      const result = await magika.identifyBytes(fileBytes);
      const resultByStream = await magika.identifyStream(
        fs.createReadStream(tempFilePath),
        fileBytes.length,
      );
      expect(result).toStrictEqual(resultByStream);

      expect(result.path).toBe("-");
      expect(result.status).toBe(exampleByPath.status);
      expect(result.prediction.dl.label).toBe(exampleByPath.prediction?.dl);
      expect(result.prediction.output.label).toBe(
        exampleByPath.prediction?.output,
      );
      expect(result.prediction.score).toBeCloseTo(
        exampleByPath.prediction!.score,
      );
      expect(result.prediction.overwrite_reason).toBe(
        exampleByPath.prediction?.overwrite_reason,
      );
    },
  );

  it.each(EXAMPLES_BY_CONTENT)(
    "check inference vs. reference - examples_by_content",
    async (exampleByContent) => {
      if (exampleByContent.prediction_mode != PredictionMode.HIGH_CONFIDENCE) {
        // We only support HIGH_CONFIDENCE mode for now.
        return;
      }

      const fileBytes = Buffer.from(exampleByContent.content_base64, "base64");
      let tempFilePath = path.join(workdir, "fileBytes.bin");
      await writeFile(tempFilePath, fileBytes);

      const result = await magika.identifyBytes(fileBytes);
      const resultByStream = await magika.identifyStream(
        fs.createReadStream(tempFilePath),
        fileBytes.length,
      );
      expect(result).toStrictEqual(resultByStream);

      expect(result.status).toBe(exampleByContent.status);
      expect(result.prediction.score).toBeCloseTo(
        exampleByContent.prediction!.score,
        1,
      );
      expect(result.prediction.dl.label).toBe(exampleByContent.prediction?.dl);
      expect(result.prediction.output.label).toBe(
        exampleByContent.prediction?.output,
      );
      expect(result.prediction.overwrite_reason).toBe(
        exampleByContent.prediction?.overwrite_reason,
      );
    },
  );
});

interface Prediction {
  dl: ContentTypeLabel;
  output: ContentTypeLabel;
  score: number; // Python float maps to TypeScript number
  overwrite_reason: OverwriteReason; // Keep snake_case to match JSON
}

interface ExampleByPath {
  prediction_mode: PredictionMode;
  path: string;
  status: Status;
  prediction: Prediction | null;
}

type ExamplesByPath = ExampleByPath[];

interface ExampleByContent {
  prediction_mode: PredictionMode;
  content_base64: string;
  status: Status;
  prediction: Prediction | null;
}

type ExamplesByContent = ExampleByContent[];

function parseGzippedExamplesByPath(): ExamplesByPath {
  const parsedData = utils.parseGzippedJSON(
    "../tests_data/reference/standard_v3_2-inference_examples_by_path.json.gz",
  );
  const examplesByPath = parsedData as ExamplesByPath;
  for (const exampleByPath of examplesByPath) {
    if (
      !validatePredictionMode(exampleByPath.prediction_mode) ||
      !validatePrediction(exampleByPath.prediction ?? undefined)
    ) {
      const error_msg = `Error parsing: ${JSON.stringify(exampleByPath)}`;
      throw new Error(error_msg);
    }
  }
  return examplesByPath;
}

function parseGzippedExamplesByContent(): ExamplesByContent {
  const parsedData = utils.parseGzippedJSON(
    "../tests_data/reference/standard_v3_2-inference_examples_by_content.json.gz",
  );
  const examplesByContent = parsedData as ExamplesByContent;
  for (const exampleByContent of examplesByContent) {
    if (
      !validatePredictionMode(exampleByContent.prediction_mode) ||
      !validatePrediction(exampleByContent.prediction ?? undefined)
    ) {
      const error_msg = `Error parsing: ${JSON.stringify(exampleByContent)}`;
      throw new Error(error_msg);
    }
  }
  return examplesByContent;
}

function validatePredictionMode(prediction_mode: PredictionMode): boolean {
  return Object.values(PredictionMode).includes(prediction_mode);
}

function validatePrediction(prediction?: Prediction): boolean {
  if (prediction === undefined) {
    return true;
  }

  if (!Object.values(ContentTypeLabel).includes(prediction.dl)) {
    return false;
  }
  if (!Object.values(ContentTypeLabel).includes(prediction.output)) {
    return false;
  }
  if (!Object.values(OverwriteReason).includes(prediction.overwrite_reason)) {
    return false;
  }
  return true;
}
