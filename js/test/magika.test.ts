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
import { MagikaNode as Magika } from "../magika-node";
import { ContentTypeLabel } from "../src/content-type-label";

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
      modelVersion: Magika.MODEL_VERSION,
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
        modelVersion: Magika.MODEL_VERSION,
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
        modelVersion: Magika.MODEL_VERSION,
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
});
