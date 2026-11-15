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
    static GGUF_V1_LE: Subformat = Subformat::with_ids("GGUF v1 (little-endian)", "fmt/2096", None);
    static GGUF_V1_BE: Subformat = Subformat::with_ids("GGUF v1 (big-endian)", "fmt/2096", None);
    static GGUF_V2_LE: Subformat = Subformat::with_ids("GGUF v2 (little-endian)", "fmt/2097", None);
    static GGUF_V2_BE: Subformat = Subformat::with_ids("GGUF v2 (big-endian)", "fmt/2097", None);
    static GGUF_V3_LE: Subformat = Subformat::with_ids("GGUF v3 (little-endian)", "fmt/2098", None);
    static GGUF_V3_BE: Subformat = Subformat::with_ids("GGUF v3 (big-endian)", "fmt/2098", None);
    const MAGIC: &[u8] = b"GGUF"; // @0..4: GGUF magic
    const VERSION: usize = 4; // @4: u32 (LE or BE) — format version (1, 2, or 3)
    const V1_MIN_LEN: usize = 16; // magic (4) + version (4) + 2 × u32 counts (8)
    const V2_MIN_LEN: usize = 24; // magic (4) + version (4) + 2 × u64 counts (16)

    require!(prefix.len() >= V1_MIN_LEN, starts(prefix, MAGIC));
    let (v_le, v_be) = (le32(prefix, VERSION)?, be32(prefix, VERSION)?);
    let sub = if v_le == 1 {
        &GGUF_V1_LE
    } else if v_be == 1 {
        &GGUF_V1_BE
    } else if prefix.len() >= V2_MIN_LEN {
        match (v_le, v_be) {
            (2, _) => &GGUF_V2_LE,
            (3, _) => &GGUF_V3_LE,
            (_, 2) => &GGUF_V2_BE,
            (_, 3) => &GGUF_V3_BE,
            _ => return None,
        }
    } else {
        return None;
    };
    with_subformat(Label::Gguf, sub)
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
    static NPY_1_0: Subformat = Subformat::new("NumPy array v1.0");
    static NPY_2_0: Subformat = Subformat::new("NumPy array v2.0");
    static NPY_3_0: Subformat = Subformat::new("NumPy array v3.0");
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
    let major = u8_at(prefix, MAJOR)?;
    let (ok, sub) = match major {
        1 => {
            let len = le16(prefix, HDR_LEN)?;
            (
                len >= 54
                    && len % 16 == 6
                    && get_slice(prefix, 10, prefix.len()).is_some_and(match_dict),
                &NPY_1_0,
            )
        }
        2 | 3 => {
            let len = le32(prefix, HDR_LEN)?;
            (
                len >= 52
                    && len % 16 == 4
                    && get_slice(prefix, 12, prefix.len()).is_some_and(match_dict),
                if major == 2 { &NPY_2_0 } else { &NPY_3_0 },
            )
        }
        _ => return None,
    };
    require!(ok);
    with_subformat(Label::Npy, sub)
}

/// Teem Nearly Raw Raster Data (NRRD): `"NRRD000"` + version `'1'`..=`'5'` followed by LF or CRLF.
pub(crate) fn nrrd(prefix: &[u8]) -> Option<Label> {
    static NRRD_1: Subformat = Subformat::with_ids("NRRD0001", "fmt/1002", None);
    static NRRD_2: Subformat = Subformat::with_ids("NRRD0002", "fmt/1003", None);
    static NRRD_3: Subformat = Subformat::with_ids("NRRD0003", "fmt/1004", None);
    static NRRD_4: Subformat = Subformat::with_ids("NRRD0004", "fmt/1005", None);
    static NRRD_5: Subformat = Subformat::with_ids("NRRD0005", "fmt/1006", None);
    const MAGIC: &[u8] = b"NRRD000"; // @0..7: NRRD version prefix
    const VERSION: usize = 7; // @7: ASCII digit '1'..='5'

    require!(starts(prefix, MAGIC));
    let sub = match u8_at(prefix, VERSION)? {
        b'1' => &NRRD_1,
        b'2' => &NRRD_2,
        b'3' => &NRRD_3,
        b'4' => &NRRD_4,
        b'5' => &NRRD_5,
        _ => return None,
    };
    let eol =
        u8_at(prefix, 8)? == b'\n' || (u8_at(prefix, 8)? == b'\r' && u8_at(prefix, 9)? == b'\n');
    require!(eol);
    with_subformat(Label::Nrrd, sub)
}

/// Unidata NetCDF classic (`"CDF\x01"`) or 64-bit offset (`"CDF\x02"`): ≥ 32-byte header with
/// either `NC_DIMENSION` tag `10` at `@8` or zero tag at `@12`.
pub(crate) fn netcdf(prefix: &[u8]) -> Option<Label> {
    static NETCDF_3_CLASSIC: Subformat =
        Subformat::with_ids("NetCDF-3 classic", "fmt/282", Some("fdd000330"));
    static NETCDF_3_64BIT: Subformat =
        Subformat::with_ids("NetCDF-3 64-bit offset", "fmt/283", Some("fdd000330"));
    const MAGIC: &[u8] = b"CDF"; // @0..3: NetCDF magic prefix
    const VERSION: usize = 3; // @3: u8 — 1 (classic) or 2 (64-bit offset)
    const DIM_TAG: usize = 8; // @8: u32 BE — NC_DIMENSION tag (10) or ABSENT (0)
    const ZERO_FIELD: usize = 12; // @12: u32 BE — nelems when absent (0)
    const MIN_LEN: usize = 32;

    require!(prefix.len() >= MIN_LEN, starts(prefix, MAGIC));
    let sub = match u8_at(prefix, VERSION)? {
        1 => &NETCDF_3_CLASSIC,
        2 => &NETCDF_3_64BIT,
        _ => return None,
    };
    require!(be32(prefix, ZERO_FIELD)? == 0 || be32(prefix, DIM_TAG)? == 10);
    with_subformat(Label::Netcdf, sub)
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
    static KERAS_HDF5: Subformat = Subformat::new("Keras HDF5 model");
    static NETCDF_4: Subformat = Subformat::with_fdd("NetCDF-4 (HDF5)", "fdd000332");
    static HDF5_SB0: Subformat =
        Subformat::with_ids("HDF5 (superblock v0)", "fmt/807", Some("fdd000229"));
    static HDF5_SB1: Subformat =
        Subformat::with_ids("HDF5 (superblock v1)", "fmt/286", Some("fdd000229"));
    static HDF5_SB2: Subformat =
        Subformat::with_ids("HDF5 (superblock v2)", "fmt/287", Some("fdd000229"));
    static HDF5_SB3: Subformat = Subformat::with_fdd("HDF5 (superblock v3)", "fdd000229");
    const MIN_HDF5_SIZE: u64 = 48; // 48B v2/v3 superblock
    const HDF5_SIG: &[u8] = b"\x89HDF\r\n\x1A\n";
    const KERAS_VERSION_ATTR: &[u8] = b"keras_version";
    const KERAS_MODEL_CONFIG_ATTR: &[u8] = b"model_config";

    require!(size >= MIN_HDF5_SIZE, prefix.len() >= 16, starts(prefix, HDF5_SIG));
    let sb_ver = u8_at(prefix, 8)?;
    require!(sb_ver <= 3);
    if contains(prefix, KERAS_VERSION_ATTR) || contains(prefix, KERAS_MODEL_CONFIG_ATTR) {
        return with_subformat(Label::Keras, &KERAS_HDF5);
    }
    if contains(prefix, b"_NCProperties")
        || contains(prefix, b"_Netcdf4Dimid")
        || contains(prefix, b"a netCDF variable.")
        || (contains(prefix, b"DIMENSION_SCALE") && contains(prefix, b"DIMENSION_LIST"))
    {
        return with_subformat(Label::Netcdf, &NETCDF_4);
    }
    let sub = match sb_ver {
        0 => &HDF5_SB0,
        1 => &HDF5_SB1,
        2 => &HDF5_SB2,
        3 => &HDF5_SB3,
        _ => unreachable!(),
    };
    with_subformat(Label::H5, sub)
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
    static PICKLE_P2: Subformat = Subformat::new("Python pickle (protocol 2)");
    static PICKLE_P3: Subformat = Subformat::new("Python pickle (protocol 3)");
    static PICKLE_P4: Subformat = Subformat::new("Python pickle (protocol 4)");
    static PICKLE_P5: Subformat = Subformat::new("Python pickle (protocol 5)");
    const PICKLE_PROTO_OPCODE: u8 = 0x80;
    require!(size >= 8, prefix.len() >= 8, u8_at(prefix, 0)? == PICKLE_PROTO_OPCODE);
    let sub = match u8_at(prefix, 1)? {
        2 => &PICKLE_P2,
        3 => &PICKLE_P3,
        4 => &PICKLE_P4,
        5 => &PICKLE_P5,
        _ => return None,
    };
    let op = u8_at(prefix, 2)?;
    require!(matches!(
        op,
        0x95 | b'}'
            | b']'
            | b'('
            | b')'
            | b'c'
            | b'C'
            | b'B'
            | b'X'
            | b'U'
            | b'K'
            | b'M'
            | b'J'
            | b'G'
            | 0x89
            | 0x88
            | b'N'
            | 0x8C
            | 0x94
    ));
    let _ = get_slice(prefix, 0, 8);
    with_subformat(Label::Pickle, sub)
}

