// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0

//! Synthetic facts stream derived from the input ahead of matching.
//!
//! Rules scan two independent streams. Stream A is the original prefix behind a 16-byte
//! size header, exactly as before preprocessing existed. Stream B is entirely synthetic:
//! a fixed facts header followed by fixed-size views, and it is only scanned when a
//! preprocessor produced something. Original bytes are never copied or rewritten.
//!
//! | stream B offset | size              | content                                   |
//! |-----------------|-------------------|-------------------------------------------|
//! | 0               | `FACTS_BYTES`     | big-endian unsigned facts, zero = absent  |
//! | `FACTS_BYTES`   | `ZIP_NAMES_BYTES` | `zip_names` view: central directory names |
//!
//! `original_size` and `prefix_size` occupy the same offsets in both streams, so the
//! size guards lower to identical patterns in either domain.

/// Size of the facts header at the start of stream B.
pub(crate) const FACTS_BYTES: usize = 64;
/// Size of the `zip_names` view following the facts header.
pub(crate) const ZIP_NAMES_BYTES: usize = 4096;

/// A named unsigned big-endian field of the facts header, addressable from conditions.
pub(crate) struct Fact {
    pub name: &'static str,
    pub offset: usize,
    pub width: usize,
}

impl Fact {
    /// Whether the fact lies in the size header, which stream A carries at the same offsets:
    /// such facts lower identically in both domains and never activate stream B.
    pub(crate) fn is_shared(&self) -> bool {
        self.offset + self.width <= super::EXTERNAL_BYTES
    }
}

const fn fact_at(name: &'static str, offset: usize, width: usize) -> Fact {
    Fact { name, offset, width }
}

/// Every fact a condition may name. Offsets are stable: they are part of the cache identity.
pub(crate) const FACTS: &[Fact] = &[
    fact_at("original_size", 0, 8),
    fact_at("prefix_size", 8, 8),
    fact_at("zip_valid", 16, 1),
    fact_at("zip_flags", 17, 1),
    fact_at("zip_entries", 18, 2),
    fact_at("zip_names_entries", 20, 2),
    fact_at("zip_names_len", 22, 2),
    fact_at("zip_comment_len", 24, 2),
    fact_at("zip_cd_size", 26, 4),
    fact_at("pe_valid", 32, 1),
    fact_at("pe_flags", 33, 1),
    fact_at("pe_machine", 34, 2),
    fact_at("pe_characteristics", 36, 2),
    fact_at("pe_subsystem", 38, 2),
    fact_at("pe_dll_characteristics", 40, 2),
    fact_at("pe_sections", 42, 2),
    fact_at("pe_magic", 44, 2),
    fact_at("pe_clr", 46, 1),
    fact_at("pe_signed", 47, 1),
    fact_at("pe_overlay", 48, 8),
    fact_at("pe_is_dll", 56, 1),
    fact_at("pe_is_executable_image", 57, 1),
];

/// Looks a fact up by the identifier used in conditions.
pub(crate) fn fact(name: &str) -> Option<&'static Fact> {
    FACTS.iter().find(|fact| fact.name == name)
}

/// A named fixed window of stream B whose bytes conditions may search with `contains`
/// and `startswith`.
pub(crate) struct View {
    pub name: &'static str,
    pub offset: usize,
    pub size: usize,
}

/// Every view a condition may name. Offsets are stable: they are part of the cache identity.
pub(crate) const VIEWS: &[View] =
    &[View { name: "zip_names", offset: FACTS_BYTES, size: ZIP_NAMES_BYTES }];

/// Looks a view up by the identifier used in conditions.
pub(crate) fn view(name: &str) -> Option<&'static View> {
    VIEWS.iter().find(|view| view.name == name)
}

/// Stores `value` as the big-endian field of `fact`. A value beyond the field's width
/// saturates to the field's maximum, so an oversized count reads as "at least the maximum"
/// rather than as an unrelated small number.
pub(crate) fn write_fact(facts: &mut [u8; FACTS_BYTES], fact: &Fact, value: u64) {
    let maximum = if fact.width == 8 { u64::MAX } else { (1 << (8 * fact.width)) - 1 };
    let bytes = value.min(maximum).to_be_bytes();
    facts[fact.offset..fact.offset + fact.width].copy_from_slice(&bytes[8 - fact.width..]);
}

/// The buffers of stream B, in stream order.
pub(crate) struct Synthetic {
    pub facts: [u8; FACTS_BYTES],
    pub names: Box<[u8; ZIP_NAMES_BYTES]>,
}

impl Synthetic {
    /// Whether a preprocessor produced anything worth scanning: any fact beyond the size
    /// header, or a nonempty `zip_names` view. Stream A already covers the sizes alone.
    pub(crate) fn is_active(&self) -> bool {
        self.facts[super::EXTERNAL_BYTES..].iter().any(|byte| *byte != 0) || self.names[0] != 0
    }
}

/// The original bytes available to preprocessing: the bounded prefix, the original size
/// and, when the caller read one, the trailing block of the input.
pub(crate) struct Blocks<'a> {
    pub prefix: &'a [u8],
    pub size: u64,
    /// Read by the container preprocessors (zip central directories live at the end).
    #[allow(dead_code)]
    pub tail: Option<&'a [u8]>,
}

