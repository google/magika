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

//! Tier 2 (`magic16`): 2-byte magic dispatch table at offset 0.

use crate::ContentType as Label;
use super::*;

/// Tier 2 (`magic16`): dispatches on the 2-byte key at offset 0.
#[inline]
pub(crate) fn probe(key: &[u8; 2], prefix: &[u8], ctx: &Ctx<'_>) -> Option<Label> {
    let size = ctx.size;
    match key {
        b"MM" => three_dsm(prefix), // Autodesk 3ds Max M3DMAGIC (1990) Autodesk
        b"BZ" => bzip(prefix), // bzip2 Compressed Stream (1996) Julian Seward
        &[0x1F, 0x8B] => gzip(prefix), // RFC 1952 GZIP Member (1992) Jean-loup Gailly
        b"P1" // Netpbm Portable BitMap ASCII (1988) Jef Poskanzer
        | b"P2" // Netpbm Portable GrayMap ASCII (1988) Jef Poskanzer
        | b"P3" // Netpbm Portable PixMap ASCII (1988) Jef Poskanzer
        | b"P4" // Netpbm Portable BitMap Binary (1988) Jef Poskanzer
        | b"P5" // Netpbm Portable GrayMap Binary (1988) Jef Poskanzer
        | b"P6" => pbm(prefix), // Netpbm Portable PixMap Binary (1988) Jef Poskanzer
        b"FW" // Macromedia/Adobe Flash Uncompressed FWS (1996) FutureWave / Macromedia
        | b"CW" // Macromedia/Adobe Flash Zlib-Compressed CWS (2002) Macromedia
        | b"ZW" => swf(prefix), // Adobe Flash LZMA-Compressed ZWS (2012) Adobe
        &[0x01, 0xF7] // IBM AIX XCOFF 64-bit Object (1998) IBM
        | &[0x01, 0xDF] => xcoff(prefix), // IBM AIX XCOFF 32-bit Object (1990) IBM
        b"BM" => bmp(prefix), // Microsoft Windows / OS/2 Bitmap (1987) Microsoft / IBM
        &[0x1F, 0x9D] => unixcompress(prefix), // Unix compress LZW Stream (1985) Spencer Thomas
        &[0x00, 0x00] => degas_or_coff(prefix, ctx), // Atari ST DEGAS Low-Res Screen Dump / COFF BigObj
        &[0x00, 0x01] // Atari ST DEGAS Medium-Res Screen Dump (1985) Batteries Included
        | &[0x00, 0x02] => degas(prefix, ctx), // Atari ST DEGAS High-Res Screen Dump (1985) Batteries Included
        &[0x00, 0x60] => koala_uncompressed(prefix, ctx), // Commodore 64 Uncompressed Koala Painter Image (1983) Koala Technologies
        MZ_MAGIC // MS-DOS MZ / Windows PE / AutoIt Executable (1981) Mark Zbikowski / Microsoft
        | ZM_MAGIC => pe_or_dos_mz(prefix, ctx), // Legacy DOS Linker ZM Executable (1981) Microsoft / IBM
        &[0x78, 0x01] // RFC 1950 ZLIB Stream No/Low Compression (1996) Jean-loup Gailly / Mark Adler
        | &[0x78, 0x5E] // RFC 1950 ZLIB Stream Fast Compression (1996) Jean-loup Gailly / Mark Adler
        | &[0x78, 0x9C] // RFC 1950 ZLIB Stream Default Compression (1996) Jean-loup Gailly / Mark Adler
        | &[0x78, 0xDA] => zlibstream(prefix, size), // RFC 1950 ZLIB Stream Best Compression (1996) Jean-loup Gailly / Mark Adler
        &[0x80, 0x02] // Python Pickle Protocol 2 (2003) Python Software Foundation
        | &[0x80, 0x03] // Python Pickle Protocol 3 (2008) Python Software Foundation
        | &[0x80, 0x04] // Python Pickle Protocol 4 (2014) Python Software Foundation
        | &[0x80, 0x05] => pickle_binary(prefix, size), // Python Pickle Protocol 5 (2019) Python Software Foundation
        &[0x08, 0x01] // ONNX ModelProto IR v1 (2017) Linux Foundation
        | &[0x08, 0x02] // ONNX ModelProto IR v2 (2017) Linux Foundation
        | &[0x08, 0x03] // ONNX ModelProto IR v3 (2018) Linux Foundation
        | &[0x08, 0x04] // ONNX ModelProto IR v4 (2019) Linux Foundation
        | &[0x08, 0x05] // ONNX ModelProto IR v5 (2019) Linux Foundation
        | &[0x08, 0x06] // ONNX ModelProto IR v6 (2019) Linux Foundation
        | &[0x08, 0x07] // ONNX ModelProto IR v7 (2020) Linux Foundation
        | &[0x08, 0x08] // ONNX ModelProto IR v8 (2021) Linux Foundation
        | &[0x08, 0x09] // ONNX ModelProto IR v9 (2023) Linux Foundation
        | &[0x08, 0x0A] // ONNX ModelProto IR v10 (2024) Linux Foundation
        | &[0x08, 0x0B] // ONNX ModelProto IR v11 (2024) Linux Foundation
        | &[0x08, 0x0C] // ONNX ModelProto IR v12 (2024) Linux Foundation
        | &[0x08, 0x0D] // ONNX ModelProto IR v13 (2024) Linux Foundation
        | &[0x08, 0x0E] // ONNX ModelProto IR v14 (2024) Linux Foundation
        | &[0x08, 0x0F] => onnx(prefix, size), // ONNX ModelProto IR v15 (2024) Linux Foundation
        &[0x4C, 0x01] // COFF i386 Object File (1993) Microsoft
        | &[0x64, 0x86] // COFF x86-64 (AMD64) Object File (2003) Microsoft
        | &[0x64, 0xAA] // COFF ARM64 Object File (2012) Microsoft
        | &[0x41, 0xA6] // COFF ARM64EC Object File (2021) Microsoft
        | &[0x42, 0xA6] // COFF ARM64X Object File (2021) Microsoft
        | &[0x4E, 0xA6] // COFF ARM64X Alternate Object File (2021) Microsoft
        | b"d:" // COFF CHPE x86 (0x3A64) Object File (2017) Microsoft
        | &[0xC0, 0x01] // COFF ARM Little-Endian Object File (1998) Microsoft
        | &[0xC4, 0x01] => coff(prefix, size), // COFF ARM Thumb-2 (ARMNT) Object File (2011) Microsoft
        &[0x9B, 0xA5] // Microsoft Word for Windows 1.x Document (1989) Microsoft
        | &[0xDB, 0xA5] // Microsoft Word for Windows 2.0 Document (1991) Microsoft
        | &[0xDC, 0xA5] => word_win12(prefix, size), // Microsoft Word for Windows 2.0c/6.0 Document (1993) Microsoft
        &[0x81, 0xFC] // UPX / COMPACT MS-DOS .COM Stub cmp sp,imm16 (1990) Markus Oberhumer / Walter Peichl
        | &[0xEB, 0x0A] => dos_com(prefix, size), // ICE MS-DOS .COM Stub jmp +0x0A (1990) MichaelL
        &[0x09, 0x00] // Microsoft Excel BIFF2 Worksheet (1987) Microsoft
        | &[0x09, 0x02] // Microsoft Excel BIFF3 Worksheet (1990) Microsoft
        | &[0x09, 0x04] // Microsoft Excel BIFF4 Worksheet (1992) Microsoft
        | &[0x09, 0x08] => xls_biff(prefix, size), // Microsoft Excel BIFF5/BIFF8 Workbook Stream (1993) Microsoft
        &[0x30, 0x80] // ASN.1 BER Indefinite-Length SEQUENCE (1988) ITU-T
        | &[0x30, 0x81] // ASN.1 DER 1-Byte Length SEQUENCE (1988) ITU-T
        | &[0x30, 0x82] // ASN.1 DER 2-Byte Length SEQUENCE (1988) ITU-T
        | &[0x30, 0x83] // ASN.1 DER 3-Byte Length SEQUENCE (1988) ITU-T
        | &[0x30, 0x84] => verify_asn1_der(prefix, size), // ASN.1 DER 4-Byte Length SEQUENCE (1988) ITU-T
        &[0x60, 0xEA] => arj(prefix, size), // Robert K. Jung ARJ Archive (1991) ARJ Software
        &[0x01, 0x00] => firebird(prefix, size), // Firebird / InterBase ODS Database Page (1985) Borland / Firebird
        &[0xD9, 0xD9] // RFC 8949 Self-Described CBOR Tag 55799 (2013) IETF
        | &[0xD8, 0x37] => cbor(prefix, size), // CBOR Tag 55 Header (2013) IETF
        &[0x03, 0x00] => oracle_dmp_text(prefix, size), // Oracle Binary Export Dump (.dmp) (1988) Oracle
        b"S0" // Motorola S-Record Header Record (1975) Motorola
        | b"S1" // Motorola S-Record 16-bit Data Record (1975) Motorola
        | b"S2" // Motorola S-Record 24-bit Data Record (1975) Motorola
        | b"S3" => srecord(prefix, size), // Motorola S-Record 32-bit Data Record (1975) Motorola
        b"*1" // Redis AOF RESP 1-Element Command Array (2009) Salvatore Sanfilippo
        | b"*2" // Redis AOF RESP 2-Element Command Array (2009) Salvatore Sanfilippo
        | b"*3" => redis_aof_resp(prefix, size), // Redis AOF RESP 3-Element Command Array (2009) Salvatore Sanfilippo
        _ => None,
    }
}
