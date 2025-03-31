#! /usr/bin/env node
// Command line tool to test MagikaJs. Please use the official command line
// tool (`pip install magika`) for normal use.

// To run this, you need to install the optional dependencies too.
import chalk from "chalk";
import { program } from "commander";
import * as fs from "fs";
import { readFile } from "fs/promises";
import { MagikaNode as Magika } from "./magika-node.js";

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
          const magika_result_by_stream = await magika.identifyStream(
            fs.createReadStream(path),
            data.length,
          );
          if (flags.jsonOutput) {
            console.log(path, magika_result_by_stream);
          } else {
            console.log(
              chalk.blue(path),
              "by_stream",
              chalk.green(
                magika_result_by_stream.prediction.dl.label,
                magika_result_by_stream.prediction.output.label,
                chalk.white(magika_result_by_stream.prediction.score),
              ),
            );
          }
        } else {
          const magika_result_by_path = await magika.identifyBytes(data);
          if (flags.jsonOutput) {
            console.log(path, magika_result_by_path);
          } else {
            console.log(
              chalk.blue(path),
              "by_path",
              chalk.green(
                magika_result_by_path.prediction.dl.label,
                magika_result_by_path.prediction.output.label,
                chalk.white(magika_result_by_path.prediction.score),
              ),
            );
          }
        }
      }
    }),
  );
})();
