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

//! Scientific data, ML model, and binary serialization format verifiers.

use crate::ContentType as Label;
use crate::resolvers::magic::*;

/// GGML Universal File (GGUF): `"GGUF"` magic with version 1 (≥ 16 bytes) or version 2/3 (≥ 24
/// bytes) in either endianness.
pub(crate) fn gguf(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"GGUF"; // @0..4: GGUF magic
    const VERSION: usize = 4; // @4: u32 (LE or BE) — format version (1, 2, or 3)
    const V1_MIN_LEN: usize = 16; // magic (4) + version (4) + 2 × u32 counts (8)
    const V2_MIN_LEN: usize = 24; // magic (4) + version (4) + 2 × u64 counts (16)

    require!(prefix.len() >= V1_MIN_LEN, starts(prefix, MAGIC));
    let (v_le, v_be) = (le32(prefix, VERSION)?, be32(prefix, VERSION)?);
    let valid = v_le == 1
        || v_be == 1
        || (prefix.len() >= V2_MIN_LEN && (matches!(v_le, 2 | 3) || matches!(v_be, 2 | 3)));
    require!(valid);
    Some(Label::Gguf)
}

/// NCSA Hierarchical Data Format v4 (HDF4): `0E 03 13 01` magic and positive signed 16-bit DD
/// count (`1..=32767`) at offset 4.
pub(crate) fn hdf4(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = &[0x0E, 0x03, 0x13, 0x01]; // @0..4: HDF4 magic (^N^C^S^A)
    const DD_COUNT: usize = 4; // @4: u16 BE — number of data descriptors in block (1..=32767)
    const MIN_LEN: usize = 22;

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, MAGIC),
        (1..=32767).contains(&be16(prefix, DD_COUNT)?),
    );
    Some(Label::Hdf4)
}

/// NumPy serialized array (`.npy`): `"\x93NUMPY"` magic, version 1.0 (16-bit header length at `@8`)
/// or 2.0/3.0 (32-bit header length at `@8`), 16-byte total header alignment, and Python dict `{`.
pub(crate) fn npy(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"\x93NUMPY"; // @0..6: NumPy format signature
    const MAJOR: usize = 6; // @6: u8 — major version (1, 2, or 3)
    const MINOR: usize = 7; // @7: u8 — minor version (0)
    const HDR_LEN: usize = 8; // @8: u16 LE (v1) or u32 LE (v2/v3) — dictionary header length
    const MIN_LEN: usize = 64;

    fn match_dict(s: &[u8]) -> bool {
        if !starts(s, b"{") {
            return false;
        }
        let ws = s[1..].iter().take(17).take_while(|&&b| matches!(b, b' ' | b'\t')).count();
        ws <= 16 && matches!(s.get(1 + ws), Some(b'\'' | b'"'))
    }

    require!(prefix.len() >= MIN_LEN, starts(prefix, MAGIC), u8_at(prefix, MINOR)? == 0);
    let ok = match u8_at(prefix, MAJOR)? {
        1 => {
            let len = le16(prefix, HDR_LEN)?;
            len >= 54 && len % 16 == 6 && get_slice(prefix, 10, prefix.len()).is_some_and(match_dict)
        }
        2 | 3 => {
            let len = le32(prefix, HDR_LEN)?;
            len >= 52 && len % 16 == 4 && get_slice(prefix, 12, prefix.len()).is_some_and(match_dict)
        }
        _ => false,
    };
    require!(ok);
    Some(Label::Npy)
}

/// Teem Nearly Raw Raster Data (NRRD): `"NRRD000"` + version `'1'`..=`'5'` followed by LF or CRLF.
pub(crate) fn nrrd(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"NRRD000"; // @0..7: NRRD version prefix
    const VERSION: usize = 7; // @7: ASCII digit '1'..='5'

    require!(starts(prefix, MAGIC), matches!(u8_at(prefix, VERSION)?, b'1'..=b'5'));
    let eol =
        u8_at(prefix, 8)? == b'\n' || (u8_at(prefix, 8)? == b'\r' && u8_at(prefix, 9)? == b'\n');
    require!(eol);
    Some(Label::Nrrd)
}

