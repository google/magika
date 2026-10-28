// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0

//! One YARA string, compiled for anchored or window-bounded matching over a byte buffer.

use std::borrow::Cow;
use std::sync::OnceLock;

use regex_automata::meta::Regex;
use regex_automata::util::syntax;
use regex_automata::{Anchored, Input as ReInput};
use regex_syntax::hir::{Class, Hir, HirKind};
use yara_x_parser::ast::{HexSubPattern, HexToken, Pattern as AstPattern};

use crate::{Error, PREFIX_LIMIT};

pub(crate) enum Pattern {
    /// Fixed width: byte `i` matches when `(input[i] & mask[i]) == bytes[i]`.
    Masked { bytes: Vec<u8>, mask: Vec<u8> },
    /// Alternation, jumps or repetition. Bytes mode, no Unicode.
    Regex(RegexPattern),
}

/// A byte regex, built the first time a scan reaches it with a byte a match can start with.
///
/// Most scans never reach most regexes, since cheaper conditions of their rules fail first, and
/// building every regex takes about 6 ms of a startup that should take none. Most regexes a scan
/// does reach cannot start at any byte of its window, and are never built either.
pub(crate) struct RegexPattern {
    pub(crate) source: Cow<'static, str>,
    pub(crate) case_insensitive: bool,
    pub(crate) dot_matches_new_line: bool,
    pub(crate) max_len: usize,
    /// The bytes a match can start with.
    pub(crate) first: ByteSet,
    /// `None` if the regex does not build, which validating the pattern already excluded.
    built: OnceLock<Option<Regex>>,
}

impl RegexPattern {
    /// A pattern whose source was validated by [`Pattern::from_ast`].
    pub(crate) fn new(
        source: impl Into<Cow<'static, str>>, case_insensitive: bool, dot_matches_new_line: bool,
        max_len: usize, first: ByteSet,
    ) -> Self {
        let source = source.into();
        RegexPattern {
            source,
            case_insensitive,
            dot_matches_new_line,
            max_len,
            first,
            built: OnceLock::new(),
        }
    }

    fn hir(&self) -> Result<Hir, String> {
        let mut parser = regex_syntax::ParserBuilder::new();
        parser
            .unicode(false)
            .utf8(false)
            .case_insensitive(self.case_insensitive)
            .dot_matches_new_line(self.dot_matches_new_line);
        parser.build().parse(&self.source).map_err(|e| format!("invalid byte regex: {e}"))
    }

    fn build(&self) -> Result<Regex, String> {
        // A scan builds most regexes it reaches on its first file, and a full DFA is most of what
        // building one costs, while the lazy DFA searches a 4 KiB prefix about as fast.
        Regex::builder()
            .syntax(syntax::Config::new().unicode(false).utf8(false))
            .configure(Regex::config().dfa(false))
            .build_from_hir(&self.hir()?)
            .map_err(|e| e.to_string())
    }

    fn regex(&self) -> Option<&Regex> {
        self.built.get_or_init(|| self.build().ok()).as_ref()
    }
}

impl Pattern {
    /// Builds the pattern. Modifiers are rejected: no shipped rule uses one.
    pub(crate) fn from_ast(pattern: &AstPattern<'_>) -> Result<Self, Error> {
        let unsupported = |reason: String| Error::Unsupported { rule: String::new(), reason };
        if !pattern.modifiers().is_empty() {
            return Err(unsupported("pattern modifiers".into()));
        }
        let (source, case_insensitive, dot_matches_new_line) = match pattern {
            AstPattern::Text(p) => (p.text.value.iter().map(|b| byte(*b)).collect(), false, false),
            AstPattern::Hex(p) => (hex_regex(&p.sub_patterns).map_err(unsupported)?, false, false),
            AstPattern::Regexp(p) => {
                (p.regexp.src.to_string(), p.regexp.case_insensitive, p.regexp.dot_matches_new_line)
            }
        };
        let pattern =
            RegexPattern::new(source, case_insensitive, dot_matches_new_line, 0, ByteSet::EMPTY);
        let hir = pattern.hir().map_err(unsupported)?;
        let properties = hir.properties();
        let Some(max_len) = properties.maximum_len() else {
            return Err(unsupported("unbounded pattern".into()));
        };
        if properties.minimum_len().unwrap_or(0) == 0 {
            return Err(unsupported("pattern matches empty input".into()));
        }
        if !properties.look_set().is_empty() {
            return Err(unsupported("regex assertions".into()));
        }
        if let Some((bytes, mask)) = fixed_masked(&hir) {
            return Ok(Pattern::Masked { bytes, mask });
        }
        // Build it now, so that a pattern that cannot build is rejected with its rule.
        let regex = pattern.build().map_err(unsupported)?;
        let first = ByteSet::first(&hir);
        Ok(Pattern::Regex(RegexPattern {
            max_len,
            first,
            built: OnceLock::from(Some(regex)),
            ..pattern
        }))
    }