/// Python `pickle` ASCII protocol 0 (`(dp0`, `(lp0`, `c__builtin__`, `ccopy_reg`).
pub(crate) fn pickle_ascii(prefix: &[u8], size: u64) -> Option<Label> {
    static PICKLE_P0: Subformat = Subformat::new("Python pickle (protocol 0)");
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
    with_subformat(Label::Pickle, &PICKLE_P0)
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
            || (starts(prefix, TF_NODE_OPEN)
                && contains(prefix, TF_NAME_FIELD)
                && contains(prefix, TF_OP_FIELD)),
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
    static GGML_LE: Subformat = Subformat::new("GGML (little-endian)");
    static GGML_BE: Subformat = Subformat::new("GGML (big-endian)");
    static GGMF_LE: Subformat = Subformat::new("GGMF (little-endian)");
    static GGMF_BE: Subformat = Subformat::new("GGMF (big-endian)");
    static GGJT_LE: Subformat = Subformat::new("GGJT (little-endian)");
    static GGJT_BE: Subformat = Subformat::new("GGJT (big-endian)");
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
        return with_subformat(
            Label::Ggml,
            if starts(prefix, GGML_LE_MAGIC) { &GGML_LE } else { &GGML_BE },
        );
    }
    let sub = if starts(prefix, GGMF_LE_MAGIC) {
        &GGMF_LE
    } else if starts(prefix, GGJT_LE_MAGIC) {
        &GGJT_LE
    } else if starts(prefix, GGMF_BE_MAGIC) {
        &GGMF_BE
    } else if starts(prefix, GGJT_BE_MAGIC) {
        &GGJT_BE
    } else {
        return None;
    };
    require!((1..=3).contains(&le32(prefix, 4)?));
    with_subformat(Label::Ggml, sub)
}

/// R statistical workspace serialization (`.RData` / `.rda`, `rdata`, 2004): `"RDX2"`, `"RDX3"`,
/// `"RDA2"`, `"RDA3"`, `"RDB2"`, or `"RDB3"` header followed by stream format tag `'X'`, `'A'`, or `'B'`.
pub(crate) fn rdata(prefix: &[u8], size: u64) -> Option<Label> {
    static RDATA_ASCII_V2: Subformat =
        Subformat::with_ids("RData v2 (ASCII, RDA2)", "fmt/1199", Some("fdd000470"));
    static RDATA_ASCII_V3: Subformat =
        Subformat::with_ids("RData v3 (ASCII, RDA3)", "fmt/1199", Some("fdd000470"));
    static RDATA_XDR_V2: Subformat =
        Subformat::with_ids("RData v2 (XDR binary, RDX2)", "fmt/1198", Some("fdd000470"));
    static RDATA_XDR_V3: Subformat =
        Subformat::with_ids("RData v3 (XDR binary, RDX3)", "fmt/1198", Some("fdd000470"));
    static RDATA_BIN_V2: Subformat =
        Subformat::with_ids("RData v2 (native binary, RDB2)", "fmt/1198", Some("fdd000470"));
    static RDATA_BIN_V3: Subformat =
        Subformat::with_ids("RData v3 (native binary, RDB3)", "fmt/1198", Some("fdd000470"));
    const RDX2_LF: &[u8] = b"RDX2\n";
    const RDX3_LF: &[u8] = b"RDX3\n";
    const RDA2_LF: &[u8] = b"RDA2\n";
    const RDA3_LF: &[u8] = b"RDA3\n";
    const RDB2_LF: &[u8] = b"RDB2\n";
    const RDB3_LF: &[u8] = b"RDB3\n";
    const RDX2_CRLF: &[u8] = b"RDX2\r\n";
    const RDX3_CRLF: &[u8] = b"RDX3\r\n";

    require!(size >= 14, prefix.len() >= 14);
    let sub = if starts(prefix, RDA2_LF) {
        &RDATA_ASCII_V2
    } else if starts(prefix, RDA3_LF) {
        &RDATA_ASCII_V3
    } else {
        let bin_sub = if starts(prefix, RDX2_LF) || starts(prefix, RDX2_CRLF) {
            &RDATA_XDR_V2
        } else if starts(prefix, RDX3_LF) || starts(prefix, RDX3_CRLF) {
            &RDATA_XDR_V3
        } else if starts(prefix, RDB2_LF) {
            &RDATA_BIN_V2
        } else if starts(prefix, RDB3_LF) {
            &RDATA_BIN_V3
        } else {
            return None;
        };
        require!(has_non_ascii_text_byte(&prefix[5..14]));
        bin_sub
    };
    require!(matches!(u8_at(prefix, 5)?, b'X' | b'A' | b'B'));
    with_subformat(Label::Rdata, sub)
}

/// RFC 8949 Concise Binary Object Representation (`cbor`, 2013): self-describe tag `55799`
/// (`D9 D9 F7`) followed by a valid CBOR initial byte at `@3`, or tag `55` indefinite array map (`D8 37 9F A1`).
pub(crate) fn cbor(prefix: &[u8], size: u64) -> Option<Label> {
    static CBOR_TAG_55799: Subformat = Subformat::new("CBOR (self-described, tag 55799)");
    static CBOR_TAG_55: Subformat = Subformat::new("CBOR (tag 55 indefinite array)");
    const CBOR_SELF_DESCRIBE_TAG: &[u8] = &[0xD9, 0xD9, 0xF7];
    const CBOR_TAG55_ARRAY_MAP: &[u8] = &[0xD8, 0x37, 0x9F, 0xA1];

    require!(size >= 8, prefix.len() >= 8);
    if starts(prefix, CBOR_SELF_DESCRIBE_TAG) {
        let b3 = u8_at(prefix, 3)?;
        let ai = b3 & 0x1F;
        require!(ai < 28 || (ai == 31 && matches!(b3 >> 5, 2 | 3 | 4 | 5 | 7)));
        let _ = get_slice(prefix, 0, 8);
        return with_subformat(Label::Cbor, &CBOR_TAG_55799);
    }
    if starts(prefix, CBOR_TAG55_ARRAY_MAP) {
        let _ = get_slice(prefix, 0, 8);
        return with_subformat(Label::Cbor, &CBOR_TAG_55);
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
    static PROTOBUF_FDS: Subformat = Subformat::new("Protobuf FileDescriptorSet");
    static PROTOBUF_FDP: Subformat = Subformat::new("Protobuf FileDescriptorProto");
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
        if !name
            .iter()
            .all(|&b| b.is_ascii_alphanumeric() || matches!(b, b'/' | b'.' | b'_' | b'-'))
        {
            return None;
        }
        p += name_len;
        let next_tag = *slice.get(p)?;
        if !matches!(
            next_tag,
            0x12 | 0x1A
                | 0x22
                | 0x2A
                | 0x32
                | 0x3A
                | 0x42
                | 0x4A
                | 0x50
                | 0x52
                | 0x58
                | 0x5A
                | 0x62
                | 0x70
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
                return with_subformat(Label::Protobuf, &PROTOBUF_FDS);
            }
        }
    }
    if let Some(read_end) = check_desc(prefix, 0) {
        let end_idx = read_end.min(prefix.len());
        require!(has_non_ascii_text_byte(&prefix[..end_idx]));
        let _ = get_slice(prefix, 0, end_idx);
        return with_subformat(Label::Protobuf, &PROTOBUF_FDP);
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
    let _ = get_slice(
        prefix,
        prefix.len().min(doc_len as usize) - 1,
        prefix.len().min(doc_len as usize),
    );
    Some(Label::Bson)
}

/// WMO GRIB meteorological data: `"GRIB"` indicator magic and edition 1 or 2 at offset 7.
pub(crate) fn grib(prefix: &[u8]) -> Option<Label> {
    static GRIB_1: Subformat = Subformat::with_ids("GRIB edition 1", "fmt/284", None);
    static GRIB_2: Subformat = Subformat::with_ids("GRIB edition 2", "fmt/285", None);
    const MAGIC: &[u8] = b"GRIB"; // @0..4: GRIB indicator section magic
    const EDITION: usize = 7; // @7: u8 — GRIB edition number (1 or 2)
    const MIN_LEN: usize = 16;

    require!(prefix.len() >= MIN_LEN, starts(prefix, MAGIC));
    let sub = match u8_at(prefix, EDITION)? {
        1 => &GRIB_1,
        2 => &GRIB_2,
        _ => return None,
    };
    with_subformat(Label::Grib, sub)
}

/// OpenStreetMap PBF binary stream: 4-byte big-endian `BlobHeader` length with zero high half-word
/// at `@0..2`, followed by protobuf field 1 (`0x0A`, length 9, `"OSMHeader"`) at `@4..15`.
pub(crate) fn osm_pbf(prefix: &[u8]) -> Option<Label> {
    static OSM_PBF: Subformat = Subformat::new("OpenStreetMap PBF");
    const LEN_HI: &[u8] = &[0x00, 0x00]; // @0..2: high 16 bits of BlobHeader length (0)
    const BLOB_TYPE_OFFSET: usize = 4; // @4..15: protobuf tag 0x0A, length 9, "OSMHeader"
    const BLOB_TYPE: &[u8] = b"\x0a\x09OSMHeader";
    const MIN_LEN: usize = 16;

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, LEN_HI),
        eq_at(prefix, BLOB_TYPE_OFFSET, BLOB_TYPE),
    );
    with_subformat(Label::Osm, &OSM_PBF)
}