/// Unidata NetCDF classic (`"CDF\x01"`) or 64-bit offset (`"CDF\x02"`): ≥ 32-byte header with
/// either `NC_DIMENSION` tag `10` at `@8` or zero tag at `@12`.
pub(crate) fn netcdf(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"CDF"; // @0..3: NetCDF magic prefix
    const VERSION: usize = 3; // @3: u8 — 1 (classic) or 2 (64-bit offset)
    const DIM_TAG: usize = 8; // @8: u32 BE — NC_DIMENSION tag (10) or ABSENT (0)
    const ZERO_FIELD: usize = 12; // @12: u32 BE — nelems when absent (0)
    const MIN_LEN: usize = 32;

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, MAGIC),
        matches!(u8_at(prefix, VERSION)?, 1 | 2),
        be32(prefix, ZERO_FIELD)? == 0 || be32(prefix, DIM_TAG)? == 10,
    );
    Some(Label::Netcdf)
}

/// Hugging Face `safetensors` model file: 64-bit little-endian header length `< 4 GiB` (high
/// dword `0` at `@4`), JSON object opening `{"` at `@8`, and `"\"dtype\""` within `@10..=512`.
pub(crate) fn safetensors(prefix: &[u8]) -> Option<Label> {
    const LEN_HI: usize = 4; // @4: u32 LE — high 32 bits of 64-bit JSON header length (0)
    const JSON_OPEN: usize = 8; // @8..10: "{\"" opening of JSON header
    const JSON_OPEN_MAGIC: &[u8] = b"{\"";
    const KEY_DTYPE: &[u8] = b"\"dtype\"";
    const DTYPE_MIN: usize = 10;
    const DTYPE_MAX: usize = 512;
    const MIN_LEN: usize = 16;

    require!(
        prefix.len() >= MIN_LEN,
        eq_at(prefix, JSON_OPEN, JSON_OPEN_MAGIC),
        le32(prefix, LEN_HI)? == 0,
        scan_range(prefix, DTYPE_MIN, DTYPE_MAX, KEY_DTYPE),
    );
    Some(Label::Safetensors)
}

/// HDF5 Hierarchical Data Format v5 (`h5`, 1998) & HDF5-backed Keras model (`keras`):
/// 8-byte signature `\x89HDF\r\n\x1A\n` at `@0` and superblock version `0..=3` at `@8`.
pub(crate) fn hdf5_or_keras(prefix: &[u8], size: u64) -> Option<Label> {
    const MIN_HDF5_SIZE: u64 = 48; // 48B v2/v3 superblock
    const HDF5_SIG: &[u8] = b"\x89HDF\r\n\x1A\n";
    const KERAS_VERSION_ATTR: &[u8] = b"keras_version";
    const KERAS_MODEL_CONFIG_ATTR: &[u8] = b"model_config";

    require!(
        size >= MIN_HDF5_SIZE,
        prefix.len() >= 16,
        starts(prefix, HDF5_SIG),
        u8_at(prefix, 8)? <= 3,
    );
    if contains(prefix, KERAS_VERSION_ATTR) || contains(prefix, KERAS_MODEL_CONFIG_ATTR) {
        return Some(Label::Keras);
    }
    Some(Label::H5)
}