    /// The longest match, in bytes.
    pub(crate) fn max_len(&self) -> usize {
        match self {
            Pattern::Masked { bytes, .. } => bytes.len(),
            Pattern::Regex(pattern) => pattern.max_len,
        }
    }

    /// Does the pattern match starting exactly at `at`?
    pub(crate) fn matches_at(&self, buf: &[u8], at: usize) -> bool {
        match self {
            Pattern::Masked { bytes, mask } => buf
                .get(at..at.saturating_add(bytes.len()))
                .is_some_and(|w| w.iter().zip(bytes).zip(mask).all(|((b, e), m)| b & m == *e)),
            Pattern::Regex(pattern) => {
                buf.get(at).is_some_and(|&b| pattern.first.contains(b))
                    && pattern.regex().is_some_and(|re| {
                        re.is_match(ReInput::new(buf).range(at..).anchored(Anchored::Yes))
                    })
            }
        }
    }

    /// Does the pattern match starting at some offset in `lo..=hi`?
    pub(crate) fn matches_in(&self, buf: &[u8], lo: usize, hi: usize) -> bool {
        let hi = hi.min(buf.len().saturating_sub(1));
        if lo > hi || buf.is_empty() {
            return false;
        }
        match self {
            Pattern::Masked { bytes, mask } if mask.iter().all(|&m| m == 0xff) => {
                memchr::memmem::find(&buf[lo..], bytes).is_some_and(|i| lo + i <= hi)
            }
            Pattern::Masked { .. } => (lo..=hi).any(|at| self.matches_at(buf, at)),
            Pattern::Regex(pattern) => {
                buf[lo..=hi].iter().any(|&b| pattern.first.contains(b))
                    && pattern.regex().is_some_and(|re| {
                        re.find(ReInput::new(buf).range(lo..)).is_some_and(|m| m.start() <= hi)
                    })
            }
        }
    }
}

/// A set of byte values, one bit each.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ByteSet(pub(crate) [u64; 4]);

impl ByteSet {
    pub(crate) const EMPTY: ByteSet = ByteSet([0; 4]);

    /// The bytes a match of `hir` can start with.
    fn first(hir: &Hir) -> ByteSet {
        let mut set = ByteSet::EMPTY;
        set.add_first(hir);
        set
    }

    /// Adds the bytes a match of `hir` can start with, and returns whether it matches empty.
    fn add_first(&mut self, hir: &Hir) -> bool {
        match hir.kind() {
            HirKind::Empty | HirKind::Look(_) => true,
            HirKind::Literal(literal) => match literal.0.first() {
                Some(&b) => {
                    self.insert(b);
                    false
                }
                None => true,
            },
            HirKind::Class(Class::Bytes(class)) => {
                class
                    .ranges()
                    .iter()
                    .flat_map(|r| r.start()..=r.end())
                    .for_each(|b| self.insert(b));
                false
            }
            // Simplifying an alternation of ASCII bytes gives a Unicode class, and so does a `(?u)`
            // flag. A character matches as its UTF-8 encoding, whose first byte grows with it.
            HirKind::Class(Class::Unicode(class)) => {
                let first = |c: char| c.encode_utf8(&mut [0; 4]).as_bytes()[0];
                for range in class.ranges() {
                    let (start, end) = (range.start(), range.end());
                    (u32::from(start)..=u32::from(end).min(0x7f))
                        .for_each(|c| self.insert(c as u8));
                    if end >= '\u{80}' {
                        (first(start.max('\u{80}'))..=first(end)).for_each(|b| self.insert(b));
                    }
                }
                false
            }
            HirKind::Repetition(repetition) => {
                self.add_first(&repetition.sub) || repetition.min == 0
            }
            HirKind::Capture(capture) => self.add_first(&capture.sub),
            HirKind::Concat(hirs) => hirs.iter().all(|hir| self.add_first(hir)),
            HirKind::Alternation(hirs) => {
                hirs.iter().fold(false, |empty, hir| self.add_first(hir) | empty)
            }
        }
    }

