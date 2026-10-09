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

use std::collections::HashSet;
use std::path::Path;
use std::sync::{Arc, LazyLock};

use anyhow::{Context, Result, anyhow, bail, ensure};
use magika_rules::{Class, Outcome, RuleSet, Source};

use crate::ContentType;

// Rules scan the first block that feature extraction reads anyway, so they never read more.
const _: () = assert!(crate::model::CONFIG.block_size >= magika_rules::PREFIX_LIMIT);

/// Format rules supplied by the caller, checked before the built-in rules.
///
/// Rules are written in the YARA subset of the `magika-rules` crate, with the same metadata: an
/// enforced rule needs a `label` that is a Magika content type, `fp_rate = 0`, and a class. When
/// every enforced rule of a content type has `class = "full"`, the rules claim to never miss
/// it, so a model prediction of that content type is vetoed when none of them matched.
///
/// Cloning is cheap: clones share the compiled rules.
#[derive(Clone)]
pub struct Rules(Arc<Compiled>);

impl PartialEq for Rules {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl std::fmt::Debug for Rules {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("Rules").field("content_types", &self.0.content_types).finish()
    }
}

impl Rules {
    /// Returns the rules bundled with Magika, compiled when `magika-rules` was built.
    pub(crate) fn builtin() -> &'static Self {
        static RULES: LazyLock<Rules> =
            LazyLock::new(|| Rules(Arc::new(Compiled::new(RuleSet::bundled()).unwrap())));
        &RULES
    }

    /// Compiles rules from YARA text.
    pub fn compile(text: &str) -> Result<Self> {
        let source = Source::parse(text)?;
        let compiled = Compiled::new(RuleSet::compile(&source)?)?;
        ensure!(!compiled.content_types.is_empty(), "no enforced rule labels a content type");
        Ok(Rules(Arc::new(compiled)))
    }

    /// Compiles the rules of several files together, as if they were one.
    ///
    /// Each file is validated on its own first, so that an error names the file and its line.
    pub fn from_files(paths: impl IntoIterator<Item = impl AsRef<Path>>) -> Result<Self> {
        let mut text = String::new();
        for path in paths {
            let path = path.as_ref();
            let file = std::fs::read_to_string(path)
                .with_context(|| format!("reading rules from {}", path.display()))?;
            Source::parse(&file).with_context(|| format!("in rules from {}", path.display()))?;
            text += &file;
            if !text.ends_with('\n') {
                text.push('\n');
            }
        }
        Self::compile(&text)
    }

    /// Returns the content types of the rules that match a file.
    ///
    /// The first block holds at least the first `PREFIX_LIMIT` bytes of the file, or the whole
    /// file if it is shorter.
    pub(crate) fn identify(
        &self, first_block: &[u8], size: u64, mut file: impl crate::Input,
    ) -> Result<Vec<ContentType>> {
        let rules = &self.0;
        let prefix = &first_block[..first_block.len().min(magika_rules::PREFIX_LIMIT)];
        let mut tail = None;
        let tail_len = rules.set.tail_len(prefix, size);
        if tail_len > 0 {
            let mut buf = vec![0; tail_len];
            file.read_at(&mut buf, size - tail_len as u64)?;
            if let Some(start) = rules.set.tail_start(&buf, size) {
                buf = vec![0; (size - start) as usize];
                file.read_at(&mut buf, start)?;
            }
            tail = Some(buf);
        }
        let tail = tail.as_deref();
        Ok(match rules.set.scan(magika_rules::Input { prefix, size, tail }) {
            Outcome::Match(labels) => labels.into_iter().map(|i| rules.content_types[i]).collect(),
            Outcome::InsufficientInput => Vec::new(),
        })
    }

    /// Returns whether the rules cover an unmatched content type with no false negatives.
    pub(crate) fn veto(&self, matched: &[ContentType], content_type: ContentType) -> bool {
        !matched.contains(&content_type) && self.0.vetos.contains(&content_type)
    }
}

/// A compiled rule set and the content type of each of its labels.
struct Compiled {
    set: RuleSet,
    /// The map from labels (index in `set.labels()`) to content types.
    content_types: Vec<ContentType>,
    /// The set of content types covered by rules without false negatives.
    vetos: HashSet<ContentType>,
}

impl Compiled {
    /// Wraps compiled rules, failing if a rule labels a content type that Magika does not know.
    fn new(set: RuleSet) -> Result<Self> {
        let mut content_types = Vec::with_capacity(set.labels().len());
        let mut vetos = HashSet::new();
        for label in set.labels() {
            let content_type = ContentType::from_label(label)
                .ok_or_else(|| anyhow!("rule label {label:?} is not a Magika content type"))?;
            content_types.push(content_type);
            let mut class = None;
            for rule in set.rules() {
                if !rule.enforced || rule.label.as_ref() != Some(label) {
                    continue;
                }
                if let Some(prev) = class.replace(rule.class) {
                    ensure!(
                        prev == rule.class,
                        "enforced rules for label {label:?} disagree on class: {prev:?} vs {:?}",
                        rule.class
                    );
                }
            }
            match class {
                Some(Class::Full) => drop(vetos.insert(content_type)),
                Some(Class::Partial) => (),
                _ => bail!("label {label:?} has no valid enforced rule class: {class:?}"),
            }
        }
        Ok(Compiled { set, content_types, vetos })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Backend, FeaturesOrRuled, FileType, Options, OverwriteReason, Runtime};

