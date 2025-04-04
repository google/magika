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
