// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0

//! Compile a bounded YARA subset into native regexes and logical combinations.
//! This module runs once when loading a pack, never while identifying a file.

use std::collections::{HashMap, HashSet};

use anyhow::{bail, ensure, Context, Result};
use regex_syntax::hir::{Hir, HirKind};
use yara_x_parser::ast::{
    BinaryExpr, Expr, HexSubPattern, HexToken, Item, MatchAnchor, MetaValue, Pattern, Rule,
    RuleFlags, AST,
};

use super::metadata::enforced;
use super::preprocess::{self, View};
use super::{EXTERNAL_BYTES, PREFIX_LIMIT};
use crate::ContentType;

const HS_FLAG_SINGLEMATCH: u32 = 8;
const HS_FLAG_COMBINATION: u32 = 512;
const HS_FLAG_QUIET: u32 = 1024;

/// The scan stream a rule executes in. Every terminal rule belongs to exactly one: a
/// condition cannot relate bytes of the original prefix to preprocessor facts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum Domain {
    /// Stream A: the size header and the original prefix; patterns and integer reads.
    Prefix,
    /// Stream B: the facts header and views, scanned only when preprocessing produced them.
    Facts,
}

/// One value per scan stream. Serialized as the pack manifest's `labels` object, so it is
/// as strict about unknown fields as the manifest itself.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Streams<T> {
    pub(super) prefix: T,
    pub(super) facts: T,
}

impl<T> Streams<T> {
    pub(super) fn get(&self, domain: Domain) -> &T {
        match domain {
            Domain::Prefix => &self.prefix,
            Domain::Facts => &self.facts,
        }
    }

    pub(super) fn get_mut(&mut self, domain: Domain) -> &mut T {
        match domain {
            Domain::Prefix => &mut self.prefix,
            Domain::Facts => &mut self.facts,
        }
    }

    pub(super) fn as_ref(&self) -> Streams<&T> {
        Streams { prefix: &self.prefix, facts: &self.facts }
    }

    pub(super) fn map<U>(self, mut f: impl FnMut(T) -> U) -> Streams<U> {
        Streams { prefix: f(self.prefix), facts: f(self.facts) }
    }

    pub(super) fn try_map<U>(self, mut f: impl FnMut(T) -> Result<U>) -> Result<Streams<U>> {
        Ok(Streams { prefix: f(self.prefix)?, facts: f(self.facts)? })
    }

    pub(super) fn zip<U>(self, other: Streams<U>) -> Streams<(T, U)> {
        Streams { prefix: (self.prefix, other.prefix), facts: (self.facts, other.facts) }
    }
}

/// Inclusive bounds on where a match may *end*: the stream offset just past its last byte,
/// as Vectorscan's `min_offset`/`max_offset` extended parameters count it. A literal of
/// `n` bytes confined to `[start, start + size)` therefore ends within
/// `start + n ..= start + size`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct Bounds {
    /// The earliest end offset of an acceptable match, inclusive.
    pub(super) min_end_offset: u64,
    /// The latest end offset of an acceptable match, inclusive.
    pub(super) max_end_offset: u64,
}

/// The native expressions of one stream, in Vectorscan's parallel-array form.
#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct Expressions {
    pub(super) expressions: Vec<String>,
    pub(super) flags: Vec<u32>,
    /// End-offset bounds for floating view literals; `None` keeps the expression's anchoring.
    pub(super) bounds: Vec<Option<Bounds>>,
    pub(super) outputs: Vec<Option<ContentType>>,
}

/// The compiled pack: one expression set per scan stream.
///
/// `prefix` targets stream A, `[16-byte size header][original prefix]`, and lowers exactly
/// as it did before the facts stream existed. `facts` targets stream B, the synthetic
/// `[facts header][views]` layout that [`preprocess`] defines. Every terminal rule lands in
/// exactly one of them; a stream with no terminal rule stays empty and is never compiled
/// into a native database.
pub(super) type Program = Streams<Expressions>;

#[cfg(test)]
thread_local! { pub(super) static COMPILATIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) }; }

