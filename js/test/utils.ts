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

import * as fs from "fs";
import * as zlib from "zlib";

export function parseGzippedJSON(filePath: string): Array<any> {
  const gzippedBuffer = fs.readFileSync(filePath);
  const jsonBuffer = zlib.gunzipSync(gzippedBuffer);
  const jsonString = jsonBuffer.toString("utf-8");
  const parsedData = JSON.parse(jsonString);
  if (!Array.isArray(parsedData)) {
    throw new Error("Parsed JSON is not an array as expected for ExampleList.");
  }
  return parsedData as Array<any>;
}