/// Open Neural Network Exchange (`onnx`, 2017): Protobuf `ModelProto` opening with field 1
/// (`ir_version`, tag `0x08`, varint `1..=15`) at `@0..2` and containing `"onnx"`,
/// `"pytorch"`, `"tf2onnx"`, `"skl2onnx"`, or `"main_graph"` in the header.
pub(crate) fn onnx(prefix: &[u8], size: u64) -> Option<Label> {
    const ONNX_PRODUCER_ONNX: &[u8] = b"onnx";
    const ONNX_PRODUCER_PYTORCH: &[u8] = b"pytorch";
    const ONNX_PRODUCER_BACKEND_TEST: &[u8] = b"backend-test";
    const ONNX_MAIN_GRAPH: &[u8] = b"main_graph";

    require!(
        size >= 32,
        prefix.len() >= 16,
        u8_at(prefix, 0)? == 0x08,
        (1..=15).contains(&u8_at(prefix, 1)?),
        matches!(u8_at(prefix, 2)?, 0x12 | 0x1A | 0x20 | 0x28 | 0x3A | 0x42),
        contains(prefix, ONNX_PRODUCER_ONNX)
            || contains(prefix, ONNX_PRODUCER_PYTORCH)
            || contains(prefix, ONNX_PRODUCER_BACKEND_TEST)
            || contains(prefix, ONNX_MAIN_GRAPH),
    );
    Some(Label::Onnx)
}

/// Python `pickle` binary serialization protocol 2..=5 (`\x80\x02`..`\x80\x05` at `@0`).
pub(crate) fn pickle_binary(prefix: &[u8], size: u64) -> Option<Label> {
    const PICKLE_PROTO_OPCODE: u8 = 0x80;
    require!(
        size >= 8,
        prefix.len() >= 8,
        u8_at(prefix, 0)? == PICKLE_PROTO_OPCODE,
        (2..=5).contains(&u8_at(prefix, 1)?),
    );
    let op = u8_at(prefix, 2)?;
    require!(matches!(
        op,
        0x95 | b'}' | b']' | b'(' | b')' | b'c' | b'C' | b'B' | b'X' | b'U' | b'K' | b'M' | b'J'
            | b'G' | 0x89 | 0x88 | b'N' | 0x8C | 0x94
    ));
    let _ = get_slice(prefix, 0, 8);
    Some(Label::Pickle)
}

/// Python `pickle` ASCII protocol 0 (`(dp0`, `(lp0`, `c__builtin__`, `ccopy_reg`).
pub(crate) fn pickle_ascii(prefix: &[u8], size: u64) -> Option<Label> {
    const PICKLE_V0_DICT_LF: &[u8] = b"(dp0\n";
    const PICKLE_V0_DICT_CRLF: &[u8] = b"(dp0\r\n";
    const PICKLE_V0_LIST_LF: &[u8] = b"(lp0\n";
    const PICKLE_V0_BUILTIN_LF: &[u8] = b"c__builtin__\n";
    const PICKLE_V0_COPY_REG_LF: &[u8] = b"ccopy_reg\n";

    require!(
        size >= 8,
        prefix.len() >= 8,
        starts(prefix, PICKLE_V0_DICT_LF)
            || starts(prefix, PICKLE_V0_DICT_CRLF)
            || starts(prefix, PICKLE_V0_LIST_LF)
            || starts(prefix, PICKLE_V0_BUILTIN_LF)
            || starts(prefix, PICKLE_V0_COPY_REG_LF),
    );
    Some(Label::Pickle)
}

/// Protocol Buffers Text Format (`textproto`): `# proto-file:` + `# proto-message:` header
/// comments or TensorFlow `node {\n  name: "` graph definition.
pub(crate) fn textproto(prefix: &[u8], size: u64) -> Option<Label> {
    const PROTO_FILE_HEADER: &[u8] = b"# proto-file:";
    const PROTO_MESSAGE_HEADER: &[u8] = b"# proto-message:";
    const TF_NODE_OPEN: &[u8] = b"node {";
    const TF_NAME_FIELD: &[u8] = b"name: \"";
    const TF_OP_FIELD: &[u8] = b"op: \"";

    require!(
        size >= 20,
        prefix.len() >= 20,
        (starts(prefix, PROTO_FILE_HEADER) && contains(prefix, PROTO_MESSAGE_HEADER))
            || (starts(prefix, TF_NODE_OPEN) && contains(prefix, TF_NAME_FIELD) && contains(prefix, TF_OP_FIELD)),
        is_utf8_text_header(prefix, size as usize),
    );
    Some(Label::Textproto)
}

