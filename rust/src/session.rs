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

use onnxruntime::session::Session;

use crate::{MagikaBuilder, MagikaConfig, MagikaInput, MagikaOutput, MagikaResult};

/// A Magika session to identify files.
#[derive(Debug)]
pub struct MagikaSession {
    pub(crate) session: Session<'static>,
    pub(crate) config: MagikaConfig,
}

impl MagikaSession {
    /// Initializes a new Magika session builder with default values.
    pub fn build() -> MagikaBuilder {
        MagikaBuilder::new()
    }

    /// Identifies a single file.
    pub fn identify(&mut self, file: impl MagikaInput) -> MagikaResult<MagikaOutput> {
        let results = self.identify_many(std::iter::once(file))?;
        let [result] = results.try_into().ok().unwrap();
        Ok(result)
    }

    /// Identifies multiple files in parallel.
    pub fn identify_many(
        &mut self,
        files: impl Iterator<Item = impl MagikaInput>,
    ) -> MagikaResult<Vec<MagikaOutput>> {
        let input = files
            .map(|x| self.config.convert_input(x))
            .collect::<MagikaResult<Vec<_>>>()?;
        let len = input.len();
        assert_eq!(len, 1, "only 1 file in parallel is supported at this time");
        let output = self.session.run::<f32, f32, _>(input)?;
        assert_eq!(output.len(), len);
        Ok(output
            .into_iter()
            .map(|x| self.config.convert_output(x))
            .collect())
    }
}
