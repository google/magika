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

use crate::{ContentType, Input, Options};

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

impl std::fmt::Debug for Rules {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("Rules").field("content_types", &self.0.content_types).finish()
    }
}

impl Rules {
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

    /// Returns the content types the rules can identify, in source order.
    pub fn content_types(&self) -> &[ContentType] {
        &self.0.content_types
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
    /// Returns the rules bundled with Magika, compiled when `magika-rules` was built.
    fn builtin() -> &'static Self {
        static RULES: LazyLock<Compiled> =
            LazyLock::new(|| Compiled::new(RuleSet::bundled()).unwrap());
        &RULES
    }

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

    /// Returns the content types of the rules that match.
    fn scan(&self, prefix: &[u8], size: u64, tail: Option<&[u8]>) -> Vec<ContentType> {
        match self.set.scan(magika_rules::Input { prefix, size, tail }) {
            Outcome::Match(labels) => labels.into_iter().map(|i| self.content_types[i]).collect(),
            Outcome::InsufficientInput => Vec::new(),
        }
    }
}

/// What the rules decided about a file.
pub(crate) enum Decision {
    /// The rules identify the file.
    Ruled(ContentType),
    /// The rules leave the decision to the model.
    Undecided(Matched),
}

/// What the rules matched in a file they did not identify, kept for the model to be vetoed.
pub(crate) struct Matched {
    /// The content types of the rules that matched.
    content_types: Vec<ContentType>,
    /// The custom rules that ran, if any.
    custom: Option<Rules>,
    /// Whether the built-in rules ran.
    builtin: bool,
}

impl Matched {
    /// Returns whether rules that ran claim to never miss `content_type`, yet none matched.
    pub(crate) fn vetoes(&self, content_type: ContentType) -> bool {
        !self.content_types.contains(&content_type)
            && (self.custom.as_ref().is_some_and(|rules| rules.0.vetos.contains(&content_type))
                || self.builtin && Compiled::builtin().vetos.contains(&content_type))
    }
}

/// Runs the enabled rules on a file: custom rules first, then built-in rules.
///
/// Custom rules identify the file when the ones that match agree on one content type. Built-in
/// rules identify it when no custom rule matched and the ones that match agree. Returns `None`
/// when no rules are enabled.
///
/// The first block holds at least the first `PREFIX_LIMIT` bytes of the file, or the whole file
/// if it is shorter.
pub(crate) fn identify(
    options: &Options, first_block: &[u8], size: u64, mut file: impl Input,
) -> Result<Option<Decision>> {
    let custom = options.custom_rules.as_ref().map(|rules| &*rules.0);
    let builtin = options.use_rules.then(Compiled::builtin);
    if custom.is_none() && builtin.is_none() {
        return Ok(None);
    }
    let prefix = &first_block[..first_block.len().min(magika_rules::PREFIX_LIMIT)];
    // Both sets read the same tail: the end of a zip archive, back to its central directory.
    let mut tail = None;
    if let Some(set) =
        custom.into_iter().chain(builtin).map(|x| &x.set).find(|set| set.tail_len(prefix, size) > 0)
    {
        let tail_len = set.tail_len(prefix, size);
        let mut buf = vec![0; tail_len];
        file.read_at(&mut buf, size - tail_len as u64)?;
        if let Some(start) = set.tail_start(&buf, size) {
            buf = vec![0; (size - start) as usize];
            file.read_at(&mut buf, start)?;
        }
        tail = Some(buf);
    }
    let tail = tail.as_deref();
    let mut content_types = Vec::new();
    if let Some(custom) = custom {
        content_types = custom.scan(prefix, size, tail);
        if let &[content_type] = &content_types[..] {
            return Ok(Some(Decision::Ruled(content_type)));
        }
    }
    if let Some(builtin) = builtin {
        let matched = builtin.scan(prefix, size, tail);
        if let (true, &[content_type]) = (content_types.is_empty(), &matched[..]) {
            return Ok(Some(Decision::Ruled(content_type)));
        }
        for content_type in matched {
            if !content_types.contains(&content_type) {
                content_types.push(content_type);
            }
        }
    }
    let custom = options.custom_rules.clone();
    Ok(Some(Decision::Undecided(Matched { content_types, custom, builtin: builtin.is_some() })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Backend, FeaturesOrRuled, FileType, OverwriteReason, Runtime};

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
    fn custom_rules_without_false_negatives_veto_the_model() {
        let wav = |class| {
            let options = options(false, Some(&rule("wav", class, "$psd at 0")));
            let FeaturesOrRuled::Features(features) = extract("wav/test.wav", &options) else {
                unreachable!()
            };
            let runtime = Runtime::builder().with_backend(Backend::Cpu).build().unwrap();
            runtime.session().unwrap().identify_features(&features).unwrap()
        };
        let FileType::Inferred(vetoed) = wav("full") else { unreachable!() };
        assert_eq!(vetoed.inferred_type, ContentType::Wav);
        assert!(matches!(vetoed.content_type, Some((_, OverwriteReason::RulesVeto))));
        assert_eq!(wav("partial").content_type(), Some(ContentType::Wav));
    }

    #[test]
    fn features_extracted_without_rules_are_not_vetoed() {
        // WAV has built-in rules without false negatives, but they did not run.
        let FeaturesOrRuled::Features(features) = extract("wav/test.wav", &options(false, None))
        else {
            unreachable!()
        };
        let runtime = Runtime::builder().with_backend(Backend::Cpu).build().unwrap();
        let file_type = runtime.session().unwrap().identify_features(&features).unwrap();
        assert_eq!(file_type.content_type(), Some(ContentType::Wav));
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
        assert_eq!(Rules::from_files([custom]).unwrap().content_types(), [ContentType::Png]);
        let error = Rules::from_files([custom, "../../tests_data/basic/png/magika_test.png"]);
        let error = format!("{:#}", error.unwrap_err());
        assert!(error.starts_with("reading rules from ../../tests_data/basic/png"), "{error}");
    }

    #[test]
    fn rules_are_shared_by_clones() {
        let rules = Rules::compile(&rule("txt", "full", "$psd at 0")).unwrap();
        assert!(Arc::ptr_eq(&rules.0, &rules.clone().0));
        assert_eq!(rules.content_types(), [ContentType::Txt]);
    }
}
