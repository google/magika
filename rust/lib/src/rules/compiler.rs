// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0

//! Compile a bounded YARA subset into native regexes and logical combinations.
//! This module runs once when loading a pack, never while identifying a file.

use std::collections::{HashMap, HashSet};

use anyhow::{bail, ensure, Context, Result};
use regex_syntax::hir::{Hir, HirKind};
use yara_x_parser::ast::{
    Expr, HexSubPattern, HexToken, Item, MatchAnchor, MetaValue, Pattern, Rule, RuleFlags, AST,
};

use super::{metadata::enforced, EXTERNAL_BYTES, PREFIX_LIMIT};
use crate::ContentType;

const HS_FLAG_SINGLEMATCH: u32 = 8;
const HS_FLAG_COMBINATION: u32 = 512;
const HS_FLAG_QUIET: u32 = 1024;

pub(super) struct Program {
    pub(super) expressions: Vec<String>,
    pub(super) flags: Vec<u32>,
    pub(super) outputs: Vec<Option<ContentType>>,
}

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
        atoms: HashMap::new(),
        visiting: HashSet::new(),
        compiled: HashMap::new(),
        program: Program { expressions: Vec::new(), flags: Vec::new(), outputs: Vec::new() },
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
        let expression =
            compiler.rule(rule).with_context(|| format!("rule {}", rule.identifier.name))?;
        let present = compiler.constant(true)?;
        compiler.push(
            format!("({expression})&{present}"),
            HS_FLAG_COMBINATION | HS_FLAG_SINGLEMATCH,
            Some(label),
        )?;
    }
    Ok(compiler.program)
}

struct Compiler<'a> {
    rules: HashMap<&'a str, &'a Rule<'a>>,
    atoms: HashMap<String, usize>,
    visiting: HashSet<&'a str>,
    compiled: HashMap<&'a str, String>,
    program: Program,
}

impl<'a> Compiler<'a> {
    fn push(
        &mut self, expression: String, flags: u32, label: Option<ContentType>,
    ) -> Result<usize> {
        ensure!(
            self.program.expressions.len() < 100_000 && expression.len() <= 1_048_576,
            "compiled rule complexity limit exceeded"
        );
        let id = self.program.expressions.len();
        self.program.expressions.push(expression);
        self.program.flags.push(flags);
        self.program.outputs.push(label);
        Ok(id)
    }

    fn atom(&mut self, pattern: String) -> Result<String> {
        if let Some(id) = self.atoms.get(&pattern) {
            return Ok(id.to_string());
        }
        let id = self.push(pattern.clone(), HS_FLAG_QUIET, None)?;
        self.atoms.insert(pattern, id);
        Ok(id.to_string())
    }

    fn rule(&mut self, rule: &'a Rule<'a>) -> Result<String> {
        if let Some(expression) = self.compiled.get(rule.identifier.name) {
            return Ok(expression.clone());
        }
        if !enforced(rule, None)? {
            // Disable the complete variant, including when used inside a larger AND/OR.
            // Only ordinary private helpers are available without explicit enablement.
            let expression = self.constant(false)?;
            self.compiled.insert(rule.identifier.name, expression.clone());
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
            self.compiled.insert(rule.identifier.name, expression.clone());
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
                    Expr::Ident(id) if id.name == "original_size" => (0, 8, true),
                    Expr::Ident(id) if id.name == "prefix_size" => (8, 8, true),
                    Expr::FuncCall(call) if call.object.is_none() && call.args.len() == 1 => {
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
