---
title: "JavaScript / TypeScript Library"
---

Use Magika in the browser or in Node!

## Installing MagikaJS

```bash
npm install magika
```

> **Node.js compatibility:** `magika/node` targets **Node.js 22 LTS** (`>=22`); Node.js 24+ is not yet supported due to an upstream `@tensorflow/tfjs-node` incompatibility ([google/magika#1577](https://github.com/google/magika/issues/1577)).

## Using Magika in JavaScript

Simple usage in Node:

```js
import { readFile } from "fs/promises";
import { MagikaNode as Magika } from "magika/node";

const data = await readFile("some file");
const magika = await Magika().create();
const prediction = await magika.identifyBytes(data);
console.log(prediction);
```

Simple usage in the browser:

```js
import { Magika } from "magika";

const file = new File(["# Hello I am a markdown file"], "hello.md");
const fileBytes = new Uint8Array(await file.arrayBuffer());
const magika = await Magika.create();
const prediction = await magika.identifyBytes(fileBytes);
console.log(prediction);
```

For more, see the API reference below.

## Content type metadata and supported outputs

Both `"magika"` and `"magika/node"` export `ContentTypeLabel` and the `ContentTypeInfo` type. Every prediction (`result.prediction.output` and `result.prediction.dl`) and `Magika.getContentTypeInfo(label)` return full knowledge-base metadata along with pipeline stage-coverage flags:

```ts
import {
  type ContentTypeInfo,
  ContentTypeLabel,
  Magika,
} from "magika";

const info: ContentTypeInfo = Magika.getContentTypeInfo(
  ContentTypeLabel.PARQUET,
);
// {
//   label: "parquet",
//   mime_type: "application/vnd.apache.parquet",
//   group: "unknown",
//   description: "Apache Parquet",
//   extensions: ["pqt", "parquet"],
//   is_text: false,
//   in_heuristics: false,
//   in_resolvers: true,
//   in_ml_model: true,
//   vetoed_by_resolvers: true,
// }
```

On a loaded `Magika` (or `MagikaNode`) instance:

- `magika.getOutputContentTypes()` returns the sorted list of `ContentTypeLabel`s that the JavaScript runtime can emit in `result.prediction.output.label` (byte/stream heuristics `empty`, `txt`, `unknown` plus the loaded ML model's output space after `overwrite_map`).
- `magika.getModelContentTypes()` returns the sorted list of `ContentTypeLabel`s supported directly by the loaded ML model.

:::note[JavaScript TF.js pipeline vs. full Rust pipeline]
`ContentTypeLabel` contains all exposed content types shared across Magika's Rust, C, Python, and JavaScript bindings. Because the JavaScript library runs byte/stream heuristics and the TensorFlow.js ML model without filesystem `stat` checks or the native Rust format resolvers, `magika.getOutputContentTypes()` excludes filesystem-only inode labels (`directory`, `symlink`, `unsupported`), resolver-only labels (`in_resolvers && !in_ml_model`), and internal labels (`undefined`, `randombytes`, `randomtxt`).

You can use the stage-coverage booleans on `ContentTypeInfo` to inspect or filter across pipelines:

```ts
// Labels emitted by this JavaScript TF.js instance:
const jsOutputs = magika.getOutputContentTypes();

// Labels emitted by the full native Rust/Python pipeline:
const rustOutputs = Object.values(ContentTypeLabel).filter((label) => {
  const info = Magika.getContentTypeInfo(label);
  return info.in_heuristics || info.in_resolvers || info.in_ml_model;
});
```
:::

## Command-line tool

Please use the official CLI as it can perform batch processing and search for files recursively.
Read more about that in the main the [Command Line Interface (CLI)](/magika/cli-and-bindings/cli/) section.
This one is useful to load the TensorflowJS model and see that it works as expected.

Install it with `npm install -g magika`. You can then run it by executing `magika-js <some files>`

```
Usage: magika-js [options] <paths...>

Magika JS - file type detection with ML. https://google.github.io/magika

Arguments:
  paths                                    Paths of the files to detect

Options:
  --json-output                            Format output in JSON
  --model-url <model-url>                  Model URL (default: "https://google.github.io/magika/models/standard_v3_2/model.json")
  --model-path <model-path>                Model file path
  --model-config-url <model-config-url>    Model config URL (default: "https://google.github.io/magika/models/standard_v3_2/config.min.json")
  --model-config-path <model-config-path>  Model config file path
  --by-stream                              Identify file via stream, not via bytes
  --debug                                  Output debug information
  -h, --help                               display help for command
```


## Loading the model and configuration

MagikaJS is designed to be flexible in how you provide the model and configuration file to it.

Both the Node and browser versions accept URLs to asynchronously load these two assets.

```js
const magika = await magika.create({
  modelURL: "https://...",
  configURL: "https://...",
});
```

The Node version also allows to load local files.

```js
const magika = await magika.create({
  modelPath: "./assets/...",
  configPath: "./assets/...",
});
```

## Development

Using the model hosted On Github:

```bash
yarn install
yarn run build
yarn run bin -- README.md
```

Using the local model:

```bash
yarn install
yarn run build
(cd ../website-ng; npm install; npm run dev) &
yarn run bin --model-url http://localhost:8080/magika/models/standard_v3_3/model.json --model-config-url http://localhost:8080/magika/models/standard_v3_3/config.min.json ../tests_data/basic/*
```

Using the local `magika` package when developing the website:

```bash
yarn install
yarn run build
npm link
(cd ../website-ng; npm install; npm link magika; npm run dev) &
```

## Testing

Execute:

```bash
yarn install
yarn run build
yarn run test
```

## API Reference

See the [JavaScript API Reference](/magika/cli-and-bindings/js-api) section.