    /// One enforced rule labeling `label` with `class`, matching when `condition` holds.
    fn rule(label: &str, class: &str, condition: &str) -> String {
        let fn_rate = if class == "full" { "0" } else { "0.5" };
        format!(
            r#"rule custom_{label} {{
                meta: label = "{label}" enforced = true class = "{class}" fp_rate = 0 fn_rate = {fn_rate}
                strings: $psd = "8BPS"
                condition: {condition}
            }}"#
        )
    }

    fn options(use_rules: bool, custom: Option<&str>) -> Options {
        let custom_rules = custom.map(|text| Rules::compile(text).unwrap());
        Options { use_rules, custom_rules, ..Options::default() }
    }

    fn extract(path: &str, options: &Options) -> FeaturesOrRuled {
        FeaturesOrRuled::extract_file(format!("../../tests_data/basic/{path}"), options).unwrap()
    }

    fn ruled(path: &str, options: &Options) -> Option<ContentType> {
        match extract(path, options) {
            FeaturesOrRuled::Ruled(FileType::Ruled(content_type)) => Some(content_type),
            _ => None,
        }
    }

    #[test]
    fn custom_rules_come_before_builtin_rules() {
        let psd = "psd/MagikaTest.psd";
        let txt = rule("txt", "full", "$psd at 0");
        assert_eq!(ruled(psd, &options(true, None)), Some(ContentType::Psd));
        assert_eq!(ruled(psd, &options(true, Some(&txt))), Some(ContentType::Txt));
        assert_eq!(ruled(psd, &options(false, Some(&txt))), Some(ContentType::Txt));
        let unmatched = rule("txt", "full", "$psd at 1");
        assert_eq!(ruled(psd, &options(true, Some(&unmatched))), Some(ContentType::Psd));
        assert_eq!(ruled(psd, &options(false, Some(&unmatched))), None);
    }

    #[test]
    fn custom_rules_mismatch_extract_identify() {
        let wav = |class| {
            let options = options(false, Some(&rule("wav", class, "$psd at 0")));
            let FeaturesOrRuled::Features(features) = extract("wav/test.wav", &options) else {
                unreachable!()
            };
            let mut runtime = Runtime::builder().with_backend(Backend::Cpu).build().unwrap();
            *runtime.options_mut() = options.clone();
            runtime.session().unwrap().identify_features(&features).unwrap()
        };
        let FileType::Inferred(vetoed) = wav("full") else { unreachable!() };
        assert_eq!(vetoed.inferred_type, ContentType::Wav);
        assert!(matches!(vetoed.content_type, Some((_, OverwriteReason::RulesVeto))));
        assert_eq!(wav("partial").content_type(), Some(ContentType::Wav));
    }

    #[test]
    fn rules_mismatch_extract_identify() {
        let FeaturesOrRuled::Features(features) = extract("wav/test.wav", &options(false, None))
        else {
            unreachable!()
        };
        let runtime = Runtime::builder().with_backend(Backend::Cpu).build().unwrap();
        let error = runtime.session().unwrap().identify_features(&features).unwrap_err();
        assert_eq!(error.to_string(), "use_rules mismatch between extract and identify");
    }

    #[test]
    fn custom_rules_read_the_end_of_a_zip_archive() {
        // The central directory, which lists the entry names, is at the end of the 290 KB file.
        let names = r#"zip_valid == 1 and zip_names contains "word/document.xml""#;
        let custom = rule("odt", "partial", names);
        assert_eq!(ruled("docx/doc.docx", &options(false, Some(&custom))), Some(ContentType::Odt));
    }

    #[test]
    fn invalid_custom_rules_are_rejected() {
        let error = |text: &str| Rules::compile(text).unwrap_err().to_string();
        assert_eq!(
            error(&rule("not_a_type", "full", "$psd at 0")),
            r#"rule label "not_a_type" is not a Magika content type"#
        );
        assert!(error("rule broken {").starts_with("invalid YARA"), "{}", error("rule broken {"));
        let no_label = r#"rule r { meta: enforced = true class = "full" fp_rate = 0 fn_rate = 0
            condition: true }"#;
        assert!(error(no_label).starts_with("rule r: "), "{}", error(no_label));
        assert_eq!(error("rule r { condition: true }"), "no enforced rule labels a content type");
    }

    #[test]
    fn rules_from_files_name_the_file_in_errors() {
        let custom = "../../tests_data/rules/custom.yar";
        assert_eq!(Rules::from_files([custom]).unwrap().0.content_types, [ContentType::Png]);
        let error = Rules::from_files([custom, "../../tests_data/basic/png/magika_test.png"]);
        let error = format!("{:#}", error.unwrap_err());
        assert!(error.starts_with("reading rules from ../../tests_data/basic/png"), "{error}");
    }

    #[test]
    fn rules_are_shared_by_clones() {
        let rules = Rules::compile(&rule("txt", "full", "$psd at 0")).unwrap();
        assert!(Arc::ptr_eq(&rules.0, &rules.clone().0));
        assert_eq!(rules.0.content_types, [ContentType::Txt]);
    }
}
