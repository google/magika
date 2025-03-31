#! /usr/bin/env node
// Command line tool to test MagikaJs. Please use the official command line
// tool (`pip install magika`) for normal use.

// To run this, you need to install the optional dependencies too.
import { program } from "commander";
import { readFile } from "fs/promises";
import * as fs from "fs";
import chalk from "chalk";
import { MagikaNode as Magika } from "./magika_node.js";

program
  .description(
    "Magika JS - file type detection with ML. https://google.github.io/magika",
  )
  .option("--json-output", "Format output in JSON")
  .option("--model-url <model-url>", "Model URL", Magika.MODEL_URL)
  .option("--model-path <model-path>", "Modle file path")
  .option("--config-url <config-url>", "Config URL", Magika.CONFIG_URL)
  .option("--config-path <config-path>", "Config file path")
  .option("--by-stream", "Identify file via stream, not via bytes")
  .argument("<paths...>", "Paths of the files to detect");

program.parse();

const flags = program.opts();
const magika = new Magika();

(async () => {
  await magika.load({
    modelURL: flags.modelUrl,
    modelPath: flags.modelPath,
    configURL: flags.configUrl,
    configPath: flags.configPath,
  });
  await Promise.all(
    program.args.map(async (path) => {
      let data = null;
      try {
        data = await readFile(path);
      } catch (error) {
        console.error("Skipping file", path, error);
      }

      if (data != null) {
        if (flags.byStream) {
          const prediction_stream = await magika.identifyStream(
            fs.createReadStream(path),
            data.length,
          );
          if (flags.jsonOutput) {
            console.log({ path, ...prediction_stream });
          } else {
            console.log(
              chalk.blue(path),
              "by_stream",
              chalk.green(
                prediction_stream?.label,
                chalk.white(prediction_stream?.score),
              ),
            );
          }
        } else {
          const prediction = await magika.identifyBytes(data);
          if (flags.jsonOutput) {
            console.log({ path, ...prediction });
          } else {
            console.log(
              chalk.blue(path),
              "by_path",
              chalk.green(prediction?.label, chalk.white(prediction?.score)),
            );
          }
        }
      }
    }),
  );
})();
