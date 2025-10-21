---
title: "JavaScript / TypeScript Library"
---

Use Magika in the browser or in Node!

## Installing MagikaJS

```bash
npm install magika
```

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
  --model-path <model-path>                Modle file path
  --model-config-url <model-config-url>    Model config URL (default: "https://google.github.io/magika/models/standard_v3_2/config.min.json")
  --model-config-path <model-config-path>  Model config file path
  --by-stream                              Identify file via stream, not via bytes
  --debug                                  Output debug information
  -h, --help                               display help for command
```


## Loading the model and configuration

MagikaJS is designed to be flexible in how you provide the model and configuration file to it.

Both the Node and browser versions accept URLs to asyncronously load these two assets.

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
(cd ../website; yarn install; yarn run dev) &
yarn run bin --model-url http://localhost:5173/magika/model/model.json --config-url http://localhost:5173/magika/model/config.json ../tests_data/basic/*
```

Using the local `magika` package when developing the website:

```bash
yarn install
yarn run build
yarn link
(cd ../website; yarn link magika; yarn install; yarn run dev) &
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