// =============================================================================
// Additional Standalone Format Verifiers
// =============================================================================

const BGSM_TILE_FLAGS_OFFSET: usize = 8;
const BGSM_VERSION_OFFSET: usize = 4;
const BIGWORLD_PACKED_XML_MAGIC: &[u8; 4] = hex2bin!("45 4E A1 62");
const CERN_ROOT_MAGIC: &[u8; 4] = b"root";
const CERN_ZEBRA_FZ_STEER_BE: &[u8; 16] =
    hex2bin!("01 23 CD EF 80 70 80 70 43 21 AB CD 80 61 80 61");
const CERN_ZEBRA_FZ_STEER_LE: &[u8; 16] =
    hex2bin!("EF CD 23 01 70 80 70 80 CD AB 21 43 61 80 61 80");
const DIABLO2_SAVE_MAGIC: &[u8; 4] = hex2bin!("55 AA 55 AA");
const DRAGONBONES_JSON_LEN_OFFSET: usize = 8;
const DRAGONBONES_MAGIC: &[u8; 4] = b"DBDT";
const DRAGONBONES_VER_OFFSET: usize = 4;
const GODOT_RSCC_MAGIC: &[u8; 4] = b"RSCC";
const GODOT_RSRC_MAGIC: &[u8; 4] = b"RSRC";
const LARIAN_LSOF_MAGIC: &[u8; 4] = b"LSOF";
const MIN_ACRONIS_MSG_CATALOG_SIZE: u64 = 64;
const MIN_ASDF_SIZE: u64 = 43;
const MIN_BETHESDA_BGSM_SIZE: u64 = 32;
const MIN_BIGWORLD_PACKED_XML_SIZE: u64 = 16;
const MIN_BINEXPORT2_SIZE: u64 = 48;
const MIN_BIOWARE_GFF_SIZE: u64 = 28;
const MIN_CERN_ROOT_SIZE: u64 = 128;
const MIN_CERN_ZEBRA_FZ_SIZE: u64 = 32;
const MIN_DESY_FPACK_BOS_SIZE: u64 = 68;
const MIN_DIABLO2_SAVE_SIZE: u64 = 96;
const MIN_DRAGONBONES_DBBIN_SIZE: u64 = 32;
const MIN_FLASH_SOL_SIZE: u64 = 20;
const MIN_FREEDOM_FORCE_SAVE_SIZE: u64 = 1024;
const MIN_GODOT_RSRC_SIZE: u64 = 32;
const MIN_LARIAN_LSOF_SIZE: u64 = 32;
const MIN_LCIO_SIO_SIZE: u64 = 28;
const MIN_MCAP_SIZE: u64 = 25;
const MIN_MEGAZEUX_WORLD_SIZE: u64 = 256;
const MIN_MINITAUR_LOG_PROTO_SIZE: u64 = 64;
const MIN_MSFS_SPB_SIZE: u64 = 32;
const MIN_MS_PRAGUE_SIZE: u64 = 68;
const MIN_NINTENDO_AAMP_SIZE: u64 = 48;
const MIN_NINTENDO_BYML_SIZE: u64 = 24;
const MIN_NMS_MBIN_SIZE: u64 = 88;
const MIN_OPENRCT2_PARK_SIZE: u64 = 48;
const MIN_PHARO_FUEL_SIZE: u64 = 32;
const MIN_RAILWORKS_SERZ_SIZE: u64 = 32;
const MIN_REDENGINE_CR2W_SIZE: u64 = 64;
const MIN_RE_ENGINE_MDF2_SIZE: u64 = 32;
const MIN_RIEGELI_SIZE: u64 = 64;
const MIN_SCS_SII_ENCRYPTED_SIZE: u64 = 32;
const MIN_STOCKFISH_NNUE_SIZE: u64 = 64;
const MIN_TERRARIA_WORLD_SIZE: u64 = 32;
const MIN_TFRECORD_SIZE: u64 = 17;
const MIN_ULTIMATE_RIDE_TSF_SIZE: u64 = 28;
const MIN_UNITY_ADDRESSABLES_CATALOG_SIZE: u64 = 36;
const MIN_UNREAL_USMAP_SIZE: u64 = 24;
const MIN_WINDOWS_RGMH_SAVE_SIZE: u64 = 0x2028;
const NINTENDO_AAMP_MAGIC: &[u8; 4] = b"AAMP";
const NINTENDO_BYML_BE_MAGIC: &[u8; 2] = b"BY";
const NINTENDO_BYML_LE_MAGIC: &[u8; 2] = b"YB";
const NMS_HASH_OFFSET: usize = 16;
const NMS_MBIN_MAGIC: &[u8; 8] = hex2bin!("CC CC CC CC CC CC CC CC");
const NMS_VERSION_OFFSET: usize = 8;
const NNUE_DESC_LEN_OFFSET: usize = 8;
const NNUE_HASH_OFFSET: usize = 4;
const RAILWORKS_SERZ_MAGIC: &[u8; 11] =
    bytes!(b"SERZ", &[0x00, 0x00, 0x01, 0x00, 0xFF, 0x43, 0x00]);
const REDENGINE_CR2W_MAGIC: &[u8; 4] = b"CR2W";
const RE_MDF2_COUNT_OFFSET: usize = 6;
const RE_MDF2_MAGIC: &[u8; 6] = bytes!(b"MDF", &[0x00, 0x01, 0x00]);
const RE_MDF2_RESERVED_OFFSET: usize = 8;
const RIEGELI_SIGNATURE_32: &[u8; 32] = &[
    0x83, 0xAF, 0x70, 0xD1, 0x0D, 0x88, 0x4A, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x91, 0xBA, 0xC2, 0x3C, 0x92, 0x87, 0xE1, 0xA9,
];
const STOCKFISH_NNUE_MAGIC: &[u8; 4] = hex2bin!("20 2F F3 7A");
const UNITY_ADDRESSABLES_CATALOG_MAGIC: &[u8; 4] = hex2bin!("42 89 E3 0D");
const UNREAL_USMAP_MAGIC_2: &[u8; 2] = hex2bin!("C4 30");

#[inline]
fn crc32c_8bytes(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0x82F6_3B78 & mask);
        }
    }
    !crc
}

#[inline]
fn masked_crc32c_8bytes(data: &[u8]) -> u32 {
    let crc = crc32c_8bytes(data);
    crc.rotate_right(15).wrapping_add(0xA282_EAD8)
}

#[inline]
fn read_binexport_varint(buf: &[u8], mut pos: usize) -> Option<(u64, usize)> {
    let mut val = 0u64;
    let mut shift = 0u32;
    for _ in 0..10 {
        let b = u8_at(buf, pos)?;
        pos += 1;
        val |= ((b & 0x7F) as u64) << shift;
        if (b & 0x80) == 0 {
            return Some((val, pos));
        }
        shift += 7;
    }
    None
}

/// Acronis Bootable Media Builder UTF-16LE Message String Catalog (`.msg`).
pub(crate) fn verify_acronis_msg_catalog(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_ACRONIS_MSG_CATALOG_SIZE, prefix.len() >= 32);
    let count = le32(prefix, 0)? as u64;
    let pool = le32(prefix, 4)? as u64;
    require!(
        (16..=500_000).contains(&count),
        pool >= 64,
        8u64.checked_add(count.checked_mul(4)?)?.checked_add(pool)? == size,
        le32(prefix, 8) == Some(0)
    );
    let mut prev = 0u64;
    for idx in 1..6usize {
        let off = le32(prefix, 8 + idx * 4)? as u64;
        require!(off >= prev, off < pool, off.is_multiple_of(2));
        prev = off;
    }
    require!(prev > 0);
    Some(Label::AcronisMsgCatalog)
}

/// ASDF: comment header `"#ASDF <version>"`, followed within the first 128 bytes by `#ASDF_STANDARD` or the `%YAML` directive.
pub(crate) fn verify_asdf(prefix: &[u8], size: u64) -> Option<Label> {
    const VERSION: usize = 6; // @6: first digit of the file-format version
    require!(
        size >= MIN_ASDF_SIZE,
        starts(prefix, b"#ASDF "),
        u8_at(prefix, VERSION)?.is_ascii_digit()
    );
    require!(
        scan_range(prefix, 6, 128, b"#ASDF_STANDARD ") || scan_range(prefix, 6, 128, b"%YAML 1.")
    );
    Some(Label::Asdf)
}

/// Bethesda Creation Engine Material file (`.bgsm` / `.bgem`).
pub(crate) fn verify_bethesda_bgsm(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_BETHESDA_BGSM_SIZE,
        prefix.len() >= 24,
        starts(prefix, b"BGSM") || starts(prefix, b"BGEM"),
        (1..=32).contains(&le32(prefix, BGSM_VERSION_OFFSET)?),
        le32(prefix, BGSM_TILE_FLAGS_OFFSET)? <= 15
    );
    Some(Label::BethesdaBgsm)
}

