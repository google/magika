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

//! Bioinformatics and life-sciences format verifiers.

use crate::ContentType as Label;
use crate::resolvers::magic::*;

/// Binary Alignment Map (BAM): `"BAM\x01"` magic and 8 bytes of header length / reference count.
pub(crate) fn bam(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"BAM\x01"; // @0..4: BAM v1 magic
    const MIN_LEN: usize = 12; // 4-byte magic + l_text (4) + n_ref (4)

    require!(prefix.len() >= MIN_LEN, starts(prefix, MAGIC));
    Some(Label::Bam)
}

/// CRAM compressed sequence alignment: `"CRAM"` magic, major/minor version `(1, 0)`, `(2, 0|1)`,
/// or `(3, 0|1)`, and complete 26-byte file definition header.
pub(crate) fn cram(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"CRAM"; // @0..4: CRAM format magic
    const MAJOR: usize = 4; // @4: u8 — major version (1..=3)
    const MINOR: usize = 5; // @5: u8 — minor version (0..=1)
    const MIN_LEN: usize = 26; // 26-byte CRAM file definition

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, MAGIC),
        matches!((u8_at(prefix, MAJOR)?, u8_at(prefix, MINOR)?), (1, 0) | (2 | 3, 0 | 1)),
    );
    Some(Label::Cram)
}

/// Brookhaven Protein Data Bank (`proteindb`, 1976): `"HEADER    "` at `@0` followed by
/// `"TITLE     "`, `"COMPND    "`, `"ATOM      "`, or `"HETATM    "`.
pub(crate) fn proteindb(prefix: &[u8], size: u64) -> Option<Label> {
    const PDB_HEADER_REC: &[u8; 10] = b"HEADER    ";
    const PDB_TITLE_LF: &[u8] = b"\nTITLE     ";
    const PDB_TITLE_CRLF: &[u8] = b"\r\nTITLE     ";
    const PDB_COMPND_LF: &[u8] = b"\nCOMPND    ";
    const PDB_ATOM_LF: &[u8] = b"\nATOM      ";
    const PDB_HETATM_LF: &[u8] = b"\nHETATM    ";

    require!(
        size >= 80,
        prefix.len() >= 80,
        starts(prefix, PDB_HEADER_REC),
        contains(prefix, PDB_TITLE_LF)
            || contains(prefix, PDB_TITLE_CRLF)
            || contains(prefix, PDB_COMPND_LF)
            || contains(prefix, PDB_ATOM_LF)
            || contains(prefix, PDB_HETATM_LF),
        is_utf8_text_header(prefix, size as usize),
    );
    Some(Label::Proteindb)
}

/// Variant Call Format (`vcf_genomics`, 2010): UTF-8 text header opening with `"##fileformat=VCFv4."`.
pub(crate) fn vcf_genomics(prefix: &[u8], size: u64) -> Option<Label> {
    const VCF_V4_HEADER: &[u8] = b"##fileformat=VCFv4.";

    require!(
        size >= 24,
        prefix.len() >= 20,
        starts(prefix, VCF_V4_HEADER),
        is_utf8_text_header(prefix, size as usize),
    );
    Some(Label::VcfGenomics)
}

/// Lipman & Pearson FASTA nucleotide/peptide sequence (`fasta`, 1985): `'>'` sequence header line
/// followed by a valid IUPAC nucleotide or amino-acid residue line.
pub(crate) fn fasta(prefix: &[u8], size: u64) -> Option<Label> {
    const FASTA_ALPHABET_MIXED: &[u8] = b"ACGTURYKMSWBDHVNacgturykmswbdhvnEFILPQXZefilpqxz-*";
    const FASTA_ALPHABET_UPPER: &[u8] = b"ACGTURYKMSWBDHVNEFILPQXZ-*";

    require!(
        size >= 14,
        prefix.len() >= 14,
        prefix[0] == b'>',
        !matches!(prefix[1], b'>' | b' ' | b'\t' | b'\r' | b'\n'),
        is_utf8_text_header(prefix, size as usize),
    );
    let nl = prefix[1..prefix.len().min(256)].iter().position(|&b| b == b'\n')? + 1;
    require!(nl >= 2 && nl + 4 <= prefix.len());
    let seq = &prefix[nl + 1..prefix.len().min(nl + 128)];
    let seq_line_end = seq.iter().position(|&b| b == b'\n' || b == b'\r').unwrap_or(seq.len());
    require!(
        seq_line_end >= 4,
        seq[..seq_line_end]
            .iter()
            .all(|&b| FASTA_ALPHABET_MIXED.contains(&b)),
        seq_line_end >= 16
            || seq[..seq_line_end]
                .iter()
                .all(|&b| FASTA_ALPHABET_UPPER.contains(&b)),
    );
    let _ = get_slice(prefix, 0, 2);
    let _ = get_slice(prefix, nl.saturating_sub(1), (nl + 1 + seq_line_end + 2).min(prefix.len()));
    Some(Label::Fasta)
}

/// Sanger Institute FASTQ sequencing reads with Phred quality scores (`fastq`, 2000): 4-line record
/// (`@<id>`, IUPAC base calls, `+<optional_id>`, and equal-length printable Phred quality string).
pub(crate) fn fastq(prefix: &[u8], size: u64) -> Option<Label> {
    const FASTQ_IUPAC_BASES: &[u8] = b"ACGTNURYKMSWBDHVacgtnurykmswbdhv";

    require!(
        size >= 16,
        prefix.len() >= 16,
        prefix[0] == b'@',
        is_utf8_text_header(prefix, size as usize),
    );
    let mut lines = prefix[..prefix.len().min(1024)].split(|&b| b == b'\n');
    let l0_raw = lines.next()?;
    let l0 = l0_raw.strip_suffix(b"\r").unwrap_or(l0_raw);
    let l1_raw = lines.next()?;
    let l1 = l1_raw.strip_suffix(b"\r").unwrap_or(l1_raw);
    let l2_raw = lines.next()?;
    let l2 = l2_raw.strip_suffix(b"\r").unwrap_or(l2_raw);
    let l3_raw = lines.next()?;
    let l3 = l3_raw.strip_suffix(b"\r").unwrap_or(l3_raw);
    require!(
        !l0.is_empty(),
        l1.len() >= 4,
        l1.iter().all(|&b| FASTQ_IUPAC_BASES.contains(&b)),
        l2.starts_with(b"+"),
        l3.len() == l1.len(),
        l3.iter().all(|&b| (0x21..=0x7E).contains(&b)),
    );
    let base = prefix.as_ptr() as usize;
    let l1_off = (l1_raw.as_ptr() as usize) - base;
    let l2_off = (l2_raw.as_ptr() as usize) - base;
    let l3_off = (l3_raw.as_ptr() as usize) - base;
    let l3_end = (l3_off + l3_raw.len() + 1).min(prefix.len());
    let _ = get_slice(prefix, 0, 1);
    let _ = get_slice(prefix, l1_off.saturating_sub(2), l1_off + l1_raw.len() + 1);
    let _ = get_slice(prefix, l2_off, l2_off + 1);
    let _ = get_slice(prefix, l3_off.saturating_sub(2), l3_end);
    Some(Label::Fastq)
}
