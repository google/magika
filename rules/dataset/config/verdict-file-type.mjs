// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0
import {pathToFileURL} from 'node:url';
import {resolve} from 'node:path';
const {fileTypeFromFile} = await import(pathToFileURL(resolve(process.argv[2])).href);
console.log(JSON.stringify(await fileTypeFromFile(process.argv[3]) ?? {matches: []}));
