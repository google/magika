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

import { describe, expect, it } from "@jest/globals";
import { spawn } from "child_process";
import path from "path";

describe("magika-cli.ts CLI Tests", () => {
  const scriptPath = path.resolve(__dirname, "../dist/mjs/magika-cli.js");
  const nodeExecutable = process.execPath;

  async function executeCli(
    args: string[] = [],
  ): Promise<{ stdout: string; stderr: string; exitCode: number | null }> {
    return new Promise((resolve, reject) => {
      const process = spawn(nodeExecutable, [scriptPath, ...args]);
      let stdout = "";
      let stderr = "";

      process.stdout.on("data", (data) => {
        stdout += data.toString();
      });

      process.stderr.on("data", (data) => {
        stderr += data.toString();
      });

      process.on("close", (code) => {
        resolve({ stdout, stderr, exitCode: code });
      });

      process.on("error", (err) => {
        reject(err);
      });
    });
  }

  it("should display help information when no arguments are provided", async () => {
    const { stdout, stderr, exitCode } = await executeCli();
    expect(exitCode).toBe(1);
    expect(stdout).toContain("Usage: ");
    expect(stdout).toContain("Options:");
    expect(stderr).toContain("error: missing required argument");

    // Check that the help is printed only once.
    const usageOccurrences = (stdout.match(/Usage:/g) || []).length;
    expect(usageOccurrences).toBe(1);
    const optionsOccurrences = (stdout.match(/Options:/g) || []).length;
    expect(optionsOccurrences).toBe(1);
  });

  it("should display help information when --help is passed", async () => {
    const { stdout, stderr, exitCode } = await executeCli(["--help"]);
    expect(exitCode).toBe(0);
    expect(stdout).toContain("Usage: ");
    expect(stdout).toContain("Options:");
  });

  it("should process (by path) a specific file and output the expected result", async () => {
    const filePath = "../README.md";
    const expectedLabel = "markdown";
    const { stdout, exitCode } = await executeCli([filePath]);
    expect(exitCode).toBe(0);
    expect(stdout).toContain(filePath);
    expect(stdout).toContain(expectedLabel);
  });

  it("should process (by stream) a specific file and output the expected result", async () => {
    const filePath = "../README.md";
    const expectedLabel = "markdown";
    const { stdout, exitCode } = await executeCli(["--by-stream", filePath]);
    expect(exitCode).toBe(0);
    expect(stdout).toContain(filePath);
    expect(stdout).toContain(expectedLabel);
  });
});