/// BigWorld Technology / Wargaming Packed Binary XML File (`.visual_processed`/`.def`, 2006 BigWorld).
pub(crate) fn verify_bigworld_packed_xml(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_BIGWORLD_PACKED_XML_SIZE,
        prefix.len() >= 12,
        starts(prefix, BIGWORLD_PACKED_XML_MAGIC),
        u8_at(prefix, 4)? == 0x00,
        u8_at(prefix, 5)?.is_ascii_graphic()
    );
    Some(Label::BigworldPackedXml)
}

/// Google BinExport v2 Binary Disassembly Protobuf (.binexport) header (`Meta` submessage + wire framing).
pub(crate) fn verify_binexport2_with_stage(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_BINEXPORT2_SIZE, prefix.len() >= 48, u8_at(prefix, 0)? == 0x0A);
    let (meta_len, mut pos) = read_binexport_varint(prefix, 1)?;
    require!((38..=1024).contains(&meta_len));
    let meta_end = pos.checked_add(meta_len as usize)?;
    require!(meta_end as u64 <= size, meta_end <= prefix.len());

    let mut has_exec_id = false;
    let mut has_arch = false;
    let mut has_ts = false;
    while pos < meta_end {
        let tag = u8_at(prefix, pos)?;
        pos += 1;
        match tag {
            // Meta.executable_name = 1 (wire type 2 = length-delimited UTF-8 string)
            0x0A => {
                let (name_len, next_pos) = read_binexport_varint(prefix, pos)?;
                let name_end = next_pos.checked_add(name_len as usize)?;
                require!(name_len <= 260, name_end <= meta_end);
                let name_bytes = get_slice(prefix, next_pos, name_end)?;
                require!(
                    core::str::from_utf8(name_bytes).is_ok(),
                    !name_bytes.iter().any(|&b| b < 0x20 || b == 0x7F)
                );
                pos = name_end;
            }
            // Meta.executable_id = 2 (wire type 2 = length-delimited MD5/SHA256 hex digest)
            0x12 => {
                let (id_len, next_pos) = read_binexport_varint(prefix, pos)?;
                let id_end = next_pos.checked_add(id_len as usize)?;
                require!((32..=96).contains(&id_len), id_end <= meta_end);
                let id_bytes = get_slice(prefix, next_pos, id_end)?;
                let hex_cnt = id_bytes.iter().filter(|b| b.is_ascii_hexdigit()).count();
                require!(
                    hex_cnt >= 32,
                    id_bytes.iter().all(|&b| b.is_ascii_hexdigit() || matches!(b, b'_' | b'-'))
                );
                has_exec_id = true;
                pos = id_end;
            }
            // Meta.architecture_name = 3 (wire type 2 = length-delimited ASCII architecture name)
            0x1A => {
                let (arch_len, next_pos) = read_binexport_varint(prefix, pos)?;
                let arch_end = next_pos.checked_add(arch_len as usize)?;
                require!((2..=32).contains(&arch_len), arch_end <= meta_end);
                let arch_bytes = get_slice(prefix, next_pos, arch_end)?;
                require!(
                    arch_bytes.iter().all(|&b| (0x20..=0x7E).contains(&b)),
                    arch_bytes.iter().any(|b| b.is_ascii_alphanumeric())
                );
                has_arch = true;
                pos = arch_end;
            }
            // Meta.timestamp = 4 (wire type 0 = varint)
            0x20 => {
                let (_ts, next_pos) = read_binexport_varint(prefix, pos)?;
                require!(next_pos <= meta_end);
                has_ts = true;
                pos = next_pos;
            }
            _ => return None,
        }
    }

    require!(pos == meta_end, has_exec_id, has_arch || has_ts);
    let record_end = if size > meta_end as u64 && prefix.len() > meta_end {
        let next_tag = u8_at(prefix, meta_end)?;
        require!(
            matches!(next_tag, 0x12 | 0x1A | 0x22 | 0x2A | 0x32 | 0x3A | 0x42 | 0x4A)
                || matches!(next_tag, 0x52 | 0x5A | 0x62 | 0x6A | 0x72 | 0x7A | 0x82)
        );
        meta_end + 1
    } else {
        meta_end
    };
    require!(has_non_ascii_text_byte(prefix.get(..record_end)?));
    Some(Label::Binexport2)
}

/// BioWare Aurora / Odyssey / Eclipse Generic File Format container (GFF V3.2, V4.0, V4.1).
///
/// Header layouts:
/// - **GFF V3.2** (Aurora / Odyssey / The Witcher 1):
///   - `@0..4`: 4-char uppercase type tag (`GFF `, `UTI `, `UTC `, `DLG `, `ARE `, `GIT `, `IFO `, `BIC `, etc.)
///   - `@4..8`: `"V3.2"`
///   - `@8..12`: `u32 LE` `struct_offset` (`56` = `0x38`)
///   - `@12..16`: `u32 LE` `struct_count` (`1..=1_000_000`)
/// - **GFF V4.0 / V4.1** (Eclipse / Dragon Age):
///   - `@0..8`: `"GFF V4.0"` or `"GFF V4.1"`
///   - `@8..12`: platform (`"PC  "`, `"X360"`, `"PS3 "`)
///   - `@20..24`: `u32 LE` `struct_count` (`1..=100_000`)
///   - `@24..28`: `u32 LE` `data_offset` (`>= 28`, `< file_size`)
pub(crate) fn verify_bioware_gff(prefix: &[u8], size: u64) -> Option<Label> {
    const V32_TAG: &[u8; 4] = b"V3.2"; // @4..8: GFF v3.2 version
    const V40_MAGIC: &[u8; 8] = b"GFF V4.0"; // @0..8: GFF v4.0 header
    const V41_MAGIC: &[u8; 8] = b"GFF V4.1"; // @0..8: GFF v4.1 header
    const V32_STRUCT_OFF: usize = 8; // @8..12: u32 LE StructOffset == 56
    const V32_STRUCT_CNT: usize = 12; // @12..16: u32 LE StructCount >= 1
    const V4_PLATFORM_OFF: usize = 8; // @8..12: platform FourCC ("PC  " / "X360" / "PS3 ")
    const V4_STRUCT_CNT: usize = 20; // @20..24: u32 LE struct count
    const V4_DATA_OFF: usize = 24; // @24..28: u32 LE data offset

    require!(size >= MIN_BIOWARE_GFF_SIZE, prefix.len() >= 28);
    if eq_at(prefix, 4, V32_TAG) {
        require!(
            size >= 56,
            prefix.len() >= 56,
            get_slice(prefix, 0, 3)?.iter().all(|b| b.is_ascii_uppercase()),
            u8_at(prefix, 3)? == b' ',
            le32(prefix, V32_STRUCT_OFF)? == 56,
            (1..=1_000_000).contains(&le32(prefix, V32_STRUCT_CNT)?)
        );
        return Some(Label::BiowareGff);
    }
    if starts(prefix, V40_MAGIC) || starts(prefix, V41_MAGIC) {
        let plat = get_slice(prefix, V4_PLATFORM_OFF, V4_PLATFORM_OFF + 4)?;
        let data_off = le32(prefix, V4_DATA_OFF)? as u64;
        require!(
            matches!(plat, b"PC  " | b"X360" | b"PS3 "),
            (1..=100_000).contains(&le32(prefix, V4_STRUCT_CNT)?),
            data_off >= 28,
            data_off < size
        );
        return Some(Label::BiowareGff);
    }
    None
}

/// CERN ROOT Binary Data File (`.root`, 1995 CERN ROOT Team).
///
/// Header layout (`TFile::WriteHeader`, all big-endian):
/// - `@0..4`: `"root"` ASCII magic
/// - `@4..8`: `fVersion` (`i32 BE`): `100..=999_999` (32-bit offsets) or `1_000_000..=9_999_999` (`>=2 GiB` 64-bit offsets)
/// - `@8..12`: `fBEGIN` (`u32 BE`): offset of first data `TKey` (`64..=1024`, canonically `100`)
/// - `@12..16` (32-bit) / `@12..20` (64-bit): `fEND` (`>= fBEGIN`)
/// - `@32` (32-bit) / `@40` (64-bit): `fUnits` (`4` for 32-bit offsets, `8` for 64-bit offsets)
/// - `@33..37` (32-bit) / `@41..45` (64-bit): `fCompress` (`0..=999`, compression algorithm * 100 + level)
pub(crate) fn verify_cern_root(prefix: &[u8], size: u64) -> Option<Label> {
    const VERSION_OFF: usize = 4; // @4: fVersion — format version (i32 BE)
    const BEGIN_OFF: usize = 8; // @8: fBEGIN — first TKey byte offset (u32 BE)
    const END_OFF: usize = 12; // @12: fEND — end of written TFile records (u32/u64 BE)
    const UNITS_32_OFF: usize = 32; // @32: fUnits — pointer width in bytes (4 for <2 GiB)
    const COMPRESS_32_OFF: usize = 33; // @33: fCompress — alg * 100 + level (u32 BE)
    const UNITS_64_OFF: usize = 40; // @40: fUnits — pointer width in bytes (8 for >=2 GiB)
    const COMPRESS_64_OFF: usize = 41; // @41: fCompress — alg * 100 + level (u32 BE)
    const LARGE_FILE_VERSION_BASE: u32 = 1_000_000;

    require!(size >= MIN_CERN_ROOT_SIZE, prefix.len() >= 100, starts(prefix, CERN_ROOT_MAGIC),);
    let version = be32(prefix, VERSION_OFF)?;
    let fbegin = be32(prefix, BEGIN_OFF)?;
    require!(
        (100..=9_999_999).contains(&version),
        (64..=1024).contains(&fbegin),
        (fbegin as u64) <= size,
    );
    if version >= LARGE_FILE_VERSION_BASE {
        let fend = be64(prefix, END_OFF)?;
        let funits = u8_at(prefix, UNITS_64_OFF)?;
        let fcompress = be32(prefix, COMPRESS_64_OFF)?;
        require!(funits == 8, fend >= fbegin as u64, fcompress <= 999);
    } else {
        let fend = be32(prefix, END_OFF)? as u64;
        let funits = u8_at(prefix, UNITS_32_OFF)?;
        let fcompress = be32(prefix, COMPRESS_32_OFF)?;
        require!(funits == 4, fend >= fbegin as u64, fcompress <= 999);
    }
    Some(Label::CernRoot)
}

