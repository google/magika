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

use anyhow::Result;

use crate::{Backend, RulesMode, Runtime};

/// Configures and creates a Magika runtime.
#[derive(Clone, Debug, Default)]
pub struct Builder {
    rules_mode: RulesMode,
    #[cfg(feature = "yara-rules")]
    ruleset: Option<crate::RuleSet>,
    backend: Option<Backend>,
    max_batch: Option<usize>,
}

impl Builder {
    /// Selects the promoted rule allowlist; defaults to off.
    pub fn with_rules_mode(mut self, mode: RulesMode) -> Self {
        self.rules_mode = mode;
        self
    }

    /// Supplies a loaded pack. It is used only when rules mode is `Enforce`.
    #[cfg(feature = "yara-rules")]
    pub fn with_ruleset(mut self, ruleset: crate::RuleSet) -> Self {
        self.ruleset = Some(ruleset);
        self
    }

    /// Selects a specific backend for inference.
    pub fn with_backend(mut self, backend: Backend) -> Self {
        self.backend = Some(backend);
        self
    }

    /// Declares the largest batch this session will ever be asked to identify.
    ///
    /// Declaring a smaller maximum skips unreachable fixed plans and makes startup cheaper. On a
    /// CPU, smaller tails are padded through the largest resident plan; GPU requests are
    /// decomposed over the original resident classes.
    pub fn with_max_batch(mut self, max_batch: usize) -> Self {
        self.max_batch = Some(max_batch);
        self
    }

    /// Consumes the builder to create a Magika runtime.
    /// Returns an error if requested rule enforcement cannot be initialized.
    pub fn build(self) -> Result<Runtime> {
        self.rules_mode.check()?;
        #[cfg(feature = "yara-rules")]
        let ruleset = match (self.rules_mode, self.ruleset) {
            (RulesMode::Enforce, None) => Some(crate::RuleSet::bundled()?),
            (_, ruleset) => ruleset,
        };
        let runtime = Runtime::new_internal(
            Backend::to_request(self.backend),
            self.max_batch,
            self.rules_mode,
        )?;
        #[cfg(feature = "yara-rules")]
        let runtime = {
            let mut runtime = runtime;
            runtime.ruleset = ruleset;
            runtime
        };
        Ok(runtime)
    }
}