/// Yandex CatBoost gradient boosting model (`.cbm`, `catboost`, 2017): `"CBM1"` magic at `@0..4`
/// followed by bounded FlatBuffers model core size at `@4..8`.
pub(crate) fn catboost(prefix: &[u8], size: u64) -> Option<Label> {
    const CATBOOST_CBM1_MAGIC: &[u8; 4] = b"CBM1";

    require!(
        size >= 12,
        prefix.len() >= 12,
        starts(prefix, CATBOOST_CBM1_MAGIC),
        (1..0x0800_0000).contains(&le32(prefix, 4)?),
    );
    Some(Label::Catboost)
}

/// Georgi Gerganov GGML / GGMF / GGJT tensor model (`ggml`, 2022): `"ggml"`/`"lmgg"` with bounded
/// vocabulary size at `@4..8`, or `"ggmf"`/`"ggjt"`/`"fmgg"`/`"tjgg"` with format version `1..=3` at `@4..8`.
pub(crate) fn ggml(prefix: &[u8], size: u64) -> Option<Label> {
    const GGML_LE_MAGIC: &[u8; 4] = b"ggml";
    const GGML_BE_MAGIC: &[u8; 4] = b"lmgg";
    const GGMF_LE_MAGIC: &[u8; 4] = b"ggmf";
    const GGJT_LE_MAGIC: &[u8; 4] = b"ggjt";
    const GGMF_BE_MAGIC: &[u8; 4] = b"fmgg";
    const GGJT_BE_MAGIC: &[u8; 4] = b"tjgg";

    require!(size >= 12, prefix.len() >= 12);
    if starts(prefix, GGML_LE_MAGIC) || starts(prefix, GGML_BE_MAGIC) {
        let vocab = le32(prefix, 4)?;
        require!(vocab <= 1_000_000 && (vocab >= 1 || size >= 64));
        return Some(Label::Ggml);
    }
    require!(
        starts(prefix, GGMF_LE_MAGIC)
            || starts(prefix, GGJT_LE_MAGIC)
            || starts(prefix, GGMF_BE_MAGIC)
            || starts(prefix, GGJT_BE_MAGIC),
        (1..=3).contains(&le32(prefix, 4)?),
    );
    Some(Label::Ggml)
}

/// R statistical workspace serialization (`.RData` / `.rda`, `rdata`, 2004): `"RDX2"`, `"RDX3"`,
/// `"RDA2"`, `"RDA3"`, `"RDB2"`, or `"RDB3"` header followed by stream format tag `'X'`, `'A'`, or `'B'`.
pub(crate) fn rdata(prefix: &[u8], size: u64) -> Option<Label> {
    const RDX2_LF: &[u8] = b"RDX2\n";
    const RDX3_LF: &[u8] = b"RDX3\n";
    const RDA2_LF: &[u8] = b"RDA2\n";
    const RDA3_LF: &[u8] = b"RDA3\n";
    const RDB2_LF: &[u8] = b"RDB2\n";
    const RDB3_LF: &[u8] = b"RDB3\n";
    const RDX2_CRLF: &[u8] = b"RDX2\r\n";
    const RDX3_CRLF: &[u8] = b"RDX3\r\n";

    require!(
        size >= 14,
        prefix.len() >= 14,
        starts(prefix, RDA2_LF)
            || starts(prefix, RDA3_LF)
            || ((starts(prefix, RDX2_LF)
                || starts(prefix, RDX3_LF)
                || starts(prefix, RDB2_LF)
                || starts(prefix, RDB3_LF)
                || starts(prefix, RDX2_CRLF)
                || starts(prefix, RDX3_CRLF))
                && has_non_ascii_text_byte(&prefix[5..14])),
        matches!(u8_at(prefix, 5)?, b'X' | b'A' | b'B'),
    );
    Some(Label::Rdata)
}