/// CERNLIB ZEBRA FZ Sequential Exchange File (`.fz` / `.xsdst`, 1985 CERN).
///
/// Header layout (32-bit words, big-endian IEEE exchange or little-endian byte-swapped):
/// - `@0..16`: 4 steering sentinel words (`0123CDEF`, `80708070`, `4321ABCD`, `80618061`)
/// - `@16..20`: `NWP` — physical record size in 32-bit words (`8..=0x100000`)
/// - `@24..28`: `NWSTEER` — steering header length in 32-bit words (`4..=NWP`)
pub(crate) fn verify_cern_zebra_fz(prefix: &[u8], size: u64) -> Option<Label> {
    const NWP_OFF: usize = 16; // @16: NWP — physical record words
    const NWSTEER_OFF: usize = 24; // @24: NWSTEER — steering header words

    require!(size >= MIN_CERN_ZEBRA_FZ_SIZE, prefix.len() >= 28);
    let (nwp, nwsteer) = if starts(prefix, CERN_ZEBRA_FZ_STEER_BE) {
        (be32(prefix, NWP_OFF)?, be32(prefix, NWSTEER_OFF)?)
    } else if starts(prefix, CERN_ZEBRA_FZ_STEER_LE) {
        (le32(prefix, NWP_OFF)?, le32(prefix, NWSTEER_OFF)?)
    } else {
        return None;
    };
    require!((8..=0x10_0000).contains(&nwp), (4..=nwp).contains(&nwsteer),);
    Some(Label::CernZebraFz)
}

/// DESY FPACK / BOS Particle Physics Event Stream (`.fpack` / `.bos`, 1985 DESY, Blobel).
///
/// Header layout (32-bit words, native BE or LE detected via `@8..12`):
/// - `@0..4`: `nphys` — physical record length in 32-bit words (`1..=0x100000`)
/// - `@4..8`: `nlog` — logical record length in 32-bit words (`1..=nphys`)
/// - `@8..12`: `0x04030201` byte-order sentinel (`04 03 02 01` BE or `01 02 03 04` LE)
/// - `@12..16`: `nhdr` — header length in 32-bit words (`11..=64`)
/// - `@16..20`: `version` — format version (`1..=200`, canonically `100`)
/// - `@20..28`: 8-byte uppercase ASCII logical record name (e.g. `"RUNEVENT"`)
pub(crate) fn verify_desy_fpack_bos(prefix: &[u8], size: u64) -> Option<Label> {
    const NPHYS_OFF: usize = 0; // @0: nphys — physical record words
    const NLOG_OFF: usize = 4; // @4: nlog — logical record words
    const BYTE_ORDER_OFF: usize = 8; // @8: 0x04030201 endianness sentinel
    const NHDR_OFF: usize = 12; // @12: nhdr — record header word count
    const VERSION_OFF: usize = 16; // @16: version — FPACK format version
    const REC_NAME_OFF: usize = 20; // @20..28: 8-char uppercase ASCII record name
    const FPACK_ORDER_BE: &[u8; 4] = hex2bin!("04 03 02 01");
    const FPACK_ORDER_LE: &[u8; 4] = hex2bin!("01 02 03 04");

    require!(size >= MIN_DESY_FPACK_BOS_SIZE, prefix.len() >= 28);
    let is_be = if eq_at(prefix, BYTE_ORDER_OFF, FPACK_ORDER_BE) {
        true
    } else if eq_at(prefix, BYTE_ORDER_OFF, FPACK_ORDER_LE) {
        false
    } else {
        return None;
    };
    let read_u32 = |off| if is_be { be32(prefix, off) } else { le32(prefix, off) };
    let nphys = read_u32(NPHYS_OFF)?;
    let nlog = read_u32(NLOG_OFF)?;
    let nhdr = read_u32(NHDR_OFF)?;
    let version = read_u32(VERSION_OFF)?;
    let rec_name = get_slice(prefix, REC_NAME_OFF, REC_NAME_OFF + 8)?;
    require!(
        (1..=0x10_0000).contains(&nphys),
        (1..=nphys).contains(&nlog),
        (11..=64).contains(&nhdr),
        (1..=200).contains(&version),
        rec_name[0].is_ascii_uppercase(),
        rec_name.iter().all(|&b| b.is_ascii_uppercase()
            || b.is_ascii_digit()
            || matches!(b, b' ' | b'_' | b'-')),
    );
    Some(Label::DesyFpackBos)
}

/// Blizzard Entertainment Diablo II / Resurrected Character Save (`.d2s`, 2000 Blizzard Entertainment).
pub(crate) fn verify_diablo2_save(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_DIABLO2_SAVE_SIZE,
        prefix.len() >= 16,
        starts(prefix, DIABLO2_SAVE_MAGIC),
        (71..=120).contains(&le32(prefix, 4)?),
        le32(prefix, 8)? as u64 == size
    );
    Some(Label::Diablo2D2s)
}

/// Egret Technology DragonBones Binary Skeleton Data file (`.dbbin`).
pub(crate) fn verify_dragonbones_dbbin(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_DRAGONBONES_DBBIN_SIZE,
        prefix.len() >= 16,
        starts(prefix, DRAGONBONES_MAGIC),
        (1..=4).contains(&be32(prefix, DRAGONBONES_VER_OFFSET)?)
            || (1..=4).contains(&le32(prefix, DRAGONBONES_VER_OFFSET)?),
        le32(prefix, DRAGONBONES_JSON_LEN_OFFSET)? >= 2,
        eq_at(prefix, 12, b"{\"")
    );
    Some(Label::DragonbonesDbbin)
}

/// Adobe Flash Player Local Shared Object (.sol, 2002) Macromedia / Adobe.
pub(crate) fn verify_flash_sol(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_FLASH_SOL_SIZE,
        prefix.len() >= 20,
        starts(prefix, b"\x00\xbf"),
        get_slice(prefix, 6, 16)? == b"TCSO\0\x04\0\0\0\0",
        be16(prefix, 16).unwrap_or(0) >= 1
    );
    require!(has_non_ascii_text_byte(prefix.get(..20)?));
    Some(Label::FlashSol)
}

/// Irrational Games Freedom Force Save Game (`.dat`).
pub(crate) fn verify_freedom_force_save(prefix: &[u8], size: u64) -> Option<Label> {
    const FREEDOM_FORCE_MAGIC: &[u8; 4] = hex2bin!("AD DB DE FA"); // @0..4: 0xFADEDBAD LE
    const BLOCK0_OFFSET: usize = 4; // @4..8: u32 LE header block size (0x400)
    const BLOCK1_OFFSET: usize = 8; // @8..12: u32 LE first block offset (0x400)
    const BLOCK2_OFFSET: usize = 12; // @12..16: u32 LE second block offset (0x400)

    require!(
        size >= MIN_FREEDOM_FORCE_SAVE_SIZE,
        prefix.len() >= 28,
        starts(prefix, FREEDOM_FORCE_MAGIC),
        le32(prefix, BLOCK0_OFFSET) == Some(0x400),
        le32(prefix, BLOCK1_OFFSET) == Some(0x400),
        le32(prefix, BLOCK2_OFFSET) == Some(0x400)
    );
    Some(Label::FreedomForceSav)
}

