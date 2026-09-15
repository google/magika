// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0

//! Lowers validated YARA text to a [`Program`]. Ported from the condition walk of #1447's
//! `compiler.rs`: the same bounded subset, with Vectorscan expressions replaced by IR nodes.

use std::collections::{HashMap, HashSet};

use yara_x_parser::ast::{Expr, Item, MatchAnchor, Rule as AstRule, RuleFlags, AST};

use crate::ir::{self, Cmp, Cond, Int, Program, Read};
use crate::matcher::Pattern;
use crate::source::explicit_enforcement;
use crate::{Error, Source, PREFIX_LIMIT};

/// Deepest chain of rule references accepted.
const MAX_DEPTH: usize = 128;

/// The program and its label table; `ir::Rule::label` indexes the table.
pub(crate) fn lower(source: &Source) -> Result<(Program, Vec<String>), Error> {
    let ast = AST::from(source.text());
    let rules: HashMap<&str, &AstRule<'_>> = ast
        .items()
        .filter_map(|item| match item {
            Item::Rule(rule) => Some((rule.identifier.name, rule)),
            _ => None,
        })
        .collect();
    let info: HashMap<&str, _> = source.rules().iter().map(|r| (r.id.as_str(), r)).collect();
    let mut lowering = Lowering {
        rules: &rules,
        enforced: info.iter().map(|(id, r)| (*id, r.enforced)).collect(),
        patterns: Vec::new(),
        pattern_ids: HashMap::new(),
        lowered: HashMap::new(),
        visiting: HashSet::new(),
    };
    let (mut labels, mut program_rules) = (Vec::<String>::new(), Vec::new());
    for rule in ast.rules() {
        let Some(info) = info.get(rule.identifier.name).filter(|r| r.enforced) else { continue };
        let label = info.label.as_deref().expect("source validation requires a label");
        let cond = lowering.rule(rule)?;
        let index = match labels.iter().position(|l| l == label) {
            Some(index) => index,
            None => {
                labels.push(label.to_string());
                labels.len() - 1
            }
        };
        program_rules.push(ir::Rule { label: index, cond });
    }
    Ok((Program { patterns: lowering.patterns, rules: program_rules }, labels))
}

struct Lowering<'a, 'src> {
    rules: &'a HashMap<&'src str, &'a AstRule<'src>>,
    /// Enforcement of every non-private rule, as validated by `Source`.
    enforced: HashMap<&'a str, bool>,
    patterns: Vec<Pattern>,
    /// `(rule, pattern identifier)` to an index into `patterns`.
    pattern_ids: HashMap<(&'src str, &'src str), usize>,
    lowered: HashMap<&'src str, Cond>,
    visiting: HashSet<&'src str>,
}

impl<'a, 'src> Lowering<'a, 'src> {
    /// Whether a rule takes part: validated enforcement for public rules; private helpers
    /// are active unless they opt out.
    fn active(&self, rule: &AstRule<'_>) -> bool {
        if rule.flags.contains(RuleFlags::Private) {
            explicit_enforcement(rule).unwrap_or(true)
        } else {
            self.enforced.get(rule.identifier.name).copied().unwrap_or(false)
        }
    }

    fn rule(&mut self, rule: &'a AstRule<'src>) -> Result<Cond, Error> {
        let name = rule.identifier.name;
        if let Some(cond) = self.lowered.get(name) {
            return Ok(cond.clone());
        }
        // An inactive rule is `false` wherever it is referenced.
        if !self.active(rule) {
            self.lowered.insert(name, Cond::False);
            return Ok(Cond::False);
        }
        if self.visiting.len() >= MAX_DEPTH || !self.visiting.insert(name) {
            return Err(unsupported(name, "cyclic or excessively deep rule references"));
        }
        let mut ids = HashSet::new();
        for pattern in rule.patterns.iter().flatten() {
            if !ids.insert(pattern.identifier().name) {
                return Err(Error::Metadata {
                    rule: name.into(),
                    reason: "duplicate pattern ID".into(),
                });
            }
        }
        let result = self.condition(&rule.condition, rule);
        self.visiting.remove(name);
        let cond = result?;
        self.lowered.insert(name, cond.clone());
        Ok(cond)
    }