    fn insert(&mut self, b: u8) {
        self.0[usize::from(b / 64)] |= 1 << (b % 64);
    }

    fn contains(&self, b: u8) -> bool {
        self.0[usize::from(b / 64)] & 1 << (b % 64) != 0
    }
}

fn byte(value: u8) -> String {
    format!("\\x{value:02x}")
}

/// Ported from `hex_regex` in #1447's `compiler.rs`: hex tokens to a byte regex.
fn hex_regex(pattern: &HexSubPattern) -> Result<String, String> {
    let mut result = String::new();
    for token in &pattern.0 {
        match token {
            HexToken::Byte(b) | HexToken::NotByte(b) => {
                let negated = matches!(token, HexToken::NotByte(_));
                let values: String = (0..=255_u8)
                    .filter(|x| ((x & b.mask) == b.value) != negated)
                    .map(byte)
                    .collect();
                if values.is_empty() {
                    return Err("empty hex byte class".into());
                }
                result.push_str(&format!("[{values}]"));
            }
            HexToken::Alternative(a) => {
                let alternatives =
                    a.alternatives.iter().map(hex_regex).collect::<Result<Vec<_>, _>>()?;
                result.push_str(&format!("(?:{})", alternatives.join("|")));
            }
            HexToken::Jump(jump) => {
                let lo = jump.start.unwrap_or(0);
                let hi = jump.end.ok_or("unbounded hex jump")?;
                if lo > hi || hi as usize > PREFIX_LIMIT {
                    return Err("invalid hex jump".into());
                }
                result.push_str(&format!("[\\x00-\\xff]{{{lo},{hi}}}"));
            }
        }
    }
    Ok(result)
}

/// `Some((bytes, mask))` when every HIR node is a literal or a mask-expressible byte class.
fn fixed_masked(hir: &Hir) -> Option<(Vec<u8>, Vec<u8>)> {
    fn walk(hir: &Hir, bytes: &mut Vec<u8>, mask: &mut Vec<u8>) -> Option<()> {
        match hir.kind() {
            HirKind::Literal(l) => {
                bytes.extend_from_slice(&l.0);
                mask.extend(std::iter::repeat_n(0xff, l.0.len()));
                Some(())
            }
            HirKind::Class(class) => {
                let (v, m) = maskable(&class_ranges(class)?)?;
                bytes.push(v);
                mask.push(m);
                Some(())
            }
            HirKind::Capture(capture) => walk(&capture.sub, bytes, mask),
            HirKind::Concat(parts) => parts.iter().try_for_each(|p| walk(p, bytes, mask)),
            HirKind::Repetition(r) if Some(r.min) == r.max => {
                (0..r.min).try_for_each(|_| walk(&r.sub, bytes, mask))
            }
            HirKind::Empty => Some(()),
            _ => None,
        }
    }
    let (mut bytes, mut mask) = (Vec::new(), Vec::new());
    walk(hir, &mut bytes, &mut mask)?;
    (!bytes.is_empty()).then_some((bytes, mask))
}

/// Byte ranges of a class. The HIR optimizer can fold ASCII alternatives into a Unicode
/// class even with Unicode disabled; its ASCII ranges keep exact byte semantics.
fn class_ranges(class: &Class) -> Option<Vec<(u8, u8)>> {
    match class {
        Class::Bytes(c) => Some(c.ranges().iter().map(|r| (r.start(), r.end())).collect()),
        Class::Unicode(c) if c.is_ascii() => {
            Some(c.ranges().iter().map(|r| (r.start() as u8, r.end() as u8)).collect())
        }
        Class::Unicode(_) => None,
    }
}

