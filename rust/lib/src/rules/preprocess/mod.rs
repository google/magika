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
//! | `FACTS_BYTES + ZIP_NAMES_BYTES` | `ZIP_FIRST_ENTRY_BYTES` | `zip_first_entry` view: first entry name and data |
//!
//! `original_size` and `prefix_size` occupy the same offsets in both streams, so the
//! size guards lower to identical patterns in either domain.

mod pe;
mod zip;

use miniz_oxide::inflate::core::DecompressorOxide;
#[cfg(test)]
pub(crate) use zip::tests::{archive, stored};

/// Size of the facts header at the start of stream B.
pub(crate) const FACTS_BYTES: usize = 64;
/// Size of the `zip_names` view following the facts header.
pub(crate) const ZIP_NAMES_BYTES: usize = 4096;
/// Size of the `zip_first_entry` view following the `zip_names` view.
pub(crate) const ZIP_FIRST_ENTRY_BYTES: usize = 16 * 1024;

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
    fact_at("zip_first_entry_len", 30, 2),
];

/// Looks a fact up by the identifier used in conditions; the compiler resolves names once.
pub(crate) fn fact(name: &str) -> Option<&'static Fact> {
    FACTS.iter().find(|fact| fact.name == name)
}

/// The facts written for every input, resolved once by position in [`FACTS`] so that
/// preparing an input never searches the table; the tests pin the name at each position.
const ORIGINAL_SIZE: &Fact = &FACTS[0];
const PREFIX_SIZE: &Fact = &FACTS[1];
/// The value a zip fact takes from an analysis.
type ZipValue = fn(&zip::ZipFacts) -> u64;
/// The zip facts, resolved the same way, each with the value it takes from an analysis.
const ZIP_FACTS: [(&Fact, ZipValue); 8] = [
    (&FACTS[2], |zip| u64::from(zip.valid)),
    (&FACTS[3], |zip| u64::from(zip.flags)),
    (&FACTS[4], |zip| u64::from(zip.entries)),
    (&FACTS[5], |zip| u64::from(zip.names_entries)),
    (&FACTS[6], |zip| u64::from(zip.names_len)),
    (&FACTS[7], |zip| u64::from(zip.comment_len)),
    (&FACTS[8], |zip| u64::from(zip.cd_size)),
    (&FACTS[22], |zip| u64::from(zip.first_entry_len)),
];
/// The value a PE fact takes from an analysis.
type PeValue = fn(&pe::PeFacts) -> u64;
/// The PE facts, resolved the same way, each with the value it takes from an analysis.
const PE_FACTS: [(&Fact, PeValue); 13] = [
    (&FACTS[9], |pe| u64::from(pe.valid)),
    (&FACTS[10], |pe| u64::from(pe.flags)),
    (&FACTS[11], |pe| u64::from(pe.machine)),
    (&FACTS[12], |pe| u64::from(pe.characteristics)),
    (&FACTS[13], |pe| u64::from(pe.subsystem)),
    (&FACTS[14], |pe| u64::from(pe.dll_characteristics)),
    (&FACTS[15], |pe| u64::from(pe.sections)),
    (&FACTS[16], |pe| u64::from(pe.magic)),
    (&FACTS[17], |pe| u64::from(pe.clr)),
    (&FACTS[18], |pe| u64::from(pe.signed)),
    (&FACTS[19], |pe| pe.overlay),
    (&FACTS[20], |pe| u64::from(pe.is_dll)),
    (&FACTS[21], |pe| u64::from(pe.is_executable_image)),
];

/// A named fixed window of stream B whose bytes conditions may search with `contains`
/// and `startswith`.
pub(crate) struct View {
    pub name: &'static str,
    pub offset: usize,
    pub size: usize,
}

/// Every view a condition may name. Offsets are stable: they are part of the cache identity.
pub(crate) const VIEWS: &[View] = &[
    View { name: "zip_names", offset: FACTS_BYTES, size: ZIP_NAMES_BYTES },
    View {
        name: "zip_first_entry",
        offset: FACTS_BYTES + ZIP_NAMES_BYTES,
        size: ZIP_FIRST_ENTRY_BYTES,
    },
];

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
    pub first_entry: Box<[u8; ZIP_FIRST_ENTRY_BYTES]>,
    /// Inflate state reused for every first entry, reset before each use.
    inflater: Box<DecompressorOxide>,
}