    fn condition(&mut self, expr: &'a Expr<'src>, rule: &'a AstRule<'src>) -> Result<Cond, Error> {
        let name = rule.identifier.name;
        Ok(match expr {
            Expr::True { .. } => Cond::True,
            Expr::False { .. } => Cond::False,
            Expr::And(node) | Expr::Or(node) => {
                let parts = node
                    .operands
                    .iter()
                    .map(|operand| self.condition(operand, rule))
                    .collect::<Result<Vec<_>, _>>()?;
                if matches!(expr, Expr::And(_)) {
                    Cond::And(parts)
                } else {
                    Cond::Or(parts)
                }
            }
            Expr::Not(node) => Cond::Not(Box::new(self.condition(&node.operand, rule)?)),
            Expr::Ident(id) => match self.rules.get(id.name) {
                Some(target) => self.rule(target)?,
                None => {
                    return Err(unsupported(name, &format!("unknown identifier `{}`", id.name)))
                }
            },
            Expr::PatternMatch(node) => {
                let key = (name, node.identifier.name);
                let pattern = match self.pattern_ids.get(&key) {
                    Some(&index) => index,
                    None => {
                        let ast = rule
                            .patterns
                            .iter()
                            .flatten()
                            .find(|p| p.identifier().name == node.identifier.name)
                            .ok_or_else(|| unsupported(name, "unknown pattern"))?;
                        let compiled = Pattern::from_ast(ast).map_err(|e| in_rule(e, name))?;
                        self.patterns.push(compiled);
                        self.pattern_ids.insert(key, self.patterns.len() - 1);
                        self.patterns.len() - 1
                    }
                };
                let (lo, hi) = match &node.anchor {
                    Some(MatchAnchor::At(at)) => {
                        let at = number(&at.expr, name)?;
                        (at, at)
                    }
                    Some(MatchAnchor::In(range)) => (
                        number(&range.range.lower_bound, name)?,
                        number(&range.range.upper_bound, name)?,
                    ),
                    None => {
                        return Err(unsupported(name, "patterns require a literal at/in bound"))
                    }
                };
                let width = self.patterns[pattern].max_len() as u64;
                if lo > hi || hi.saturating_add(width) > PREFIX_LIMIT as u64 {
                    return Err(unsupported(name, "pattern placement exceeds the prefix"));
                }
                let (lo, hi) = (Int::Const(lo as i64), Int::Const(hi as i64));
                if lo == hi {
                    Cond::At { pattern, offset: lo }
                } else {
                    Cond::In { pattern, lo, hi }
                }
            }
            Expr::Eq(node)
            | Expr::Ne(node)
            | Expr::Lt(node)
            | Expr::Le(node)
            | Expr::Gt(node)
            | Expr::Ge(node) => {
                let op = match expr {
                    Expr::Eq(_) => Cmp::Eq,
                    Expr::Ne(_) => Cmp::Ne,
                    Expr::Lt(_) => Cmp::Lt,
                    Expr::Le(_) => Cmp::Le,
                    Expr::Gt(_) => Cmp::Gt,
                    _ => Cmp::Ge,
                };
                let lhs = match &node.lhs {
                    Expr::Mod(remainder) => {
                        if op != Cmp::Eq || remainder.operands.len() != 2 {
                            return Err(unsupported(
                                name,
                                "modulo requires equality and one literal divisor",
                            ));
                        }
                        let divisor = number(&remainder.operands[1], name)?;
                        if !divisor.is_power_of_two() {
                            return Err(unsupported(name, "modulo divisor must be a power of two"));
                        }
                        Int::And(
                            Box::new(operand(&remainder.operands[0], name)?),
                            Box::new(Int::Const((divisor - 1) as i64)),
                        )
                    }
                    lhs => operand(lhs, name)?,
                };
                Cond::Cmp(op, lhs, operand(&node.rhs, name)?)
            }
            _ => {
                return Err(unsupported(
                    name,
                    "unsupported YARA condition; no predicates were discarded",
                ))
            }
        })
    }
}

/// An integer operand: a literal, a size, or a fixed unsigned read inside the prefix.
fn operand(expr: &Expr<'_>, rule: &str) -> Result<Int, Error> {
    Ok(match expr {
        Expr::LiteralInteger(_) => Int::Const(number(expr, rule)? as i64),
        Expr::Filesize { .. } => Int::FileSize,
        Expr::Ident(id) if id.name == "original_size" => Int::FileSize,
        Expr::Ident(id) if id.name == "prefix_size" => Int::PrefixSize,
        Expr::FuncCall(call) if call.object.is_none() && call.args.len() == 1 => {
            let (read, width) = match call.identifier.name {
                "uint8" => (Read::U8, 1),
                "uint16" => (Read::U16Le, 2),
                "uint16be" => (Read::U16Be, 2),
                "uint32" => (Read::U32Le, 4),
                "uint32be" => (Read::U32Be, 4),
                _ => return Err(unsupported(rule, "unsupported integer function")),
            };
            let offset = number(&call.args[0], rule)?;
            if offset + width > PREFIX_LIMIT as u64 {
                return Err(unsupported(rule, "integer read exceeds the prefix"));
            }
            Int::Read(read, Box::new(Int::Const(offset as i64)))
        }
        _ => {
            return Err(unsupported(
                rule,
                "comparisons take literals, sizes, or fixed unsigned reads",
            ))
        }
    })
}

fn number(expr: &Expr<'_>, rule: &str) -> Result<u64, Error> {
    match expr {
        Expr::LiteralInteger(literal) => u64::try_from(literal.value)
            .map_err(|_| unsupported(rule, "expected a nonnegative integer literal")),
        _ => Err(unsupported(rule, "expected a nonnegative integer literal")),
    }
}

fn unsupported(rule: &str, reason: &str) -> Error {
    Error::Unsupported { rule: rule.into(), reason: reason.into() }
}

/// Names the rule in an error raised before it was known.
fn in_rule(error: Error, rule: &str) -> Error {
    match error {
        Error::Unsupported { reason, .. } => Error::Unsupported { rule: rule.into(), reason },
        other => other,
    }
}