#[cfg(test)]
thread_local! {
    /// Inputs preprocessed on this thread.
    pub(crate) static PREPARATIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Derives stream B for one input. Only the size facts are populated so far; container
/// preprocessors will fill the remaining facts and the `zip_names` view.
pub(crate) fn prepare(blocks: &Blocks<'_>) -> Synthetic {
    #[cfg(test)]
    PREPARATIONS.set(PREPARATIONS.get() + 1);
    let Blocks { prefix, size, tail: _ } = *blocks;
    let mut facts = [0; FACTS_BYTES];
    write_fact(&mut facts, fact("original_size").unwrap(), size);
    write_fact(&mut facts, fact("prefix_size").unwrap(), prefix.len() as u64);
    Synthetic { facts, names: Box::new([0; ZIP_NAMES_BYTES]) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn facts_are_disjoint_and_fit_the_header() {
        for (index, fact) in FACTS.iter().enumerate() {
            assert!(fact.width > 0 && fact.width <= 8, "{}", fact.name);
            assert!(fact.offset + fact.width <= FACTS_BYTES, "{}", fact.name);
            for other in &FACTS[..index] {
                assert_ne!(fact.name, other.name);
                assert!(
                    fact.offset >= other.offset + other.width
                        || other.offset >= fact.offset + fact.width,
                    "{} overlaps {}",
                    fact.name,
                    other.name
                );
            }
        }
        assert_eq!(fact("original_size").map(|x| (x.offset, x.width)), Some((0, 8)));
        assert_eq!(fact("prefix_size").map(|x| (x.offset, x.width)), Some((8, 8)));
        assert!(fact("filesize").is_none());
        // Exactly the stream A header fields are shared with stream B.
        let shared: Vec<_> = FACTS.iter().filter(|x| x.is_shared()).map(|x| x.name).collect();
        assert_eq!(shared, ["original_size", "prefix_size"]);
    }

    #[test]
    fn views_are_named_fixed_windows_of_the_facts_stream() {
        assert_eq!(
            view("zip_names").map(|x| (x.offset, x.size)),
            Some((FACTS_BYTES, ZIP_NAMES_BYTES))
        );
        assert!(view("pe_machine").is_none());
        for view in VIEWS {
            assert!(view.offset >= FACTS_BYTES, "{}", view.name);
            assert!(fact(view.name).is_none(), "{} is both a fact and a view", view.name);
        }
    }

    #[test]
    fn synthetic_is_active_only_when_a_preprocessor_produced_something() {
        // The size facts alone never activate the facts stream: stream A already carries them.
        let mut synthetic = prepare(&Blocks { prefix: b"x", size: u64::MAX >> 1, tail: None });
        assert!(!synthetic.is_active());
        synthetic.facts[15] = 0xff;
        assert!(!synthetic.is_active());
        synthetic.names[1] = b'a';
        assert!(!synthetic.is_active());
        synthetic.names[0] = b'\n';
        assert!(synthetic.is_active());
        let mut synthetic = prepare(&Blocks { prefix: b"x", size: 1, tail: None });
        write_fact(&mut synthetic.facts, fact("pe_is_executable_image").unwrap(), 1);
        assert!(synthetic.is_active());
    }

    #[test]
    fn prepare_writes_only_the_size_facts() {
        let prefix = [7; 4096];
        let synthetic = prepare(&Blocks { prefix: &prefix, size: 1 << 40, tail: Some(&[1, 2]) });
        assert_eq!(synthetic.facts[..8], (1_u64 << 40).to_be_bytes());
        assert_eq!(synthetic.facts[8..16], 4096_u64.to_be_bytes());
        assert!(synthetic.facts[16..].iter().all(|x| *x == 0));
        assert!(synthetic.names.iter().all(|x| *x == 0));
        assert_eq!(synthetic.names.len(), ZIP_NAMES_BYTES);
    }

    #[test]
    fn write_fact_uses_big_endian_field_width() {
        let mut facts = [0; FACTS_BYTES];
        write_fact(&mut facts, fact("pe_machine").unwrap(), 0x8664);
        write_fact(&mut facts, fact("zip_valid").unwrap(), 1);
        write_fact(&mut facts, fact("pe_overlay").unwrap(), 0x0102030405060708);
        assert_eq!(facts[34..36], [0x86, 0x64]);
        assert_eq!(facts[16], 1);
        assert_eq!(facts[48..56], [1, 2, 3, 4, 5, 6, 7, 8]);
        assert!(facts[..16].iter().all(|x| *x == 0));
    }

    #[test]
    fn write_fact_saturates_to_the_field_maximum() {
        let mut facts = [0; FACTS_BYTES];
        write_fact(&mut facts, fact("zip_valid").unwrap(), 300);
        write_fact(&mut facts, fact("zip_entries").unwrap(), 70_000);
        write_fact(&mut facts, fact("pe_overlay").unwrap(), u64::MAX);
        assert_eq!(facts[16], 255);
        assert_eq!(facts[18..20], [255, 255]);
        assert_eq!(facts[48..56], [255; 8]);
        // Neighbouring fields are untouched by the saturated writes.
        assert_eq!(facts[17], 0);
        assert_eq!(facts[20..22], [0, 0]);
        assert_eq!(facts[56], 0);
    }
}
