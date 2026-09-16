// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0

//! Evaluates a program over one input. A read past the prefix is `None` and makes its
//! comparison `false`, matching YARA's undefined value.

use crate::ir::{Cmp, Cond, Int, Program, Read};
use crate::{Input, Outcome, PREFIX_LIMIT};

pub(crate) fn scan(program: &Program, input: Input<'_>) -> Outcome {
    let Input { prefix, size, .. } = input;
    if prefix.is_empty() || prefix.len() as u64 != size.min(PREFIX_LIMIT as u64) {
        return Outcome::InsufficientInput;
    }
    let ctx = Ctx { program, prefix, size };
    let mut outcome = Outcome::NoMatch;
    for rule in &program.rules {
        if ctx.cond(&rule.cond) {
            outcome = match outcome {
                Outcome::NoMatch => Outcome::Match(rule.label),
                Outcome::Match(label) if label == rule.label => outcome,
                _ => return Outcome::Conflict,
            };
        }
    }
    outcome
}

struct Ctx<'a> {
    program: &'a Program,
    prefix: &'a [u8],
    size: u64,
}

impl Ctx<'_> {
    fn cond(&self, cond: &Cond) -> bool {
        match cond {
            Cond::True => true,
            Cond::False => false,
            Cond::Not(x) => !self.cond(x),
            Cond::And(xs) => xs.iter().all(|x| self.cond(x)),
            Cond::Or(xs) => xs.iter().any(|x| self.cond(x)),
            Cond::Cmp(op, a, b) => match (self.int(a), self.int(b)) {
                (Some(a), Some(b)) => match op {
                    Cmp::Eq => a == b,
                    Cmp::Ne => a != b,
                    Cmp::Lt => a < b,
                    Cmp::Le => a <= b,
                    Cmp::Gt => a > b,
                    Cmp::Ge => a >= b,
                },
                _ => false,
            },
            Cond::At { pattern, offset } => self
                .offset(offset)
                .is_some_and(|at| self.program.patterns[*pattern].matches_at(self.prefix, at)),
            Cond::In { pattern, lo, hi } => match (self.offset(lo), self.offset(hi)) {
                (Some(lo), Some(hi)) => {
                    self.program.patterns[*pattern].matches_in(self.prefix, lo, hi)
                }
                _ => false,
            },
        }
    }

    fn offset(&self, int: &Int) -> Option<usize> {
        usize::try_from(self.int(int)?).ok()
    }

    fn int(&self, int: &Int) -> Option<i128> {
        Some(match int {
            Int::Const(value) => i128::from(*value),
            Int::FileSize => i128::from(self.size),
            Int::PrefixSize => self.prefix.len() as i128,
            Int::Read(kind, at) => {
                let at = self.offset(at)?;
                let get = |n: usize| self.prefix.get(at..at.checked_add(n)?);
                match kind {
                    Read::U8 => i128::from(get(1)?[0]),
                    Read::U16Le => i128::from(u16::from_le_bytes(get(2)?.try_into().ok()?)),
                    Read::U16Be => i128::from(u16::from_be_bytes(get(2)?.try_into().ok()?)),
                    Read::U32Le => i128::from(u32::from_le_bytes(get(4)?.try_into().ok()?)),
                    Read::U32Be => i128::from(u32::from_be_bytes(get(4)?.try_into().ok()?)),
                }
            }
            Int::And(a, b) => self.int(a)? & self.int(b)?,
        })
    }
}