pub(super) fn compile(source: &str) -> Result<Program> {
    #[cfg(test)]
    COMPILATIONS.set(COMPILATIONS.get() + 1);
    ensure!(source.len() <= 4 * 1024 * 1024, "YARA source exceeds 4 MiB");
    let ast = AST::from(source);
    ensure!(ast.errors().is_empty(), "invalid YARA: {:?}", ast.errors());
    let mut rules = HashMap::new();
    for item in ast.items() {
        let Item::Rule(rule) = item else { bail!("imports and includes are not supported") };
        ensure!(!rule.flags.contains(RuleFlags::Global), "global rules are not supported");
        ensure!(rules.insert(rule.identifier.name, rule).is_none(), "duplicate rule ID");
    }
    let mut compiler = Compiler {
        rules,
        requirements: HashMap::new(),
        visiting: HashSet::new(),
        compiled: HashMap::new(),
        domain: Domain::Prefix,
        targets: Streams::default(),
    };
    for rule in ast.rules() {
        if rule.flags.contains(RuleFlags::Private) {
            continue;
        }
        let enabled = enforced(rule, None)?;
        let mut label = None;
        for meta in rule.meta.iter().flatten() {
            match (meta.identifier.name, &meta.value) {
                ("label", MetaValue::String((value, _))) => {
                    ensure!(label.replace(*value).is_none(), "duplicate label metadata");
                }
                ("label", _) => bail!("label must be a string"),
                _ => (),
            }
        }
        let label = label.and_then(ContentType::from_label).with_context(|| {
            format!("rule {} needs a canonical Magika label", rule.identifier.name)
        })?;
        // Explicit opt-in for each terminal rule. Metadata is configuration, never evidence.
        if !enabled {
            continue;
        }
        let context = || format!("rule {}", rule.identifier.name);
        // A rule using only sizes and constants scans the prefix stream, as it always did.
        compiler.domain =
            compiler.requirement(rule).with_context(context)?.unwrap_or(Domain::Prefix);
        let expression = compiler.rule(rule).with_context(context)?;
        let present = compiler.constant(true)?;
        compiler.push(
            format!("({expression})&{present}"),
            HS_FLAG_COMBINATION | HS_FLAG_SINGLEMATCH,
            None,
            Some(label),
        )?;
    }
    Ok(compiler.targets.map(|target| target.expressions))
}

/// The expressions being accumulated for one stream, with its atom cache: an identical
/// pattern under identical bounds shares one expression ID within a stream, never across
/// streams, since IDs index each stream's own output table.
#[derive(Default)]
struct Target {
    expressions: Expressions,
    atoms: HashMap<(String, Option<Bounds>), usize>,
}

struct Compiler<'a> {
    rules: HashMap<&'a str, &'a Rule<'a>>,
    /// The domain each rule's condition needs, transitively; `None` when neutral.
    requirements: HashMap<&'a str, Option<Domain>>,
    visiting: HashSet<&'a str>,
    /// Lowered rules per domain: a neutral helper may serve terminals of both.
    compiled: HashMap<(&'a str, Domain), String>,
    /// The domain of the terminal rule being lowered.
    domain: Domain,
    targets: Streams<Target>,
}

fn merge(known: Option<Domain>, found: Option<Domain>, rule: &str) -> Result<Option<Domain>> {
    match (known, found) {
        (Some(known), Some(found)) if known != found => {
            bail!("rule `{rule}` mixes prefix patterns with preprocessor facts")
        }
        (Some(known), _) => Ok(Some(known)),
        (None, found) => Ok(found),
    }
}

impl<'a> Compiler<'a> {
    fn push(
        &mut self, expression: String, flags: u32, bounds: Option<Bounds>,
        label: Option<ContentType>,
    ) -> Result<usize> {
        let program = &mut self.targets.get_mut(self.domain).expressions;
        ensure!(
            program.expressions.len() < 100_000 && expression.len() <= 1_048_576,
            "compiled rule complexity limit exceeded"
        );
        let id = program.expressions.len();
        program.expressions.push(expression);
        program.flags.push(flags);
        program.bounds.push(bounds);
        program.outputs.push(label);
        Ok(id)
    }

    /// An atom anchored by its own `^` prefix, or a constant.
    fn atom(&mut self, pattern: String) -> Result<String> {
        self.bounded_atom(pattern, None)
    }

    fn bounded_atom(&mut self, pattern: String, bounds: Option<Bounds>) -> Result<String> {
        let key = (pattern, bounds);
        if let Some(id) = self.targets.get(self.domain).atoms.get(&key) {
            return Ok(id.to_string());
        }
        let id = self.push(key.0.clone(), HS_FLAG_QUIET, bounds, None)?;
        self.targets.get_mut(self.domain).atoms.insert(key, id);
        Ok(id.to_string())
    }

    /// The domain a rule needs, transitively through references, or `None` when it uses
    /// only sizes and constants. A rule needing both domains is an error.
    fn requirement(&mut self, rule: &'a Rule<'a>) -> Result<Option<Domain>> {
        let name = rule.identifier.name;
        if let Some(domain) = self.requirements.get(name) {
            return Ok(*domain);
        }
        // Unenforced variants lower to a constant, which is neutral.
        let domain = if enforced(rule, None)? {
            ensure!(
                self.visiting.len() < 128 && self.visiting.insert(name),
                "cyclic or excessively deep rule references"
            );
            let result = self.requires(&rule.condition, name);
            self.visiting.remove(name);
            result?
        } else {
            None
        };
        self.requirements.insert(name, domain);
        Ok(domain)
    }

