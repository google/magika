// Copyright 2026 Google LLC
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

//! Regenerates `include/magika.h` using `cbindgen`.

use std::path::Path;

use anyhow::Result;

fn main() -> Result<()> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let _ = cbindgen::generate(dir)?.write_to_file(dir.join("include/magika.h"));
    Ok(())
}
