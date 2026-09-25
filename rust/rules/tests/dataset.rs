// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0

//! The bundled rules over the evaluation dataset (`dataset/` on the dataset branch): zero
//! false positives against validated labels, plus throughput on real files. Not run in CI.
//!
//! A false positive is an enforced match or conflict on a sample whose `label_status` is
//! `validated_*` when the rule's label is not one the sample's class accepts: its
//! `format_id`, the Magika labels the dataset maps it to, or a label in `REFINES`. On
//! unverified samples (`need_review`, `conflicting`) a mismatch is printed as a disagreement
//! to adjudicate in the dataset, never treated as a label here.
//!
//! Export the manifest (`sha256<TAB>format_id<TAB>label_status<TAB>magika_labels<TAB>path`,
//! labels comma-separated) from `dataset/`:
//!
//! ```text
//! uv run --no-sync python - manifest.tsv <<'EOF'
//! import sys, os, json, pyarrow.parquet as pq
//! root = os.path.abspath('local/corpus/objects')
//! kb = {r['format_id']: (json.loads(r['metadata_json'] or '{}').get('magika') or {}).get('kb_labels') or []
//!       for r in pq.read_table('classes.parquet', columns=['format_id', 'metadata_json']).to_pylist()}
//! t = pq.read_table('samples.parquet', columns=['sha256', 'format_id', 'label_status'])
//! with open(sys.argv[1], 'w') as out:
//!     for sha, fmt, status in zip(*(t.column(c).to_pylist() for c in t.column_names)):
//!         h = sha.hex()
//!         out.write(f'{h}\t{fmt}\t{status}\t{",".join(kb[fmt])}\t{root}/{h[:2]}/{h}\n')
//! EOF
//! ```
//!
//! Then: `MAGIKA_RULES_DATASET=manifest.tsv cargo test --release --test dataset -- --ignored --nocapture`

#![cfg(feature = "bundled")]

use std::collections::BTreeMap;
use std::io::{Read, Seek, SeekFrom};
use std::time::{Duration, Instant};

use magika_rules::{Input, Outcome, RuleSet, PREFIX_LIMIT};

/// Dataset classes that refine a Magika label with no label of their own.
const REFINES: &[(&str, &str)] = &[
    // GeoParquet is Apache Parquet with geometry metadata.
    ("geoparquet", "parquet"),
];