    fn requires(&mut self, expr: &'a Expr<'a>, rule: &str) -> Result<Option<Domain>> {
        Ok(match expr {
            Expr::And(node) | Expr::Or(node) => {
                let mut domain = None;
                for operand in &node.operands {
                    domain = merge(domain, self.requires(operand, rule)?, rule)?;
                }
                domain
            }
            Expr::Ident(id) => match self.rules.get(id.name).copied() {
                Some(target) => self.requirement(target)?,
                None => None, // Lowering reports the unknown reference.
            },
            Expr::PatternMatch(_) => Some(Domain::Prefix),
            Expr::Eq(node)
            | Expr::Ne(node)
            | Expr::Lt(node)
            | Expr::Le(node)
            | Expr::Gt(node)
            | Expr::Ge(node) => {
                let read = match &node.lhs {
                    Expr::Mod(remainder) => remainder.operands.first(),
                    lhs => Some(lhs),
                };
                match read {
                    Some(Expr::FuncCall(_)) => Some(Domain::Prefix),
                    Some(Expr::Ident(id)) => {
                        preprocess::fact(id.name).filter(|x| !x.is_shared()).map(|_| Domain::Facts)
                    }
                    _ => None,
                }
            }
            Expr::Contains(_) | Expr::StartsWith(_) => Some(Domain::Facts),
            _ => None, // Unsupported conditions are rejected by lowering.
        })
    }

    /// Lowering is only valid in the domain the requirement pass assigned. `requires`
    /// classifies every construct `condition` lowers and `merge` already rejected mixed
    /// rules with the user-facing message, so this failing means the two passes disagree:
    /// an internal invariant violation, not a diagnosable rule.
    fn needs(&self, domain: Domain, rule: &Rule<'_>) -> Result<()> {
        ensure!(
            self.domain == domain,
            "internal: rule `{}` lowered a {domain:?} construct in the {:?} stream",
            rule.identifier.name,
            self.domain
        );
        Ok(())
    }

    fn rule(&mut self, rule: &'a Rule<'a>) -> Result<String> {
        let key = (rule.identifier.name, self.domain);
        if let Some(expression) = self.compiled.get(&key) {
            return Ok(expression.clone());
        }
        if !enforced(rule, None)? {
            // Disable the complete variant, including when used inside a larger AND/OR.
            // Only ordinary private helpers are available without explicit enablement.
            let expression = self.constant(false)?;
            self.compiled.insert(key, expression.clone());
            return Ok(expression);
        }
        ensure!(
            self.visiting.len() < 128 && self.visiting.insert(rule.identifier.name),
            "cyclic or excessively deep rule references"
        );
        let mut ids = HashSet::new();
        for pattern in rule.patterns.iter().flatten() {
            ensure!(ids.insert(pattern.identifier().name), "duplicate pattern ID");
        }
        let result = self.condition(&rule.condition, rule);
        self.visiting.remove(rule.identifier.name);
        if let Ok(expression) = &result {
            self.compiled.insert(key, expression.clone());
        }
        result
    }