impl Synthetic {
    /// An empty stream B, to be filled by [`Self::prepare`] once per input.
    pub(crate) fn new() -> Self {
        Self {
            facts: [0; FACTS_BYTES],
            names: Box::new([0; ZIP_NAMES_BYTES]),
            first_entry: Box::new([0; ZIP_FIRST_ENTRY_BYTES]),
            inflater: Box::default(),
        }
    }

    /// Whether a preprocessor produced anything worth scanning: any fact beyond the size
    /// header, or a nonempty `zip_names` view. Stream A already covers the sizes alone.
    pub(crate) fn is_active(&self) -> bool {
        self.facts[super::EXTERNAL_BYTES..].iter().any(|byte| *byte != 0) || self.names[0] != 0
    }

    /// Derives stream B for one input, forgetting whatever the previous input left behind.
    ///
    /// The size facts always describe the input. The zip facts and the `zip_names` view are
    /// derived by [`zip::analyze`] from the prefix, the size and the tail, only for a prefix
    /// that [`wants_tail`]; the tail is the prefix itself when it holds the whole input and
    /// empty when the caller read none, in which case only the first block is known. The PE
    /// facts are derived by [`pe::analyze`] from the prefix and the size alone, only for a
    /// prefix that [`pe::wants`]. The signatures are exclusive, so at most one preprocessor
    /// runs, and any other input pays nothing beyond the size facts.
    pub(crate) fn prepare(&mut self, blocks: &Blocks<'_>) {
        #[cfg(test)]
        PREPARATIONS.set(PREPARATIONS.get() + 1);
        let Blocks { prefix, size, tail } = *blocks;
        self.facts = [0; FACTS_BYTES];
        // A nonempty view opens with `\n` and is zero beyond its length, so a view whose
        // first byte is clear is clear throughout: most inputs skip the 4 KiB clear.
        if self.names[0] != 0 {
            self.names.fill(0);
        }
        // Likewise: the first entry view opens with a sanitized name byte or `\n`.
        if self.first_entry[0] != 0 {
            self.first_entry.fill(0);
        }
        write_fact(&mut self.facts, ORIGINAL_SIZE, size);
        write_fact(&mut self.facts, PREFIX_SIZE, prefix.len() as u64);
        if wants_tail(prefix) {
            let tail = tail.unwrap_or(if size == prefix.len() as u64 { prefix } else { &[] });
            let mut zip = zip::analyze(prefix, size, tail, &mut self.names);
            let (len, flags) = zip::first_entry(prefix, &mut self.first_entry, &mut self.inflater);
            zip.first_entry_len = len;
            zip.flags |= flags;
            for (fact, value) in ZIP_FACTS {
                write_fact(&mut self.facts, fact, value(&zip));
            }
        } else if pe::wants(prefix) {
            let pe = pe::analyze(prefix, size);
            for (fact, value) in PE_FACTS {
                write_fact(&mut self.facts, fact, value(&pe));
            }
        }
    }
}

/// The original bytes available to preprocessing: the bounded prefix, the original size
/// and, when the caller read one, the trailing block of the input.
pub(crate) struct Blocks<'a> {
    pub prefix: &'a [u8],
    pub size: u64,
    /// The last `ZIP_TAIL_BYTES` (or fewer) bytes of the input, read by [`read_tail`].
    pub tail: Option<&'a [u8]>,
}

#[cfg(test)]
thread_local! {
    /// Inputs preprocessed on this thread.
    pub(crate) static PREPARATIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// The little-endian u16 at `at`. Get-based and checked: a field not lying entirely
/// inside `bytes` (or an `at` that overflows) reads as `None`, never panics.
fn u16_at(bytes: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_le_bytes(bytes.get(at..at.checked_add(2)?)?.try_into().ok()?))
}

/// The little-endian u32 at `at`; see [`u16_at`].
fn u32_at(bytes: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(bytes.get(at..at.checked_add(4)?)?.try_into().ok()?))
}

/// Derives stream B for one input into a fresh allocation; see [`Synthetic::prepare`].
/// Production scans reuse a thread-local `Synthetic` instead.
#[cfg(test)]
pub(crate) fn prepare(blocks: &Blocks<'_>) -> Synthetic {
    let mut synthetic = Synthetic::new();
    synthetic.prepare(blocks);
    synthetic
}

/// Size of the trailing window read for archives: the central directory of most documents
/// fits in it, and the model's end block is its suffix.
pub(crate) const ZIP_TAIL_BYTES: usize = 16 * 1024;