/// RFC 8949 Concise Binary Object Representation (`cbor`, 2013): self-describe tag `55799`
/// (`D9 D9 F7`) followed by a valid CBOR initial byte at `@3`, or tag `55` indefinite array map (`D8 37 9F A1`).
pub(crate) fn cbor(prefix: &[u8], size: u64) -> Option<Label> {
    const CBOR_SELF_DESCRIBE_TAG: &[u8] = &[0xD9, 0xD9, 0xF7];
    const CBOR_TAG55_ARRAY_MAP: &[u8] = &[0xD8, 0x37, 0x9F, 0xA1];

    require!(size >= 8, prefix.len() >= 8);
    if starts(prefix, CBOR_SELF_DESCRIBE_TAG) {
        let b3 = u8_at(prefix, 3)?;
        let ai = b3 & 0x1F;
        require!(ai < 28 || (ai == 31 && matches!(b3 >> 5, 2 | 3 | 4 | 5 | 7)));
        let _ = get_slice(prefix, 0, 8);
        return Some(Label::Cbor);
    }
    if starts(prefix, CBOR_TAG55_ARRAY_MAP) {
        let _ = get_slice(prefix, 0, 8);
        return Some(Label::Cbor);
    }
    None
}

/// Sadayuki Furuhashi MessagePack binary object (`msgpack`, 2009): `fixmap` (`0x81..=0x8F`) at `@0`
/// opening with a printable ASCII `fixstr` key (`0xA2..=0xBF`) at `@1` and free of reserved byte `0xC1`.
pub(crate) fn msgpack(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        (8..=256).contains(&size),
        prefix.len() >= 8,
        (0x81..=0x8F).contains(&prefix[0]),
        (0xA2..=0xBF).contains(&prefix[1]),
    );
    let klen = (prefix[1] & 0x1F) as usize;
    require!(
        2 + klen < prefix.len(),
        prefix[2..2 + klen].iter().all(|&b| b.is_ascii_graphic()),
        !prefix[..prefix.len().min(128)].contains(&0xC1),
    );
    let _ = get_slice(prefix, 0, (4 + klen).max(8).min(prefix.len()));
    Some(Label::Msgpack)
}

/// Google Protocol Buffers `FileDescriptorSet` / `FileDescriptorProto` binary (`protobuf`, 2008):
/// field 1 (`0x0A`) opening a descriptor whose `name` (`0x0A`) ends with `".proto"` or `".protodevel"`.
pub(crate) fn protobuf(prefix: &[u8], size: u64) -> Option<Label> {
    const PROTO_EXT: &[u8] = b".proto";
    const PROTODEVEL_EXT: &[u8] = b".protodevel";

    require!(size >= 16, prefix.len() >= 12, prefix[0] == 0x0A);
    let read_varint = |slice: &[u8], p: &mut usize| -> Option<u64> {
        let mut val = 0u64;
        for shift in (0..64).step_by(7) {
            let b = *slice.get(*p)?;
            *p += 1;
            if shift > 0 && b == 0 {
                return None;
            }
            val |= ((b & 0x7F) as u64) << shift;
            if (b & 0x80) == 0 {
                return Some(val);
            }
        }
        None
    };
    let check_desc = |slice: &[u8], start: usize| -> Option<usize> {
        if *slice.get(start)? != 0x0A {
            return None;
        }
        let mut p = start + 1;
        let name_len = read_varint(slice, &mut p)? as usize;
        if !(7..=250).contains(&name_len) || p + name_len >= slice.len() {
            return None;
        }
        let name = &slice[p..p + name_len];
        if !(name.ends_with(PROTO_EXT) || name.ends_with(PROTODEVEL_EXT)) {
            return None;
        }
        if !name.iter().all(|&b| b.is_ascii_alphanumeric() || matches!(b, b'/' | b'.' | b'_' | b'-')) {
            return None;
        }
        p += name_len;
        let next_tag = *slice.get(p)?;
        if !matches!(
            next_tag,
            0x12 | 0x1A | 0x22 | 0x2A | 0x32 | 0x3A | 0x42 | 0x4A | 0x50 | 0x52 | 0x58 | 0x5A | 0x62 | 0x70
        ) {
            return None;
        }
        Some(p + 1)
    };
    let mut pos = 1usize;
    if let Some(flen) = read_varint(prefix, &mut pos) {
        let flen = flen as usize;
        if flen >= 9 && (pos + flen) as u64 <= size {
            let end = (pos + flen).min(prefix.len());
            if let Some(read_end) = check_desc(&prefix[..end], pos) {
                let end_idx = read_end.min(prefix.len());
                require!(has_non_ascii_text_byte(&prefix[..end_idx]));
                let _ = get_slice(prefix, 0, end_idx);
                return Some(Label::Protobuf);
            }
        }
    }
    if let Some(read_end) = check_desc(prefix, 0) {
        let end_idx = read_end.min(prefix.len());
        require!(has_non_ascii_text_byte(&prefix[..end_idx]));
        let _ = get_slice(prefix, 0, end_idx);
        return Some(Label::Protobuf);
    }
    None
}

