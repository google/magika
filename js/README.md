# Magika TypeScript/JavaScript library

Use Magika in the browser or in Node!

# Installing MagikaJS

```bash
npm install magika
```

# Using MagikaJS

Simple usage in Node:

```js
import { readFile } from "fs/promises";
import { MagikaNode as Magika } from "magika";

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

For more, see our [documentation](https://github.com/google/magika/blob/main/docs/js.md).

# Command-line tool

Please use the official CLI (with `pip install magika`) as it can perform batch processing and search for files recursively.
Read more about that in the main [README](https://github.com/google/magika/blob/main/README.md).
This one is useful to load the TensorflowJS model and see that it works as expected.

Install it with `npm install -g magika`. You can then run it by executing `magika-js <some files>`

```help
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

# Reporting errors in detections

Please open an issue on [Github](https://github.com/google/magika/issues).

# Citation

If you use this software for your research, please cite it as:

```bibtex
@InProceedings{fratantonio25:magika,
  author = {Yanick Fratantonio and Luca Invernizzi and Loua Farah and Kurt Thomas and Marina Zhang and Ange Albertini and Francois Galilee and Giancarlo Metitieri and Julien Cretin and Alexandre Petit-Bianco and David Tao and Elie Bursztein},
  title = {{Magika: AI-Powered Content-Type Detection}},
  booktitle = {Proceedings of the International Conference on Software Engineering (ICSE)},
  month = {April},
  year = {2025}
}
```

# Loading the model and configuration

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

# Development

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
