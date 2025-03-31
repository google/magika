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
import { TfnMock } from "./tfnHook";

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
const TEST_FILES: Array<[string, string, Dirent]> = [
  ...getTestFilesWithLabels("../tests_data/basic"),
  // ...(getTestFilesWithLabels('../tests_data/mitra'))
];

/**
 * File types for Magika V2 or for corner cases that are not handled by the
 * model. Skip them in the tests for now.
 */
const SKIP_FUTURE_CONTENT_TYPES = new Set([
  "dockerfile",
  "empty",
  "toml",
  "typescript",
  "yara",
]);

describe("Magika class", () => {
  const workdir = {
    root: "",
    config: "",
    model: "",
  };
  beforeAll(async () => {
    workdir.root = await mkdtemp(path.join(os.tmpdir(), "magika-"));
    workdir.config = path.join(workdir.root, "config.json");
    workdir.model = path.join(workdir.root, "model.json");

    const config = Readable.fromWeb(
      (await fetch(Magika.CONFIG_URL)).body as ReadableStream<any>,
    );
    const model = Readable.fromWeb(
      (await fetch(Magika.MODEL_URL)).body as ReadableStream<any>,
    );
    await Promise.all([
      await finished(config.pipe(fs.createWriteStream(workdir.config))),
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
        const config = Readable.fromWeb(
          (await fetch(weight.url)).body as ReadableStream<any>,
        );
        await finished(
          config.pipe(
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
    const magika = new Magika();
    await magika.load();
    expect(magika.model.model).toBeDefined();
    expect(magika.config.target_labels_space.length).toBeGreaterThan(0);
    expect(Object.values(TfnMock.accessed).reduce((a, b) => a + b, 0)).toBe(0);
  });

  it("should load model from file path", async () => {
    const magika = new Magika();
    await magika.load({ configPath: workdir.config, modelPath: workdir.model });
    expect(magika.model.model).toBeDefined();
    expect(magika.config.target_labels_space.length).toBeGreaterThan(0);
    expect(TfnMock.accessed.io).toBe(1);
    expect(Object.values(TfnMock.accessed).reduce((a, b) => a + b, 0)).toBe(1);
  });

  it("scores should be in the expected range", async () => {
    const magika = new Magika();
    await magika.load();
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

  it("features should result in known value", async () => {
    const magika = new Magika();
    await magika.load({ configPath: workdir.config, modelPath: workdir.model });
    const featuresMock = jest.spyOn(magika.model, "predict");

    const streamResult = await magika.identifyStream(
      fs.createReadStream("../tests_data/basic/javascript/code.js"),
      (await fs.promises.stat("../tests_data/basic/javascript/code.js")).size,
    );

    const input = await fs.promises.readFile(
      "../tests_data/basic/javascript/code.js",
    );
    const byteResult = await magika.identifyBytes(input);
    expect(streamResult.prediction).toStrictEqual(byteResult.prediction);
    expect(featuresMock.mock.calls[0][0]).toStrictEqual(
      featuresMock.mock.calls[1][0],
    );
    const featuresChunk = [
      40, 102, 117, 110, 99, 116, 105, 111, 110, 40, 41, 123, 47, 42, 10, 10,
      32, 67, 111, 112, 121, 114, 105, 103, 104, 116, 32, 84, 104, 101, 32, 67,
      108, 111, 115, 117, 114, 101, 32, 76, 105, 98, 114, 97, 114, 121, 32, 65,
      117, 116, 104, 111, 114, 115, 46, 10, 32, 83, 80, 68, 88, 45, 76, 105, 99,
      101, 110, 115, 101, 45, 73, 100, 101, 110, 116, 105, 102, 105, 101, 114,
      58, 32, 65, 112, 97, 99, 104, 101, 45, 50, 46, 48, 10, 42, 47, 10, 118,
      97, 114, 32, 110, 61, 116, 104, 105, 115, 124, 124, 115, 101, 108, 102,
      44, 112, 61, 102, 117, 110, 99, 116, 105, 111, 110, 40, 97, 44, 98, 41,
      123, 97, 61, 97, 46, 115, 112, 108, 105, 116, 40, 34, 46, 34, 41, 59, 118,
      97, 114, 32, 99, 61, 110, 59, 97, 91, 48, 93, 105, 110, 32, 99, 124, 124,
      34, 117, 110, 100, 101, 102, 105, 110, 101, 100, 34, 61, 61, 116, 121,
      112, 101, 111, 102, 32, 99, 46, 101, 120, 101, 99, 83, 99, 114, 105, 112,
      116, 124, 124, 99, 46, 101, 120, 101, 99, 83, 99, 114, 105, 112, 116, 40,
      34, 118, 97, 114, 32, 34, 43, 97, 91, 48, 93, 41, 59, 102, 111, 114, 40,
      118, 97, 114, 32, 100, 59, 97, 46, 108, 101, 110, 103, 116, 104, 38, 38,
      40, 100, 61, 97, 46, 115, 104, 105, 102, 116, 40, 41, 41, 59, 41, 97, 46,
      108, 101, 110, 103, 116, 104, 124, 124, 118, 111, 105, 100, 32, 48, 61,
      61, 61, 98, 63, 99, 61, 99, 91, 100, 93, 38, 38, 99, 91, 100, 93, 33, 61,
      61, 79, 98, 106, 101, 99, 116, 46, 112, 114, 111, 116, 111, 116, 121, 112,
      101, 91, 100, 93, 63, 99, 91, 100, 93, 58, 99, 91, 100, 93, 61, 123, 125,
      58, 99, 91, 100, 93, 61, 98, 125, 59, 102, 117, 110, 99, 116, 105, 111,
      110, 32, 113, 40, 41, 123, 102, 111, 114, 40, 118, 97, 114, 32, 97, 61,
      114, 44, 98, 61, 123, 125, 44, 99, 61, 48, 59, 99, 60, 97, 46, 108, 101,
      110, 103, 116, 104, 59, 43, 43, 99, 41, 98, 91, 97, 91, 99, 93, 93, 61,
      99, 59, 114, 101, 116, 117, 114, 110, 32, 98, 125, 102, 117, 110, 99, 116,
      105, 111, 110, 32, 117, 40, 41, 123, 118, 97, 114, 32, 97, 61, 34, 65, 66,
      67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84,
      85, 86, 87, 88, 89, 90, 34, 59, 97, 43, 61, 97, 46, 116, 111, 76, 111,
      119, 101, 114, 67, 97, 115, 101, 40, 41, 43, 34, 48, 49, 50, 51, 52, 53,
      54, 55, 56, 57, 45, 95, 34, 59, 114, 101, 116, 117, 114, 110, 32, 97, 43,
      34, 46, 34, 125, 118, 97, 114, 32, 114, 44, 118, 59, 10, 102, 117, 110,
      99, 116, 105, 111, 110, 32, 97, 97, 40, 97, 41, 123, 102, 117, 110, 99,
      116, 105, 111, 110, 32, 98, 40, 107, 41, 123, 102, 111, 114, 40, 59, 100,
      60, 97, 46, 108, 101, 110, 103, 116, 104, 59, 41, 123, 118, 97, 114, 32,
      109, 61, 97, 46, 99, 104, 97, 114, 65, 116, 40, 100, 43, 43, 41, 44, 108,
      61, 118, 91, 109, 93, 59, 105, 102, 40, 110, 117, 108, 108, 33, 61, 108,
      41, 114, 101, 116, 117, 114, 110, 32, 108, 59, 105, 102, 40, 33, 47, 94,
      91, 92, 115, 92, 120, 97, 48, 93, 42, 36, 47, 46, 116, 101, 115, 116, 40,
      109, 41, 41, 116, 104, 114, 111, 119, 32, 69, 114, 114, 111, 114, 40, 34,
      85, 110, 107, 110, 111, 119, 110, 32, 98, 97, 115, 101, 54, 52, 32, 101,
      110, 99, 111, 100, 105, 110, 103, 32, 97, 116, 32, 99, 104, 97, 114, 58,
      32, 34, 43, 109, 41, 59, 125, 114, 101, 116, 117, 114, 110, 32, 107, 125,
      114, 61, 114, 124, 124, 117, 40, 41, 59, 118, 61, 118, 124, 124, 113, 40,
      41, 59, 102, 111, 114, 40, 118, 97, 114, 32, 99, 61, 34, 34, 44, 100, 61,
      48, 59, 59, 41, 123, 118, 97, 114, 32, 101, 61, 98, 40, 45, 49, 41, 44,
      102, 61, 98, 40, 48, 41, 44, 104, 61, 98, 40, 54, 52, 41, 44, 103, 61, 98,
      40, 54, 52, 41, 59, 105, 102, 40, 54, 52, 61, 61, 61, 103, 38, 38, 45, 49,
      61, 61, 61, 101, 41, 114, 101, 116, 117, 114, 110, 32, 99, 59, 99, 43, 61,
      83, 116, 114, 105, 110, 103, 46, 102, 114, 111, 109, 67, 104, 97, 114, 67,
      111, 100, 101, 40, 101, 60, 60, 50, 124, 102, 62, 62, 52, 41, 59, 54, 52,
      33, 61, 104, 38, 38, 40, 99, 43, 61, 83, 116, 114, 105, 110, 103, 46, 102,
      114, 111, 109, 67, 104, 97, 114, 67, 111, 100, 101, 40, 102, 60, 60, 52,
      38, 50, 52, 48, 124, 104, 62, 62, 50, 41, 44, 54, 52, 33, 61, 103, 38, 38,
      40, 99, 43, 61, 83, 116, 114, 105, 110, 103, 46, 102, 114, 111, 109, 67,
      104, 97, 114, 67, 111, 100, 101, 40, 104, 60, 60, 54, 38, 49, 57, 50, 124,
      103, 41, 41, 41, 125, 125, 59, 118, 97, 114, 32, 119, 61, 123, 125, 44,
      121, 61, 102, 117, 110, 99, 116, 105, 111, 110, 40, 97, 41, 123, 119, 46,
      84, 65, 71, 71, 73, 78, 71, 61, 119, 46, 84, 65, 71, 71, 73, 78, 71, 124,
      124, 91, 93, 59, 119, 46, 84, 65, 71, 71, 73, 78, 71, 91, 97, 93, 61, 33,
      48, 125, 59, 118, 97, 114, 32, 98, 97, 61, 65, 114, 114, 97, 121, 46, 105,
      115, 65, 114, 114, 97, 121, 44, 99, 97, 61, 102, 117, 110, 99, 116, 105,
      111, 110, 40, 97, 44, 98, 41, 123, 105, 102, 40, 97, 38, 38, 98, 97, 101,
      40, 98, 44, 49, 41, 59, 78, 46, 104, 91, 97, 93, 61, 110, 117, 108, 108,
      59, 98, 114, 101, 97, 107, 125, 125, 59, 78, 46, 106, 61, 102, 117, 110,
      99, 116, 105, 111, 110, 40, 97, 41, 123, 114, 101, 116, 117, 114, 110, 32,
      78, 46, 104, 91, 97, 93, 125, 59, 78, 46, 103, 101, 116, 65, 108, 108, 61,
      102, 117, 110, 99, 116, 105, 111, 110, 40, 41, 123, 114, 101, 116, 117,
      114, 110, 32, 78, 46, 80, 46, 115, 108, 105, 99, 101, 40, 48, 41, 125, 59,
      10, 78, 46, 78, 61, 102, 117, 110, 99, 116, 105, 111, 110, 40, 41, 123,
      34, 103, 97, 34, 33, 61, 103, 98, 38, 38, 74, 40, 52, 57, 41, 59, 118, 97,
      114, 32, 97, 61, 79, 91, 103, 98, 93, 59, 105, 102, 40, 33, 97, 124, 124,
      52, 50, 33, 61, 97, 46, 97, 110, 115, 119, 101, 114, 41, 123, 78, 46, 76,
      61, 97, 38, 38, 97, 46, 108, 59, 78, 46, 121, 97, 61, 49, 42, 110, 101,
      119, 32, 68, 97, 116, 101, 59, 78, 46, 108, 111, 97, 100, 101, 100, 61,
      33, 48, 59, 118, 97, 114, 32, 98, 61, 97, 38, 38, 97, 46, 113, 44, 99, 61,
      107, 97, 40, 98, 41, 59, 97, 61, 91, 93, 59, 99, 63, 97, 61, 98, 46, 115,
      108, 105, 99, 101, 40, 48, 41, 58, 74, 40, 53, 48, 41, 59, 78, 46, 113,
      61, 99, 63, 98, 58, 91, 93, 59, 78, 46, 113, 46, 115, 112, 108, 105, 99,
      101, 40, 48, 41, 59, 78, 46, 113, 100, 61, 48, 59, 98, 61, 79, 91, 103,
      98, 93, 61, 78, 59, 88, 40, 34, 99, 114, 101, 97, 116, 101, 34, 44, 98,
      44, 98, 46, 99, 114, 101, 97, 116, 101, 41, 59, 88, 40, 34, 114, 101, 109,
      111, 118, 101, 34, 44, 98, 44, 98, 46, 114, 101, 109, 111, 118, 101, 41,
      59, 88, 40, 34, 103, 101, 116, 66, 121, 78, 97, 109, 101, 34, 44, 98, 44,
      98, 46, 106, 44, 53, 41, 59, 88, 40, 34, 103, 101, 116, 65, 108, 108, 34,
      44, 98, 44, 98, 46, 103, 101, 116, 65, 108, 108, 44, 54, 41, 59, 98, 61,
      112, 99, 46, 112, 114, 111, 116, 111, 116, 121, 112, 101, 59, 88, 40, 34,
      103, 101, 116, 34, 44, 98, 44, 98, 46, 103, 101, 116, 44, 55, 41, 59, 88,
      40, 34, 115, 101, 116, 34, 44, 98, 44, 98, 46, 115, 101, 116, 44, 52, 41,
      59, 88, 40, 34, 115, 101, 110, 100, 34, 44, 98, 44, 98, 46, 115, 101, 110,
      100, 41, 59, 88, 40, 34, 114, 101, 113, 117, 105, 114, 101, 83, 121, 110,
      99, 34, 44, 98, 44, 98, 46, 109, 97, 41, 59, 98, 61, 89, 97, 46, 112, 114,
      111, 116, 111, 116, 121, 112, 101, 59, 88, 40, 34, 103, 101, 116, 34, 44,
      98, 44, 98, 46, 103, 101, 116, 41, 59, 88, 40, 34, 115, 101, 116, 34, 44,
      98, 44, 98, 46, 115, 101, 116, 41, 59, 105, 102, 40, 34, 104, 116, 116,
      112, 115, 58, 34, 33, 61, 77, 46, 108, 111, 99, 97, 116, 105, 111, 110,
      46, 112, 114, 111, 116, 111, 99, 111, 108, 38, 38, 33, 66, 97, 41, 123,
      97, 58, 123, 98, 61, 77, 46, 103, 101, 116, 69, 108, 101, 109, 101, 110,
      116, 115, 66, 121, 84, 97, 103, 78, 97, 109, 101, 40, 34, 115, 99, 114,
      105, 112, 116, 34, 41, 59, 10, 102, 111, 114, 40, 99, 61, 48, 59, 99, 60,
      98, 46, 108, 101, 110, 103, 116, 104, 38, 38, 49, 48, 48, 62, 99, 59, 99,
      43, 43, 41, 123, 118, 97, 114, 32, 100, 61, 98, 91, 99, 93, 46, 115, 114,
      99, 59, 105, 102, 40, 100, 38, 38, 48, 61, 61, 100, 46, 105, 110, 100,
      101, 120, 79, 102, 40, 98, 100, 40, 33, 48, 41, 43, 34, 47, 97, 110, 97,
      108, 121, 116, 105, 99, 115, 34, 41, 41, 123, 98, 61, 33, 48, 59, 98, 114,
      101, 97, 107, 32, 97, 125, 125, 98, 61, 33, 49, 125, 98, 38, 38, 40, 66,
      97, 61, 33, 48, 41, 125, 40, 79, 46, 103, 97, 112, 108, 117, 103, 105,
      110, 115, 61, 79, 46, 103, 97, 112, 108, 117, 103, 105, 110, 115, 124,
      124, 123, 125, 41, 46, 76, 105, 110, 107, 101, 114, 61, 68, 99, 59, 98,
      61, 68, 99, 46, 112, 114, 111, 116, 111, 116, 121, 112, 101, 59, 67, 40,
      34, 108, 105, 110, 107, 101, 114, 34, 44, 68, 99, 41, 59, 88, 40, 34, 100,
      101, 99, 111, 114, 97, 116, 101, 34, 44, 98, 44, 98, 46, 99, 97, 44, 50,
      48, 41, 59, 88, 40, 34, 97, 117, 116, 111, 76, 105, 110, 107, 34, 44, 98,
      44, 98, 46, 83, 44, 50, 53, 41, 59, 88, 40, 34, 112, 97, 115, 115, 116,
      104, 114, 111, 117, 103, 104, 34, 44, 98, 44, 98, 46, 36, 44, 50, 53, 41,
      59, 67, 40, 34, 100, 105, 115, 112, 108, 97, 121, 102, 101, 97, 116, 117,
      114, 101, 115, 34, 44, 102, 100, 41, 59, 67, 40, 34, 97, 100, 102, 101,
      97, 116, 117, 114, 101, 115, 34, 44, 102, 100, 41, 59, 90, 46, 68, 46, 97,
      112, 112, 108, 121, 40, 78, 44, 97, 41, 125, 125, 59, 118, 97, 114, 32,
      120, 102, 61, 78, 46, 78, 44, 121, 102, 61, 79, 91, 103, 98, 93, 59, 121,
      102, 38, 38, 121, 102, 46, 114, 63, 120, 102, 40, 41, 58, 122, 40, 120,
      102, 41, 59, 122, 40, 102, 117, 110, 99, 116, 105, 111, 110, 40, 41, 123,
      90, 46, 68, 40, 91, 34, 112, 114, 111, 118, 105, 100, 101, 34, 44, 34,
      114, 101, 110, 100, 101, 114, 34, 44, 117, 97, 93, 41, 125, 41, 59, 125,
      41, 40, 119, 105, 110, 100, 111, 119, 41, 59,
    ];
    expect(featuresMock.mock.calls[0][0].toArray()).toStrictEqual(
      featuresChunk,
    );
    expect(TfnMock.accessed.io).toBe(1);
    expect(Object.values(TfnMock.accessed).reduce((a, b) => a + b, 0)).toBe(1);
  });

  it.each(TEST_FILES)(
    'by_stream vs by_byte should return the same (correct) features/label for "%s" "%s"',
    async (label, testPath, testFile) => {
      if (SKIP_FUTURE_CONTENT_TYPES.has(label)) return;
      const magika = new Magika();
      await magika.load({
        configPath: workdir.config,
        modelPath: workdir.model,
      });
      const featuresMock = jest.spyOn(magika.model, "predict");
      const filePath = path.join(testFile.parentPath, testFile.name);
      const streamResult = await magika.identifyStream(
        fs.createReadStream(filePath),
        (await fs.promises.stat(filePath)).size,
      );
      const input = await fs.promises.readFile(filePath);
      const byteResult = await magika.identifyBytes(input);

      expect(streamResult.prediction).toStrictEqual(byteResult.prediction);
      expect(featuresMock.mock.calls[0][0]).toStrictEqual(
        featuresMock.mock.calls[1][0],
      );
      expect(streamResult.prediction.output.label).toBe(label);
      expect(TfnMock.accessed.io).toBe(1);
      expect(Object.values(TfnMock.accessed).reduce((a, b) => a + b, 0)).toBe(
        1,
      );
    },
  );

  it.each(TEST_FILES)(
    'Magika is agnostic to the format of the input bytes for "%s" "%s"',
    async (label, testPath, testFile) => {
      if (SKIP_FUTURE_CONTENT_TYPES.has(label)) return;
      const magika = new Magika();
      await magika.load({
        configPath: workdir.config,
        modelPath: workdir.model,
      });
      const featuresMock = jest.spyOn(magika.model, "predict");
      const filePath = path.join(testFile.parentPath, testFile.name);
      const inputBuffer = await fs.promises.readFile(filePath);
      const inputUint8 = new Uint8Array(inputBuffer);
      const inputUint16 = new Uint16Array(inputBuffer);
      const resultFromBuffer = await magika.identifyBytes(inputBuffer);
      const resultFromUint8 = await magika.identifyBytes(inputUint8);
      const resultFromUint16 = await magika.identifyBytes(inputUint16);
      expect(resultFromBuffer.prediction).toStrictEqual(
        resultFromUint8.prediction,
      );
      expect(resultFromBuffer.prediction).toStrictEqual(
        resultFromUint16.prediction,
      );
    },
  );
});