    fn condition(&mut self, expr: &'a Expr<'a>, rule: &'a Rule<'a>) -> Result<String> {
        match expr {
            Expr::And(node) | Expr::Or(node) => {
                let parts: Vec<_> =
                    node.operands.iter().map(|x| self.condition(x, rule)).collect::<Result<_>>()?;
                let result = format!(
                    "({})",
                    parts.join(if matches!(expr, Expr::And(_)) { "&" } else { "|" })
                );
                ensure!(result.len() <= 1_048_576, "condition expansion exceeds budget");
                Ok(result)
            }
            Expr::Ident(id) => {
                let target = *self.rules.get(id.name).context("unknown rule reference")?;
                self.rule(target)
            }
            Expr::PatternMatch(node) => {
                self.needs(Domain::Prefix, rule)?;
                let pattern = rule
                    .patterns
                    .iter()
                    .flatten()
                    .find(|x| x.identifier().name == node.identifier.name)
                    .context("unknown pattern")?;
                let (lo, hi) = match &node.anchor {
                    Some(MatchAnchor::At(at)) => {
                        let n = number(&at.expr)?;
                        (n, n)
                    }
                    Some(MatchAnchor::In(range)) => {
                        (number(&range.range.lower_bound)?, number(&range.range.upper_bound)?)
                    }
                    None => bail!("patterns require a literal at/in bound"),
                };
                ensure!(lo <= hi && hi <= PREFIX_LIMIT as u64, "invalid pattern offset bound");
                let pattern = pattern_regex(pattern)?;
                let hir = regex_syntax::ParserBuilder::new()
                    .unicode(false)
                    .utf8(false)
                    .build()
                    .parse(&pattern)
                    .context("invalid byte regex")?;
                let width = hir.properties().maximum_len().context("unbounded regex")?;
                ensure!(
                    hir.properties().minimum_len().unwrap_or(0) > 0
                        && width as u64 + hi <= PREFIX_LIMIT as u64,
                    "pattern exceeds prefix budget or matches empty input"
                );
                // Re-emit the parsed byte expression: source dialect escapes cannot change native semantics.
                self.atom(format!(
                    "^{}{{{},{}}}(?:{})",
                    ANY,
                    lo + EXTERNAL_BYTES as u64,
                    hi + EXTERNAL_BYTES as u64,
                    byte_regex(&hir)?
                ))
            }
            Expr::Eq(node)
            | Expr::Ne(node)
            | Expr::Lt(node)
            | Expr::Le(node)
            | Expr::Gt(node)
            | Expr::Ge(node) => {
                let (lhs, modulus) = if let Expr::Mod(remainder) = &node.lhs {
                    ensure!(
                        matches!(expr, Expr::Eq(_)) && remainder.operands.len() == 2,
                        "modulo requires equality and one literal divisor"
                    );
                    let divisor = number(&remainder.operands[1])?;
                    ensure!(divisor.is_power_of_two(), "modulo divisor must be a power of two");
                    (&remainder.operands[0], Some(divisor))
                } else {
                    (&node.lhs, None)
                };
                let (offset, width, big_endian) = match lhs {
                    Expr::Ident(id) => {
                        let fact = preprocess::fact(id.name)
                            .with_context(|| format!("unknown fact `{}`", id.name))?;
                        // The size header is at the same offsets in both streams.
                        if !fact.is_shared() {
                            self.needs(Domain::Facts, rule)?;
                        }
                        (fact.offset, fact.width, true)
                    }
                    Expr::FuncCall(call) if call.object.is_none() && call.args.len() == 1 => {
                        self.needs(Domain::Prefix, rule)?;
                        let (width, be) = match call.identifier.name {
                            "uint8" => (1, true),
                            "uint16" => (2, false),
                            "uint16be" => (2, true),
                            "uint32" => (4, false),
                            "uint32be" => (4, true),
                            _ => bail!("unsupported integer function"),
                        };
                        let offset = number(&call.args[0])?;
                        ensure!(
                            offset <= PREFIX_LIMIT as u64 - width,
                            "integer read exceeds prefix budget"
                        );
                        (offset as usize + EXTERNAL_BYTES, width as usize, be)
                    }
                    _ => bail!("comparisons require an external size or fixed unsigned read"),
                };
                let value = number(&node.rhs)?;
                let max = if width == 8 { i64::MAX as u64 } else { (1_u64 << (8 * width)) - 1 };
                if let Some(divisor) = modulus {
                    if value >= divisor || value > max {
                        return self.constant(false);
                    }
                    let mut bytes = Vec::new();
                    for i in 0..width {
                        let mask = ((divisor - 1) >> (i * 8)) as u8;
                        let wanted = (value >> (i * 8)) as u8;
                        bytes.push(match mask {
                            0 => ANY.to_string(),
                            255 => byte(wanted),
                            _ => format!(
                                "[{}]",
                                (0..=255_u8)
                                    .filter(|x| x & mask == wanted)
                                    .map(byte)
                                    .collect::<String>()
                            ),
                        });
                    }
                    if big_endian {
                        bytes.reverse();
                    }
                    // Consume the entire field, even for modulo 1: an absent read is
                    // undefined in YARA and must not turn into an unconditional match.
                    let skip =
                        if offset == 0 { String::new() } else { format!("{ANY}{{{offset}}}") };
                    return self.atom(format!("^{skip}{}", bytes.concat()));
                }
                let mut ranges = Vec::new();
                if matches!(expr, Expr::Eq(_) | Expr::Le(_) | Expr::Ge(_)) && value <= max {
                    ranges.push((value, value));
                }
                if matches!(expr, Expr::Ne(_) | Expr::Lt(_) | Expr::Le(_)) && value > 0 {
                    ranges.push((0, (value - 1).min(max)));
                }
                if matches!(expr, Expr::Ne(_) | Expr::Gt(_) | Expr::Ge(_)) && value < max {
                    ranges.push((value + 1, max));
                }
                if ranges.is_empty() {
                    return self.constant(false);
                }
                let alternatives: Vec<_> = ranges
                    .into_iter()
                    .flat_map(|(lo, hi)| interval(lo, hi, width, big_endian))
                    .collect();
                let skip = if offset == 0 { String::new() } else { format!("{ANY}{{{offset}}}") };
                self.atom(format!("^{skip}(?:{})", alternatives.join("|")))
            }
            Expr::Contains(node) | Expr::StartsWith(node) => {
                self.needs(Domain::Facts, rule)?;
                let (view, literal) = view_literal(node)?;
                // A floating literal placed by the end offset of its last byte: it may
                // neither start before the view nor extend past it.
                let earliest = (view.offset + literal.len()) as u64;
                let latest = match expr {
                    Expr::Contains(_) => (view.offset + view.size) as u64,
                    _ => earliest,
                };
                let pattern = literal.iter().map(|x| byte(*x)).collect();
                let bounds = Bounds { min_end_offset: earliest, max_end_offset: latest };
                self.bounded_atom(pattern, Some(bounds))
            }
            Expr::IContains(_)
            | Expr::IStartsWith(_)
            | Expr::EndsWith(_)
            | Expr::IEndsWith(_)
            | Expr::Matches(_) => {
                bail!("views support only `contains` and `startswith` with a literal string")
            }
            Expr::True { .. } => self.constant(true),
            Expr::False { .. } => self.constant(false),
            _ => bail!("unsupported YARA condition; no predicates were discarded"),
        }
    }

