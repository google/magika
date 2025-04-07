#! /usr/bin/env node
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

// Command line tool to test the JavaScript implementation of Magika. Please use
// the official command line tool (`pip install magika`) for normal use.

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
  .option(
    "--model-config-url <model-config-url>",
    "Model config URL",
    Magika.MODEL_CONFIG_URL,
  )
  .option("--model-config-path <model-config-path>", "Model config file path")
  .option("--by-stream", "Identify file via stream, not via bytes")
  .option("--debug", "Output debug information")
  .argument("<paths...>", "Paths of the files to detect");

program.exitOverride();

try {
  program.parse(process.argv);
} catch (error: any) {
  // There was an error parsing the arguments, let's print the help.
  try {
    program.help();
  } catch (error: any) {
    // Avoid that commander shows some weird exception.
    process.exit(1);
  }
}

const flags = program.opts();

(async () => {
  const magika = await Magika.create({
    modelURL: flags.modelUrl,
    modelPath: flags.modelPath,
    modelConfigURL: flags.configUrl,
    modelConfigPath: flags.configPath,
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
          } else if (flags.debug) {
            console.log(
              chalk.yellow(path),
              "by_stream",
              chalk.green(
                magika_result_by_stream.prediction.dl.label,
                magika_result_by_stream.prediction.output.label,
              ),
              chalk.white(magika_result_by_stream.prediction.score),
            );
          } else {
            console.log(
              chalk.yellow(path),
              chalk.green(magika_result_by_stream.prediction.output.label),
              chalk.white(magika_result_by_stream.prediction.score.toFixed(3)),
            );
          }
        } else {
          const magika_result_by_path = await magika.identifyBytes(data);
          if (flags.jsonOutput) {
            console.log(path, magika_result_by_path);
          } else if (flags.debug) {
            console.log(
              chalk.yellow(path),
              "by_path",
              chalk.green(
                magika_result_by_path.prediction.dl.label,
                magika_result_by_path.prediction.output.label,
              ),
              chalk.white(magika_result_by_path.prediction.score),
            );
          } else {
            console.log(
              chalk.yellow(path),
              chalk.green(magika_result_by_path.prediction.output.label),
              chalk.white(magika_result_by_path.prediction.score.toFixed(3)),
            );
          }
        }
      }
    }),
  );
})();
