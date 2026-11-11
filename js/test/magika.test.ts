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

import { TfnMock } from "./tfnHook";
// TfnMock must be imported first; leave this line here to avoid imports
// sorting.
import {
  afterAll,
  beforeAll,
  beforeEach,
  describe,
  expect,
  it,
  jest,
} from "@jest/globals";
import * as fc from "fast-check";
import * as fs from "fs";
import { Dirent, readdirSync } from "fs";
import { mkdtemp, readFile, rm } from "fs/promises";
import * as os from "os";
import * as path from "path";
import { Readable } from "stream";
import { finished } from "stream/promises";
import { ReadableStream } from "stream/web";
import {
  type ContentTypeInfo,
  ContentTypeLabel as BrowserContentTypeLabel,
  Magika as BrowserMagika,
} from "../magika";
import {
  type ContentTypeInfo as NodeContentTypeInfo,
  ContentTypeLabel,
  MagikaNode as Magika,
} from "../magika-node";

/**
 * Returns a list of test files and their correct labels.
 *
 * @param directory the directory to recursively scan for test files.
 * @returns the list of file paths and labels.
 */
const getTestFilesWithLabels = (
  directory: string,
): Array<[string, string, Dirent]> =>
  readdirSync(directory, { recursive: true, withFileTypes: true })
    .filter((dirent) => dirent.isFile())
    .map<[string, string, Dirent]>((dirent) => {
      const label = dirent.parentPath.split("/").pop() || "UNKNOWN";
      const filePath = path.join(dirent.parentPath, dirent.name);
      return [label, filePath, dirent];
    });

/**
 * Array of all our test files and their labels.
 */
const BASIC_TEST_FILES: Array<[string, string, Dirent]> = [
  ...getTestFilesWithLabels("../tests_data/basic"),
];