/// Google TensorFlow Lite FlatBuffers model (`.tflite`, `tflite`, 2017): `"TFL3"` file identifier
/// at `@4..8` and bounded root table offset at `@0..4`.
pub(crate) fn tflite(prefix: &[u8], size: u64) -> Option<Label> {
    const TFLITE_V3_ID: &[u8; 4] = b"TFL3";

    require!(
        size >= 16,
        prefix.len() >= 16,
        eq_at(prefix, 4, TFLITE_V3_ID),
        (8..size.min(0x0800_0000) as u32).contains(&le32(prefix, 0)?),
    );
    Some(Label::Tflite)
}

/// MongoDB Binary JSON document (`bson`, 2009): `u32 LE` document length matching `size` at `@0..4`,
/// valid element type `1..=19` at `@4`, NUL-terminated printable ASCII key at `@5..`, and trailing `0x00`.
pub(crate) fn bson(prefix: &[u8], size: u64) -> Option<Label> {
    require!((8..=16_777_216).contains(&size), prefix.len() >= 10);
    let doc_len = u32::from_le_bytes([prefix[0], prefix[1], prefix[2], prefix[3]]) as u64;
    let elem_type = prefix[4];
    require!(
        (doc_len == size || (size == 10 && doc_len == 8)) && (1..=19).contains(&elem_type),
        prefix[prefix.len().min(doc_len as usize) - 1] == 0,
    );
    let nul_pos = prefix[5..prefix.len().min(64)].iter().position(|&b| b == 0)?;
    require!(
        (1..=48).contains(&nul_pos),
        prefix[5..5 + nul_pos].iter().all(|&b| b.is_ascii_graphic()),
    );
    let _ = get_slice(prefix, 0, (6 + nul_pos).min(prefix.len()));
    let _ = get_slice(prefix, prefix.len().min(doc_len as usize) - 1, prefix.len().min(doc_len as usize));
    Some(Label::Bson)
}

/// WMO GRIB meteorological data: `"GRIB"` indicator magic and edition 1 or 2 at offset 7.
pub(crate) fn grib(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"GRIB"; // @0..4: GRIB indicator section magic
    const EDITION: usize = 7; // @7: u8 — GRIB edition number (1 or 2)
    const MIN_LEN: usize = 16;

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, MAGIC),
        matches!(u8_at(prefix, EDITION)?, 1 | 2),
    );
    Some(Label::Grib)
}

/// OpenStreetMap PBF binary stream: 4-byte big-endian `BlobHeader` length with zero high half-word
/// at `@0..2`, followed by protobuf field 1 (`0x0A`, length 9, `"OSMHeader"`) at `@4..15`.
pub(crate) fn osm_pbf(prefix: &[u8]) -> Option<Label> {
    const LEN_HI: &[u8] = &[0x00, 0x00]; // @0..2: high 16 bits of BlobHeader length (0)
    const BLOB_TYPE_OFFSET: usize = 4; // @4..15: protobuf tag 0x0A, length 9, "OSMHeader"
    const BLOB_TYPE: &[u8] = b"\x0a\x09OSMHeader";
    const MIN_LEN: usize = 16;

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, LEN_HI),
        eq_at(prefix, BLOB_TYPE_OFFSET, BLOB_TYPE),
    );
    Some(Label::Osm)
}