    fn constant(&mut self, value: bool) -> Result<String> {
        if value {
            self.atom(format!("^{ANY}"))
        } else {
            // Contradictory positive facts, rather than temporal negation in a combination.
            Ok(format!("({}&{})", self.atom("^\\x00".into())?, self.atom("^\\x01".into())?))
        }
    }
}

/// The view and literal bytes of a `contains`/`startswith` membership test.
fn view_literal<'e>(node: &'e BinaryExpr<'_>) -> Result<(&'static View, &'e [u8])> {
    let Expr::Ident(id) = &node.lhs else {
        bail!("view membership requires a view identifier on the left")
    };
    let view = preprocess::view(id.name).with_context(|| format!("unknown view `{}`", id.name))?;
    let Expr::LiteralString(literal) = &node.rhs else {
        bail!("view membership requires a literal string on the right")
    };
    let bytes: &[u8] = &literal.value;
    ensure!(
        !bytes.is_empty() && bytes.len() <= view.size,
        "view literal must be 1 to {} bytes",
        view.size
    );
    Ok((view, bytes))
}

const ANY: &str = "[\\x00-\\xff]";
fn number(expr: &Expr<'_>) -> Result<u64> {
    let Expr::LiteralInteger(value) = expr else { bail!("expected a nonnegative integer literal") };
    u64::try_from(value.value).context("expected a nonnegative integer literal")
}
fn byte(value: u8) -> String {
    format!("\\x{value:02x}")
}
fn class(lo: u8, hi: u8) -> String {
    if lo == hi {
        byte(lo)
    } else {
        format!("[{}-{}]", byte(lo), byte(hi))
    }
}

// Partition an unsigned interval into aligned binary prefixes at compile time. Each prefix is
// a bounded native byte pattern; no Rust comparison of input fields occurs during scanning.
fn interval(mut lo: u64, hi: u64, width: usize, big_endian: bool) -> Vec<String> {
    let mut result = Vec::new();
    loop {
        let space = hi as u128 - lo as u128 + 1;
        let bits = lo.trailing_zeros().min(127 - space.leading_zeros());
        let mut bytes = Vec::new();
        for i in 0..width {
            let shift = i * 8;
            let free = (bits as usize).saturating_sub(shift).min(8);
            let value = (lo >> shift) as u8;
            bytes.push(class(value, value | ((1_u16 << free) - 1) as u8));
        }
        if big_endian {
            bytes.reverse();
        }
        result.push(bytes.concat());
        let next = lo as u128 + (1_u128 << bits);
        if next > hi as u128 {
            break;
        }
        lo = next as u64;
    }
    result
}