/// Godot Engine Binary Resource / Scene (`"RSRC"` uncompressed or `"RSCC"` compressed).
///
/// Header layouts:
/// - Uncompressed `"RSRC"`:
///   - `@4..8`: `u32 LE` `big_endian` (`0` or `1`)
///   - `@8..12`: `u32 LE` `use_real64` (`0` or `1`)
///   - `@12..16`: `u32 LE` `ver_major` (`2..=5`)
///   - `@16..20`: `u32 LE` `ver_minor` (`0..=20`)
///   - `@20..24`: `u32 LE` `format_ver` (`1..=10`)
///   - `@24..28`: `u32 LE` `type_str_len` (`1..=128`)
/// - Compressed `"RSCC"`:
///   - `@4..8`: `u32 LE` `compression_mode` (`0..=3`: FastLZ, Deflate, Zstd, Gzip)
///   - `@8..12`: `u32 LE` `block_size` (`4096..=65536`, power of two)
///   - `@12..16`: `u32 LE` `uncompressed_size` (`>= 16`)
pub(crate) fn verify_godot_rsrc(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_GODOT_RSRC_SIZE, prefix.len() >= 28);
    if starts(prefix, GODOT_RSRC_MAGIC) {
        let big_endian = le32(prefix, 4)?; // @4..8: endian flag (0/1)
        let use_real64 = le32(prefix, 8)?; // @8..12: 64-bit float flag (0/1)
        let ver_major = le32(prefix, 12)?; // @12..16: Godot major version (2..=5)
        let ver_minor = le32(prefix, 16)?; // @16..20: Godot minor version (0..=20)
        let fmt_ver = le32(prefix, 20)?; // @20..24: resource format version (1..=10)
        let type_len = le32(prefix, 24)?; // @24..28: NUL-terminated resource class string len
        require!(
            big_endian <= 1,
            use_real64 <= 1,
            (2..=5).contains(&ver_major),
            ver_minor <= 20,
            (1..=10).contains(&fmt_ver),
            (1..=128).contains(&type_len)
        );
        return Some(Label::GodotRsrc);
    }
    if starts(prefix, GODOT_RSCC_MAGIC) {
        let comp_mode = le32(prefix, 4)?; // @4..8: compression algorithm (0..=3)
        let block_size = le32(prefix, 8)?; // @8..12: chunk size (4096..=65536, power of two)
        let uncomp_size = le32(prefix, 12)?; // @12..16: total uncompressed payload length
        require!(
            comp_mode <= 3,
            (4096..=65536).contains(&block_size),
            block_size.is_power_of_two(),
            uncomp_size >= 16
        );
        return Some(Label::GodotRsrc);
    }
    None
}

/// Java KeyStore (`.jks`, `FE ED FE ED`) or JCEKS (`CE CE CE CE`) container.
pub(crate) fn verify_java_keystore(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= 32,
        prefix.len() >= 8,
        starts(prefix, hex2bin!("FE ED FE ED")) || starts(prefix, hex2bin!("CE CE CE CE")),
    );
    Some(Label::Jks)
}

/// Larian Studios LSOF Binary Resource (`.lsf`, Divinity: Original Sin 1/2, Baldur's Gate 3).
///
/// Header layout:
/// - `@0..4`: `"LSOF"` magic
/// - `@4..8`: `u32 LE` `version` (`1..=10`; v7 in Baldur's Gate 3)
pub(crate) fn verify_larian_lsof(prefix: &[u8], size: u64) -> Option<Label> {
    const VERSION_OFF: usize = 4; // @4..8: u32 LE LSOF version (1..=10)

    require!(
        size >= MIN_LARIAN_LSOF_SIZE,
        prefix.len() >= 16,
        starts(prefix, LARIAN_LSOF_MAGIC),
        (1..=10).contains(&le32(prefix, VERSION_OFF)?)
    );
    Some(Label::LarianLsof)
}

/// Linear Collider I/O SIO Event Stream (`.slcio`, 2003 DESY / SLAC iLCSoft).
///
/// SIO record header layout (big-endian):
/// - `@0..4`: `hdr_len` (`u32 BE`, 4-byte aligned, `28..=1024`, `<= size`)
/// - `@4..8`: `0xABADCAFE` record marker (`AB AD CA FE` BE)
/// - `@8..12`: `options` (`u32 BE`, bit 0 = zlib compressed)
/// - `@12..16`: `comp_len` (`u32 BE`)
/// - `@16..20`: `uncomp_len` (`u32 BE`, equals `comp_len` when uncompressed)
/// - `@20..24`: `name_len` (`u32 BE`, exact invariant `24 + ((name_len + 3) & !3) == hdr_len`)
/// - `@24..hdr_len`: printable ASCII record name (e.g. `"LCRunHeader"`) + `0x00` padding
pub(crate) fn verify_lcio_sio(prefix: &[u8], size: u64) -> Option<Label> {
    const HDR_LEN_OFF: usize = 0; // @0: hdr_len — SIO header length (u32 BE)
    const SIO_MAGIC_OFF: usize = 4; // @4: SIO record magic 0xABADCAFE BE
    const OPTIONS_OFF: usize = 8; // @8: options — flags (bit 0 = compressed)
    const COMP_LEN_OFF: usize = 12; // @12: comp_len — stored payload byte length
    const UNCOMP_LEN_OFF: usize = 16; // @16: uncomp_len — uncompressed payload byte length
    const NAME_LEN_OFF: usize = 20; // @20: name_len — record name byte length
    const NAME_OFF: usize = 24; // @24: ASCII record name + NUL pad to 4B boundary
    const SIO_RECORD_MAGIC_BE: &[u8; 4] = hex2bin!("AB AD CA FE");

    require!(
        size >= MIN_LCIO_SIO_SIZE,
        prefix.len() >= 28,
        eq_at(prefix, SIO_MAGIC_OFF, SIO_RECORD_MAGIC_BE),
    );
    let hdr_len = be32(prefix, HDR_LEN_OFF)?;
    let options = be32(prefix, OPTIONS_OFF)?;
    let comp_len = be32(prefix, COMP_LEN_OFF)?;
    let uncomp_len = be32(prefix, UNCOMP_LEN_OFF)?;
    let name_len = be32(prefix, NAME_LEN_OFF)?;
    require!(
        (28..=1024).contains(&hdr_len),
        (hdr_len & 3) == 0,
        (hdr_len as u64) <= size,
        prefix.len() >= hdr_len as usize,
        (1..=256).contains(&name_len),
    );
    let expected_hdr_len = 24u32.checked_add((name_len + 3) & !3)?;
    require!(expected_hdr_len == hdr_len);
    if (options & 1) == 0 {
        require!(comp_len == uncomp_len);
    }
    let name_end = NAME_OFF + name_len as usize;
    require!(
        get_slice(prefix, NAME_OFF, name_end)?.iter().all(|&b| b.is_ascii_graphic()),
        get_slice(prefix, name_end, hdr_len as usize)?.iter().all(|&b| b == 0),
    );
    Some(Label::LcioSio)
}

/// Foxglove Robotics MCAP Container (`.mcap`, `application/vnd.mcap`).
///
/// Header layout:
/// - `@0..5`: `\x89MCAP` (`89 4D 43 41 50`)
/// - `@5`: Major specification version ASCII byte (`'0'` = `0x30` or `'1'` = `0x31`)
/// - `@6..8`: `\r\n` (`0D 0A`)
/// - `@8`: First record opcode (`0x01` = Header record)
/// - `@9..17`: `u64 LE` Header record byte length (`8..=65536`)
/// - `@17..21`: `u32 LE` `profile` string byte length (`profile_len`), followed by `profile` bytes and `u32 LE` `library_len` + `library` bytes (`8 + profile_len + library_len == header_record_len`).
pub(crate) fn verify_mcap(prefix: &[u8], size: u64) -> Option<Label> {
    const MCAP_PREFIX: &[u8; 5] = b"\x89MCAP"; // @0..5: 0x89 + "MCAP"
    const VER_OFFSET: usize = 5; // @5: major version ASCII digit ('0' or '1')
    const CRLF_OFFSET: usize = 6; // @6..8: "\r\n"
    const HDR_OPCODE_OFFSET: usize = 8; // @8: Header record opcode (0x01)
    const HDR_LEN_OFFSET: usize = 9; // @9..17: u64 LE Header record length
    const PROFILE_LEN_OFFSET: usize = 17; // @17..21: u32 LE profile string length

    let hdr_len = le64(prefix, HDR_LEN_OFFSET)?;
    let profile_len = le32(prefix, PROFILE_LEN_OFFSET)? as u64;
    require!(
        size >= MIN_MCAP_SIZE,
        prefix.len() >= 25,
        starts(prefix, MCAP_PREFIX),
        matches!(u8_at(prefix, VER_OFFSET)?, b'0' | b'1'),
        eq_at(prefix, CRLF_OFFSET, b"\r\n"),
        u8_at(prefix, HDR_OPCODE_OFFSET) == Some(0x01),
        (8..=65536).contains(&hdr_len),
        size >= 17 + hdr_len,
        profile_len + 8 <= hdr_len
    );
    Some(Label::Mcap)
}

/// MegaZeux Game Creation System World File (`.mzx`, 1994 Alexis Janson / DigitalMZX).
///
/// Header layout:
/// - `@0..25`: 25-byte world title
/// - `@25`: protection method (`0x00` = unprotected; `1..=3` = 16-byte password block at `@26..42`)
/// - Unprotected (`@25 == 0`): `@26..29 = "M\x02<minor>"` (with `"PK\x03\x04"` at `@29..33` for v2.90+ ZIP worlds)
///   or `"MZX\0\0\0\0"` at `@26..33`
/// - Protected (`@25 in 1..=3`): `"MZX\0\0\0\0"` at `@42..49` (or `@41..48` on legacy 15B password block)
pub(crate) fn verify_megazeux_world(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_MEGAZEUX_WORLD_SIZE, prefix.len() >= 49);
    let prot = u8_at(prefix, 25)?;
    if prot == 0 {
        if u8_at(prefix, 26)? == b'M'
            && u8_at(prefix, 27)? == 0x02
            && (0x5A..=0x70).contains(&u8_at(prefix, 28)?)
            && eq_at(prefix, 29, b"PK\x03\x04")
        {
            return Some(Label::MegazeuxWorld);
        }
        if eq_at(prefix, 26, b"MZX\0") && eq_at(prefix, 30, b"\0\0\0") {
            return Some(Label::MegazeuxWorld);
        }
    } else if (1..=3).contains(&prot) {
        if eq_at(prefix, 42, b"MZX\0\0\0\0") {
            return Some(Label::MegazeuxWorld);
        }
        if eq_at(prefix, 41, b"MZX\0\0\0\0") {
            return Some(Label::MegazeuxWorld);
        }
    }
    None
}

