# Magika JavaScript library

Use Magika in the browser or in Node!

# Installing MagikaJS

```bash
npm install magika
```

# Using MagikaJS

Simple usage in Node:

```js
import { readFile } from "fs/promises";
import { Magika } from "magika";

const data = await readFile("some file");
const magika = new Magika();
await magika.load();
const prediction = await magika.identifyBytes(data);
console.log(prediction);
```

Simple usage in the browser:

```js
import { Magika } from "magika";

const file = new File(["# Hello I am a markdown file"], "hello.md");
const fileBytes = new Uint8Array(await file.arrayBuffer());
const magika = new Magika();
await magika.load();
const prediction = await magika.identifyBytes(fileBytes);
console.log(prediction);
```

For more, see our [documentation](../docs/js.md).

# Commmand-line tool

Please use the official CLI (with `pip install magika`) as it's considerably faster than this one.
Read more about that in the main [README](../README.md).
This one is useful to load the TensorflowJS model and see that it works as expected.

Install it with `npm install -g magika`. You can then run it by executing `magika-js <some files>`

```help
Magika JS - file type detection with ML. https://google.github.io/magika

Arguments:
  paths                      Paths of the files to detect

Options:
  --json-output              Format output in JSON
  --model-url <model-url>    Model URL (default: "https://google.github.io/magika/model/model.json")
  --config-url <config-url>  Config  URL (default: "https://google.github.io/magika/model/config.json")
  -h, --help                 display help for command

```

# Reporting false positives

Please open an issue on Github.

# Citation

If you use this software for your research, please cite it as:

```bibtex
@software{magika,
author = {Fratantonio, Yanick and Bursztein, Elie and Invernizzi, Luca and Zhang, Marina and Metitieri, Giancarlo and Kurt, Thomas and Galilee, Francois and Petit-Bianco, Alexandre and Farah, Loua and Albertini, Ange},
title = {{Magika content-type scanner}},
url = {https://github.com/google/magika}
}
```

# Development

Using the model hosted On Github:

```bash
yarn install
yarn run bin -- README.md
```

Using the local model:

```bash
yarn install
(cd ../website; yarn install; yarn run dev) &
yarn run bin --model-url http://localhost:5173/magika/model/model.json --config-url http://localhost:5173/magika/model/config.json ../tests_data/mitra/*
```