fn pattern_regex(pattern: &Pattern<'_>) -> Result<String> {
    ensure!(pattern.modifiers().is_empty(), "pattern modifiers are not supported yet");
    Ok(match pattern {
        Pattern::Text(p) => p.text.value.iter().map(|b| byte(*b)).collect(),
        Pattern::Hex(p) => hex_regex(&p.sub_patterns)?,
        Pattern::Regexp(p) => {
            let hir = regex_syntax::ParserBuilder::new()
                .unicode(false)
                .utf8(false)
                .case_insensitive(p.regexp.case_insensitive)
                .dot_matches_new_line(p.regexp.dot_matches_new_line)
                .build()
                .parse(p.regexp.src)
                .context("invalid byte regex")?;
            byte_regex(&hir)?
        }
    })
}
fn hex_regex(pattern: &HexSubPattern) -> Result<String> {
    let mut result = String::new();
    for token in &pattern.0 {
        result.push_str(&match token {
            HexToken::Byte(b) | HexToken::NotByte(b) => {
                let values: Vec<_> = (0..=255_u8)
                    .filter(|x| ((*x & b.mask) == b.value) != matches!(token, HexToken::NotByte(_)))
                    .map(byte)
                    .collect();
                ensure!(!values.is_empty(), "empty hex byte class");
                format!("[{}]", values.concat())
            }
            HexToken::Alternative(a) => {
                let alternatives: Vec<_> =
                    a.alternatives.iter().map(hex_regex).collect::<Result<_>>()?;
                format!("(?:{})", alternatives.join("|"))
            }
            HexToken::Jump(jump) => {
                let lo = jump.start.unwrap_or(0);
                let hi = jump.end.context("unbounded hex jump")?;
                ensure!(lo <= hi && hi <= PREFIX_LIMIT as u32, "invalid hex jump");
                format!("{ANY}{{{lo},{hi}}}")
            }
        });
    }
    Ok(result)
}
fn byte_regex(hir: &Hir) -> Result<String> {
    Ok(match hir.kind() {
        HirKind::Empty => String::new(),
        HirKind::Literal(literal) => literal.0.iter().map(|b| byte(*b)).collect(),
        HirKind::Class(regex_syntax::hir::Class::Bytes(bytes)) => {
            let parts: Vec<_> =
                bytes.iter().map(|r| format!("{}-{}", byte(r.start()), byte(r.end()))).collect();
            format!("[{}]", parts.concat())
        }
        // The HIR optimizer folds ASCII literal alternatives into a Unicode class even
        // with Unicode parsing disabled. Its ASCII ranges still have exact byte semantics.
        HirKind::Class(regex_syntax::hir::Class::Unicode(chars)) if chars.is_ascii() => {
            let parts: Vec<_> = chars
                .iter()
                .map(|r| format!("{}-{}", byte(r.start() as u8), byte(r.end() as u8)))
                .collect();
            format!("[{}]", parts.concat())
        }
        HirKind::Capture(capture) => byte_regex(&capture.sub)?,
        HirKind::Concat(parts) => {
            parts.iter().map(byte_regex).collect::<Result<Vec<_>>>()?.concat()
        }
        HirKind::Alternation(parts) => {
            format!("(?:{})", parts.iter().map(byte_regex).collect::<Result<Vec<_>>>()?.join("|"))
        }
        HirKind::Repetition(repeat) => {
            let max = repeat.max.context("unbounded repetition")?;
            ensure!(max <= PREFIX_LIMIT as u32, "repetition exceeds prefix budget");
            format!("(?:{}){{{},{max}}}", byte_regex(&repeat.sub)?, repeat.min)
        }
        _ => bail!("regex assertions and Unicode classes are unsupported"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const META: &str =
        "meta: label = \"png\" enabled = true class = \"full\" fp_rate = 0 fn_rate = 0";

    fn compiled(patterns: &str, condition: &str) -> Result<Program> {
        compile(&format!("rule fact {{ {META} {patterns} condition: {condition} }}"))
    }

    fn error(patterns: &str, condition: &str) -> String {
        format!("{:#}", compiled(patterns, condition).unwrap_err())
    }

    /// Expressions with their bounds, in stream order.
    fn atoms(stream: &Expressions) -> Vec<(&str, Option<Bounds>)> {
        stream.expressions.iter().map(String::as_str).zip(stream.bounds.iter().copied()).collect()
    }

    /// A match ending anywhere in `min_end_offset..=max_end_offset`.
    fn ends(min_end_offset: u64, max_end_offset: u64) -> Option<Bounds> {
        Some(Bounds { min_end_offset, max_end_offset })
    }

    fn labels(stream: &Expressions) -> Vec<&'static str> {
        stream.outputs.iter().flatten().map(|x| x.info().label).collect()
    }

    #[test]
    fn stream_a_lowers_exactly_as_before_the_facts_stream_existed() {
        // Expected values are the output of the compiler at 96e16f91, which knew a single
        // stream: the 16-byte size header followed by the prefix.
        let source = format!(
            r#"
            private rule helper {{ condition: original_size >= 4 and prefix_size % 4 == 0 }}
            rule disabled {{ meta: label = "gif" enabled = false condition: uint8(0) == 1 }}
            rule fixture {{ {META}
                strings: $a = "AB" $b = {{ 41 [1-2] 42 }} $c = /A(B|CD)/
                condition: ($a at 5 or $b in (3..5)) and $c at 0 and uint16be(3) == 0x4142
                    and uint8(0) % 4 == 1 and uint32(8) > 7 and helper and disabled or true }}
            rule other {{ meta: label = "gif" enabled = true class = "full" fp_rate = 0 fn_rate = 0
                condition: helper and uint8(0) == 1 or false }}
            "#
        );
        let program = compile(&source).unwrap();
        let modulo = |divisor: u8, remainder: u8| {
            format!(
                "[{}]",
                (0..=255_u8).filter(|x| x % divisor == remainder).map(byte).collect::<String>()
            )
        };
        let size_at_least_4 = format!(
            "^(?:{})",
            [interval(4, 4, 8, true), interval(5, i64::MAX as u64, 8, true)].concat().join("|")
        );
        let uint32_above_7 =
            format!("^{ANY}{{24}}(?:{})", interval(8, u32::MAX as u64, 4, false).join("|"));
        let expected = [
            ("^[\\x00-\\xff]{21,21}(?:\\x41\\x42)".to_string(), 1024, None),
            ("^[\\x00-\\xff]{19,21}(?:\\x41(?:[\\x00-\\xff]){1,2}\\x42)".to_string(), 1024, None),
            ("^[\\x00-\\xff]{16,16}(?:\\x41(?:\\x42|\\x43\\x44))".to_string(), 1024, None),
            ("^[\\x00-\\xff]{19}(?:\\x41\\x42)".to_string(), 1024, None),
            (format!("^[\\x00-\\xff]{{16}}{}", modulo(4, 1)), 1024, None),
            (uint32_above_7, 1024, None),
            (size_at_least_4, 1024, None),
            (format!("^[\\x00-\\xff]{{8}}{}{}", ANY.repeat(7), modulo(4, 0)), 1024, None),
            ("^\\x00".to_string(), 1024, None),
            ("^\\x01".to_string(), 1024, None),
            ("^[\\x00-\\xff]".to_string(), 1024, None),
            ("((((0|1)&2&3&4&5&(6&7)&(8&9))|10))&10".to_string(), 520, Some("png")),
            ("^[\\x00-\\xff]{16}(?:\\x01)".to_string(), 1024, None),
            ("((((6&7)&12)|(8&9)))&10".to_string(), 520, Some("gif")),
        ];
        let actual: Vec<_> = program
            .prefix
            .expressions
            .iter()
            .zip(&program.prefix.flags)
            .zip(&program.prefix.outputs)
            .map(|((e, f), o)| (e.clone(), *f, o.map(|x| x.info().label)))
            .collect();
        assert_eq!(actual, expected);
        assert!(program.prefix.bounds.iter().all(Option::is_none));
        assert_eq!(program.facts, Expressions::default());
    }

    #[test]
    fn bundled_rules_compile_entirely_into_the_prefix_stream() {
        let program = compile(super::super::DEFAULT_RULES).unwrap();
        assert_eq!(program.facts, Expressions::default());
        assert!(program.prefix.bounds.iter().all(Option::is_none));
        assert!(!labels(&program.prefix).is_empty());
        for (expression, flags) in program.prefix.expressions.iter().zip(&program.prefix.flags) {
            if *flags == HS_FLAG_QUIET {
                assert!(expression.starts_with('^'), "{expression}");
            } else {
                assert_eq!(*flags, HS_FLAG_COMBINATION | HS_FLAG_SINGLEMATCH, "{expression}");
                assert!(expression.starts_with('('), "{expression}");
            }
        }
    }

    #[test]
    fn facts_compile_into_the_facts_stream_at_their_header_offsets() {
        let program = compiled("", "pe_machine == 0x8664 and zip_valid % 2 == 1").unwrap();
        assert_eq!(program.prefix, Expressions::default());
        let odd = format!(
            "^{ANY}{{16}}[{}]",
            (0..=255_u8).filter(|x| x % 2 == 1).map(byte).collect::<String>()
        );
        assert_eq!(
            atoms(&program.facts)[..2],
            [("^[\\x00-\\xff]{34}(?:\\x86\\x64)", None), (odd.as_str(), None)]
        );
        assert_eq!(labels(&program.facts), ["png"]);
        // The size header lowers identically in both domains.
        let sizes = "original_size >= 4 and prefix_size % 4 == 0";
        let prefix = compiled("", sizes).unwrap();
        let facts = compiled("", &format!("{sizes} and pe_overlay > 0")).unwrap();
        assert_eq!(facts.prefix, Expressions::default());
        assert_eq!(atoms(&prefix.prefix)[..2], atoms(&facts.facts)[..2]);
        assert!(prefix.facts.expressions.is_empty());
    }

    #[test]
    fn view_membership_lowers_to_floating_literals_bounded_to_the_view() {
        let program = compiled(
            "",
            "zip_names contains \"word/\" or zip_names startswith \"\\nword/\" or zip_names contains \"a\\\"b\"",
        )
        .unwrap();
        assert_eq!(program.prefix, Expressions::default());
        assert_eq!(
            atoms(&program.facts)[..3],
            [
                ("\\x77\\x6f\\x72\\x64\\x2f", ends(69, 4160)),
                ("\\x0a\\x77\\x6f\\x72\\x64\\x2f", ends(70, 70)),
                ("\\x61\\x22\\x62", ends(67, 4160)),
            ]
        );
        assert!(program.facts.flags[..3].iter().all(|x| *x == HS_FLAG_QUIET));
        // The same literal under a different bound is a different atom; identical uses share one.
        let program = compiled(
            "",
            "zip_names contains \"x\" or zip_names startswith \"x\" or zip_names contains \"x\"",
        )
        .unwrap();
        assert_eq!(
            atoms(&program.facts)[..2],
            [("\\x78", ends(65, 4160)), ("\\x78", ends(65, 65))]
        );
        assert_eq!(program.facts.expressions.iter().filter(|x| *x == "\\x78").count(), 2);
        // A literal may fill the whole view, but never exceed it.
        let full = "x".repeat(4096);
        let program = compiled("", &format!("zip_names startswith \"{full}\"")).unwrap();
        assert_eq!(program.facts.bounds[0], ends(4160, 4160));
        assert!(error("", &format!("zip_names contains \"{full}x\"")).contains("1 to 4096 bytes"));
    }

    #[test]
    fn unsupported_view_forms_are_rejected_with_reasons() {
        for (condition, message) in [
            ("zip_names icontains \"a\"", "only `contains` and `startswith`"),
            ("zip_names istartswith \"a\"", "only `contains` and `startswith`"),
            ("zip_names endswith \"a\"", "only `contains` and `startswith`"),
            ("zip_names iendswith \"a\"", "only `contains` and `startswith`"),
            ("zip_names matches /a/", "only `contains` and `startswith`"),
            ("zip_names contains zip_names", "literal string on the right"),
            ("zip_names contains pe_machine", "literal string on the right"),
            ("zip_names startswith 1", "literal string on the right"),
            ("pe_machine contains \"a\"", "unknown view `pe_machine`"),
            ("bogus startswith \"a\"", "unknown view `bogus`"),
            ("\"a\" contains \"b\"", "view identifier on the left"),
            ("zip_names contains \"\"", "1 to 4096 bytes"),
            ("zip_names == \"a\"", "unknown fact `zip_names`"),
            ("pe_bogus == 1", "unknown fact `pe_bogus`"),
        ] {
            let error = error("", condition);
            assert!(error.contains(message), "{condition}: {error}");
        }
    }

    #[test]
    fn terminal_rules_are_assigned_one_domain() {
        let program = compiled("strings: $a = \"AB\"", "$a at 0 and original_size >= 4").unwrap();
        assert_eq!((labels(&program.prefix), labels(&program.facts)), (vec!["png"], vec![]));
        let program = compiled("", "pe_valid == 1 and original_size >= 4").unwrap();
        assert_eq!((labels(&program.prefix), labels(&program.facts)), (vec![], vec!["png"]));
        let program = compiled("", "original_size >= 4 and true").unwrap();
        assert_eq!((labels(&program.prefix), labels(&program.facts)), (vec!["png"], vec![]));
        for (patterns, condition) in [
            ("strings: $a = \"AB\"", "$a at 0 and pe_valid == 1"),
            ("strings: $a = \"AB\"", "$a at 0 or zip_names contains \"x\""),
            ("", "uint8(0) == 1 and zip_names startswith \"x\""),
            ("", "uint8(0) % 2 == 1 or (original_size > 0 and pe_valid == 1)"),
        ] {
            let error = error(patterns, condition);
            assert!(
                error.contains("rule `fact` mixes prefix patterns with preprocessor facts"),
                "{condition}: {error}"
            );
        }
    }

    #[test]
    fn helpers_are_compiled_per_domain_of_their_referrers() {
        let source = format!(
            r#"
            private rule sized {{ condition: original_size >= 4 }}
            private rule signed {{ strings: $a = "AB" condition: $a at 0 }}
            private rule packaged {{ condition: zip_valid == 1 and zip_names contains "x" }}
            rule image {{ {META} condition: sized and signed }}
            rule archive {{ meta: label = "zip" enabled = true class = "full" fp_rate = 0 fn_rate = 0
                condition: sized and packaged }}
            "#
        );
        let program = compile(&source).unwrap();
        assert_eq!((labels(&program.prefix), labels(&program.facts)), (vec!["png"], vec!["zip"]));
        let sized = &compiled("", "original_size >= 4").unwrap().prefix.expressions[0];
        assert_eq!(program.prefix.expressions.iter().filter(|x| *x == sized).count(), 1);
        assert_eq!(program.facts.expressions.iter().filter(|x| *x == sized).count(), 1);
        assert!(program.prefix.expressions.iter().any(|x| x.ends_with("(?:\\x41\\x42)")));
        assert!(program.facts.expressions.iter().any(|x| x == "\\x78"));
        // Mixing through references is reported on the rule that combines both domains.
        for condition in ["signed and packaged", "sized and signed and zip_valid == 1"] {
            let mixed = source.replace("sized and signed", condition);
            let error = format!("{:#}", compile(&mixed).unwrap_err());
            assert!(
                error.contains("rule `image` mixes prefix patterns with preprocessor facts"),
                "{condition}: {error}"
            );
        }
        let helper = source.replace(
            "condition: zip_valid == 1 and zip_names contains \"x\"",
            "condition: uint8(0) == 1 and zip_valid == 1",
        );
        let error = format!("{:#}", compile(&helper).unwrap_err());
        assert!(error.starts_with("rule archive: rule `packaged` mixes"), "{error}");
        // An unenforced variant is a neutral constant wherever it is referenced.
        let disabled = source.replace(
            "private rule packaged {",
            "rule packaged { meta: label = \"zip\" enabled = false",
        );
        let program = compile(&disabled).unwrap();
        assert_eq!((labels(&program.prefix), labels(&program.facts)), (vec!["png", "zip"], vec![]));
    }
}