/// Microsoft Fluid Framework / Loop Prague Binary Serialization (`.loop` / `.fluid`) file.
pub(crate) fn verify_microsoft_prague_binary(prefix: &[u8], size: u64) -> Option<Label> {
    const PRAGUE_GUID_MAGIC: &[u8; 16] =
        hex2bin!("57 8F 1C D0 D0 67 34 4B 9F FB 4F 5A 48 06 9E C5"); // @0..16: 16-byte Prague container GUID
    const PRAGUE_NAME_OFFSET: usize = 30; // @30..64: UTF-16LE "Microsoft.Prague "
    const PRAGUE_NAME_UTF16LE: &[u8; 34] = b"M\0i\0c\0r\0o\0s\0o\0f\0t\0.\0P\0r\0a\0g\0u\0e\0 \0";

    require!(
        size >= MIN_MS_PRAGUE_SIZE,
        prefix.len() >= 65,
        starts(prefix, PRAGUE_GUID_MAGIC),
        eq_at(prefix, PRAGUE_NAME_OFFSET, PRAGUE_NAME_UTF16LE)
    );
    Some(Label::MsPrague)
}

/// Minitaur Robotics Latency Telemetry Protobuf Log (.proto).
pub(crate) fn verify_minitaur_log_proto(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MINITAUR_LOG_PROTO_SIZE,
        prefix.len() >= 32,
        get_slice(prefix, 0, 13)? == hex2bin!("0A 90 03 0A 05 10 80 9B EE 02 12 1B 09"),
        get_slice(prefix, 19, 22)? == hex2bin!("82 BF 11"),
        get_slice(prefix, 28, 31)? == hex2bin!("10 BF 19")
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));
    Some(Label::MinitaurLogProto)
}

/// Microsoft Flight Simulator SimPropBinary Compiled XML (`.spb`, FSX/P3D/MSFS, 2006 Microsoft).
pub(crate) fn verify_msfs_spb(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MSFS_SPB_SIZE,
        prefix.len() >= 20,
        starts(prefix, hex2bin!("AC EB 14 00 00 00 01 00 00 00 00 00 00 00 00 02 00 00 01 00"))
    );
    Some(Label::MsfsSpb)
}

/// Nintendo EPD Binary Parameter Archive (`.bxml`, `.baatarc`, `.bgparamlist`, `.aamp`).
///
/// Header layout:
/// - `@0..4`: `"AAMP"` magic
/// - `@4..8`: `u32 LE` `version` (`1..=2`)
/// - `@8..12`: `u32 LE` `flags` (`<= 0xFF`)
/// - `@12..16`: `u32 LE` `file_size` (`>= 32`)
pub(crate) fn verify_nintendo_aamp(prefix: &[u8], size: u64) -> Option<Label> {
    const VERSION_OFF: usize = 4; // @4..8: u32 LE format version (1 or 2)
    const FLAGS_OFF: usize = 8; // @8..12: u32 LE bitflags (<= 0xFF)
    const FILE_SIZE_OFF: usize = 12; // @12..16: u32 LE declared file size

    require!(
        size >= MIN_NINTENDO_AAMP_SIZE,
        prefix.len() >= 16,
        starts(prefix, NINTENDO_AAMP_MAGIC),
        (1..=2).contains(&le32(prefix, VERSION_OFF)?),
        le32(prefix, FLAGS_OFF)? <= 0xFF,
        le32(prefix, FILE_SIZE_OFF)? >= 32
    );
    Some(Label::NintendoAamp)
}

/// Nintendo Binary YAML file (`.byml`, `.byaml`, `.bgyml`, Tier 2 `magic16`).
///
/// Header layout:
/// - `@0..2`: `"YB"` (little-endian: Switch / 3DS) or `"BY"` (big-endian: Wii U)
/// - `@2..4`: `u16` `version` (`1..=10`; v1 has a 20-byte header with path table offset, v2+ has a 16-byte header)
/// - `@4..8`: `u32` `hash_key_table_offset` (`0` or `>= hdr_len`, pointing to `0xC2` `StringTable`)
/// - `@8..12`: `u32` `string_table_offset` (`0` or `>= hdr_len`, pointing to `0xC2` `StringTable`)
/// - `@12..16` (v2+) or `@16..20` (v1): `u32` `root_node_offset` (`< file_size`)
pub(crate) fn verify_nintendo_byml(prefix: &[u8], size: u64) -> Option<Label> {
    const NODE_ARRAY: u8 = 0xC0;
    const NODE_DICT: u8 = 0xC1;
    const NODE_STRING_TABLE: u8 = 0xC2;

    require!(size >= MIN_NINTENDO_BYML_SIZE, prefix.len() >= 21);
    let is_le = if starts(prefix, NINTENDO_BYML_LE_MAGIC) {
        true
    } else if starts(prefix, NINTENDO_BYML_BE_MAGIC) {
        false
    } else {
        return None;
    };
    let rd16 = |off| if is_le { le16(prefix, off) } else { be16(prefix, off) };
    let rd32 = |off| if is_le { le32(prefix, off) } else { be32(prefix, off) };

    let version = rd16(2)?;
    require!((1..=10).contains(&version));
    let hdr_len: usize = if version == 1 { 20 } else { 16 };
    let hkt_off = rd32(4)? as usize;
    let st_off = rd32(8)? as usize;
    let root_off = rd32(hdr_len - 4)? as usize;

    require!((hkt_off as u64) < size, (st_off as u64) < size, (root_off as u64) < size);
    // At least one table/root must start immediately after the header with a valid BYML container node tag.
    let mut verified_node_off: Option<usize> = None;
    if hkt_off == hdr_len {
        require!(u8_at(prefix, hkt_off)? == NODE_STRING_TABLE);
        verified_node_off = Some(hkt_off);
    } else if st_off == hdr_len {
        require!(u8_at(prefix, st_off)? == NODE_STRING_TABLE);
        verified_node_off = Some(st_off);
    } else if root_off == hdr_len {
        require!(matches!(u8_at(prefix, root_off)?, NODE_ARRAY | NODE_DICT));
        verified_node_off = Some(root_off);
    }
    let _node_off = verified_node_off?;
    if st_off != 0 && st_off < prefix.len() {
        require!(u8_at(prefix, st_off)? == NODE_STRING_TABLE);
    }
    Some(Label::NintendoByml)
}

/// Hello Games No Man's Sky MBIN Serialized Data file (`.mbin`).
pub(crate) fn verify_nms_mbin(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_NMS_MBIN_SIZE,
        prefix.len() >= 32,
        starts(prefix, NMS_MBIN_MAGIC),
        (2000..=4000).contains(&le32(prefix, NMS_VERSION_OFFSET)?),
        le64(prefix, NMS_HASH_OFFSET)? != 0
    );
    Some(Label::NmsMbin)
}

/// OpenRCT2 Park Save / Scenario Container (`.park`, `"PARK"`, 2021 OpenRCT2).
pub(crate) fn verify_openrct2_park(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_OPENRCT2_PARK_SIZE, prefix.len() >= 32, starts(prefix, b"PARK"));
    let target_ver = le32(prefix, 4)?;
    let min_ver = le32(prefix, 8)?;
    let num_chunks = le32(prefix, 12)?;
    let compression = le32(prefix, 24)?;
    require!(
        (1..=1000).contains(&target_ver),
        (1..=target_ver).contains(&min_ver),
        (1..=1024).contains(&num_chunks),
        compression <= 4
    );
    Some(Label::Openrct2Park)
}

/// Pharo Smalltalk FUEL Object Serializer Stream (.fuel, 2011) RMoD INRIA.
pub(crate) fn verify_pharo_fuel(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PHARO_FUEL_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"FUEL"),
        get_slice(prefix, 6, 10)? == b"\0\0\0\0",
        contains(prefix.get(16..prefix.len().min(64))?, b"FLGlobal")
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));
    Some(Label::PharoFuel)
}

/// Kuju / Dovetail Games RailWorks / Train Simulator SERZ Binary XML file (`.bin` / `.geopcdx` / `.tgpcdx`).
pub(crate) fn verify_railworks_serz(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_RAILWORKS_SERZ_SIZE,
        prefix.len() >= 20,
        starts(prefix, RAILWORKS_SERZ_MAGIC)
    );
    Some(Label::RailworksSerz)
}

/// Capcom RE Engine Material Definition file (`.mdf2`).
pub(crate) fn verify_re_engine_mdf2(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_RE_ENGINE_MDF2_SIZE,
        prefix.len() >= 16,
        starts(prefix, RE_MDF2_MAGIC),
        le16(prefix, RE_MDF2_COUNT_OFFSET)? >= 1,
        le64(prefix, RE_MDF2_RESERVED_OFFSET) == Some(0)
    );
    Some(Label::ReEngineMdf2)
}