/// Validated samples whose label the bytes refute, checked outside the rules. Each must scan
/// to the proven label. Fix them in the dataset and drop the group.
const REFUTED: &[(&str, &str, &str, &[&str])] = &[
    (
        "ar",
        "vib",
        "ar archives whose first member is descriptor.xml with a `<vib version=\"5.0\">` root: VMware VIBs.",
        &[
            "72f2911c080b22d100aa2514c5e4e276691c3806b52794b1a923a5f9f4b50d54",
            "86ee8df25377693e4f08eb4bcdc242c8afd384522ae69aa8db1f74184b34e49f",
            "a9e0558c12f9492cadbbfa64b2c3b2cf9f6d8c7820112db36c7736f00eb0c13e",
            "e9a4eead8e5d4082248831dcb01b76f7c64c868d2b048f85922e2b26cba1e6cb",
            "ea5905e813777b2c7f1637af091bd19935be658cdb676cd220015a10442a3da4",
            "f59e252b779d86779d1bfd0dcdc86eec1b8e7d2308aab9cc92e81c6ff367fefa",
        ],
    ),
    (
        "crt",
        "pem",
        "Text beginning `-----BEGIN CERTIFICATE-----`: PEM (RFC 7468); Magika `crt` is binary DER.",
        &[
            "02521ff63937c94c3ebee1a4336b1d6ed8431cbbaea467a90ba889ed174fc49a",
            "1e53f9f517b5d0d3db0b59318c6ddeaae7c0f5dc4ce30b724b5a154c79639386",
            "1ec49f2f263a75b35b6e02e0d336ee9ead2b992053047c2150dec449c1791ea0",
            "20242441a7883730be5e2301d919435f5b8cb864bfeba449136dfb8cbc699e2a",
            "251b9bf761e2adb9acaed33f5a72a5e26d1398f42a50e9c2f4e6d327cc412dfa",
            "2907261efb8c04ef71e46b88f825d2545b05c3640e7ce373462e8874db29b097",
            "2d7988add8de890b044135296b775bf530b9acb54f7a65df78e1b78aeacdce68",
            "2f6e552db6b47b6d90b4153e5121133d3e815feddae4bf482ed3286065e626fd",
            "32349be298b582594549162d982372560de48a963cb19e431ce9920cf4603b85",
            "33b9040dca5b7f60eeaa05c737de384719c93466585c3b5660c9e305be78ea13",
            "364bdb3e699047bc67dd285cee4700fb49d8cd7a265ec32ae4bd835bb6c7aa5c",
            "3931989cecd0cdd3377da8a478c6674522f88797651bacf3eeda295ef4f4fe87",
            "3f16169b579714e62223ed743cc07a31fd86c280450204f9cad2a622c4bbd24d",
            "3fd3048358d45036c0714f095070d8b9da72f334b9c2f6666af48f274b526ec5",
            "41091be9d4f9a3cf87b51c99f249fc73a0d369fcdefbab5793e5af635fbd42b9",
            "44f02bd5fd08bdf7c652f59b8bc5e9961fe343f96b80ec6328b07087fc5ea6ae",
            "5def3b21b7d8dbf207b4de1cd320097ad61e2a39b8a18f2c51685e1e301d2a5b",
            "634c52103a77ef703b29341a69f69a5c8f3bf9659fd2d176ad78c10572f3fe8d",
            "63e4696c692b7d23822623f0cedd6b7bd72b8978324d26bbf45e1d3da8a425c0",
            "69767dde53d043687e186ee2b13129066a79311cb89b85e876fff3f388109864",
            "722f35326d53cb4d5258ad58d4f7b646c803fac143327cfc482c6e138ff85b21",
            "72e3a142bbc6a15be3f13af5a1513cfe323cab56e6f167dcef383d8adb0e4f51",
            "78f4ce31f1874113edaff300e128028e049907571c579193ddd28b58bcd141c1",
            "7c78668bd0986717610818340cdbd8c3a3ef75ce5e88684841650fef8b0a2052",
            "7f5d222e160d1ed51dc88c43c0de506683f21feeb4b2aacf6dc6c32c35445207",
            "818d7b36b7a46fa6b054947bd261d0703ae543d262637920f6c6e944adf11ef5",
            "904a401383919fcada528755912fcb914a399770e343039a2823ecb74e7a4b4d",
            "941cde894ff38f0d5478663ad332133d8ea509586f034204861c446c57cda346",
            "9b5b9b740ac112c56a5e1d54daa4823216416a675943ac5b11ca5b6e67d67bb7",
            "a579daebe246a885495c97c9128bfeaac09ded4b14a2fcd1d75be3fb05e08e05",
            "a84facbaad7d757333348b97319b2ac48b7dbe7a6ca07afc86ffcfcca7b8e821",
            "a8bbff408bdcaf737e460675e9cece78051e11c0efe59516ecba2c6ee2439357",
            "a934ef479bd0574179fc150392c88b43838abda06e82ffbde57f8207495027b8",
            "aa85ab6e06a712c07ee04e41e5e4d898d487509bc8dea06397c4c99a5c05ad13",
            "bb396e06116509c5341b13f5e77d1d4bc1668f25c594f83d4c046341991b1d71",
            "cac6a925e783ba5d063295e971d5556d71e28cf1fbb36813f38cf662665c24bb",
            "ce9ac74b899c22f035622a45497190c5a912ce19eefaf04f4a8ded8ead9e539a",
            "d1d6c5094d97dd4c8b06d7d59cd9a7c18899858fa6e20ac3312d1cf948209104",
            "d20cd87f81415e56c28021724dbe34228566d4f81e1285c54f99c23fc402a810",
            "dae3b4f3891032ef94f36e4b0cb4b70be6b949cd1e297420fe59deae9f695b10",
            "dbb7015e4c2814bb110cec1def64f56e686f7d9563383ec24ce86437a8f129be",
            "dc657b2965cd4094bb2dd40afd46956029c5e03ff14b86cf522f9fa5f630c316",
            "e5dd841d71cfa8780010a57975d5bddb4499fbb820af7db8e6960bd9fa590854",
            "f79490338be542bd19bc18c84d8e348e75af1bb25799aa5468e2bb52c5efe6cd",
            "fe99f632725aa880dfa401b016db16a442b3343abc75d47e0caa6f39a5780182",
        ],
    ),
    (
        "zip",
        "gzip",
        "alibaba/fastjson `issue859.zip` is a gzip stream: `gzip -t` passes, no zip end of central directory.",
        &[
            "b0d2436a3a8021f76c3d85f9f4b0b512f1f8d53a74096b546e678899de8bca6e",
        ],
    ),
];