/// Whether the prefix opens a zip archive (a local file header, or the end record of an
/// empty one), the only inputs whose tail is read ahead of matching.
pub(crate) fn wants_tail(prefix: &[u8]) -> bool {
    prefix.starts_with(b"PK\x03\x04") || prefix.starts_with(b"PK\x05\x06")
}

/// Reads the trailing window `[size - min(size, ZIP_TAIL_BYTES), size)` of an input whose
/// prefix [`wants_tail`], unless the prefix already holds the whole input. When that window
/// holds the end record of an archive whose central directory starts before it and spans at
/// most `ZIP_DIRECTORY_MAX_BYTES` ([`zip::directory_start`]), one more read extends the
/// window back to the directory's start. Any other input costs no read at all.
pub(crate) fn read_tail(
    input: &mut impl crate::Input, size: u64, prefix: &[u8],
) -> anyhow::Result<Option<Vec<u8>>> {
    if !wants_tail(prefix) || size <= prefix.len() as u64 {
        return Ok(None);
    }
    let _span = crate::startup_trace::span("input_tail_read");
    let len = size.min(ZIP_TAIL_BYTES as u64);
    let mut tail = vec![0; len as usize];
    input.read_at(&mut tail, size - len)?;
    if let Some(start) = zip::directory_start(&tail, size) {
        let before = (size - len - start) as usize;
        let mut extended = vec![0; before + tail.len()];
        input.read_at(&mut extended[..before], start)?;
        extended[before..].copy_from_slice(&tail);
        tail = extended;
    }
    Ok(Some(tail))
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    fn read_tail_extends_to_a_directory_beyond_the_window() {
        let entries: Vec<_> =
            (0..400).map(|i| stored(format!("{i:0>60}").as_bytes(), b"")).collect();
        let bytes = archive(&entries, b"");
        let (size, prefix) = (bytes.len() as u64, &bytes[..4096]);
        let tail = read_tail(&mut bytes.as_slice(), size, prefix).unwrap().unwrap();
        assert!(tail.len() > ZIP_TAIL_BYTES && bytes.ends_with(&tail), "{}", tail.len());
        let synthetic = prepare(&Blocks { prefix, size, tail: Some(&tail) });
        assert_eq!(synthetic.facts[16], 1, "zip_valid");
    }

    #[test]
    fn hot_path_facts_are_resolved_to_the_named_entries() {
        let resolved = [(ORIGINAL_SIZE, "original_size"), (PREFIX_SIZE, "prefix_size")]
            .into_iter()
            .chain(ZIP_FACTS.iter().map(|(fact, _)| *fact).zip([
                "zip_valid",
                "zip_flags",
                "zip_entries",
                "zip_names_entries",
                "zip_names_len",
                "zip_comment_len",
                "zip_cd_size",
                "zip_first_entry_len",
            ]))
            .chain(PE_FACTS.iter().map(|(fact, _)| *fact).zip([
                "pe_valid",
                "pe_flags",
                "pe_machine",
                "pe_characteristics",
                "pe_subsystem",
                "pe_dll_characteristics",
                "pe_sections",
                "pe_magic",
                "pe_clr",
                "pe_signed",
                "pe_overlay",
                "pe_is_dll",
                "pe_is_executable_image",
            ]));
        for (resolved, name) in resolved {
            let named = fact(name).unwrap();
            assert_eq!(resolved.name, name);
            assert_eq!((resolved.offset, resolved.width), (named.offset, named.width), "{name}");
        }
    }

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
    fn only_zip_signatures_want_a_tail() {
        assert!(wants_tail(b"PK\x03\x04\x14\x00"));
        assert!(wants_tail(b"PK\x05\x06"));
        for prefix in [&b""[..], b"PK", b"PK\x03", b"PK\x01\x02", b"PK\x07\x08", b"\x89PNG"] {
            assert!(!wants_tail(prefix), "{prefix:?}");
        }
    }

    /// An input of `size` bytes of `A`, opening with a local file header when `zip`, or
    /// of the bytes given to [`Self::serving`]; it records its reads and fails them all
    /// when `fail`.
    pub(crate) struct Probe {
        pub size: u64,
        pub zip: bool,
        pub fail: bool,
        pub reads: Vec<(u64, usize)>,
        bytes: Option<Vec<u8>>,
    }
    impl Probe {
        pub(crate) fn new(size: u64, zip: bool) -> Self {
            Probe { size, zip, fail: false, reads: Vec::new(), bytes: None }
        }

        /// An input serving exactly `bytes`.
        pub(crate) fn serving(bytes: &[u8]) -> Self {
            Probe { bytes: Some(bytes.to_vec()), ..Self::new(bytes.len() as u64, false) }
        }
    }
    impl crate::Input for Probe {
        fn length(&self) -> anyhow::Result<u64> {
            Ok(self.size)
        }
        fn read_at(&mut self, buffer: &mut [u8], offset: u64) -> anyhow::Result<()> {
            self.reads.push((offset, buffer.len()));
            anyhow::ensure!(!self.fail, "input read failed");
            if let Some(bytes) = &self.bytes {
                return crate::Input::read_at(&mut bytes.as_slice(), buffer, offset);
            }
            buffer.fill(b'A');
            if self.zip && offset == 0 {
                let head = buffer.len().min(4);
                buffer[..head].copy_from_slice(&b"PK\x03\x04"[..head]);
            }
            Ok(())
        }
    }
    fn prefix_of(input: &mut Probe) -> Vec<u8> {
        use crate::Input;
        let mut prefix = vec![0; input.size.min(4096) as usize];
        if !prefix.is_empty() {
            input.read_at(&mut prefix, 0).unwrap();
        }
        input.reads.clear();
        prefix
    }

    #[test]
    fn read_tail_reads_one_bounded_window_only_for_unheld_zips() {
        for (size, expected) in [
            (100_000, Some((83_616, 16_384))),
            (16_384, Some((0, 16_384))),
            (10_000, Some((0, 10_000))),
            (4097, Some((0, 4097))),
            (4096, None),
            (4000, None),
            (4, None),
        ] {
            let mut input = Probe::new(size, true);
            let prefix = prefix_of(&mut input);
            let tail = read_tail(&mut input, size, &prefix).unwrap();
            assert_eq!(tail.as_ref().map(|x| x.len()), expected.map(|x| x.1), "{size}");
            assert_eq!(input.reads, expected.into_iter().collect::<Vec<_>>(), "{size}");
            if let Some(tail) = tail {
                assert!(tail.iter().skip(4).all(|x| *x == b'A'), "tail bytes are the input's");
            }
        }
        let mut other = Probe::new(100_000, false);
        let prefix = prefix_of(&mut other);
        assert!(read_tail(&mut other, 100_000, &prefix).unwrap().is_none());
        assert_eq!(other.reads, []);
        let mut broken = Probe::new(100_000, true);
        let prefix = prefix_of(&mut broken);
        broken.fail = true;
        let error = read_tail(&mut broken, 100_000, &prefix).unwrap_err();
        assert!(error.to_string().contains("input read failed"));
    }

    fn zip_facts(synthetic: &Synthetic) -> [u8; 14] {
        synthetic.facts[16..30].try_into().unwrap()
    }

    #[test]
    fn prepare_analyzes_zips_from_the_held_blocks() {
        let bytes = zip::tests::archive(
            &[zip::tests::stored(b"word/document.xml", b"<w/>"), zip::tests::stored(b"x", b"")],
            b"!!",
        );
        let size = bytes.len() as u64;
        let cd_size = (2 * 46 + 17 + 1_u32).to_be_bytes();
        let expected_facts =
            [1, 0, 0, 2, 0, 2, 0, 21, 0, 2, cd_size[0], cd_size[1], cd_size[2], cd_size[3]];
        // Top-level names are written before nested ones.
        let expected_view = b"\nx\nword/document.xml\n";
        // A file held entirely in the prefix needs no tail.
        let held = prepare(&Blocks { prefix: &bytes, size, tail: None });
        assert_eq!(zip_facts(&held), expected_facts);
        assert_eq!(&held.names[..expected_view.len()], expected_view);
        assert!(held.names[expected_view.len()..].iter().all(|x| *x == 0));
        assert!(held.is_active());
        // The same archive behind a stub, seen through the bounded blocks.
        let mut big = vec![0; 100_000];
        big[..4].copy_from_slice(b"PK\x03\x04");
        big.extend_from_slice(&bytes);
        let prefix = &big[..4096];
        let tail = &big[big.len() - 16 * 1024..];
        let bounded = prepare(&Blocks { prefix, size: big.len() as u64, tail: Some(tail) });
        let mut prepended = expected_facts;
        prepended[1] = 1 << 5;
        assert_eq!(zip_facts(&bounded), prepended);
        assert_eq!(&bounded.names[..expected_view.len()], expected_view);
        // Without a tail nothing beyond the first block is known: the zip facts stay zero.
        let blind = prepare(&Blocks { prefix, size: big.len() as u64, tail: None });
        assert_eq!(zip_facts(&blind), [0; 14]);
        assert!(blind.names.iter().all(|x| *x == 0));
        // The size facts are written alongside.
        assert_eq!(bounded.facts[..8], (big.len() as u64).to_be_bytes());
        assert_eq!(bounded.facts[8..16], 4096_u64.to_be_bytes());
        // A prefix without the zip signature never takes the zip path, whatever the tail holds.
        let mut foreign = big.clone();
        foreign[..4].copy_from_slice(b"\x89PNG");
        let skipped = prepare(&Blocks {
            prefix: &foreign[..4096],
            size: foreign.len() as u64,
            tail: Some(&foreign[foreign.len() - 16 * 1024..]),
        });
        assert_eq!(zip_facts(&skipped), [0; 14]);
        assert!(!skipped.is_active());
    }

    fn pe_facts(synthetic: &Synthetic) -> [u8; 26] {
        synthetic.facts[32..58].try_into().unwrap()
    }

    #[test]
    fn prepare_analyzes_executables_from_the_first_block() {
        let mut bytes = pe::tests::pe32().build();
        bytes.extend_from_slice(&[0; 0x1234]);
        let size = bytes.len() as u64;
        let overlay = 0x1234_u64.to_be_bytes();
        let mut expected = [0; 26];
        expected[..14]
            .copy_from_slice(&[1, 0, 0x01, 0x4c, 0x01, 0x02, 0, 3, 0x81, 0x40, 0, 1, 0x01, 0x0b]);
        expected[16..24].copy_from_slice(&overlay);
        expected[25] = 1; // pe_is_executable_image
        let held = prepare(&Blocks { prefix: &bytes, size, tail: None });
        assert_eq!(pe_facts(&held), expected);
        assert!(held.is_active());
        assert!(held.names.iter().all(|x| *x == 0), "no view is written for executables");
        // Only the first block is used: the same facts come from the bounded prefix.
        let bounded = prepare(&Blocks { prefix: &bytes[..4096], size, tail: None });
        assert_eq!(pe_facts(&bounded), expected);
        // The size facts are written alongside, and the zip facts stay clear.
        assert_eq!(bounded.facts[..8], size.to_be_bytes());
        assert_eq!(bounded.facts[8..16], 4096_u64.to_be_bytes());
        assert!(bounded.facts[16..32].iter().all(|x| *x == 0));
        // A plain DOS executable is wanted but yields nothing: stream B stays inactive.
        let dos = prepare(&Blocks { prefix: &bytes[..0x40], size: 0x40, tail: None });
        assert_eq!(pe_facts(&dos), [0; 26]);
        assert!(!dos.is_active());
        // A prefix without the `MZ` signature never takes the PE path, whatever follows.
        let mut foreign = bytes.clone();
        foreign[..2].copy_from_slice(b"\x7fE");
        let skipped = prepare(&Blocks { prefix: &foreign[..4096], size, tail: None });
        assert_eq!(pe_facts(&skipped), [0; 26]);
        assert!(!skipped.is_active());
        // An archive prefix takes the zip path only: a PE header behind it is not read.
        let mut archive = bytes.clone();
        archive[..4].copy_from_slice(b"PK\x03\x04");
        let zipped = prepare(&Blocks { prefix: &archive[..4096], size, tail: Some(&[]) });
        assert_eq!(pe_facts(&zipped), [0; 26]);
        assert!(zipped.names.iter().all(|x| *x == 0));
    }

    #[test]
    fn a_reused_synthetic_forgets_the_previous_input() {
        let bytes = zip::tests::archive(&[zip::tests::stored(b"word/document.xml", b"")], b"");
        let mut synthetic =
            prepare(&Blocks { prefix: &bytes, size: bytes.len() as u64, tail: None });
        assert!(synthetic.is_active());
        synthetic.prepare(&Blocks { prefix: b"\x89PNG", size: 4, tail: None });
        assert!(!synthetic.is_active());
        assert!(synthetic.facts[16..].iter().all(|x| *x == 0));
        assert!(synthetic.names.iter().all(|x| *x == 0));
        assert_eq!(synthetic.facts[..8], 4_u64.to_be_bytes());
        // An executable followed by an archive: each leaves only its own facts.
        let executable = pe::tests::pe32().build();
        synthetic.prepare(&Blocks {
            prefix: &executable,
            size: executable.len() as u64,
            tail: None,
        });
        assert_eq!(synthetic.facts[32], 1, "pe_valid");
        assert!(synthetic.facts[16..32].iter().all(|x| *x == 0));
        synthetic.prepare(&Blocks { prefix: &bytes, size: bytes.len() as u64, tail: None });
        assert_eq!(synthetic.facts[16], 1, "zip_valid");
        assert!(synthetic.facts[32..].iter().all(|x| *x == 0));
    }

    /// Repository fixtures whose native facts and view pin the Python oracle: containers
    /// decided from their names, executables, the crafted probes and one plain image.
    const GOLDEN_FIXTURES: &[&str] = &[
        "basic/docx/doc.docx",
        "basic/epub/doc.epub",
        "basic/odp/magika_test.odp",
        "basic/ods/magika_test.ods",
        "basic/odt/doc.odt",
        "basic/png/magika_test.png",
        "basic/pptx/magika_test.pptx",
        "basic/xlsx/magika_test.xlsx",
        "basic/zip/magika_test.zip",
        "mitra/pebin/pe32.exe",
        "mitra/pebin/pe64.exe",
        "mitra/zip/simple.zip",
        "mitra/zip/zip64.zip",
        "rules_negative/aar_manifest_classes_jar.zip",
        "rules_negative/docx_names_in_comment.zip",
        "rules_negative/jar_manifest_only.zip",
        "rules_negative/mz_garbage.bin",
        "rules_negative/odf_deflated_mimetype.zip",
        "rules_negative/odf_text_template.ott",
        "rules_negative/pe_lfanew_out_of_range.bin",
        "rules_negative/zip64_docx_names.zip",
        "rules_negative/zip_plain_names.zip",
    ];

    fn repository() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    /// The golden file the Python oracle reads (`rules/benchmark/tests/test_preprocess.py`).
    fn golden_file() -> std::path::PathBuf {
        repository().join("rules/benchmark/tests/golden/preprocess.json")
    }

    /// The facts and the view digest of one fixture, derived exactly as identification
    /// does: the bounded prefix, the size and the tail window an archive reads.
    fn golden_record(relative: &str) -> serde_json::Value {
        use sha2::Digest;
        let bytes = std::fs::read(repository().join("tests_data").join(relative)).unwrap();
        let size = bytes.len() as u64;
        let prefix = &bytes[..bytes.len().min(crate::rules::PREFIX_LIMIT)];
        let tail = read_tail(&mut bytes.as_slice(), size, prefix).unwrap();
        let synthetic = prepare(&Blocks { prefix, size, tail: tail.as_deref() });
        let facts: serde_json::Map<_, _> = FACTS
            .iter()
            .map(|fact| {
                let mut value = [0; 8];
                value[8 - fact.width..]
                    .copy_from_slice(&synthetic.facts[fact.offset..fact.offset + fact.width]);
                (fact.name.to_string(), u64::from_be_bytes(value).into())
            })
            .collect();
        let digest = |view: &[u8]| data_encoding::HEXLOWER.encode(&sha2::Sha256::digest(view));
        serde_json::json!({
            "path": format!("tests_data/{relative}"),
            "facts": facts,
            "view_sha256": digest(synthetic.names.as_slice()),
            "first_entry_sha256": digest(synthetic.first_entry.as_slice()),
        })
    }

    #[test]
    fn golden_facts_are_written_on_request() {
        // `MAGIKA_WRITE_GOLDEN=<path>` records the native facts for the Python oracle.
        let Some(path) = std::env::var_os("MAGIKA_WRITE_GOLDEN") else { return };
        let records: Vec<_> = GOLDEN_FIXTURES.iter().map(|x| golden_record(x)).collect();
        let text = serde_json::to_string_pretty(&records).unwrap() + "\n";
        std::fs::write(path, text).unwrap();
    }

    #[test]
    fn golden_facts_are_reproduced() {
        let Ok(text) = std::fs::read_to_string(golden_file()) else { return };
        let records: Vec<serde_json::Value> = serde_json::from_str(&text).unwrap();
        assert_eq!(records.len(), GOLDEN_FIXTURES.len(), "regenerate with MAGIKA_WRITE_GOLDEN");
        for (record, relative) in records.iter().zip(GOLDEN_FIXTURES) {
            assert_eq!(*record, golden_record(relative), "{relative}");
        }
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