describe("Magika class", () => {
  const workdir = {
    root: "",
    model_config: "",
    model: "",
  };
  beforeAll(async () => {
    workdir.root = await mkdtemp(path.join(os.tmpdir(), "magika-"));
    workdir.model_config = path.join(workdir.root, "config.json");
    workdir.model = path.join(workdir.root, "model.json");

    const model_config = Readable.fromWeb(
      (await fetch(Magika.MODEL_CONFIG_URL)).body as ReadableStream<any>,
    );
    const model = Readable.fromWeb(
      (await fetch(Magika.MODEL_URL)).body as ReadableStream<any>,
    );
    await Promise.all([
      await finished(
        model_config.pipe(fs.createWriteStream(workdir.model_config)),
      ),
      await finished(model.pipe(fs.createWriteStream(workdir.model))),
    ]);
    const weights = JSON.parse((await readFile(workdir.model)).toString())
      .weightsManifest.filter(
        (weights: { paths?: string[] }) => weights?.paths != null,
      )
      .map((weights: { paths: string[] }) => {
        return weights.paths.map((path) => {
          return {
            name: path,
            url: Magika.MODEL_URL.replace(/model\.json$/, path),
          };
        });
      })
      .flat();
    await Promise.all(
      weights.map(async (weight: { name: string; url: string }) => {
        const model_config = Readable.fromWeb(
          (await fetch(weight.url)).body as ReadableStream<any>,
        );
        await finished(
          model_config.pipe(
            fs.createWriteStream(path.join(workdir.root, weight.name)),
          ),
        );
      }),
    );
  });

  beforeEach(async () => {
    TfnMock.reset();
  });

  afterAll(async () => {
    if (workdir.root) {
      await rm(workdir.root, { recursive: true, force: true });
    }
  });

  it("should load default model from url", async () => {
    const magika = await Magika.create();
    expect(magika.model.model).toBeDefined();
    expect(magika.model_config.target_labels_space.length).toBeGreaterThan(0);
    expect(Object.values(TfnMock.accessed).reduce((a, b) => a + b, 0)).toBe(0);
  });

  it("should load model from file path", async () => {
    const magika = await Magika.create({
      modelConfigPath: workdir.model_config,
      modelPath: workdir.model,
    });
    expect(magika.model.model).toBeDefined();
    expect(magika.model_config.target_labels_space.length).toBeGreaterThan(0);
    expect(TfnMock.accessed.io).toBe(1);
    expect(Object.values(TfnMock.accessed).reduce((a, b) => a + b, 0)).toBe(1);
  });

  it("scores should be in the expected range", async () => {
    const magika = await Magika.create();
    fc.assert(
      fc.asyncProperty(
        fc.array(fc.integer({ min: 0, max: 255 }), {
          minLength: 0,
          maxLength: 10,
        }),
        async (bytesContent) => {
          const result = await magika.identifyBytes(
            Uint8Array.from(bytesContent),
          );
          expect(result.prediction.score).toBeGreaterThanOrEqual(0);
          expect(result.prediction.score).toBeLessThanOrEqual(1);
        },
      ),
    );
  });

  it.each(BASIC_TEST_FILES)(
    'by_stream vs by_byte should return the same (correct) features/label for "%s" "%s"',
    async (label, testPath, testFile) => {
      const magika = await Magika.create({
        modelConfigPath: workdir.model_config,
        modelPath: workdir.model,
      });
      const featuresMock = jest.spyOn(magika.model, "predict");

      // Do predictions by stream and by path.
      const filePath = path.join(testFile.parentPath, testFile.name);
      const streamResult = await magika.identifyStream(
        fs.createReadStream(filePath),
        (await fs.promises.stat(filePath)).size,
      );
      const fileBytes = await fs.promises.readFile(filePath);
      const byteResult = await magika.identifyBytes(fileBytes);

      // Compare the results; they should match between them
      expect(streamResult).toStrictEqual(byteResult);
      if (streamResult.prediction.dl.label != ContentTypeLabel.UNDEFINED) {
        expect(featuresMock.mock.calls[0][0]).toStrictEqual(
          featuresMock.mock.calls[1][0],
        );
      }
      // Check that the predictions make the expectations.
      expect(streamResult.prediction.output.label).toBe(label);

      // Check properties on the TfnMock object.
      expect(TfnMock.accessed.io).toBe(1);
      expect(Object.values(TfnMock.accessed).reduce((a, b) => a + b, 0)).toBe(
        1,
      );

      // The predictions are the same via bytes and via stream, let's just take one.
      const prediction = byteResult.prediction;
      expect(prediction).not.toBeUndefined();
      expect(prediction.dl).not.toBeUndefined();
      expect(prediction.output).not.toBeUndefined();
      expect(prediction.score).not.toBeUndefined();

      if (prediction.dl.label == ContentTypeLabel.UNDEFINED) {
        // If dl.label == UNDEFINED, scores_map should not be set.
        expect(prediction.scores_map).toBeUndefined();
      } else {
        // If dl.label is not UNDEFINED, scores_map should be set.
        expect(prediction.scores_map).not.toBeUndefined();
        // Check that the max score and label associated to max score matches
        // what's returned in the prediction.
        const scores = Object.values(prediction?.scores_map ?? {});
        let curr_max_score = scores[0];
        let argmax_idx = 0;
        for (let i = 1; i < scores.length; i++) {
          if (scores[i] > curr_max_score) {
            curr_max_score = scores[i];
            argmax_idx = i;
          }
        }
        const predicted_label =
          magika.model_config.target_labels_space[argmax_idx];
        expect(predicted_label).toBe(prediction.dl.label);
        expect(curr_max_score).toBe(prediction.score);
      }
    },
  );

  it.each(BASIC_TEST_FILES)(
    'Magika is agnostic to the format of the input bytes for "%s" "%s"',
    async (label, testPath, testFile) => {
      const magika = await Magika.create({
        modelConfigPath: workdir.model_config,
        modelPath: workdir.model,
      });
      const featuresMock = jest.spyOn(magika.model, "predict");
      const filePath = path.join(testFile.parentPath, testFile.name);
      const inputBuffer = await fs.promises.readFile(filePath);
      const inputUint8 = new Uint8Array(inputBuffer);
      const resultFromBuffer = await magika.identifyBytes(inputBuffer);
      const resultFromUint8 = await magika.identifyBytes(inputUint8);
      expect(resultFromBuffer).toStrictEqual(resultFromUint8);

      if (resultFromBuffer.prediction.dl.label != ContentTypeLabel.UNDEFINED) {
        expect(featuresMock.mock.calls[0][0]).toStrictEqual(
          featuresMock.mock.calls[1][0],
        );
      }
    },
  );

  it("should expose full ContentTypeInfo metadata for all exposed labels", async () => {
    const exposedPath = path.resolve(
      __dirname,
      "../../assets/exposed_content_types.txt",
    );
    const kbPath = path.resolve(
      __dirname,
      "../../assets/content_types_kb.min.json",
    );
    const expectedExposed = (await readFile(exposedPath, "utf-8"))
      .split(/\r?\n/)
      .map((line) => line.trim())
      .filter((line) => line.length > 0)
      .sort();
    const rawKb = JSON.parse(await readFile(kbPath, "utf-8")) as Record<
      string,
      {
        mime_type: string | null;
        group: string | null;
        description: string | null;
        extensions: string[];
        is_text: boolean;
        in_heuristics: boolean;
        in_resolvers: boolean;
        in_ml_model: boolean;
        vetoed_by_resolvers: boolean;
      }
    >;

    expect(BrowserContentTypeLabel).toBe(ContentTypeLabel);
    expect(Object.values(ContentTypeLabel)).toStrictEqual(expectedExposed);

    for (const ct of Object.values(ContentTypeLabel)) {
      const entry = rawKb[ct];
      expect(entry).toBeDefined();

      const info: ContentTypeInfo = Magika.getContentTypeInfo(ct);
      const nodeInfo: NodeContentTypeInfo = BrowserMagika.getContentTypeInfo(
        ct as string,
      );
      expect(nodeInfo).toStrictEqual(info);

      const expectedMime =
        entry.mime_type ??
        (entry.is_text ? "text/plain" : "application/octet-stream");
      expect(info).toStrictEqual({
        label: ct,
        mime_type: expectedMime,
        group: entry.group ?? "unknown",
        description: entry.description ?? ct,
        extensions: entry.extensions,
        is_text: entry.is_text,
        in_heuristics: entry.in_heuristics,
        in_resolvers: entry.in_resolvers,
        in_ml_model: entry.in_ml_model,
        vetoed_by_resolvers: entry.vetoed_by_resolvers,
      });
      if (info.vetoed_by_resolvers) {
        expect(info.in_resolvers).toBe(true);
      }
    }

    // Spot-check filesystem heuristic, resolver-vetoed ML type, resolver-only
    // type, and inactive exposed labels.
    const dirInfo = Magika.getContentTypeInfo(ContentTypeLabel.DIRECTORY);
    expect(dirInfo.in_heuristics).toBe(true);
    expect(dirInfo.in_resolvers).toBe(false);
    expect(dirInfo.in_ml_model).toBe(false);
    expect(dirInfo.vetoed_by_resolvers).toBe(false);

    const parquetInfo = Magika.getContentTypeInfo("parquet");
    expect(parquetInfo.in_resolvers).toBe(true);
    expect(parquetInfo.in_ml_model).toBe(true);
    expect(parquetInfo.vetoed_by_resolvers).toBe(true);

    const llvmBitcodeInfo = Magika.getContentTypeInfo(
      ContentTypeLabel.LLVM_BITCODE,
    );
    expect(llvmBitcodeInfo.in_heuristics).toBe(false);
    expect(llvmBitcodeInfo.in_resolvers).toBe(true);
    expect(llvmBitcodeInfo.in_ml_model).toBe(false);

    const undefinedInfo = Magika.getContentTypeInfo(ContentTypeLabel.UNDEFINED);
    expect(undefinedInfo.in_heuristics).toBe(false);
    expect(undefinedInfo.in_resolvers).toBe(false);
    expect(undefinedInfo.in_ml_model).toBe(false);

    // Defensive copies prevent caller mutations from polluting cached entries.
    const pythonInfo = Magika.getContentTypeInfo(ContentTypeLabel.PYTHON);
    const origExtensions = [...pythonInfo.extensions];
    pythonInfo.extensions.push("mutated_ext");
    pythonInfo.description = "mutated_desc";
    expect(Magika.getContentTypeInfo(ContentTypeLabel.PYTHON)).toMatchObject({
      description: "Python source",
      extensions: origExtensions,
    });

    // Unknown, unexposed catalog-only, prototype-key, and non-string inputs.
    for (const badLabel of [
      "not_a_real_content_type",
      "abnf",
      "toString",
      "__proto__",
      "constructor",
    ]) {
      expect(() => Magika.getContentTypeInfo(badLabel)).toThrow(
        /Unknown content type label/,
      );
    }
    expect(() => Magika.getContentTypeInfo(123 as unknown as string)).toThrow(
      TypeError,
    );
  });

  it("should list output and model content types and match prediction metadata", async () => {
    class UnloadedMagika extends BrowserMagika {
      public static createUnloaded(): UnloadedMagika {
        return new UnloadedMagika();
      }
    }
    const unloaded = UnloadedMagika.createUnloaded();
    expect(() => unloaded.getOutputContentTypes()).toThrow(
      /model config has not been loaded/,
    );
    expect(() => unloaded.getModelContentTypes()).toThrow(
      /model config has not been loaded/,
    );

    const magika = await Magika.create({
      modelConfigPath: workdir.model_config,
      modelPath: workdir.model,
    });
    const modelTypes = magika.getModelContentTypes();
    const outputTypes = magika.getOutputContentTypes();

    expect(modelTypes).toStrictEqual([...new Set(modelTypes)].sort());
    expect(outputTypes).toStrictEqual([...new Set(outputTypes)].sort());

    const expectedModelTypes = Object.values(ContentTypeLabel)
      .filter((ct) => Magika.getContentTypeInfo(ct).in_ml_model)
      .sort();
    expect(modelTypes).toStrictEqual(expectedModelTypes);

    const expectedOutputTypes = Array.from(
      new Set<ContentTypeLabel>([
        ContentTypeLabel.EMPTY,
        ContentTypeLabel.TXT,
        ContentTypeLabel.UNKNOWN,
        ...magika.model_config.target_labels_space.map(
          (ct) => magika.model_config.overwrite_map[ct] ?? ct,
        ),
      ]),
    ).sort();
    expect(outputTypes).toStrictEqual(expectedOutputTypes);

    // JS TF.js pipeline includes byte/stream heuristics + ML types, and
    // excludes inactive labels, filesystem-only inode types, and resolver-only
    // types.
    for (const included of [
      ContentTypeLabel.EMPTY,
      ContentTypeLabel.TXT,
      ContentTypeLabel.UNKNOWN,
      ContentTypeLabel.PYTHON,
      ContentTypeLabel.PDF,
    ]) {
      expect(outputTypes).toContain(included);
    }
    for (const excluded of [
      ContentTypeLabel.UNDEFINED,
      ContentTypeLabel.RANDOMBYTES,
      ContentTypeLabel.RANDOMTXT,
      ContentTypeLabel.DIRECTORY,
      ContentTypeLabel.SYMLINK,
      ContentTypeLabel.UNSUPPORTED,
      ContentTypeLabel.LLVM_BITCODE,
    ]) {
      expect(outputTypes).not.toContain(excluded);
    }

    // Predictions carry full ContentTypeInfo matching getContentTypeInfo.
    const emptyRes = await magika.identifyBytes(new Uint8Array(0));
    expect(emptyRes.prediction.output).toStrictEqual(
      Magika.getContentTypeInfo(ContentTypeLabel.EMPTY),
    );
    expect(emptyRes.prediction.dl).toStrictEqual(
      Magika.getContentTypeInfo(ContentTypeLabel.UNDEFINED),
    );

    const pyBytes = Buffer.from("def hello() -> int:\n    return 42\n");
    const pyRes = await magika.identifyBytes(pyBytes);
    expect(pyRes.prediction.output).toStrictEqual(
      Magika.getContentTypeInfo(pyRes.prediction.output.label),
    );
    expect(pyRes.prediction.dl).toStrictEqual(
      Magika.getContentTypeInfo(pyRes.prediction.dl.label),
    );

    // Mutating prediction metadata must not corrupt subsequent predictions.
    pyRes.prediction.output.extensions.push("mutated_ext");
    const pyResAgain = await magika.identifyBytes(pyBytes);
    expect(pyResAgain.prediction.output).toStrictEqual(
      Magika.getContentTypeInfo(pyResAgain.prediction.output.label),
    );
  });
});