#[test]
#[ignore = "needs MAGIKA_RULES_DATASET, a manifest exported from the evaluation dataset"]
fn bundled_rules_never_contradict_a_validated_label() {
    let manifest = std::env::var("MAGIKA_RULES_DATASET").expect("MAGIKA_RULES_DATASET");
    let rules = RuleSet::bundled();
    let (mut files, mut verified, mut hits, mut agreements) = (0usize, 0usize, 0usize, 0usize);
    let (mut bytes_scanned, mut elapsed) = (0usize, Duration::ZERO);
    let mut false_positives = Vec::new();
    let mut disagreements: BTreeMap<(String, String), usize> = BTreeMap::new();
    let mut prefix = vec![0u8; PREFIX_LIMIT];
    for line in std::fs::read_to_string(&manifest).unwrap().lines() {
        let [sha256, format_id, status, magika_labels, path] =
            line.split('\t').collect::<Vec<_>>()[..]
        else {
            panic!("malformed manifest line: {line}")
        };
        let refuted = REFUTED.iter().find(|(_, _, _, samples)| samples.contains(&sha256));
        let accepts = |label: &str| match refuted {
            Some((_, proven, _, _)) => label == *proven,
            None => {
                label == format_id
                    || magika_labels.split(',').any(|l| l == label)
                    || REFINES.contains(&(format_id, label))
            }
        };
        let mut file = std::fs::File::open(path).unwrap();
        let size = file.metadata().unwrap().len();
        let wanted = size.min(PREFIX_LIMIT as u64) as usize;
        file.read_exact(&mut prefix[..wanted]).unwrap();
        // The tail a caller reads: the last `tail_len` bytes, back to `tail_start` if needed.
        let mut tail = vec![0; rules.tail_len(&prefix[..wanted], size)];
        file.seek(SeekFrom::Start(size - tail.len() as u64)).unwrap();
        file.read_exact(&mut tail).unwrap();
        if let Some(start) = rules.tail_start(&tail, size) {
            tail = vec![0; (size - start) as usize];
            file.seek(SeekFrom::Start(start)).unwrap();
            file.read_exact(&mut tail).unwrap();
        }
        let validated = status.starts_with("validated_");
        files += 1;
        verified += usize::from(validated);
        bytes_scanned += wanted;
        let start = Instant::now();
        let tail = (!tail.is_empty()).then_some(tail.as_slice());
        let outcome = rules.scan(Input { prefix: &prefix[..wanted], size, tail });
        elapsed += start.elapsed();
        let said = match outcome {
            Outcome::Match(i) if accepts(&rules.labels()[i]) => {
                if validated {
                    hits += 1;
                } else {
                    agreements += 1;
                }
                continue;
            }
            Outcome::Match(i) => rules.labels()[i].clone(),
            Outcome::Conflict => "<conflict>".to_string(),
            _ if refuted.is_some() => "<no match>".to_string(),
            Outcome::NoMatch | Outcome::InsufficientInput => continue,
        };
        if validated {
            false_positives.push(format!("{sha256} {format_id} ({status}): rules say {said}"));
        } else {
            *disagreements.entry((format_id.to_string(), said)).or_default() += 1;
        }
    }
    let mean = elapsed / files.max(1) as u32;
    eprintln!(
        "dataset: {files} files ({verified} validated), {hits} validated hits, \
         {agreements} unverified agreements, {} false positives, {} unverified disagreements; \
         {bytes_scanned} prefix bytes, mean scan {mean:?}",
        false_positives.len(),
        disagreements.values().sum::<usize>(),
    );
    for ((format_id, said), count) in &disagreements {
        eprintln!("  unverified {format_id} -> {said}: {count}");
    }
    assert!(false_positives.is_empty(), "false positives:\n{}", false_positives.join("\n"));
}
