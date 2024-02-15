# Magika JavaScript library
Use Magika in the browser or in Node!

Simple usage in Node:
```js
import { readFile } from "fs/promises";
import { Magika } from "magika";
const data = await readFile('some file');
const magika = new Magika();
await magika.load();
const prediction = await magika.identifyBytes(data);
console.log(prediction);
```


Simple usage in the browser:
```js
import { Magika } from "magika";

const someFile = new File(["# Hello I am a markdown file"], "hello.md")
const magika = new Magika();
await magika.load();
const prediction = await magika.identifyBytes(await someFile.text());
console.log(prediction);
```

See our [DOCS.md|documentation].


# Commmand line tool
Please use the official CLI (with `pip install magika`) as it's considerably faster than this one.
This is useful to load the TensorflowJS model and see that it works as expected.
```
Magika JS - file type detection with ML. https://google.github.io/magika

Arguments:
  paths                      Paths of the files to detect

Options:
  --json-output              Format output in JSON
  --model-url <model-url>    Model URL (default: "https://google.github.io/magika/model/model.json")
  --config-url <config-url>  Config  URL (default: "https://google.github.io/magika/model/config.json")
  -h, --help                 display help for command

```

# Development
Using the model hosted On Github
```bash
yarn install
yarn run bin -- README.md
```

Using the local model
```bash
yarn install
(cd ../docs; yarn install; yarn run dev) &
 yarn run bin --model-url http://localhost:5173/magika/model/model.json --config-url http://localhost:5173/magika/model/config.json test_data/*
```