/// Nibble wildcards produce {b | b & 0xf0 == v}, {b | b & 0x0f == v} and all bytes.
fn maskable(ranges: &[(u8, u8)]) -> Option<(u8, u8)> {
    let member = |b: u8| ranges.iter().any(|&(s, e)| s <= b && b <= e);
    let first = (0..=255u8).find(|&b| member(b))?;
    [0xffu8, 0xf0, 0x0f, 0x00]
        .into_iter()
        .map(|m| (first & m, m))
        .find(|&(v, m)| (0..=255u8).all(|b| member(b) == (b & m == v)))
}

#[cfg(test)]
mod tests {
    use yara_x_parser::ast::{Item, AST};

    use super::*;

    fn pattern(yara: &str) -> Pattern {
        let text = format!("rule r {{ strings: $a = {yara} condition: $a }}");
        let ast = AST::from(text.as_str());
        let Item::Rule(rule) = &ast.items().next().unwrap() else { unreachable!() };
        Pattern::from_ast(rule.patterns.as_ref().unwrap().iter().next().unwrap()).unwrap()
    }

    #[test]
    fn hex_literal_becomes_masked_compare() {
        let p = pattern("{ 89 50 4E 47 }");
        assert!(matches!(p, Pattern::Masked { .. }));
        assert!(p.matches_at(b"\x89PNG....", 0));
        assert!(!p.matches_at(b".\x89PNG...", 0));
        assert!(p.matches_at(b".\x89PNG...", 1));
        assert!(!p.matches_at(b"\x89PN", 0), "truncated input never matches");
    }

    #[test]
    fn hex_wildcards_mask_nibbles_and_bytes() {
        let p = pattern("{ 4D 5A ?? 00 1? }");
        assert!(matches!(p, Pattern::Masked { .. }));
        assert!(p.matches_at(b"MZ\x99\x00\x1f", 0));
        assert!(!p.matches_at(b"MZ\x99\x00\x2f", 0));
        assert!(!p.matches_at(b"MZ\x99\x01\x1f", 0));
        let low = pattern("{ ?7 }");
        assert!(low.matches_at(b"\xa7", 0) && !low.matches_at(b"\xa8", 0));
    }

    #[test]
    fn text_literal_is_masked_too() {
        let p = pattern("\"GIF8\"");
        assert!(matches!(p, Pattern::Masked { .. }));
        assert!(p.matches_at(b"GIF89a", 0));
    }

    #[test]
    fn alternation_and_jump_become_regex() {
        let p = pattern("{ 47 49 46 38 ( 37 | 39 ) 61 }");
        assert!(matches!(p, Pattern::Regex(_)));
        assert!(
            p.matches_at(b"GIF87a", 0) && p.matches_at(b"GIF89a", 0) && !p.matches_at(b"GIF88a", 0)
        );
        let j = pattern("{ 41 [2] 42 }");
        assert!(j.matches_at(b"AxxB", 0) && !j.matches_at(b"AxB", 0));
    }

    /// The bytes that move the regex's own anchored DFA out of its start state to a live one.
    fn dfa_first_bytes(pattern: &RegexPattern) -> ByteSet {
        use regex_automata::hybrid::dfa::DFA;
        use regex_automata::nfa::thompson;
        let nfa = thompson::Compiler::new().build_from_hir(&pattern.hir().unwrap()).unwrap();
        let dfa = DFA::builder().build_from_nfa(nfa).unwrap();
        let mut cache = dfa.create_cache();
        let start = ReInput::new(b"").anchored(Anchored::Yes);
        let start = dfa.start_state_forward(&mut cache, &start).unwrap();
        let mut set = ByteSet::EMPTY;
        for b in 0..=255 {
            if !dfa.next_state(&mut cache, start, b).unwrap().is_dead() {
                set.insert(b);
            }
        }
        set
    }