/// CD Projekt RED REDengine 2/3/4 CR2W Binary Resource (`.w2ent`, `.w2mesh`, `.w2scene`, `.xbm`, `.ent`).
///
/// Header layout:
/// - `@0..4`: `"CR2W"` magic
/// - `@4..8`: `u32 LE` `version` (`100..=250`, e.g., `162`–`163` Witcher 3, `195` Cyberpunk 2077)
/// - `@8..12`: `u32 LE` `flags` (`0`)
pub(crate) fn verify_redengine_cr2w(prefix: &[u8], size: u64) -> Option<Label> {
    const VERSION_OFF: usize = 4; // @4..8: u32 LE REDengine CR2W version (100..=250)
    const FLAGS_OFF: usize = 8; // @8..12: u32 LE flags (0)

    require!(
        size >= MIN_REDENGINE_CR2W_SIZE,
        prefix.len() >= 16,
        starts(prefix, REDENGINE_CR2W_MAGIC),
        (100..=250).contains(&le32(prefix, VERSION_OFF)?),
        le32(prefix, FLAGS_OFF)? == 0
    );
    Some(Label::WitcherCr2w)
}

/// Google Riegeli Records Structured Container (.riegeli).
pub(crate) fn verify_riegeli(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_RIEGELI_SIZE && prefix.len() >= 32 && starts(prefix, RIEGELI_SIGNATURE_32)
    );
    Some(Label::Riegeli)
}

/// SCS Software Encrypted (`"ScsC"`) or Binary (`"BSII"`) SII Unit File (`.sii`, 2012 SCS Software).
pub(crate) fn verify_scs_sii_encrypted(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_SCS_SII_ENCRYPTED_SIZE, prefix.len() >= 16);
    if starts(prefix, b"ScsC") {
        require!(
            size >= 72,
            prefix.len() >= 56,
            (size - 56).is_multiple_of(16),
            le32(prefix, 52)? > 0,
            has_non_ascii_text_byte(prefix.get(..56)?)
        );
        return Some(Label::ScsSiiEnc);
    }
    if starts(prefix, b"BSII") {
        require!((1..=3).contains(&le32(prefix, 4)?), has_non_ascii_text_byte(prefix.get(..16)?));
        return Some(Label::ScsSiiEnc);
    }
    None
}

/// Stockfish / Shogi NNUE Neural Network Weights file (`.nnue`).
pub(crate) fn verify_stockfish_nnue(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_STOCKFISH_NNUE_SIZE,
        prefix.len() >= 20,
        starts(prefix, STOCKFISH_NNUE_MAGIC),
        le32(prefix, NNUE_HASH_OFFSET)? != 0
    );
    let desc_len = le32(prefix, NNUE_DESC_LEN_OFFSET)? as usize;
    require!(
        (1..=512).contains(&desc_len),
        get_slice(prefix, 12, 12 + desc_len.min(8))?.iter().all(|&b| (0x20..=0x7E).contains(&b))
    );
    Some(Label::StockfishNnue)
}

/// TensorFlow TFRecord Dataset (`.tfrecord`).
pub(crate) fn verify_tensorflow_record(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_TFRECORD_SIZE, prefix.len() >= 16);
    let rec_len = le64(prefix, 0)?;
    require!((1..=size.saturating_sub(16)).contains(&rec_len));
    let expected_crc = masked_crc32c_8bytes(get_slice(prefix, 0, 8)?);
    require!(le32(prefix, 8) == Some(expected_crc));
    Some(Label::Tfrecord)
}

/// Re-Logic Terraria Binary World/Map Save (`.wld`, `"relogic"` at `@4`, 2011 Re-Logic).
pub(crate) fn verify_terraria_world(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_TERRARIA_WORLD_SIZE,
        prefix.len() >= 16,
        (80..=1000).contains(&le32(prefix, 0)?),
        eq_at(prefix, 4, b"relogic"),
        (1..=3).contains(&u8_at(prefix, 11)?)
    );
    Some(Label::TerrariaRelogic)
}

/// Disney / Walt Disney Imagineering Ultimate Ride Text Strings (`.tsf`) file.
pub(crate) fn verify_ultimate_ride_text(prefix: &[u8], size: u64) -> Option<Label> {
    const TWFB_TWTS_MAGIC: &[u8; 16] = bytes!(b"TWFB", le32_bytes!(2), b"TWTS", le32_bytes!(0x20)); // @0..16: "TWFB" + v2 + "TWTS" + 0x20

    require!(
        size >= MIN_ULTIMATE_RIDE_TSF_SIZE,
        prefix.len() >= 20,
        starts(prefix, TWFB_TWTS_MAGIC)
    );
    Some(Label::TwfbTsf)
}

/// Unity Engine Addressables Binary Content Catalog (`catalog.bin`, 2023 Unity Technologies).
///
/// Header layout (`ContentCatalogData` binary format, little-endian):
/// - `@0..4`: `u32 LE` `magic` (`0x0DE38942` = `42 89 E3 0D`)
/// - `@4..8`: `u32 LE` `version` (`1` or `2`)
/// - `@8..12`: `u32 LE` `keys_offset` (`0x20` for v1, `0x24` for v2)
/// - `@12..16`: `u32 LE` `id_offset` (`>= keys_offset`)
/// - `@16..20`: `u32 LE` `instance_provider_offset` (`>= keys_offset`)
/// - `@20..24`: `u32 LE` `scene_provider_offset` (`>= keys_offset`)
/// - `@24..28`: `u32 LE` `init_objects_array_offset` (`>= keys_offset`)
pub(crate) fn verify_unity_addressables_catalog(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_UNITY_ADDRESSABLES_CATALOG_SIZE,
        prefix.len() >= 32,
        starts(prefix, UNITY_ADDRESSABLES_CATALOG_MAGIC)
    );
    let version = le32(prefix, 4)?;
    let keys_off = le32(prefix, 8)?;
    let id_off = le32(prefix, 12)?;
    let inst_off = le32(prefix, 16)?;
    let scene_off = le32(prefix, 20)?;
    let init_off = le32(prefix, 24)?;
    require!(
        matches!(version, 1 | 2),
        matches!(keys_off, 0x20 | 0x24),
        id_off >= keys_off,
        inst_off >= keys_off,
        scene_off >= keys_off,
        init_off >= keys_off
    );
    Some(Label::UnityAddressablesCatalog)
}

/// Unreal Engine Reflection Property Mapping File (`.usmap`, `0x30C4` LE, 2020 FModel / CUE4Parse).
pub(crate) fn verify_unreal_usmap(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_UNREAL_USMAP_SIZE,
        prefix.len() >= 16,
        starts(prefix, UNREAL_USMAP_MAGIC_2)
    );
    let ver = u8_at(prefix, 2)?;
    require!(ver <= 4);
    let hdr_off = if ver == 0 {
        3usize
    } else if le32(prefix, 3) == Some(0) && u8_at(prefix, 7)? <= 3 {
        7usize
    } else if u8_at(prefix, 3)? == 0 && u8_at(prefix, 4)? <= 3 {
        4usize
    } else if le32(prefix, 3) == Some(1) && prefix.len() >= 24 {
        let custom_cnt = le32(prefix, 15)? as usize;
        require!(custom_cnt <= 256);
        let o = 19 + custom_cnt * 20;
        require!(o + 9 <= prefix.len());
        o
    } else {
        return None;
    };
    let comp_method = u8_at(prefix, hdr_off)?;
    let comp_sz = le32(prefix, hdr_off + 1)? as u64;
    let decomp_sz = le32(prefix, hdr_off + 5)? as u64;
    let payload_off = (hdr_off + 9) as u64;
    require!(
        comp_method <= 3,
        comp_sz >= 8,
        decomp_sz >= comp_sz,
        comp_method != 0 || comp_sz == decomp_sz,
        payload_off.saturating_add(comp_sz) == size
    );
    Some(Label::UnrealUsmap)
}

/// Microsoft Windows Game Explorer Rich Saved Game (`"RGMH"` / `"HMGR"`, `RICH_GAME_MEDIA_HEADER`).
pub(crate) fn verify_windows_rich_saved_game(prefix: &[u8], size: u64) -> Option<Label> {
    const RGMH_MAGIC: &[u8; 4] = b"RGMH"; // @0..4: 0x484D4752 LE ("RGMH")
    const HMGR_MAGIC: &[u8; 4] = b"HMGR"; // @0..4: 0x52474D48 LE ("HMGR")
    const HDR_VERSION_OFFSET: usize = 4; // @4..8: u32 LE dwHeaderVersion == 1
    const HDR_SIZE_OFFSET: usize = 8; // @8..12: u32 LE dwHeaderSize == 0x2028 (8232 bytes)
    const EXPECTED_HDR_SIZE: u32 = 0x2028; // sizeof(RICH_GAME_MEDIA_HEADER)

    require!(
        size >= MIN_WINDOWS_RGMH_SAVE_SIZE,
        prefix.len() >= 24,
        starts(prefix, RGMH_MAGIC) || starts(prefix, HMGR_MAGIC),
        le32(prefix, HDR_VERSION_OFFSET) == Some(1),
        le32(prefix, HDR_SIZE_OFFSET) == Some(EXPECTED_HDR_SIZE)
    );
    Some(Label::RgmhSave)
}