    #[test]
    fn first_bytes_are_the_bytes_a_match_can_start_with() {
        let patterns = [
            "{ 47 49 46 38 ( 37 | 39 ) 61 }",
            "{ ( 41 | 42 ?? ) 43 }",
            "/(ab)?c/",
            "/a{0,2}b{0,3}c/",
            "/[^\\x00]x/",
            "/hello/i",
            "/(?u)é/",
            "/(?u)[\\x{70}-\\x{3000}]/",
        ];
        #[cfg(feature = "bundled")]
        let bundled = crate::RuleSet::bundled().program.patterns;
        #[cfg(not(feature = "bundled"))]
        let bundled = Vec::new();
        for pattern in patterns.map(pattern).iter().chain(&bundled) {
            let Pattern::Regex(pattern) = pattern else { continue };
            assert_eq!(pattern.first, dfa_first_bytes(pattern), "{}", pattern.source);
        }
    }

    #[test]
    fn a_regex_is_not_built_where_no_match_can_start() {
        let Pattern::Regex(built) = pattern("/[ab]{2}[0-9]{1,3}x/") else { unreachable!() };
        let p = Pattern::Regex(RegexPattern::new(built.source, false, false, 0, built.first));
        assert!(!p.matches_at(b"zzab1x", 0) && !p.matches_in(b"zzzz", 0, 3));
        let Pattern::Regex(regex) = &p else { unreachable!() };
        assert!(regex.built.get().is_none());
        assert!(p.matches_in(b"zzab1x", 0, 3));
        assert!(regex.built.get().is_some());
    }

    #[test]
    fn regex_patterns_are_byte_oriented() {
        let p = pattern("/BLENDER[_-][vV][1-4][0-9]{2}/");
        assert!(p.matches_at(b"BLENDER-v302", 0));
        assert!(!p.matches_at("BLENDER-v3é2".as_bytes(), 0));
        let c = pattern("/[\\x00-\\xff]/");
        assert!(c.matches_at(b"\xff", 0), "\\xff is one byte, not a UTF-8 sequence");
    }

    #[test]
    fn in_window_bounds_the_match_start() {
        let p = pattern("\"moov\"");
        assert!(p.matches_in(b"....moov", 0, 8));
        assert!(p.matches_in(b"....moov", 4, 4));
        assert!(!p.matches_in(b"....moov", 0, 3));
        assert!(!p.matches_in(b"....moov", 5, 100));
    }

    #[test]
    fn nocase_wide_and_xor_modifiers_are_rejected() {
        for modifier in ["nocase", "wide", "xor"] {
            let text = format!("rule r {{ strings: $a = \"x\" {modifier} condition: $a }}");
            let ast = AST::from(text.as_str());
            let Item::Rule(rule) = &ast.items().next().unwrap() else { unreachable!() };
            let pattern = rule.patterns.as_ref().unwrap().iter().next().unwrap();
            assert!(Pattern::from_ast(pattern).is_err(), "{modifier}");
        }
    }

    // Ported from #1447's `stream_a_lowers_exactly_as_before_the_facts_stream_existed`
    // fixture: a ranged hex jump and a regex alternation with a capture group.
    #[test]
    fn ported_hex_jump_range_and_regex_group() {
        let b = pattern("{ 41 [1-2] 42 }");
        assert!(b.matches_at(b"AxB", 0) && b.matches_at(b"AxxB", 0));
        assert!(!b.matches_at(b"AB", 0) && !b.matches_at(b"AxxxB", 0));
        let c = pattern("/A(B|CD)/");
        assert!(c.matches_at(b"AB", 0) && c.matches_at(b"ACD", 0) && !c.matches_at(b"AC", 0));
        let masked = pattern("/AC10(09|12)\\x00{5}/");
        assert!(masked.matches_at(b"AC1012\x00\x00\x00\x00\x00", 0));
        assert!(!masked.matches_at(b"AC1012\x00\x00\x00\x00", 0));
    }

    #[test]
    fn empty_unbounded_and_asserting_patterns_are_rejected() {
        for yara in ["/A+/", "/A*/", "/^A/"] {
            let text = format!("rule r {{ strings: $a = {yara} condition: $a }}");
            let ast = AST::from(text.as_str());
            let Item::Rule(rule) = &ast.items().next().unwrap() else { unreachable!() };
            let pattern = rule.patterns.as_ref().unwrap().iter().next().unwrap();
            assert!(Pattern::from_ast(pattern).is_err(), "{yara}");
        }
    }
}
