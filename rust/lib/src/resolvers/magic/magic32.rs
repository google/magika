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

//! Tier 1 (`magic32`): 4-byte magic dispatch table at offset 0.

use super::*;
use crate::ContentType as Label;

/// Tier 1 (`magic32`): dispatches on the 4-byte key at offset 0.
#[inline]
pub(crate) fn probe(key: &[u8; 4], prefix: &[u8], ctx: &Ctx<'_>) -> Option<Label> {
    let size = ctx.size;
    match key {
        // =====================================================================
        // Section 1: Pure Printable ASCII (0x20..=0x7E)
        // =====================================================================
        b"3DSX" => threedsx(prefix), // Nintendo 3DS Homebrew Executable (2014) fincs / devkitPro
        b"3D G" => rhinoceros(prefix, size), // Rhinoceros 3D Model (.3dm) (1998) Robert McNeel & Associates
        b"2IMG" => dmg_header(prefix, size), // Apple II Universal Disk Image (1995) Apple
        b"bpli" => applebplist(prefix), // Apple Binary Property List (2002) Apple
        b".snd" // Sun/NeXT AU Audio Big-Endian (1989) Sun Microsystems / NeXT
        | b"dns." => au(prefix), // DEC AU Audio Little-Endian (1989) DEC
        RIFF_MAGIC // Resource Interchange File Format LE (1991) Microsoft / IBM
        | RIFX_MAGIC // Resource Interchange File Format BE (1991) Microsoft / IBM
        | b"RF64" => riff_or_webp(prefix, size), // EBU RF64 64-bit RIFF Audio (2006) EBU
        IFF_FOR1_MAGIC => beam(prefix), // Erlang BEAM Module IFF Container (1997) Ericsson
        IFF_FOR4_MAGIC // Autodesk Maya IFF 32-bit FOR4 Container (1998) Alias|Wavefront / Autodesk
        | IFF_FOR8_MAGIC => maya_binary(prefix, size), // Autodesk Maya IFF 64-bit FOR8 Container (2013) Autodesk
        b"//Ma" => maya_ascii(prefix, size), // Autodesk Maya ASCII Scene (.ma) (1998) Alias|Wavefront / Autodesk
        b"BLEN" => blend(prefix), // Blender 3D Scene File (1994) NaN / Blender Foundation
        b"CRAM" => cram(prefix), // CRAM Genomic Alignment Archive (2011) EBI / GA4GH
        b"AC10" => dwg(prefix), // AutoCAD DWG Drawing (1982) Autodesk
        b"Kayd" // Autodesk FBX Binary Scene (2006) Kaydara / Autodesk
        | b"; FB" => fbx(prefix), // Autodesk FBX ASCII Scene (2006) Kaydara / Autodesk
        b"SIMP" => fits(prefix), // FITS Flexible Image Transport System (1981) NASA / IAU
        b"SQLi" => sqlite_or_geopackage(prefix, size), // SQLite 3 / OGC GeoPackage / QGIS (2004) D. Richard Hipp
        b"GGUF" => gguf(prefix), // GGML Universal Model File (2023) Georgi Gerganov
        b"ggml" // GGML Legacy Tensor File LE (2022) Georgi Gerganov
        | b"ggmf" // GGMF Versioned Tensor File LE (2023) Georgi Gerganov
        | b"ggjt" // GGJT Mmap-Aligned Tensor File LE (2023) Georgi Gerganov
        | b"lmgg" // GGML Legacy Tensor File BE (2022) Georgi Gerganov
        | b"fmgg" // GGMF Versioned Tensor File BE (2023) Georgi Gerganov
        | b"tjgg" => ggml(prefix, size), // GGJT Mmap-Aligned Tensor File BE (2023) Georgi Gerganov
        b"glTF" => glb(prefix), // Khronos Binary glTF 2.0 Asset (2017) Khronos Group
        b"GRIB" => grib(prefix), // WMO GRIB Meteorological Data (1985) WMO
        b"regf" => hve(prefix), // Windows NT Registry Hive (1993) Microsoft
        b"icns" => icns(prefix), // Apple Icon Image Container (1999) Apple
        PDF_MAGIC => pdf_or_ai(prefix, size), // Adobe PDF / Illustrator AI (1993) Adobe
        RTF_MAGIC => rtf(prefix, size), // Microsoft Rich Text Format (1987) Microsoft
        b"ITSF" => chm(prefix, size), // Microsoft Compiled HTML Help (1997) Microsoft
        TTF_TRUE_MAGIC // Apple TrueType Font sfnt (1991) Apple
        | TTF_TYP1_MAGIC // Apple PostScript Type 1 sfnt (1991) Apple
        | TTCF_MAGIC => ttf(prefix, size), // TrueType Font Collection (1994) Microsoft / Apple
        OTF_MAGIC => otf(prefix, size), // OpenType CFF Font (1996) Adobe / Microsoft
        b"hsqs" // SquashFS Little-Endian v4 (2009) Phillip Lougher
        | b"sqsh" // SquashFS Big-Endian (2002) Phillip Lougher
        | b"shsq" // SquashFS Swapped-Endian (2002) Phillip Lougher
        | b"qshs" => squashfs(prefix, size), // SquashFS Alternate Variant (2002) Phillip Lougher
        b"(dp0" // Python Pickle Protocol 0 Dict (1995) Python Software Foundation
        | b"(lp0" // Python Pickle Protocol 0 List (1995) Python Software Foundation
        | b"c__b" // Python Pickle Protocol 0 __builtin__ (1995) Python Software Foundation
        | b"ccop" => pickle_ascii(prefix, size), // Python Pickle Protocol 0 copy_reg (1995) Python Software Foundation
        b"!BDN" => outlook_pst(prefix, size), // Microsoft Outlook PST/OST Personal Folders (1996) Microsoft
        b"CMMM" => thumbcache(prefix, size), // Windows Explorer Thumbnail Cache (2006) Microsoft
        b"soli" => stl_ascii(prefix, size), // ASCII Stereolithography 3D Mesh (1987) 3D Systems
        b"Auto" => dxf_or_stl(prefix, size), // AutoCAD Binary DXF Drawing / AutoCAD Exported Binary STL (1988) Autodesk
        b"<?ph" => php_tag(prefix, size), // PHP Source Code Tag (1995) PHP Group
        b"MIME" // RFC 2557 MHTML / RFC 2045 MIME Mail (1996) IETF
        | b"From" => mht_or_eml(prefix, size), // RFC 822 / MHTML From Header (1982) IETF
        b"HEAD" => proteindb(prefix, size), // Brookhaven Protein Data Bank (1976) Brookhaven / wwPDB
        b"# pr" // Protocol Buffers Text Format Comment (2008) Google
        | b"node" => textproto(prefix, size), // TensorFlow GraphDef Textproto (2015) Google
        b"#cs " // AutoIt v3 Block Comment Space (2004) Jonathan Bennett
        | b"#inc" => autoit_script(prefix, size), // AutoIt v3 Include Directive (2004) Jonathan Bennett
        b"LRZI" => lrz(prefix), // Long Range ZIP Archive (2006) Con Kolivas
        b"LZIP" => lzip(prefix, ctx), // Lzip LZMA Compressed Stream (2008) Antonio Diaz Diaz
        b"MATL" => mat(prefix), // MATLAB Level 5 MAT-File (1999) MathWorks
        b"MThd" => midi(prefix), // Standard MIDI Audio File (1988) MMA
        b"MDMP" => minidump(prefix), // Windows Minidump Crash Dump (2001) Microsoft
        b"NRRD" => nrrd(prefix), // Teem Nearly Raw Raster Data (2002) Gordon Kindlmann
        b"PAR1" => parquet(prefix), // Apache Parquet Columnar Storage (2013) Apache
        b"----" => pem_or_stl(prefix, size), // RFC 7468 PEM / OpenPGP ASCII Armor / Binary STL Banner (1993) IETF
        b"PGDM" => postgres_dump(prefix), // PostgreSQL pg_dump Custom Archive (2000) PostgreSQL
        b"8BPS" => psd(prefix), // Adobe Photoshop PSD/PSB Document (1990) Adobe
        b"qoif" => qoi(prefix), // Quite OK Image Format (2021) Dominic Szablewski
        b"Rar!" // WinRAR Archive v1.5–v5.0 (1993) Eugene Roshal
        | b"RE~^" => rar(prefix), // RAR v1.4 Legacy Archive (1993) Eugene Roshal
        b"REDI" => redis_rdb(prefix), // Redis RDB Database Snapshot (2009) Salvatore Sanfilippo
        b"RZIP" => rzip(prefix), // rzip Large-Window Compressed File (2003) Andrew Tridgell
        b"$FL2" // SPSS System Data File (1968) SPSS / IBM
        | b"$FL3" => spss(prefix), // SPSS ZSAV Compressed Data File (2012) IBM
        b"ISO-" => step(prefix), // ISO 10303-21 STEP CAD Exchange Model (1994) ISO
        b"cone" => vhd(prefix), // Microsoft Virtual Hard Disk (2004) Connectix / Microsoft
        b"!<ar" => vib_or_deb(prefix, size), // Unix ar Archive / Debian .deb / VMware VIB (1993) Debian / VMware
        b"IWAD" // Doom Internal WAD Archive (1993) id Software
        | b"PWAD" => wad(prefix), // Doom Patch WAD Archive (1993) id Software
        b"wOFF" => woff(prefix), // W3C Web Open Font Format 1.0 (2009) Mozilla / W3C
        b"wOF2" => woff2(prefix), // W3C Web Open Font Format 2.0 (2014) Google / W3C
        b"xar!" => xar(prefix), // OpenDarwin / Apple eXtensible ARchive (2005) Apple
        b"gimp" => xcf(prefix), // GIMP XCF Native Image (1997) GIMP Team
        b"ARRO" => arrow(prefix), // Apache Arrow IPC File Format (2016) Apache
        b"MSCF" => cab(prefix), // Microsoft Cabinet Archive (1995) Microsoft
        b"Cr24" => crx(prefix), // Google Chrome Extension Package (2009) Google
        b"fLaC" => flac(prefix), // Free Lossless Audio Codec (2001) Josh Coalson / Xiph.Org
        b"GIF8" => gif(prefix), // CompuServe GIF87a / GIF89a Image (1987) CompuServe
        IFF_FORM_MAGIC => ilbm_or_iff(prefix, size), // EA IFF 85 FORM ILBM/ACBM/PBM / Cinema4D / LightWave (1985) Electronic Arts
        b"SZDD" => mscompress(prefix), // Microsoft Compress SZDD Archive (1990) Microsoft
        b"OggS" => ogg(prefix), // Xiph.Org Ogg Bitstream Container (2002) Xiph.Org
        b"Micr" // Microsoft Program Database (1995) Microsoft
        | b"BSJB" // .NET Portable PDB ECMA-335 Metadata Root (2015) Microsoft
        | b"MPDB" => pdb(prefix), // .NET Compressed Portable PDB (2015) Microsoft
        b"Adam" => pe_or_autoit(prefix, ctx), // TMT Pascal 32-bit Adam Executable (1995) TMT Development
        b"<sta" => stata(prefix), // Stata XML Dataset (2003) StataCorp
        b"%!PS" => postscript(prefix), // Adobe PostScript DSC Document (1984) Adobe
        b"d13:" // BitTorrent Metainfo Dict 13-char Key (2001) Bram Cohen
        | b"d4:i" // BitTorrent Metainfo Dict info Key (2001) Bram Cohen
        | b"d8:a" // BitTorrent Metainfo Dict announce Key (2001) Bram Cohen
        | b"d7:c" => torrent(prefix), // BitTorrent Metainfo Dict comment Key (2001) Bram Cohen
        b"BEGI" => ics_or_vcard(prefix, size), // IETF RFC 5545 iCalendar / vCalendar / vCard (1998) IETF
        b"[Int" // Windows Internet Shortcut Direct Section (1995) Microsoft
        | b"[{00" => internetshortcut(prefix, size), // Windows Internet Shortcut GUID Section (1995) Microsoft
        b"#EXT" => m3u(prefix, size), // Extended M3U / HLS Playlist (1997) Fraunhofer / Apple
        b"WEBV" => vtt(prefix, size), // W3C WebVTT Timed Text Subtitle (2011) W3C
        b"REGE" // Windows 95/NT4 Registry Script REGEDIT4 (1995) Microsoft
        | b"Wind" => winregistry(prefix, size), // Windows 2000+ Registry Script Version 5.00 (1999) Microsoft
        b"SET " => aff_hunspell(prefix, size), // Hunspell Affix Dictionary File (2003) Laszlo Nemeth
        b"Ogaw" => alembic(prefix, size), // Alembic Ogawa 3D Geometry Cache (2013) Sony / ILM
        b"ASD0" => asd(prefix, size), // Microsoft Word AutoRecovery Save (.asd) (1995) Microsoft
        b"BZ3v" => bzip3(prefix, size), // BZip3 Compressed Stream (2022) Kamila Szewczyk
        b"CBM1" => catboost(prefix, size), // CatBoost Gradient Boosting Model (2017) Yandex
        b"DB2B" // IBM DB2 Database Backup Image (1993) IBM
        | b"BIND" // IBM DB2 Bind File v2 (1993) IBM
        | b"SQLO" => db2(prefix, size), // IBM DB2 Conversion Table (1993) IBM
        b"EHFA" => erdas(prefix, size), // ERDAS IMAGINE Hierarchical File Format (.img) (1991) ERDAS / Hexagon
        b"H:2," => h2(prefix, size), // H2 MVStore Database Engine (2014) Thomas Mueller
        b"HWP " => hwp_v3(prefix, size), // Hangul Word Processor v3.0 Document (1995) Hancom
        b"LASF" => las(prefix, size), // ASPRS LAS LiDAR Point Cloud (2003) ASPRS
        b"EXPO" => oracle_dmp_text(prefix, size), // Oracle Classic Export Dump (.dmp) (1988) Oracle
        b"PMTi" => pmtiles(prefix, size), // Protomaps PMTiles Tile Archive (2021) Brandon Liu / Protomaps
        b"RDX2" // R Serialization XDR v2 (2004) R Core Team
        | b"RDX3" // R Serialization XDR v3 (2018) R Core Team
        | b"RDA2" // R Serialization ASCII v2 (2004) R Core Team
        | b"RDA3" // R Serialization ASCII v3 (2018) R Core Team
        | b"RDB2" // R Serialization Binary v2 (2004) R Core Team
        | b"RDB3" => rdata(prefix, size), // R Serialization Binary v3 (2018) R Core Team
        b"T-DB" => realm(prefix, size), // Realm Mobile Database (2014) Realm / MongoDB
        b"file" => redis_aof_manifest(prefix, size), // Redis Append-Only File v7+ Manifest (2022) Redis
        b"PXR-" // Pixar USD Binary Crate (.usdc) (2016) Pixar
        | b"#usd" => usd(prefix, size), // Pixar USD ASCII Scene (.usda) (2016) Pixar
        b"#@~^" => vbe(prefix, size), // Microsoft VBScript/JScript Encoded Script (.vbe/.jse) (1999) Microsoft
        b"##fi" => vcf_genomics(prefix, size), // Variant Call Format v4 Genomics Text (2010) 1000 Genomes / GA4GH
        b"# vt" => vtk(prefix, size), // Visualization Toolkit Legacy DataFile (.vtk) (1993) Kitware
        b"MSWI" // Microsoft Windows Imaging Format (.wim) (2006) Microsoft
        | b"WLPW" => wim(prefix, size), // Windows Local Patch WIM (2006) Microsoft
        b"WT_F" => wiredtiger(prefix, size), // WiredTiger Storage Engine Data File (2012) WiredTiger / MongoDB
        b"ZBr4" // Pixologic ZBrush v4 Document/Tool (.zbr/.ztl) (2010) Pixologic / Maxon
        | b"ZBru" => zbrush(prefix, size), // Pixologic ZBrush File Header (1999) Pixologic / Maxon
        b"scre" // Microsoft Remote Desktop Connection screen mode (2001) Microsoft
        | b"full" => rdp(prefix, size), // Microsoft Remote Desktop Connection full address (2001) Microsoft

        // =====================================================================
        // Section 2: Mixed Printable ASCII & Control / High Bytes
        // =====================================================================
        b"3DS\0" => three_dsm_legacy(prefix, size), // Autodesk 3D Studio DOS Legacy Scene (.3ds) (1990) Autodesk
        b"Obj\x01" => avro(prefix), // Apache Avro Object Container File (2009) Apache
        b"BAM\x01" => bam(prefix), // Binary Alignment Map (2009) SAMtools
        b"\x0f7BE" => beam(prefix), // Erlang BEAM Compressed Module (1997) Ericsson
        b"BPG\xfb" => bpg(prefix), // Better Portable Graphics (2014) Fabrice Bellard
        b"\x7fELF" => cubin_or_elf(prefix, size), // System V ELF / NVIDIA CUBIN (1988) AT&T / USL
        b"dex\n" => dex(prefix), // Android Dalvik Executable (2007) Google
        b"dey\n" => odex(prefix, size), // Android Optimized Dalvik Executable (.odex) (2008) Google
        b"FLV\x01" => flv(prefix), // Adobe Flash Video v1 (2003) Macromedia / Adobe
        b"\x89HDF" => hdf5_or_keras(prefix, size), // NCSA HDF5 / Keras Model (1998) NCSA
        b"\x8bJNG" => jng(prefix), // JPEG Network Graphics (2001) Glenn Randers-Pehrson
        b"\x89PNG" => png(prefix, size), // W3C Portable Network Graphics (1996) W3C
        TIFF_LE_MAGIC // TIFF Little-Endian (1986) Aldus / Adobe
        | TIFF_BE_MAGIC // TIFF Big-Endian (1986) Aldus / Adobe
        | BIGTIFF_LE_MAGIC // BigTIFF 64-bit Little-Endian (2007) AWare Systems
        | BIGTIFF_BE_MAGIC // BigTIFF 64-bit Big-Endian (2007) AWare Systems
        | b"II\x55\x00" // Panasonic RW2 Raw Camera TIFF (2008) Panasonic
        | b"IIRO" // Olympus ORF Raw Camera TIFF IIR (2001) Olympus
        | b"IIRS" // Olympus ORF Raw Camera TIFF IIRS (2004) Olympus
        | b"MMOR" // Olympus ORF Raw Camera Big-Endian (2001) Olympus
        | b"CR\x02\x00" => tiff(prefix, size), // Canon CR2 Raw Camera Header (2004) Canon
        b"\xfd7zX" => xz(prefix, size), // Tukaani XZ Compressed Stream (2009) Lasse Collin
        b"ER\x08\x00" // Apple Driver Descriptor Map 2048B Block (1987) Apple
        | b"ER\x02\x00" => dmg_header(prefix, size), // Apple Driver Descriptor Map 512B Block (1987) Apple
        b"ID3\x02" // ID3v2.2 Tagged MP3 Audio (1998) Martin Nilsson
        | b"ID3\x03" // ID3v2.3 Tagged MP3 Audio (1999) Martin Nilsson
        | b"ID3\x04" => id3_mp3(prefix, size), // ID3v2.4 Tagged MP3 Audio (2000) Martin Nilsson
        b"  0\n" // AutoCAD ASCII DXF Space-Padded LF (1982) Autodesk
        | b"  0\r" // AutoCAD ASCII DXF Space-Padded CR (1982) Autodesk
        | b" 0\r\n" // AutoCAD ASCII DXF 1-Space CRLF (1982) Autodesk
        | b" 0\nS" // AutoCAD ASCII DXF 1-Space LF (1982) Autodesk
        | b"0\nSE" // AutoCAD ASCII DXF Unpadded LF (1982) Autodesk
        | b"0\r\nS" => dxf_ascii(prefix, size), // AutoCAD ASCII DXF Unpadded CRLF (1982) Autodesk
        b"#cs\n" // AutoIt v3 Block Comment LF (2004) Jonathan Bennett
        | b"#cs\r" => autoit_script(prefix, size), // AutoIt v3 Block Comment CR (2004) Jonathan Bennett
        b"BC\xc0\xde" => llvm_bitcode(prefix, ctx), // LLVM Raw Bitcode Stream (2007) LLVM Project
        b"\x1bLua" // Lua Compiled Bytecode (1994) PUC-Rio
        | b"\x1bLJ\x01" // LuaJIT 1.x/2.0 Bytecode (2008) Mike Pall
        | b"\x1bLJ\x02" => luabytecode(prefix), // LuaJIT 2.1 Bytecode (2015) Mike Pall
        b"\x02!L\x18" // LZ4 Legacy Frame v0.2 (2011) Yann Collet
        | b"\x03!L\x18" // LZ4 Legacy Frame v0.3 (2012) Yann Collet
        | b"\x04\"M\x18" => lz4(prefix, ctx), // LZ4 Standard Frame v1.4+ (2013) Yann Collet
        b"\x93NUM" => npy(prefix), // NumPy .npy Array Binary (2007) Travis Oliphant
        b"ply\n" // Stanford Polygon File Format LF (1994) Greg Turk
        | b"ply\r" => ply(prefix), // Stanford Polygon File Format CR (1994) Greg Turk
        b"\x0eSke" => sketchup(prefix), // Trimble SketchUp ASCII Header (2000) @Last Software
        b"UF2\n" => uf2(prefix), // Microsoft UF2 Bootloader Firmware (2016) Microsoft
        b"\x00asm" => wasm(prefix), // W3C WebAssembly Binary Module (2017) W3C
        b"?_\x03\x00" => hlp(prefix), // Microsoft Windows WinHelp (.hlp) (1990) Microsoft
        b"CDF\x01" // Unidata NetCDF Classic v1 (1989) UCAR
        | b"CDF\x02" => netcdf(prefix), // Unidata NetCDF 64-bit Offset v2 (2004) UCAR
        b"CDF\x05" => cdf(prefix, size), // Unidata NetCDF 64-bit Data CDF-5 (2016) UCAR
        b"fgb\x03" // FlatGeobuf v3 Spatial Binary (2020) Bjorn Harrtell
        | b"fgb\x02" // FlatGeobuf v2 Spatial Binary (2019) Bjorn Harrtell
        | b"fgb\x01" => flatgeobuf(prefix, size), // FlatGeobuf v1 Spatial Binary (2018) Bjorn Harrtell
        b"LZX\x00" => lzx(prefix, size), // Amiga LZX Archive (1995) Jonathan Forbes & Tomi Poutanen
        b"LWS\n" // NewTek LightWave 3D Scene LF (1990) NewTek
        | b"LWS\r" => lightwave_scene(prefix, size), // NewTek LightWave 3D Scene CR (1990) NewTek
        b"ORC\0" // Apache ORC Columnar Storage NUL (2013) Apache
        | b"ORC\n" // Apache ORC Columnar Storage LF (2013) Apache
        | b"ORC\x01" // Apache ORC Columnar Storage v1 (2013) Apache
        | b"ORC\x08" // Apache ORC Columnar Storage Protobuf (2013) Apache
        | b"ORC\x82" => orc(prefix, size, ctx.tail), // Apache ORC Snappy/Zlib Stream (2013) Apache
        ZIP_LOCAL_SIGNATURE // PKZIP Local File Header (1989) Phil Katz / PKWARE
        | ZIP_EOCD_SIGNATURE => zip(prefix, ctx), // PKZIP End of Central Directory (1989) Phil Katz / PKWARE

        // =====================================================================
        // Section 3: Pure Binary Signatures
        // =====================================================================
        &[0x00, 0x05, 0x16, 0x07] => appledouble(prefix), // AppleDouble Resource File (1990) Apple
        &[0x00, 0x05, 0x16, 0x00] => applesingle(prefix), // AppleSingle Archive (1990) Apple
        &[0x00, 0x00, 0x00, 0x01] => dsstore_or_isobmff(prefix, size), // macOS Finder .DS_Store / 64-bit ISO-BMFF Box (1999) Apple / ISO
        &[0x0E, 0x03, 0x13, 0x01] => hdf4(prefix), // NCSA HDF4 Scientific Data (1988) NCSA
        &[0xD7, 0xCD, 0xC6, 0x9A] // Aldus Placeable Windows Metafile (1987) Aldus
        | &[0x02, 0x00, 0x09, 0x00] // Windows Metafile Disk-Based v3.0 (1990) Microsoft
        | &[0x01, 0x00, 0x09, 0x00] => wmf(prefix, size), // Windows Metafile Memory-Based v3.0 (1990) Microsoft
        TTF_SFNT_V1_MAGIC => ttf_or_access(prefix, size), // TrueType 1.0 sfnt Font / Microsoft Access Jet/ACE DB
        &[0xCA, 0xFE, 0xBA, 0xBE] => cafebabe(prefix, size), // Java Class / Mach-O Universal (1995) Sun / Apple
        &[0xA3, 0x48, 0x4B, 0xBE] => autoit_compiled(prefix, size), // AutoIt v3 Compiled Script (2004) Jonathan Bennett
        &[0xD0, 0xCF, 0x11, 0xE0] => verify_ole2(prefix, size, ctx.tail), // Microsoft OLE2 Compound File (1992) Microsoft
        &[0x31, 0xBE, 0x00, 0x00] // Microsoft Word for DOS Unformatted (1983) Microsoft
        | &[0x32, 0xBE, 0x00, 0x00] => word_dos(prefix, size), // Microsoft Word for DOS Formatted (1983) Microsoft
        &[0xDE, 0xC0, 0x17, 0x0B] => llvm_bitcode(prefix, ctx), // LLVM Bitcode Darwin Wrapper (2007) Apple / LLVM
        &[0x4C, 0x00, 0x00, 0x00] => lnk(prefix), // Microsoft Windows Shell Link (.lnk) (1995) Microsoft
        &[0xE4, 0x52, 0x5C, 0x7B] => one(prefix), // Microsoft OneNote Section (.one) (2003) Microsoft
        &[0x37, 0x7A, 0xBC, 0xAF] => sevenzip(prefix), // 7-Zip Archive (1999) Igor Pavlov
        &[0x00, 0x00, 0x27, 0x0A] => shapefile(prefix), // ESRI Shapefile Main/Index (.shp/.shx) (1998) Esri
        &[0xFF, 0xFE, 0xFF, 0x0E] => sketchup(prefix), // Trimble SketchUp UTF-16LE Model (2000) @Last Software
        &[0x03, 0x02, 0x23, 0x07] // Khronos SPIR-V Little-Endian (2015) Khronos Group
        | &[0x07, 0x23, 0x02, 0x03] => spirv(prefix, ctx), // Khronos SPIR-V Big-Endian (2015) Khronos Group
        &[0xC9, 0xC3, 0xE2, 0xC1] => spss(prefix), // SPSS EBCDIC Portable Dataset (1968) SPSS
        &[0x22, 0xB5, 0x2F, 0xFD] // Zstandard Frame v0.2 (2015) Yann Collet
        | &[0x23, 0xB5, 0x2F, 0xFD] // Zstandard Frame v0.3 (2015) Yann Collet
        | &[0x24, 0xB5, 0x2F, 0xFD] // Zstandard Frame v0.4 (2015) Yann Collet
        | &[0x25, 0xB5, 0x2F, 0xFD] // Zstandard Frame v0.5 (2016) Yann Collet
        | &[0x26, 0xB5, 0x2F, 0xFD] // Zstandard Frame v0.6 (2016) Yann Collet
        | &[0x27, 0xB5, 0x2F, 0xFD] // Zstandard Frame v0.7 (2016) Yann Collet
        | &[0x28, 0xB5, 0x2F, 0xFD] => zst(prefix), // Zstandard Frame v0.8+ RFC 8878 (2016) Yann Collet / Meta
        &[0x0A, 0x0D, 0x0D, 0x0A] => pcapng(prefix), // IETF PCAPNG Section Header Block (2004) IETF
        &[0x00, 0x00, 0x01, 0x00] => ico(prefix), // Microsoft Windows Icon Resource (.ico) (1985) Microsoft
        &[0x00, 0x00, 0x02, 0x00] => ico_or_tga(prefix, ctx), // Microsoft Windows Cursor (.cur) / Uncompressed True-Color TGA (1985) Microsoft / Truevision
        &[0x00, 0x00, 0x00, 0x0C] => jp2_or_jxl(prefix), // ISO/IEC 15444-1 JP2 / 18181-2 JXL Box (2000) ISO
        &[0xFF, 0x80, 0xC9, 0xC7] => koala(prefix), // Commodore 64 Koala Painter Image (1983) Koala Technologies
        &[0xCE, 0xFA, 0xED, 0xFE] // Apple Mach-O 32-bit Little-Endian (1988) NeXT / Apple
        | &[0xCF, 0xFA, 0xED, 0xFE] // Apple Mach-O 64-bit Little-Endian (2003) Apple
        | &[0xFE, 0xED, 0xFA, 0xCE] // Apple Mach-O 32-bit Big-Endian (1988) NeXT / Apple
        | &[0xFE, 0xED, 0xFA, 0xCF] => macho(prefix), // Apple Mach-O 64-bit Big-Endian (2003) Apple
        EBML_MAGIC => mkv_or_webm(prefix, size), // EBML Matroska / WebM Container (2002) Matroska / Google
        &[0xD4, 0xC3, 0xB2, 0xA1] // libpcap Microsecond Little-Endian (1994) LBL / tcpdump
        | &[0x4D, 0x3C, 0xB2, 0xA1] // libpcap Nanosecond Little-Endian (2005) tcpdump
        | &[0xA1, 0xB2, 0xC3, 0xD4] // libpcap Microsecond Big-Endian (1994) LBL / tcpdump
        | &[0xA1, 0xB2, 0x3C, 0x4D] => pcap(prefix), // libpcap Nanosecond Big-Endian (2005) tcpdump
        &[0xED, 0xAB, 0xEE, 0xDB] => rpm(prefix), // Red Hat Package Manager v3/v4 (1997) Red Hat
        &[0x00, 0x00, 0x00, 0x00] => null_prefixed(prefix, ctx), // SAS7BDAT / DuckDB / MPEG-TS / Binary STL / DICOM / ISO 9660 / MBR / HFS+ / Kodak Photo CD
        &[0x01, 0x00, 0x00, 0x00] => emf_or_wmf(prefix, size), // Microsoft Enhanced Metafile (1992) Microsoft
        &[0xFF, 0xFE, 0x57, 0x00] => winregistry(prefix, size), // Windows Registry Script UTF-16LE (1999) Microsoft
        &[0xCD, 0xF3, 0x00, 0x01] // NASA Common Data Format v3 (2004) NASA GSFC
        | &[0xCD, 0xF2, 0x06, 0x00] => cdf(prefix, size), // NASA Common Data Format v2.6 (1996) NASA GSFC
        &[0x30, 0x26, 0xB2, 0x75] => asf_family(prefix, size), // Microsoft Advanced Systems Format / WMA / WMV (1996) Microsoft
        b"\x7FDB2" // IBM DB2 Check/Internal Header (1993) IBM
        | &[0xFF, 0xFB, 0x49, 0x4E] => db2(prefix, size), // IBM DB2 Index File (1993) IBM
        &[0xE0, 0x0D, 0x00, 0x00] // WiredTiger Write-Ahead Log (2012) WiredTiger / MongoDB
        | &[0x41, 0xD8, 0x01, 0x00] => wiredtiger(prefix, size), // WiredTiger B-Tree Block (2012) WiredTiger / MongoDB
        &[0xA8, 0x00, 0x00, 0x00] => zbrush(prefix, size), // Pixologic ZBrush ZProject (.zpr) (2010) Pixologic / Maxon
        &[0xFF, 0xFE, 0x73, 0x00] => rdp(prefix, size), // Microsoft Remote Desktop Connection UTF-16LE (2001) Microsoft
        &[0xCC, 0x00, 0x64, 0x00] // Linux/BSD QMAGIC i386 a.out Executable (1993) Linux / BSD
        | &[0x07, 0x01, 0x00, 0x00] // Research Unix / BSD OMAGIC LE a.out Executable (1971) Bell Labs / UCB
        | &[0x08, 0x01, 0x00, 0x00] // Research Unix / BSD NMAGIC LE a.out Executable (1975) Bell Labs / UCB
        | &[0x0B, 0x01, 0x00, 0x00] // BSD ZMAGIC Demand-Paged LE a.out Executable (1979) UCB
        | &[0x00, 0x00, 0x01, 0x07] // Research Unix / BSD OMAGIC BE a.out Executable (1971) Bell Labs / UCB
        | &[0x00, 0x00, 0x01, 0x08] // Research Unix / BSD NMAGIC BE a.out Executable (1975) Bell Labs / UCB
        | &[0x00, 0x00, 0x01, 0x0B] => aout(prefix, size), // BSD ZMAGIC Demand-Paged BE a.out Executable (1979) UCB

        // =====================================================================
        // Section 4: Shortcuts (Offset Probes & Heuristics)
        // =====================================================================

        // --- ISO 9660 Sector 0 Non-Zero Padding & MBR / El Torito IsoHybrid ---
        // ISO 9660 reserves System Area sectors 0..15 (`0x0000..0x7FFF`) before the Primary Volume
        // Descriptor (`\x01CD001\x01`) at `@0x8000`. All-zero Sector 0 is handled by `null_prefixed`;
        // disc mastering tools that fill Sector 0 with `0xAD` or `'0'`, or embed an x86 MBR / El Torito
        // IsoHybrid bootstrap at `@0..4`, are dispatched here directly at Tier 1:
        &[0xAD, 0xAD, 0xAD, 0xAD] // ISO 9660 Sector 0 `0xAD` fill pattern (1988) ECMA-119 / ISO 9660
        | b"0000" => iso(prefix, size), // ISO 9660 Sector 0 ASCII `'0'` fill pattern (1988) ECMA-119 / ISO 9660
        &[0x33, 0xED, 0x90, 0x90] // ISOLINUX / SYSLINUX IsoHybrid MBR: `xor bp,bp; nop; nop` (2008) H. Peter Anvin
        | &[0x31, 0xED, 0x90, 0x90] // ISOLINUX / SYSLINUX IsoHybrid MBR (GNU as encoding): `xor bp,bp; nop; nop` (2008) H. Peter Anvin
        | &[0x33, 0xED, 0xFA, 0x8E] // ISOLINUX IsoHybrid MBR: `xor bp,bp; cli; mov ss,...` (2008) H. Peter Anvin
        | &[0x31, 0xED, 0xFA, 0x8E] // ISOLINUX IsoHybrid MBR (GNU as encoding): `xor bp,bp; cli; mov ss,...` (2008) H. Peter Anvin
        | &[0xEB, 0x63, 0x90, 0x90] // GRUB2 / xorriso IsoHybrid MBR: `jmp short 0x65; nop; nop` (2010) GNU GRUB
        | &[0xEB, 0x63, 0x90, 0x00] // GRUB2 / xorriso IsoHybrid MBR: `jmp short 0x65; nop; 0x00` (2010) GNU GRUB
        | &[0xEB, 0x63, 0x90, 0xD0] // Windows Vista+ / DiskPart MBR: `jmp short 0x65; nop; 0xD0` (2006) Microsoft
        | &[0xEB, 0x48, 0x90, 0x00] // GRUB Legacy Stage 1 MBR: `jmp short 0x4A; nop; 0x00` over BPB (1999) GNU GRUB
        | &[0xEB, 0x4C, 0x90, 0x00] // GRUB Legacy Stage 1 Patched MBR: `jmp short 0x4E; nop; 0x00` (2000) GNU GRUB
        | &[0xEB, 0x3C, 0x90, 0x4C] // LILO Boot Loader MBR: `jmp short 0x3E; nop; "LILO"` OEM ID (1992) Werner Almesberger
        | &[0xFC, 0x31, 0xC0, 0x8E] // FreeBSD `boot0` Boot Manager MBR: `cld; xor ax,ax; mov es,ax` (1998) FreeBSD Project
        | &[0x33, 0xC0, 0xFA, 0x8E] // MS-DOS / Windows 95/98 MBR: `xor ax,ax; cli; mov ss,ax` (1983) Microsoft
        | &[0x33, 0xC0, 0x8E, 0xD0] // Windows 2000 / XP / Server 2003 MBR: `xor ax,ax; mov ss,ax` (2000) Microsoft
        | &[0x31, 0xC0, 0x8E, 0xD8] // SeaBIOS / QEMU MBR: `xor ax,ax; mov ds,ax` (2008) Kevin O'Connor
        | &[0xFA, 0x31, 0xC0, 0x8E] // NetBSD / OpenBSD `fdisk` MBR: `cli; xor ax,ax; mov ss,ax` (1995) NetBSD / OpenBSD
        | &[0xFA, 0x33, 0xC0, 0x8E] // MS-DOS 3.30–6.22 MBR: `cli; xor ax,ax; mov ss,ax` (1987) Microsoft
        | &[0xFA, 0xFC, 0x31, 0xC0] // TestDisk / util-linux MBR: `cli; cld; xor ax,ax` (1998) Christophe Grenier
        | &[0xEA, 0x05, 0x00, 0xC0] // Real-Mode Far Jump MBR: `jmp 0xC000:0x0005` (1985) IBM
        | &[0xEA, 0x05, 0x7C, 0x00] // Canonical Far Jump MBR: `jmp 0x0000:0x7C05` to normalize CS:IP (1995) FreeBSD / OpenBSD
        | &[0xE8, 0x0B, 0x01, 0xB9] // Near-Call MBR Bootstrap: `call +0x010B; mov cx,...` (1990) Phoenix / Award
        | &[0xFA, 0xEA, 0x6C, 0x7C] // OS/2 Boot Manager Far-Jump MBR: `cli; jmp 0x0000:0x7C6C` (1995) IBM
        | &[0xFA, 0xB8, 0xB0, 0x07] => boot_or_isohybrid(prefix, size), // Segment Setup MBR: `cli; mov ax,0x07B0` (1988) Compaq / IBM

        // --- ISO-BMFF / QuickTime / HEIC / AVIF / 3GP Box Lengths (u32 BE Box Size at @0..4) ---
        // In ISO/IEC 14496-12 (ISO-BMFF), bytes @0..4 are the 32-bit big-endian byte length of the
        // first box (followed by `"ftyp"`, `"wide"`, `"free"`, or `"moov"` at `@4..8`). An `ftyp` box
        // has length `16 + 4 * N` bytes (`8`B header + `4`B `major_brand` + `4`B `minor_version` + `N`
        // 4-byte `compatible_brands`, so `0x10..=0x40` step `4` covers `0..=12` compatible brands):
        &[0x00, 0x00, 0x00, 0x08] // 8-byte leading `wide` / `free` / `skip` placeholder box before `mdat`/`moov` (2001) Apple / ISO
        | &[0x00, 0x00, 0x00, 0x10] // 16-byte `ftyp` box (`16 = 8 + 8`, 0 compatible brands) (2001) Apple / ISO
        | &[0x00, 0x00, 0x00, 0x14] // 20-byte `ftyp` box (`20 = 16 + 1*4`, 1 compatible brand) (2001) Apple / ISO
        | &[0x00, 0x00, 0x00, 0x18] // 24-byte `ftyp` box (`24 = 16 + 2*4`, 2 compatible brands) (2001) Apple / ISO
        | &[0x00, 0x00, 0x00, 0x1C] // 28-byte `ftyp` box (`28 = 16 + 3*4`, 3 compatible brands) (2001) Apple / ISO
        | &[0x00, 0x00, 0x00, 0x20] // 32-byte `ftyp` box (`32 = 16 + 4*4`, 4 compatible brands) (2001) Apple / ISO
        | &[0x00, 0x00, 0x00, 0x24] // 36-byte `ftyp` box (`36 = 16 + 5*4`, 5 compatible brands) (2001) Apple / ISO
        | &[0x00, 0x00, 0x00, 0x28] // 40-byte `ftyp` box (`40 = 16 + 6*4`, 6 compatible brands) (2001) Apple / ISO
        | &[0x00, 0x00, 0x00, 0x2C] // 44-byte `ftyp` box (`44 = 16 + 7*4`, 7 compatible brands) (2001) Apple / ISO
        | &[0x00, 0x00, 0x00, 0x30] // 48-byte `ftyp` box (`48 = 16 + 8*4`, 8 compatible brands) (2001) Apple / ISO
        | &[0x00, 0x00, 0x00, 0x34] // 52-byte `ftyp` box (`52 = 16 + 9*4`, 9 compatible brands) (2001) Apple / ISO
        | &[0x00, 0x00, 0x00, 0x38] // 56-byte `ftyp` box (`56 = 16 + 10*4`, 10 compatible brands) (2001) Apple / ISO
        | &[0x00, 0x00, 0x00, 0x3C] // 60-byte `ftyp` box (`60 = 16 + 11*4`, 11 compatible brands) (2001) Apple / ISO
        | &[0x00, 0x00, 0x00, 0x40] => isobmff(prefix, size), // 64-byte `ftyp` box (`64 = 16 + 12*4`, 12 compatible brands) (2001) Apple / ISO

        // --- OpenStreetMap PBF BlobHeader Lengths (u32 BE Length at @0..4) ---
        // In OSM `.osm.pbf`, bytes @0..4 are the 32-bit big-endian byte length of the first Protobuf
        // `BlobHeader` (`\n\x09OSMHeader\x18<datasize>`), which is 13 or 14 bytes depending on the
        // varint length of `datasize`:
        &[0x00, 0x00, 0x00, 0x0D] // 13-byte `BlobHeader` (`\n\x09OSMHeader\x18<2B varint>`) (2010) Scott Crosby / OSM
        | &[0x00, 0x00, 0x00, 0x0E] => osm_pbf(prefix), // 14-byte `BlobHeader` (`\n\x09OSMHeader\x18<3B varint>`) (2010) Scott Crosby / OSM

        // --- POSIX / GNU / PAX TAR Archive Member Name Prefixes ---
        // TAR archives have `ustar` at `@257`, while bytes @0..100 hold the first member's path name:
        b".\0\0\0" // POSIX `ustar` root `".\0"` directory entry (1988) IEEE POSIX.1
        | b"./\0\0" // GNU / BSD `tar` `"./\0"` root directory entry (1989) Free Software Foundation
        | b"pax_" => tar(prefix, size), // POSIX.1-2001 `"pax_global_header"` extended header entry (2001) IEEE / Open Group

        // --- Truevision TGA Image Headers (id_len, color_map_type, image_type, first_entry_lo) ---
        // TGA has no magic signature; for images with `id_length == 0` (`@0 = 0x00`) and `first_entry_index == 0`
        // (`@3 = 0x00`), bytes @1..3 encode `(color_map_type, image_type)` (`(0, 2)` is shared with Windows
        // `.cur` in `ico_or_tga` above):
        &[0x00, 0x00, 0x0A, 0x00] // `id_len=0, cmap=0, image_type=10`: RLE True-Color TGA (1984) Truevision
        | &[0x00, 0x01, 0x01, 0x00] // `id_len=0, cmap=1, image_type=1`: Uncompressed Color-Mapped TGA (1984) Truevision
        | &[0x00, 0x01, 0x09, 0x00] // `id_len=0, cmap=1, image_type=9`: RLE Color-Mapped TGA (1984) Truevision
        | &[0x00, 0x00, 0x03, 0x00] // `id_len=0, cmap=0, image_type=3`: Uncompressed Grayscale TGA (1984) Truevision
        | &[0x00, 0x00, 0x0B, 0x00] => tga(prefix, size, ctx.tail), // `id_len=0, cmap=0, image_type=11`: RLE Grayscale TGA (1984) Truevision

        // --- Binary STL 80-Byte CAD Exporter Preambles ---
        // Binary STL begins with an 80-byte free-form header followed by `u32 LE` `triangle_count` at `@80`
        // where `size == 84 + 50 * triangle_count`. Major CAD/slicer exporters write a known banner at `@0..4`:
        b"STL " // Generic `"STL ..."` binary STL header preamble (1987) 3D Systems
        | b"STLB" // `"STLB..."` binary STL header preamble (1987) 3D Systems
        | b"Bin " // `"Bin ..."` binary STL header preamble (1987) 3D Systems
        | b"BINA" // `"BINARY ..."` binary STL header preamble (1987) 3D Systems
        | b"bina" // `"binary ..."` binary STL header preamble (1987) 3D Systems
        | b"MESH" // MeshLab / Netfabb `"MESH-MESH-MESH..."` binary STL preamble (2005) ISTI-CNR / Netfabb
        | b"3D S" // 3D Systems / Autodesk 3ds Max `"3D Systems..."` binary STL preamble (1987) 3D Systems
        | b"VCG\0" // ISTI-CNR VCG Lib / MeshLab `"VCG\0..."` binary STL preamble (2004) ISTI-CNR
        | b"Exp " // CAD `"Exp ..."` binary STL preamble (2002) Blender Foundation
        | b"Expo" // Blender `"Exported from Blender..."` binary STL preamble (2002) Blender Foundation
        | b"Crea" // CAD `"Created by ..."` binary STL preamble (1995) Autodesk / SolidWorks
        | b"Gene" // OpenSCAD / FreeCAD `"Generated by ..."` binary STL preamble (1995) OpenSCAD / FreeCAD
        | b"Open" // OpenSCAD / OpenCASCADE `"OpenSCAD Model..."` binary STL preamble (2010) OpenSCAD
        | b"Soli" // SolidWorks / SolidEdge `"SolidWorks..."` binary STL preamble (1995) Dassault Systèmes
        | b"SOLI" // SOLIDWORKS `"SOLIDWORKS..."` uppercase binary STL preamble (1995) Dassault Systèmes
        | b" Rh " // Rhinoceros 3D `" Rh ..."` binary STL preamble (1998) Robert McNeel & Associates
        | b"Rhin" // Rhinoceros 3D `"Rhinoceros..."` binary STL preamble (1998) Robert McNeel & Associates
        | b"ibal" // Materialise Magics `"ibal..."` binary STL preamble (1992) Materialise
        | b"Mat " // Materialise Magics `"Mat ..."` binary STL preamble (1992) Materialise
        | b"Mate" // Materialise Magics `"Materialise..."` binary STL preamble (1992) Materialise
        | b"Pro/" // PTC Pro/ENGINEER `"Pro/ENGINEER..."` binary STL preamble (1988) PTC
        | b"CATI" // Dassault CATIA `"CATIA..."` binary STL preamble (1988) Dassault Systèmes
        | b"FREE" // FreeCAD `"FREECAD..."` uppercase binary STL preamble (2002) FreeCAD Team
        | b"Free" // FreeCAD `"FreeCAD..."` binary STL preamble (2002) FreeCAD Team
        | b" Cur" // Ultimaker Cura `" Cura..."` binary STL preamble (2012) Ultimaker
        | b"Cura" // Ultimaker Cura `"Cura_SteamEngine..."` binary STL preamble (2012) Ultimaker
        | b"Prus" // PrusaSlicer `"PrusaSlicer..."` binary STL preamble (2016) Prusa Research
        | b"Slic" // Slic3r `"Slic3r..."` binary STL preamble (2011) Alessandro Ranellucci
        | b"Netf" // Netfabb Studio `"Netfabb..."` binary STL preamble (2009) Netfabb / Autodesk
        | b"netf" => stl_binary(prefix, size), // Netfabb `"netfabb..."` lowercase binary STL preamble (2009) Netfabb

        // --- XML / HTML / SVG / GIS / Package Document Root Tags ---
        // Unindented XML/HTML documents starting at `@0` with `<?xml`, UTF-8 BOM + `<`, `<!DOCTYPE`,
        // `<!--`, or a known root element tag (whitespace-indented documents fall back to `heuristics`):
        b"<?xm" // W3C XML 1.0 `<?xml` declaration (1998) W3C
        | b"\xEF\xBB\xBF<" // UTF-8 BOM (`EF BB BF`) + `<` XML / HTML tag (1998) W3C
        | b"<!DO" // SGML / HTML / XML uppercase `<!DOCTYPE` declaration (1993) IETF / W3C
        | b"<!do" // HTML lowercase `<!doctype` declaration (1993) IETF / W3C
        | b"<!--" // XML / HTML leading comment (1993) IETF / W3C
        | b"<svg" // W3C Scalable Vector Graphics root element (2001) W3C
        | b"<SVG" // W3C Scalable Vector Graphics uppercase root element (2001) W3C
        | b"<htm" // HTML `<html` root element (1993) IETF / W3C
        | b"<HTM" // HTML `<HTML` uppercase root element (1993) IETF / W3C
        | b"<hea" // HTML `<head` fragment root element (1993) IETF / W3C
        | b"<HEA" // HTML `<HEAD` uppercase fragment root element (1993) IETF / W3C
        | b"<bod" // HTML `<body` fragment root element (1993) IETF / W3C
        | b"<BOD" // HTML `<BODY` uppercase fragment root element (1993) IETF / W3C
        | b"<pli" // Apple XML Property List `<plist` root element (2001) Apple
        | b"<rss" // RSS 2.0 Syndication Feed root element (2002) Dave Winer / Harvard
        | b"<RSS" // RSS 2.0 Syndication Feed uppercase root element (2002) Dave Winer / Harvard
        | b"<fee" // RFC 4287 Atom Syndication Feed `<feed` root element (2005) IETF
        | b"<kml" // OGC Keyhole Markup Language root element (2004) Keyhole / Google / OGC
        | b"<gpx" // TopoGrafix GPS Exchange Format root element (2002) TopoGrafix
        | b"<osm" // OpenStreetMap XML root element (2005) Steve Coast / OSM
        | b"<COL" // Khronos COLLADA 3D Asset `<COLLADA` root element (2004) Sony / Khronos Group
        | b"<rdf" // W3C RDF/XML / RSS 1.0 `<rdf:RDF` root element (1999) W3C
        | b"<RDF" // W3C RDF/XML uppercase root element (1999) W3C
        | b"<xs:" // W3C XML Schema `<xs:schema` root element (2001) W3C
        | b"<xsd" // W3C XML Schema `<xsd:schema` root element (2001) W3C
        | b"<sch" // W3C XML Schema / ISO Schematron `<schema` root element (2001) W3C / ISO
        | b"<ass" // Microsoft WinSxS / ClickOnce `<assembly` manifest root element (2001) Microsoft
        | b"<pac" // NuGet / Chocolatey `<package` specification root element (2010) Microsoft
        | b"<job" // Windows Script File root element (1999) Microsoft
        | b"<jnl" // Java Network Launching Protocol `<jnlp` root element (2001) Sun Microsystems
        | b"<net" // KiCad / Eagle XML netlist root element (2010) KiCad
        | b"<dev" // UPnP `<device` description XML root element (2000) UPnP Forum
        | b"<gml" => xml_document(prefix, size), // OGC Geography Markup Language root element (2000) OGC

        // --- Intel MCS-86 Hexadecimal Object File Records (':' + 2-Hex-Digit Byte Count + Address High Byte) ---
        // Intel HEX records start with `':'` followed by byte count (`02` for extended address records,
        // `10` = 16 bytes for standard data lines, `00` for EOF) and the high nibble of the 16-bit load address:
        b":020" // `":02000002..."` / `":02000004..."` Extended Segment/Linear Address Record (1973) Intel
        | b":100" // `":100..."` 16-byte Data Record at `0x00xx` (1973) Intel
        | b":000" => intelhex(prefix), // `":00000001FF"` End-of-File Record (1973) Intel

        // --- MS-DOS & CP/M-80 .COM Flat Binary Executables (ORG 0x0100 Entry Prologues) ---
        // Flat `.COM` binaries lack a header magic; these 4-byte x86/Z80 instruction sequences at `@0`
        // identify standard compiler runtimes, DOS `INT 21h` entry stubs, and `.COM` executable packers:
        &[0x9C, 0x55, 0x56, 0x8C] // MS-DOS paragraph-relocatable `.COM` entry: `pushf; push bp; push si; mov ...` (1985) Microsoft
        | &[0xEB, 0x3D, 0x90, 0x00] // TheDraw self-displaying ANSI art `.COM` viewer stub: `jmp short 0x3F; nop; 0x00` (1986) TheSoft
        | &[0xE9, 0x79, 0x2C, 0x90] // Borland Turbo Pascal 3.0 MS-DOS `.COM` runtime entry: `jmp 0x2D7C; nop` (1985) Borland
        | &[0xC3, 0xE2, 0x20, 0xCD] // Borland Turbo Pascal 3.0 CP/M-80 `.COM` runtime entry: `jp 0x20E2; call ...` (1985) Borland
        | &[0xE9, 0x87, 0x2B, 0x90] // Borland Turbo Pascal 3.01A MS-DOS `.COM` runtime entry: `jmp 0x2C8A; nop` (1986) Borland
        | &[0xB4, 0x30, 0xCD, 0x21] // MS-DOS version check `.COM` entry: `mov ah, 30h; int 21h` (1983) Microsoft
        | &[0xEB, 0x0A, 0x00, 0x00] // ICE `.COM` executable packer stub (`jmp short +0x0A` over `0x0000` word) (1990) MichaelL
        | &[0xEB, 0x0A, 0x27, 0x7D] // ICE `.COM` executable packer stub (`jmp short +0x0A` over `"'}"` marker) (1990) MichaelL
        | &[0x68, 0x00, 0x01, 0xFD] // Eliashim T-PACK `.COM` decompressor stub: `push 0100h; std` (1992) Eliashim
        | &[0x50, 0x9C, 0xFC, 0xBE] // Thomasikkink SHRINK `.COM` decompressor stub: `push ax; pushf; cld; mov si,...` (1991) Thomasikkink
        | &[0x81, 0xFC, 0xE8, 0x80] => dos_com(prefix, size), // Walter Peichl COMPACT `.COM` decompressor stub: `cmp sp, 80E8h` (1990) Walter Peichl

        // =====================================================================
        // Section 5: 16-Bit & 8-Bit Magics Expanded to 32-Bit
        // =====================================================================

        // --- DOS MZ / Windows PE / Win16 NE / OS/2 LE & LX / ZM Executables ---
        // Bytes @0..2 are "MZ" (or "ZM"); bytes @2..4 are u16 LE `e_cblp` (bytes on last 512-byte
        // page of the DOS stub), a demoscene/packer tag overlapping the DOS header, or `PE\0\0` at @2.
        // (General 2-byte "MZ" / "ZM" fallback remains in Tier 2 `magic16`.)
        //
        // Group A: Windows PE/COFF Linker, Packer & Reflective Loader Stubs
        b"MZ\x90\x00" // e_cblp=0x0090 (144): Microsoft link.exe / LLVM lld-link / GNU ld (MinGW) / Rust / Go / .NET standard PE stub
        | b"MZP\x00" // e_cblp=0x0050 (80, "MZP\0"): Borland TLINK32 / Delphi / C++Builder PE stub
        | b"MZx\x00" // e_cblp=0x0078 (120): LLVM lld / minimal PE stub
        | b"MZ\x0A\x00" // e_cblp=0x000A (10): PowerBASIC / Watcom PE stub
        | b"MZk\x00" // e_cblp=0x006B (107): UniLink (ulink) 107-byte PE stub
        | b"MZ\x89\x00" // e_cblp=0x0089 (137): EuroASM 137-byte PE stub
        | b"MZ<\x00" // e_cblp=0x003C (60): NOS Packer / minimal 60-byte ("Win32 reqd.\r\n$") PE stub
        | b"MZfa" // e_cblp=0x6166: Farbrausch kkrunchy demoscene PE packer stub
        | b"MZR\xC3" // "MZR\xC3": BeRoEXEPacker demoscene PE stub (e_lfanew=0x0C)
        | b"MZ.l" // "MZ.l": Logicoma squishy / kkrunchy 64k intro PE stub ("MZ.logicoma.PE\0\0", e_lfanew=0x0C)
        | b"MZ. " // "MZ. ": Logicoma squishy 0.2.0 x64 64k intro PE stub ("MZ. squished by.\n...", e_lfanew=0x40)
        | b"MZ\x68\x00" // "MZ\x68\0": Crinkler 0.6–1.2 demoscene 4k intro PE linker (e_lfanew=0x0C)
        | b"MZ\xBF\x00" // "MZ\xBF\0": Crinkler 1.3–3.0 compatibility mode demoscene 4k intro PE linker (e_lfanew=0x0C)
        | b"MZ04" // "MZ04": Crinkler 0.4 demoscene 4k intro PE linker ("MZ04PE\0\0", e_lfanew=4)
        | b"MZ05" // "MZ05": Crinkler 0.5 demoscene 4k intro PE linker ("MZ05PE\0\0", e_lfanew=4)
        | b"MZ06" // "MZ06": Crinkler 0.6 demoscene 4k intro PE linker ("MZ06PE\0\0", e_lfanew=4)
        | b"MZ10" // "MZ10": Crinkler 1.0 demoscene 4k intro PE linker ("MZ10PE\0\0", e_lfanew=4)
        | b"MZ11" // "MZ11": Crinkler 1.1 demoscene 4k intro PE linker ("MZ11PE\0\0", e_lfanew=4)
        | b"MZ12" // "MZ12": Crinkler 1.2 demoscene 4k intro PE linker ("MZ12PE\0\0", e_lfanew=4)
        | b"MZ13" // "MZ13": Crinkler 1.3 demoscene 4k intro PE linker ("MZ13PE\0\0", e_lfanew=4)
        | b"MZ14" // "MZ14": Crinkler 1.4 demoscene 4k intro PE linker ("MZ14PE\0\0", e_lfanew=4)
        | b"MZ20" // "MZ20": Crinkler 2.0 demoscene 4k intro PE linker ("MZ20PE\0\0", e_lfanew=4)
        | b"MZ21" // "MZ21": Crinkler 2.1 demoscene 4k intro PE linker ("MZ21PE\0\0", e_lfanew=4)
        | b"MZ22" // "MZ22": Crinkler 2.2 demoscene 4k intro PE linker ("MZ22PE\0\0", e_lfanew=4)
        | b"MZ23" // "MZ23": Crinkler 2.3 demoscene 4k intro PE linker ("MZ23PE\0\0", e_lfanew=4)
        | b"MZ30" // "MZ30": Crinkler 3.0 demoscene 4k intro PE linker ("MZ30PE\0\0", e_lfanew=4)
        | b"MZ31" // "MZ31": Crinkler 3.1 demoscene 4k intro PE linker ("MZ31PE\0\0", e_lfanew=4)
        | b"MZPE" // "MZPE": TinyPE maximally-collapsed MZ + PE\0\0 header overlapping at @2 (e_lfanew=2)
        | b"MZAR" // "MZAR": x64 Reflective DLL bootstrap ("MZAR\x55\x48\x89\xE5", dec ebp; pop edx; push rbp)
        | b"MZ\xE8\x00" // "MZ\xE8\0": x86 Reflective DLL call $+5 bootstrap ("MZ\xE8\x00\x00\x00\x00\x5B")
        | b"MZRE" // "MZRE": Stephen Fewer / Metasploit Reflective DLL bootstrap ("MZRE\xE8\x00\x00\x00\x00")
        | b"MZER" // "MZER": pe_to_shellcode (pe2shc) Reflective PE bootstrap ("MZER\xE8\x00\x00\x00\x00")
        // Group B: Win16 New Executable (NE) & Font Resource (.FON / .FOT) Stubs
        | b"MZ\x80\x00" // e_cblp=0x0080 (128): Windows 1.x–3.x .FON font resource & Microsoft Win16 / OS/2 1.x 128-byte NE stub
        | b"MZ\x01\x00" // e_cblp=0x0001 (1): Windows 3.1 .FOT TrueType Font Resource 128-byte NE wrapper stub
        | b"MZ`\x00" // e_cblp=0x0060 (96): Microsoft Win16 96-byte NE stub
        | b"MZ\xB8\x00" // e_cblp=0x00B8 (184): Windows 3.x .FON font resource & Win16 184-byte NE stub
        | b"MZ\xF8\x00" // e_cblp=0x00F8 (248): Microsoft Visual Basic 1.0 Win16 NE stub
        | b"MZ\xC4\x01" // e_cblp=0x01C4 (452): Microsoft Visual Basic 2.0 Win16 NE stub
        // Group C: 32-Bit Linear Executable (LE / LX), DOS Extender & VxD Bundle Stubs
        | b"MZ\xA0\x01" // e_cblp=0x01A0 (416): Rational Systems / Tenberry DOS/4GW 32-bit LE DOS extender stub (1991)
        | b"MZ\x90\x01" // e_cblp=0x0190 (400): Narech Koumar DOS/32A LE linear extender stub (1996)
        | b"MZ\xF8\x01" // e_cblp=0x01F8 (504): OpenWatcom / CauseWay / WDOSX LE DOS extender stub
        | b"MZ\xC0\x00" // e_cblp=0x00C0 (192): Windows 3.x WIN386.EXE W3 VxD bundle & VxD LE driver stub
        | b"MZla" // e_cblp=0x616C: Lattice C / DOS MZ stub
        // Group D: 16-Bit MS-DOS MZ & IBM Pascal ZM Executables
        | b"MZ\x00\x00" // e_cblp=0x0000 (0): Full 512-byte page-aligned MS-DOS MZ / PE stub
        | b"MZ@\x00" // e_cblp=0x0040 (64): Minimal 64-byte header-only MS-DOS MZ executable
        | b"MZ\x00\x01" // e_cblp=0x0100 (256): 256-byte last-page MS-DOS MZ executable
        | b"MZ\x00\x02" // e_cblp=0x0200 (512): 512-byte last-page MS-DOS MZ executable
        | b"MZ\xE0\x00" // e_cblp=0x00E0 (224): Charles Scheffold & Thomas Pytel PMODE/W DOS extender MZ stub (1994)
        | b"MZ,\x00" // e_cblp=0x002C (44): Narech Koumar DOS/32A LC compressed extender MZ stub (1996)
        | b"MZt\x00" // e_cblp=0x0074 (116): DJGPP GO32 / COFF 32-bit DOS extender MZ stub (1991)
        | b"MZP\x01" // e_cblp=0x0150 (336): Phar Lap 286|386 DOS-Extender MZ stub (1986)
        | b"MZ\xFC\x01" // e_cblp=0x01FC (508): Microsoft EXEPACK-compressed MS-DOS MZ executable
        | b"ZM\x04\x00" => pe_or_dos_mz(prefix, ctx), // e_cblp=0x0004: IBM PC Pascal Compiler v1.00/v2.00 ZM DOS Executable (1981) IBM / Microsoft

        // --- RFC 1952 GZIP Compressed Streams (1F 8B + CM=0x08 Deflate + FLG Bitmask) ---
        // Bytes @0..2 are `1F 8B`, byte @2 is `CM=8` (Deflate), and byte @3 is the RFC 1952 `FLG`
        // bitmask (`0x01`=FTEXT, `0x02`=FHCRC, `0x04`=FEXTRA, `0x08`=FNAME, `0x10`=FCOMMENT):
        &[0x1F, 0x8B, 0x08, 0x00] // CM=8 (Deflate), FLG=0x00 (bare stream, no optional header fields)
        | &[0x1F, 0x8B, 0x08, 0x01] // CM=8 (Deflate), FLG=0x01 (FTEXT ASCII hint)
        | &[0x1F, 0x8B, 0x08, 0x02] // CM=8 (Deflate), FLG=0x02 (FHCRC 16-bit header CRC)
        | &[0x1F, 0x8B, 0x08, 0x04] // CM=8 (Deflate), FLG=0x04 (FEXTRA: SAMtools BGZF / dictzip / Unity .unitypackage)
        | &[0x1F, 0x8B, 0x08, 0x08] // CM=8 (Deflate), FLG=0x08 (FNAME original filename, standard `gzip` CLI default)
        | &[0x1F, 0x8B, 0x08, 0x09] // CM=8 (Deflate), FLG=0x09 (FTEXT | FNAME)
        | &[0x1F, 0x8B, 0x08, 0x0A] // CM=8 (Deflate), FLG=0x0A (FHCRC | FNAME)
        | &[0x1F, 0x8B, 0x08, 0x0C] // CM=8 (Deflate), FLG=0x0C (FEXTRA | FNAME)
        | &[0x1F, 0x8B, 0x08, 0x10] // CM=8 (Deflate), FLG=0x10 (FCOMMENT header comment, e.g. OpenBSD `signify -z`)
        | &[0x1F, 0x8B, 0x08, 0x14] // CM=8 (Deflate), FLG=0x14 (FEXTRA | FCOMMENT, e.g. Kubernetes Helm Chart `.tgz`)
        | &[0x1F, 0x8B, 0x08, 0x18] // CM=8 (Deflate), FLG=0x18 (FNAME | FCOMMENT, e.g. Unity WebGL `.gz`)
        | &[0x1F, 0x8B, 0x08, 0x1C] => gzip(prefix), // CM=8 (Deflate), FLG=0x1C (FEXTRA | FNAME | FCOMMENT, e.g. `gztool` index)

        // --- bzip2 Burrows-Wheeler Compressed Streams ("BZh" + Block Size '1'..'9') ---
        // Bytes @0..3 are `"BZh"` (`'h'` = Huffman-coded BWT), byte @3 is `'1'..'9'` (100 KB..900 KB block size):
        b"BZh1" // Level 1 (100 KB BWT block size) (1996) Julian Seward
        | b"BZh2" // Level 2 (200 KB BWT block size) (1996) Julian Seward
        | b"BZh3" // Level 3 (300 KB BWT block size) (1996) Julian Seward
        | b"BZh4" // Level 4 (400 KB BWT block size) (1996) Julian Seward
        | b"BZh5" // Level 5 (500 KB BWT block size) (1996) Julian Seward
        | b"BZh6" // Level 6 (600 KB BWT block size) (1996) Julian Seward
        | b"BZh7" // Level 7 (700 KB BWT block size) (1996) Julian Seward
        | b"BZh8" // Level 8 (800 KB BWT block size) (1996) Julian Seward
        | b"BZh9" => bzip(prefix), // Level 9 (900 KB BWT block size, default) (1996) Julian Seward

        // --- RFC 1950 ZLIB Streams (CMF + FLG + Initial Deflate Block Header) ---
        // Bytes @0..2 are the RFC 1950 header (`0x78` = 32 KiB Deflate window; `0x01`/`0x9C`/`0xDA` =
        // level 0–1 / 6 default / 9 best compression), and bytes @2..4 are the first Deflate block header:
        &[0x78, 0x9C, 0xEC, 0xDD] // Level 6 default (`78 9C`) + dynamic Huffman block (`EC DD`)
        | &[0x78, 0x9C, 0x63, 0x60] // Level 6 default (`78 9C`) + fixed Huffman literal `0x00` (`63 60`)
        | &[0x78, 0x9C, 0x63, 0x64] // Level 6 default (`78 9C`) + fixed Huffman Sub-filter scanline (`63 64`)
        | &[0x78, 0xDA, 0xED, 0xDC] // Level 9 best (`78 DA`) + non-final dynamic Huffman block (`ED DC`)
        | &[0x78, 0xDA, 0xED, 0xDD] // Level 9 best (`78 DA`) + final dynamic Huffman block (`ED DD`)
        | &[0x78, 0xDA, 0x63, 0x60] // Level 9 best (`78 DA`) + fixed Huffman literal `0x00` (`63 60`)
        | &[0x78, 0x01, 0xED, 0xC0] // Level 1 fastest (`78 01`) + dynamic Huffman block (`ED C0`)
        | &[0x78, 0x01, 0x01, 0x00] => zlibstream(prefix, size), // Level 0 store (`78 01`) + final stored uncompressed block (`BFINAL=1, BTYPE=00`)

        // --- BSD Unix compress LZW Streams (1F 9D + 0x90 Flags + First Literal Byte) ---
        // Bytes @0..2 are `1F 9D`, byte @2 is `0x90` (`block_mode=1`, `maxbits=16`), and byte @3 is
        // the first uncompressed byte (since the first 9-bit LZW code is a 7-bit ASCII literal):
        &[0x1F, 0x9D, 0x90, 0x00] // 16-bit LZW block mode (`0x90`), first byte = `0x00` (null-prefixed or TAR stream)
        | &[0x1F, 0x9D, 0x90, b'.'] // 16-bit LZW block mode (`0x90`), first byte = `'.'` (`"./..."` `.tar.Z` archive)
        | &[0x1F, 0x9D, 0x90, b'/'] // 16-bit LZW block mode (`0x90`), first byte = `'/'` (`"/*..."` C source or absolute path)
        | &[0x1F, 0x9D, 0x90, b'#'] // 16-bit LZW block mode (`0x90`), first byte = `'#'` (`"#!..."` script or comment)
        | &[0x1F, 0x9D, 0x90, b'p'] // 16-bit LZW block mode (`0x90`), first byte = `'p'` (`"pax_global_header"` or package)
        | &[0x1F, 0x9D, 0x90, b'u'] => unixcompress(prefix), // 16-bit LZW block mode (`0x90`), first byte = `'u'` (`"usr/..."` `.tar.Z` archive)

        // --- ITU-T T.81 / ISO/IEC 10918-1 JPEG Codestreams (FF D8 SOI + FF xx First Marker) ---
        // Bytes @0..2 are `FF D8` (`SOI` Start of Image); bytes @2..4 are the first marker (`FF xx`):
        &[0xFF, 0xD8, 0xFF, 0xE0] // SOI + APP0 (`FF E0`): JFIF (1991) / JFXX / AVI1 Motion JPEG
        | &[0xFF, 0xD8, 0xFF, 0xE1] // SOI + APP1 (`FF E1`): JEIDA/CIPA Exif (1998) / Adobe XMP (2001)
        | &[0xFF, 0xD8, 0xFF, 0xE2] // SOI + APP2 (`FF E2`): ICC_PROFILE color profile / Kodak FlashPix / Canon CIFF
        | &[0xFF, 0xD8, 0xFF, 0xE3] // SOI + APP3 (`FF E3`): JPS Stereoscopic / Kodak Meta
        | &[0xFF, 0xD8, 0xFF, 0xE4] // SOI + APP4 (`FF E4`): Scalado / FlashPix extension
        | &[0xFF, 0xD8, 0xFF, 0xE5] // SOI + APP5 (`FF E5`): Ricoh RMETA
        | &[0xFF, 0xD8, 0xFF, 0xE6] // SOI + APP6 (`FF E6`): Toshiba EPPIM / GoPro GPMF
        | &[0xFF, 0xD8, 0xFF, 0xE7] // SOI + APP7 (`FF E7`): Pentax / Qualcomm camera data
        | &[0xFF, 0xD8, 0xFF, 0xE8] // SOI + APP8 (`FF E8`): ITU-T T.84 SPIFF (Still Picture Interchange File Format, 1996)
        | &[0xFF, 0xD8, 0xFF, 0xE9] // SOI + APP9 (`FF E9`): MediaJukebox
        | &[0xFF, 0xD8, 0xFF, 0xEA] // SOI + APP10 (`FF EA`): Photostudio ActiveObject comment
        | &[0xFF, 0xD8, 0xFF, 0xEB] // SOI + APP11 (`FF EB`): JPEG-HDR / ISO/IEC 19566-5 JUMBF (C2PA)
        | &[0xFF, 0xD8, 0xFF, 0xEC] // SOI + APP12 (`FF EC`): Agfa/Olympus Picture Info / Adobe Save-for-Web "Ducky"
        | &[0xFF, 0xD8, 0xFF, 0xED] // SOI + APP13 (`FF ED`): Adobe Photoshop 3.0 Image Resource Block (IRB / IPTC)
        | &[0xFF, 0xD8, 0xFF, 0xEE] // SOI + APP14 (`FF EE`): Adobe DCT color transform marker
        | &[0xFF, 0xD8, 0xFF, 0xEF] // SOI + APP15 (`FF EF`): Lemke Software GraphicConverter
        | &[0xFF, 0xD8, 0xFF, 0xFE] // SOI + COM (`FF FE`): Comment segment
        | &[0xFF, 0xD8, 0xFF, 0xDB] // SOI + DQT (`FF DB`): Define Quantization Table (headerless raw JPEG)
        | &[0xFF, 0xD8, 0xFF, 0xC0] // SOI + SOF0 (`FF C0`): Baseline DCT frame
        | &[0xFF, 0xD8, 0xFF, 0xC2] // SOI + SOF2 (`FF C2`): Progressive DCT frame
        | &[0xFF, 0xD8, 0xFF, 0xC4] // SOI + DHT (`FF C4`): Define Huffman Table
        | &[0xFF, 0xD8, 0xFF, 0xDD] // SOI + DRI (`FF DD`): Define Restart Interval
        | &[0xFF, 0xD8, 0xFF, 0xFF] => jpeg(prefix, size), // SOI + `0xFF` fill byte padding before first marker

        // --- Microsoft Windows & IBM OS/2 Device-Independent Bitmap ("BM" + u16 LE at @2..4) ---
        // Bytes @0..2 are `"BM"`; bytes @2..4 are the low 16 bits of `bfSize` (total file size in LE,
        // or in OS/2 1.x/2.x often set to `cbSize`, the combined file+DIB header size). Because
        // uncompressed pixel payloads for power-of-two dimensions (e.g. 128x128x32bpp = 64 KiB,
        // 256x256, 512x512, 1024x1024) are exact multiples of 65,536 (`0x10000`) bytes, `(bfSize & 0xFFFF)`
        // equals the exact header/palette size `bfOffBits` (Group A) or `bfOffBits + W*H*bpp/8` for
        // standard icon/tile dimensions (Group B). General 2-byte `"BM"` fallback remains in `magic16`.
        //
        // Group A: Standard Windows (V3 / V4 / V5) & OS/2 (1.x / 2.x) Header Sizes (+ 64 KiB-Aligned Payloads)
        b"BM\x36\x00" // 0x0036 (54 = 14 + 40): Windows V3 BITMAPINFOHEADER + 64 KiB-aligned pixel payload
        | b"BM\x3E\x00" // 0x003E (62 = 54 + 8): Windows V3 1-bpp monochrome header + 2-entry RGBQUAD palette
        | b"BM\x42\x00" // 0x0042 (66 = 54 + 12): Windows V3 BI_BITFIELDS header (54B + 3x4B RGB masks)
        | b"BM\x46\x00" // 0x0046 (70 = 54 + 16): Windows V3 BI_ALPHABITFIELDS header (54B + 4x4B RGBA masks)
        | b"BM\x76\x00" // 0x0076 (118 = 54 + 64): Windows V3 4-bpp 16-color palette (16x4B) + 64 KiB-aligned payload
        | b"BM\x36\x04" // 0x0436 (1078 = 54 + 1024): Windows V3 8-bpp 256-color palette (256x4B) + 64 KiB-aligned payload
        | b"BM\x7A\x00" // 0x007A (122 = 14 + 108): Windows 95/NT4 V4 BITMAPV4HEADER + 64 KiB-aligned payload
        | b"BM\x7A\x80" // 0x807A (32890 = 122 + 32768): Windows V4 BITMAPV4HEADER + 32 KiB-aligned payload
        | b"BM\x8A\x00" // 0x008A (138 = 14 + 124): Windows 98/2000 V5 BITMAPV5HEADER + 64 KiB-aligned payload
        | b"BM\x96\x00" // 0x0096 (150 = 138 + 12): Windows V5 BITMAPV5HEADER + 3x4B BI_BITFIELDS masks + 64 KiB-aligned payload
        | b"BM\x98\x00" // 0x0098 (152 = 138 + 14): Windows V5 BITMAPV5HEADER with 2-byte aligned profile/masks
        | b"BM\x9A\x00" // 0x009A (154 = 138 + 16): Windows V5 BITMAPV5HEADER + 4x4B RGBA bitmasks + 64 KiB-aligned payload
        | b"BM\x1A\x00" // 0x001A (26 = 14 + 12): OS/2 1.x BITMAPCOREHEADER (`cbSize = 26`)
        | b"BM\x1E\x00" // 0x001E (30 = 26 + 4): OS/2 1.x 1-bpp header with 2-entry RGBTRIPLE palette (`cbSize = 30`)
        | b"BM\x4E\x00" // 0x004E (78 = 14 + 64): OS/2 2.x BITMAPINFOHEADER2 (`cbSize = 78`)
        // Group B: Standard Windows V3 / V4 / V5 Icon, Cursor & Tile Dimensions (`bfOffBits + W * H * bpp / 8`)
        | b"BM\x76\x02" // 0x0276 (630 = 118 + 512): Windows V3 32x32x4bpp 16-color icon/tile
        | b"BM\x36\x03" // 0x0336 (822 = 54 + 768): Windows V3 16x16x24bpp icon/tile
        | b"BM\x38\x03" // 0x0338 (824 = 54 + 768 + 2): Windows V3 16x16x24bpp icon/tile + 2B EOF alignment padding
        | b"BM\x76\x06" // 0x0676 (1654 = 118 + 1536): Windows V3 24x24x8bpp / 64x32x4bpp icon/tile
        | b"BM\x36\x08" // 0x0836 (2102 = 1078 + 1024): Windows V3 32x32x8bpp 256-color icon/tile
        | b"BM\x36\x09" // 0x0936 (2358 = 54 + 2304): Windows V3 24x32x24bpp / small icon/tile
        | b"BM\x3E\x0B" // 0x0B3E (2878 = 62 + 2816): Windows V3 4-bpp/1-bpp small icon/tile
        | b"BM\x36\x0C" // 0x0C36 (3126 = 54 + 3072): Windows V3 32x32x24bpp icon/tile
        | b"BM\x38\x0C" // 0x0C38 (3128 = 54 + 3072 + 2): Windows V3 32x32x24bpp icon/tile + 2B EOF alignment padding
        | b"BM\x36\x10" // 0x1036 (4150 = 54 + 4096): Windows V3 32x32x32bpp icon/tile
        | b"BM\xF6\x12" // 0x12F6 (4854 = 54 + 4800): Windows V3 40x40x24bpp icon/tile
        | b"BM\x36\x14" // 0x1436 (5174 = 1078 + 4096): Windows V3 64x64x8bpp 256-color tile
        | b"BM\x38\x14" // 0x1438 (5176 = 1078 + 4096 + 2): Windows V3 64x64x8bpp tile + 2B EOF alignment padding
        | b"BM\x36\x24" // 0x2436 (9270 = 54 + 9216): Windows V3 48x48x32bpp icon/tile
        | b"BM\x36\x30" // 0x3036 (12342 = 54 + 12288): Windows V3 64x64x24bpp tile
        | b"BM\x36\x40" // 0x4036 (16438 = 54 + 16384): Windows V3 64x64x32bpp tile
        | b"BM\x7A\x03" // 0x037A (890 = 122 + 768): Windows V4 16x16x24bpp icon/tile
        | b"BM\x7A\x04" // 0x047A (1146 = 122 + 1024): Windows V4 16x16x32bpp icon/tile
        | b"BM\x7A\x0C" // 0x0C7A (3194 = 122 + 3072): Windows V4 32x32x24bpp icon/tile
        | b"BM\x7A\x40" // 0x407A (16506 = 122 + 16384): Windows V4 64x64x32bpp tile
        | b"BM\x7E\x40" // 0x407E (16510 = 126 + 16384): Windows V4 64x64x32bpp BI_BITFIELDS tile
        | b"BM\x7A\xC0" // 0xC07A (49274 = 122 + 49152): Windows V4 128x128x24bpp tile
        | b"BM\x8A\x04" // 0x048A (1162 = 138 + 1024): Windows V5 16x16x32bpp icon/tile
        | b"BM\x8A\x10" // 0x108A (4234 = 138 + 4096): Windows V5 32x32x32bpp icon/tile
        | b"BM\x8A\x40" // 0x408A (16522 = 138 + 16384): Windows V5 64x64x32bpp tile
        | b"BM\x96\x10" // 0x1096 (4246 = 150 + 4096): Windows V5 32x32x32bpp BI_BITFIELDS icon/tile
        | b"BM\x96\x40" // 0x4096 (16534 = 150 + 16384): Windows V5 64x64x32bpp BI_BITFIELDS tile
        | b"BM\x96\x90" // 0x9096 (37014 = 150 + 36864): Windows V5 96x96x32bpp BI_BITFIELDS avatar/tile
        // Group C: Minimal 1x1 / Strip / Unfilled-Size Bitmaps
        | b"BM\x00\x00" // 0x0000: bfSize == 0 (unfilled size field or exact 64 KiB-multiple file size)
        | b"BM\x3C\x00" // 0x003C (60 = 54 + 6): Minimal Windows V3 1x1/2x1 24-bpp bitmap (54B header + 6B padded row)
        | b"BM\x4A\x00" // 0x004A (74 = 62 + 12): Minimal Windows V3 1-bpp monochrome 3-row icon/glyph
        | b"BM\x7E\x00" // 0x007E (126 = 122 + 4): Minimal Windows V4 1x1x32bpp single-pixel bitmap
        | b"BM\x8E\x00" // 0x008E (142 = 138 + 4): Minimal Windows V5 1x1x32bpp single-pixel bitmap
        | b"BM\xF6\x00" => bmp(prefix), // 0x00F6 (246 = 122 + 124): Windows V4 1x31x32bpp UI gradient strip

        // --- Netpbm Portable AnyMap ("P1".."P7" + LF + Comment or Header Token) ---
        // Bytes @0..2 are `"P1"`..`"P7"`, byte @2 is `\n`, and byte @3 is `'#'` (creator comment) or width digit:
        b"P1\n#" // Netpbm Portable BitMap ASCII (`P1`) with creator comment line (1988) Jef Poskanzer
        | b"P2\n#" // Netpbm Portable GrayMap ASCII (`P2`) with creator comment line (1988) Jef Poskanzer
        | b"P3\n#" // Netpbm Portable PixMap ASCII (`P3`) with creator comment line (1988) Jef Poskanzer
        | b"P4\n#" // Netpbm Portable BitMap Binary (`P4`) with creator comment line (1988) Jef Poskanzer
        | b"P5\n#" // Netpbm Portable GrayMap Binary (`P5`) with creator comment line (1988) Jef Poskanzer
        | b"P6\n#" // Netpbm Portable PixMap Binary (`P6`) with creator comment line (1988) Jef Poskanzer
        | b"P6\n2" => pbm(prefix), // Netpbm Portable PixMap Binary (`P6`) starting with width `2xx` (1988) Jef Poskanzer

        // --- Macromedia / Adobe Shockwave Flash SWF ("FWS" / "CWS" / "ZWS" + Version Byte) ---
        // Bytes @0..3 identify compression (`"FWS"` = uncompressed, `"CWS"` = zlib-compressed Flash 6+,
        // `"ZWS"` = LZMA-compressed Flash 13+), and byte @3 is the 1-byte SWF format version:
        b"FWS\x01" // SWF v1 uncompressed (1996 FutureSplash Animator)
        | b"FWS\x02" // SWF v2 uncompressed (1997 Macromedia Flash 2)
        | b"FWS\x03" // SWF v3 uncompressed (1998 Macromedia Flash 3)
        | b"FWS\x04" // SWF v4 uncompressed (1999 Macromedia Flash 4)
        | b"FWS\x05" // SWF v5 uncompressed (2000 Macromedia Flash 5)
        | b"FWS\x06" // SWF v6 uncompressed (2002 Macromedia Flash MX)
        | b"FWS\x07" // SWF v7 uncompressed (2003 Macromedia Flash MX 2004)
        | b"FWS\x08" // SWF v8 uncompressed (2005 Macromedia Flash 8)
        | b"FWS\x09" // SWF v9 uncompressed (2006 Adobe Flash CS3 / ActionScript 3)
        | b"FWS\x0A" // SWF v10 uncompressed (2008 Adobe Flash CS4)
        | b"CWS\x06" // SWF v6 zlib-compressed (2002 Macromedia Flash MX)
        | b"CWS\x07" // SWF v7 zlib-compressed (2003 Macromedia Flash MX 2004)
        | b"CWS\x08" // SWF v8 zlib-compressed (2005 Macromedia Flash 8)
        | b"CWS\x09" // SWF v9 zlib-compressed (2006 Adobe Flash CS3)
        | b"CWS\x0A" // SWF v10 zlib-compressed (2008 Adobe Flash CS4)
        | b"CWS\x0B" // SWF v11 zlib-compressed (2010 Adobe Flash CS5)
        | b"CWS\x0C" // SWF v12 zlib-compressed (2011 Adobe Flash CS5.5)
        | b"CWS\x0D" // SWF v13 zlib-compressed (2012 Adobe Flash CS6)
        | b"CWS\x0E" // SWF v14 zlib-compressed (2013 Adobe Flash CC)
        | b"CWS\x0F" // SWF v15 zlib-compressed (2014 Adobe Flash CC)
        | b"ZWS\x0D" // SWF v13 LZMA-compressed (2012 Adobe Flash CS6)
        | b"ZWS\x0E" // SWF v14 LZMA-compressed (2013 Adobe Flash CC)
        | b"ZWS\x0F" // SWF v15 LZMA-compressed (2014 Adobe Flash CC)
        | b"ZWS\x10" // SWF v16 LZMA-compressed (2014 Adobe Flash CC)
        | b"ZWS\x11" // SWF v17 LZMA-compressed (2015 Adobe Flash CC)
        | b"ZWS\x12" // SWF v18 LZMA-compressed (2015 Adobe Animate)
        | b"ZWS\x13" // SWF v19 LZMA-compressed (2015 Adobe Animate)
        | b"ZWS\x14" => swf(prefix), // SWF v20 LZMA-compressed (2016 Adobe Animate)

        // --- Microsoft / System V COFF Object Files (u16 LE Machine + u16 LE NumberOfSections) ---
        // Bytes @0..2 are the COFF `Machine` ID in little-endian (`0x014C` = i386, `0x8664` = x86-64/AMD64,
        // `0xAA64` = ARM64), and bytes @2..4 are `NumberOfSections` in little-endian (`1..=12` / `1..=4`).
        // (Objects with more sections or other COFF architectures fall back to Tier 2 `magic16`.)
        &[0x4C, 0x01, 0x01, 0x00] // COFF i386 (`0x014C` LE), 1 section
        | &[0x4C, 0x01, 0x02, 0x00] // COFF i386 (`0x014C` LE), 2 sections
        | &[0x4C, 0x01, 0x03, 0x00] // COFF i386 (`0x014C` LE), 3 sections
        | &[0x4C, 0x01, 0x04, 0x00] // COFF i386 (`0x014C` LE), 4 sections
        | &[0x4C, 0x01, 0x05, 0x00] // COFF i386 (`0x014C` LE), 5 sections
        | &[0x4C, 0x01, 0x06, 0x00] // COFF i386 (`0x014C` LE), 6 sections
        | &[0x4C, 0x01, 0x07, 0x00] // COFF i386 (`0x014C` LE), 7 sections
        | &[0x4C, 0x01, 0x08, 0x00] // COFF i386 (`0x014C` LE), 8 sections
        | &[0x4C, 0x01, 0x09, 0x00] // COFF i386 (`0x014C` LE), 9 sections
        | &[0x4C, 0x01, 0x0A, 0x00] // COFF i386 (`0x014C` LE), 10 sections
        | &[0x4C, 0x01, 0x0B, 0x00] // COFF i386 (`0x014C` LE), 11 sections
        | &[0x4C, 0x01, 0x0C, 0x00] // COFF i386 (`0x014C` LE), 12 sections
        | &[0x64, 0x86, 0x01, 0x00] // COFF x86-64 / AMD64 (`0x8664` LE), 1 section
        | &[0x64, 0x86, 0x02, 0x00] // COFF x86-64 / AMD64 (`0x8664` LE), 2 sections
        | &[0x64, 0x86, 0x03, 0x00] // COFF x86-64 / AMD64 (`0x8664` LE), 3 sections
        | &[0x64, 0x86, 0x04, 0x00] // COFF x86-64 / AMD64 (`0x8664` LE), 4 sections
        | &[0x64, 0x86, 0x05, 0x00] // COFF x86-64 / AMD64 (`0x8664` LE), 5 sections
        | &[0x64, 0x86, 0x06, 0x00] // COFF x86-64 / AMD64 (`0x8664` LE), 6 sections
        | &[0x64, 0x86, 0x07, 0x00] // COFF x86-64 / AMD64 (`0x8664` LE), 7 sections
        | &[0x64, 0x86, 0x08, 0x00] // COFF x86-64 / AMD64 (`0x8664` LE), 8 sections
        | &[0x64, 0x86, 0x09, 0x00] // COFF x86-64 / AMD64 (`0x8664` LE), 9 sections
        | &[0x64, 0x86, 0x0A, 0x00] // COFF x86-64 / AMD64 (`0x8664` LE), 10 sections
        | &[0x64, 0x86, 0x0B, 0x00] // COFF x86-64 / AMD64 (`0x8664` LE), 11 sections
        | &[0x64, 0x86, 0x0C, 0x00] // COFF x86-64 / AMD64 (`0x8664` LE), 12 sections
        | &[0x64, 0xAA, 0x01, 0x00] // COFF ARM64 (`0xAA64` LE), 1 section
        | &[0x64, 0xAA, 0x02, 0x00] // COFF ARM64 (`0xAA64` LE), 2 sections
        | &[0x64, 0xAA, 0x03, 0x00] // COFF ARM64 (`0xAA64` LE), 3 sections
        | &[0x64, 0xAA, 0x04, 0x00] => coff(prefix, size), // COFF ARM64 (`0xAA64` LE), 4 sections

        // --- IBM AIX Extended COFF Object Files (u16 BE f_magic + u16 BE f_nscns) ---
        // Bytes @0..2 are the big-endian XCOFF magic (`0x01DF` = 32-bit XCOFF, `0x01F7` = 64-bit XCOFF64),
        // and bytes @2..4 are `f_nscns` (`u16 BE` section count `1..=4`, typically `.text`, `.data`, `.bss`, `.loader`):
        &[0x01, 0xDF, 0x00, 0x01] // 32-bit IBM AIX XCOFF (`0x01DF` BE) with 1 section (`f_nscns = 1`)
        | &[0x01, 0xDF, 0x00, 0x02] // 32-bit IBM AIX XCOFF (`0x01DF` BE) with 2 sections (`f_nscns = 2`)
        | &[0x01, 0xDF, 0x00, 0x03] // 32-bit IBM AIX XCOFF (`0x01DF` BE) with 3 sections (`f_nscns = 3`)
        | &[0x01, 0xDF, 0x00, 0x04] // 32-bit IBM AIX XCOFF (`0x01DF` BE) with 4 sections (`f_nscns = 4`)
        | &[0x01, 0xF7, 0x00, 0x01] // 64-bit IBM AIX XCOFF64 (`0x01F7` BE) with 1 section (`f_nscns = 1`)
        | &[0x01, 0xF7, 0x00, 0x02] // 64-bit IBM AIX XCOFF64 (`0x01F7` BE) with 2 sections (`f_nscns = 2`)
        | &[0x01, 0xF7, 0x00, 0x03] // 64-bit IBM AIX XCOFF64 (`0x01F7` BE) with 3 sections (`f_nscns = 3`)
        | &[0x01, 0xF7, 0x00, 0x04] => xcoff(prefix), // 64-bit IBM AIX XCOFF64 (`0x01F7` BE) with 4 sections (`f_nscns = 4`)

        // --- Legacy Microsoft Office (Standalone Excel BIFF BOF & Word for Windows FIB) ---
        // Excel BIFF: bytes @0..2 are u16 LE `BOF` record type (`0x0009` BIFF2, `0x0209` BIFF3, `0x0409` BIFF4,
        // `0x0809` BIFF5/8), and bytes @2..4 are u16 LE `BOF` record payload length (`4`, `6`, `8`, or `16` bytes):
        &[0x09, 0x00, 0x04, 0x00] // Excel 2.x BIFF2 standalone worksheet (`BOF=0x0009` LE, `len=4` LE) (1987) Microsoft
        | &[0x09, 0x02, 0x06, 0x00] // Excel 3.0 BIFF3 standalone worksheet (`BOF=0x0209` LE, `len=6` LE) (1990) Microsoft
        | &[0x09, 0x04, 0x06, 0x00] // Excel 4.0 BIFF4 standalone worksheet/workbook (`BOF=0x0409` LE, `len=6` LE) (1992) Microsoft
        | &[0x09, 0x08, 0x08, 0x00] // Excel 5.0/95 BIFF5 stream (`BOF=0x0809` LE, `len=8` LE) (1993) Microsoft
        | &[0x09, 0x08, 0x10, 0x00] => xls_biff(prefix, size), // Excel 97–2003 BIFF8 raw stream (`BOF=0x0809` LE, `len=16` LE) (1997) Microsoft
        // Word for Windows 1.x/2.0: bytes @0..2 are u16 LE `wIdent` magic, bytes @2..4 are u16 LE `nFib` version:
        &[0x9B, 0xA5, 0x21, 0x00] // Word for Windows 1.0 pre-release (`wIdent=0xA59B` LE, `nFib=0x0021` LE) (1989) Microsoft
        | &[0xDB, 0xA5, 0x21, 0x00] // Word for Windows 1.0 / 1.1 (`wIdent=0xA5DB` LE, `nFib=0x0021` LE) (1989) Microsoft
        | &[0xDC, 0xA5, 0x2D, 0x00] => word_win12(prefix, size), // Word for Windows 2.0 (`wIdent=0xA5DC` LE, `nFib=0x002D` LE) (1991) Microsoft

        // --- RFC 8949 Self-Described CBOR (D9 D9 F7 Tag 55799 + Root Data Item Initial Byte) ---
        // Bytes @0..3 are `D9 D9 F7` (CBOR Major Type 6, 2-byte Tag 55799), and byte @3 is the initial
        // byte of the root CBOR item (Major Type 5 map `0xA0..0xBF`, Major Type 4 array `0x81..0x9F`,
        // or Major Type 6 nested semantic tag `0xD2`..`0xD9`). General `D9 D9` fallback in `magic16`:
        &[0xD9, 0xD9, 0xF7, 0xA0] // Self-Described CBOR + empty map (`0xA0`, 0 key-value pairs)
        | &[0xD9, 0xD9, 0xF7, 0xA1] // Self-Described CBOR + 1-pair map (`0xA1`)
        | &[0xD9, 0xD9, 0xF7, 0xA2] // Self-Described CBOR + 2-pair map (`0xA2`)
        | &[0xD9, 0xD9, 0xF7, 0xA3] // Self-Described CBOR + 3-pair map (`0xA3`)
        | &[0xD9, 0xD9, 0xF7, 0xA4] // Self-Described CBOR + 4-pair map (`0xA4`)
        | &[0xD9, 0xD9, 0xF7, 0xA5] // Self-Described CBOR + 5-pair map (`0xA5`)
        | &[0xD9, 0xD9, 0xF7, 0xA6] // Self-Described CBOR + 6-pair map (`0xA6`)
        | &[0xD9, 0xD9, 0xF7, 0xA7] // Self-Described CBOR + 7-pair map (`0xA7`)
        | &[0xD9, 0xD9, 0xF7, 0xA8] // Self-Described CBOR + 8-pair map (`0xA8`)
        | &[0xD9, 0xD9, 0xF7, 0xB8] // Self-Described CBOR + 1-byte length map (`0xB8`, 24..255 pairs)
        | &[0xD9, 0xD9, 0xF7, 0xB9] // Self-Described CBOR + 2-byte length map (`0xB9`, 256..65535 pairs)
        | &[0xD9, 0xD9, 0xF7, 0xBF] // Self-Described CBOR + indefinite-length streaming map (`0xBF`)
        | &[0xD9, 0xD9, 0xF7, 0x81] // Self-Described CBOR + 1-element array (`0x81`)
        | &[0xD9, 0xD9, 0xF7, 0x82] // Self-Described CBOR + 2-element array (`0x82`)
        | &[0xD9, 0xD9, 0xF7, 0x83] // Self-Described CBOR + 3-element array (`0x83`)
        | &[0xD9, 0xD9, 0xF7, 0x84] // Self-Described CBOR + 4-element array (`0x84`)
        | &[0xD9, 0xD9, 0xF7, 0x85] // Self-Described CBOR + 5-element array (`0x85`)
        | &[0xD9, 0xD9, 0xF7, 0x86] // Self-Described CBOR + 6-element array (`0x86`)
        | &[0xD9, 0xD9, 0xF7, 0x98] // Self-Described CBOR + 1-byte length array (`0x98`, 24..255 elements)
        | &[0xD9, 0xD9, 0xF7, 0x99] // Self-Described CBOR + 2-byte length array (`0x99`, 256..65535 elements)
        | &[0xD9, 0xD9, 0xF7, 0x9F] // Self-Described CBOR + indefinite-length streaming array (`0x9F`)
        | &[0xD9, 0xD9, 0xF7, 0xD2] // Self-Described CBOR + RFC 8152 `COSE_Sign1` Tag 18 (`0xD2`)
        | &[0xD9, 0xD9, 0xF7, 0xD8] // Self-Described CBOR + 1-byte inner semantic tag (`0xD8`)
        | &[0xD9, 0xD9, 0xF7, 0xD9] => cbor(prefix, size), // Self-Described CBOR + 2-byte inner semantic tag (`0xD9`)

        // --- ONNX ModelProto Wire Prefix (Field 1 `ir_version` + Field 2 `producer_name` Tag & Length) ---
        // In `onnx.ModelProto` (Protobuf), bytes @0..2 encode field 1 `ir_version` (`0x08` = field 1 varint,
        // `0x03..=0x0A` = ONNX IR v3–v10), and bytes @2..4 encode field 2 `producer_name` (`0x12` = field 2
        // length-delimited string, followed by string byte length `0x07` for `"pytorch"`/`"tf2onnx"` or `0x04`
        // for `"onnx"`). Other producer lengths and IR versions fall back to Tier 2 `magic16` (`08 01..0F`):
        &[0x08, 0x03, 0x12, 0x07] // ONNX IR v3 (`08 03`) + 7-byte `producer_name` (`12 07`, e.g. `"pytorch"` / `"tf2onnx"`)
        | &[0x08, 0x04, 0x12, 0x07] // ONNX IR v4 (`08 04`) + 7-byte `producer_name` (`12 07`, e.g. `"pytorch"` / `"tf2onnx"`)
        | &[0x08, 0x05, 0x12, 0x07] // ONNX IR v5 (`08 05`) + 7-byte `producer_name` (`12 07`, e.g. `"pytorch"` / `"tf2onnx"`)
        | &[0x08, 0x06, 0x12, 0x07] // ONNX IR v6 (`08 06`) + 7-byte `producer_name` (`12 07`, e.g. `"pytorch"` / `"tf2onnx"`)
        | &[0x08, 0x07, 0x12, 0x07] // ONNX IR v7 (`08 07`) + 7-byte `producer_name` (`12 07`, e.g. `"pytorch"` / `"tf2onnx"`)
        | &[0x08, 0x08, 0x12, 0x07] // ONNX IR v8 (`08 08`) + 7-byte `producer_name` (`12 07`, e.g. `"pytorch"` / `"tf2onnx"`)
        | &[0x08, 0x09, 0x12, 0x07] // ONNX IR v9 (`08 09`) + 7-byte `producer_name` (`12 07`, e.g. `"pytorch"` / `"tf2onnx"`)
        | &[0x08, 0x0A, 0x12, 0x07] // ONNX IR v10 (`08 0A`) + 7-byte `producer_name` (`12 07`, e.g. `"pytorch"` / `"tf2onnx"`)
        | &[0x08, 0x06, 0x12, 0x04] // ONNX IR v6 (`08 06`) + 4-byte `producer_name` (`12 04`, e.g. `"onnx"`)
        | &[0x08, 0x07, 0x12, 0x04] // ONNX IR v7 (`08 07`) + 4-byte `producer_name` (`12 04`, e.g. `"onnx"`)
        | &[0x08, 0x08, 0x12, 0x04] // ONNX IR v8 (`08 08`) + 4-byte `producer_name` (`12 04`, e.g. `"onnx"`)
        | &[0x08, 0x09, 0x12, 0x04] => onnx(prefix, size), // ONNX IR v9 (`08 09`) + 4-byte `producer_name` (`12 04`, e.g. `"onnx"`)

        // --- Python Pickle Binary Serialization (0x80 PROTO + Version 2/3 + Root Opcode) ---
        // Bytes @0..2 are `80 02` (Protocol 2) or `80 03` (Protocol 3); bytes @2..4 are the root object
        // construction opcodes (`'}'` `EMPTY_DICT` + `'q'` `BINPUT`, `']'` `EMPTY_LIST` + `'q'` `BINPUT`,
        // or `'c'` `GLOBAL` + `'_'` for `c__builtin__` / `c_codecs`). Protocol 4/5 (`80 04 95`, `80 05 95`)
        // and general `80 02..05` streams fall back to Tier 2 `magic16`:
        &[0x80, 0x02, b'}', b'q'] // Protocol 2 (`80 02`) + `EMPTY_DICT` (`'}'`) + `BINPUT` (`'q'`) (2003) Python
        | &[0x80, 0x02, b']', b'q'] // Protocol 2 (`80 02`) + `EMPTY_LIST` (`']'`) + `BINPUT` (`'q'`) (2003) Python
        | &[0x80, 0x02, b'c', b'_'] // Protocol 2 (`80 02`) + `GLOBAL` (`"c__builtin__\n"` / `"c_codecs\n"`) (2003) Python
        | &[0x80, 0x03, b'}', b'q'] // Protocol 3 (`80 03`) + `EMPTY_DICT` (`'}'`) + `BINPUT` (`'q'`) (2008) Python
        | &[0x80, 0x03, b']', b'q'] // Protocol 3 (`80 03`) + `EMPTY_LIST` (`']'`) + `BINPUT` (`'q'`) (2008) Python
        | &[0x80, 0x03, b'c', b'_'] => pickle_binary(prefix, size), // Protocol 3 (`80 03`) + `GLOBAL` (`"c_codecs\n"` / `"c__builtin__\n"`) (2008) Python

        // --- Bioinformatics FASTQ & FASTA Sequence Headers ---
        // FASTQ files begin with `'@'` + sequencing run/instrument prefix; FASTA files begin with
        // `'>'` + sequence database accession prefix (general `'@'` / `'>'` fallback in `magic8`):
        b"@SRR" // NCBI Sequence Read Archive (`@SRR...`) Run FASTQ (2008) NCBI
        | b"@ERR" // EBI European Nucleotide Archive (`@ERR...`) Run FASTQ (2008) EMBL-EBI
        | b"@DRR" // DDBJ Sequence Read Archive (`@DRR...`) Run FASTQ (2008) DDBJ
        | b"@HIS" // Illumina HiSeq (`@HISEQ...`) Instrument FASTQ (2010) Illumina
        | b"@M00" // Illumina MiSeq (`@M00...`) Instrument FASTQ (2011) Illumina
        | b"@NS5" // Illumina NextSeq 500/550 (`@NS5...`) Instrument FASTQ (2014) Illumina
        | b"@A00" // Illumina NovaSeq 6000 (`@A00...`) Instrument FASTQ (2017) Illumina
        | b"@V30" // MGI DNBSEQ / BGISEQ (`@V30...`) Instrument FASTQ (2018) MGI Tech
        | b"@LH0" // Illumina NovaSeq X (`@LH0...`) Instrument FASTQ (2022) Illumina
        | b"@NB5" // Illumina NextSeq 500 (`@NB5...`) Instrument FASTQ (2014) Illumina
        | b"@FS1" // Illumina iSeq 100 (`@FS1...`) Instrument FASTQ (2018) Illumina
        | b"@SEQ" => fastq(prefix, size), // Generic Read Identifier FASTQ (2000) Sanger Institute
        b">gi|" // NCBI GenInfo (`>gi|...`) Identifier FASTA (1992) NCBI
        | b">sp|" // UniProtKB/Swiss-Prot (`>sp|...`) Curated Protein FASTA (1986) SIB / EBI
        | b">tr|" // UniProtKB/TrEMBL (`>tr|...`) Translated Protein FASTA (1996) EBI
        | b">ref" // NCBI RefSeq (`>ref|...`) Reference Sequence FASTA (1999) NCBI
        | b">gb|" // GenBank (`>gb|...`) Accession FASTA (1982) NCBI
        | b">emb" // EMBL (`>emb|...`) Nucleotide Sequence Database FASTA (1982) EMBL-EBI
        | b">dbj" // DNA Data Bank of Japan (`>dbj|...`) FASTA (1986) DDBJ
        | b">pir" // Protein Information Resource (`>pir|...`) FASTA (1984) NBRF
        | b">prf" // Protein Research Foundation (`>prf|...`) FASTA (1985) PRF
        | b">gnl" // NCBI General Database (`>gnl|...`) Identifier FASTA (1992) NCBI
        | b">lcl" // NCBI Local Sequence (`>lcl|...`) Identifier FASTA (1992) NCBI
        | b">chr" => fasta(prefix, size), // Reference Chromosome (`>chr...`) Assembly FASTA (2000) UCSC / Ensembl


        // =====================================================================
        // Section 3: Additional Standalone Binary Formats
        // =====================================================================
        b"1CDB" => verify_1c_enterprise_1cd(prefix, size), // 1C:Enterprise 8 Infobase Database (1Cv8.1CD)
        &[0xFF, 0xFF, 0xFF, 0x7F] => verify_1c_enterprise_cf(prefix, size), // 1C:Enterprise 8 Configuration Container (.cf)
        b"2DKG" => verify_2dkgt2_demo(prefix, size), // 2D Fighter Maker 2nd (2DKGT2) Replay Demo (.demo, 2002) Enterbrain
        b"AAPT" => verify_aapt2_flat(prefix, size), // Google Android AAPT2 Compiled Flat Resource Container (.flat)
        b"AARU" // Aaru Data Preservation Suite / DiscImageChef Media Image (.aaru / .dicf)
        | b"DICF" // Aaru Data Preservation Suite / DiscImageChef Media Image (.aaru / .dicf)
        | b"DICM" => verify_aaru_disk_image(prefix, size), // Aaru Data Preservation Suite / DiscImageChef Media Image (.aaru / .dicf)
        b"\x03AB2" => verify_abcomp(prefix, size), // ABComp v2.x Compressed Data (.abp, 1990s)
        b"ampf" => verify_ableton_amxd(prefix, size), // Ableton Live / Cycling '74 Max for Live Device (.amxd, 2009) Ableton & Cycling '74
        b"ABS0" => verify_absolute_db_qc(prefix, size), // ComponentAce Absolute Database File (.abs / .qc)
        b"ADSE" => verify_accessdata_ftk_ad1(prefix, size), // AccessData Forensic Toolkit (FTK) Logical Evidence Image (.ad1, ADSEGMENTEDFILE)
        b"AM01" => verify_ace_tracker_module(prefix, size), // Amiga Ace Tracker Module (.am)
        b"ACR5" => verify_acr5_icdb(prefix, size), // ACDSee ACR5 Image Catalog Database (.icdb, ACDSee)
        b"\xB4ngD" // Acronis True Image backup container (.tib)
        | &[0xCE, 0x24, 0xB9, 0xA2] => verify_acronis_tib(prefix, size), // Acronis True Image backup container (.tib)
        b"ACT " => verify_act_apricot(prefix, size), // ACT Apricot PC/Xi MS-DOS Floppy Disk Image (1983) Applied Computer Techniques ("ACT Apricot disk image")
        b"win-" => verify_adexplorer_snapshot(prefix, size), // Sysinternals Active Directory Explorer Snapshot (.dat)
        b"_A2m" => verify_adlib_a2m(prefix, size), // Subz3ro AdLib Tracker II Module (.a2m)
        b"CTMF" => verify_adlib_cmf(prefix, size), // Creative Labs AdLib / Sound Blaster Creative Music File (.cmf, 1990) Creative Labs
        b"A.H." => verify_adlib_cmf_ah(prefix, size), // Creative Labs AdLib / Sound Blaster Creative Music File (.cmf)
        b"DBRA" => verify_adlib_dro(prefix, size), // DOSBox AdLib OPL2/OPL3 Raw Capture (.dro, 2004) Harekiet / DOSBox Team ("DBRAWOPL")
        b"RAD " => verify_adlib_rad(prefix, size), // Reality AdLib Tracker OPL2/OPL3 Module (.rad, 1994) Shayde / Reality ("RAD by REALiTY!!")
        b"RAWA" => verify_adlib_raw(prefix, size), // RdosPlay AdLib OPL2 Raw Register Capture (.raw, 1995) ("RAWADATA")
        b"2OP\x1A" // Creative Labs Sound Blaster FM Instrument Patch (.sbi)
        | b"4OP\x1A" // Creative Labs Sound Blaster FM Instrument Patch (.sbi)
        | b"SBI\x1A" => verify_adlib_sbi(prefix, size), // Creative Labs Sound Blaster FM Instrument Patch (.sbi)
        b"Adob" => verify_adobe_asn_sif(prefix, size), // Adobe Developer Association (ASN) Search Index File (.sif, 1995) Adobe
        &[0xD7, 0x01, 0x00, 0x00] // Adobe Acrobat Reader RNA IPC Message (.13ec / .7a4)
        | &[0xDD, 0x01, 0x00, 0x00] => verify_adobe_rna(prefix, size), // Adobe Acrobat Reader RNA IPC Message (.13ec / .7a4)
        &[0xFE, 0xCA, 0xBE, 0xBA] => verify_adobe_techpool_tutorial_tpp(prefix, size), // Adobe / TechPool Interactive Tutorial Presentation (.tpp, 1995) TechPool Studios / Adobe
        b"90\x00\x00" => verify_aerofly_tmt(prefix, size), // IPACS Aerofly Texture Map (.tmt)
        &[0x00, 0x44, 0x00, 0x01] => verify_agfa_intellifont(prefix, size), // Agfa Compugraphic Intellifont Scalable Font (.lib / .fnt, 1990) Agfa
        b"AGST" => verify_ags_tra(prefix, size), // Adventure Game Studio Compiled Translation (.tra, 2001) Chris Jones
        b"X\xC7\xC1Q" => verify_agt_agx(prefix, size), // AGT Adventure Game Toolkit AGX Game (.agx)
        b"AAMV" => verify_aidaim_sfs(prefix, size), // AidAim Software SingleFileSystem (SFS) Container (AAMVHFSS, .edb / .plr / .loader)
        b"ALAN" => verify_alan3_game(prefix, size), // Alan 3 Compiled Adventure (.a3c)
        b"ALCH" => verify_alchemy_gallery(prefix, size), // Image Gallery (.gal, 1990s) Alchemy Mindworks ("ALCHGLRY")
        b"MEDI" => verify_alcohol_mds(prefix, size), // Alcohol 120% / DAEMON Tools Media Descriptor (.mds / .mdx, 2002) Alcohol Soft ("MEDIA DESCRIPTOR")
        b"\x08AST" => verify_all_sound_tracker(prefix, size), // All Sound Tracker Module (.ast)
        b"IMAG" => verify_allwinner_imagewty(prefix, size), // Allwinner LiveSuit / PhoenixSuit IMAGEWTY Firmware Image (.img)
        &[0xFF, 0xFF, 0x00, 0x01] => verify_alpha_bmp(prefix, size), // Alpha Microsystems AMOS Bitmap (.bmp)
        b"CoSA" => verify_alsa_soc_topology(prefix, size), // Linux ALSA SoC Audio DSP Topology Binary (.tplg, CoSA)
        b"POF\x00" => verify_altera_quartus_pof(prefix, size), // Altera / Intel Quartus Programmer Object File (.pof, POF\\0)
        b"ALZ\x01" // ESTsoft ALZip compressed archive
        | b"ALZ\x02" => verify_alzip(prefix, size), // ESTsoft ALZip compressed archive
        b"AM6\xEC" => verify_am6_amp(prefix, size), // AM6 Wrapped Windows DIB Bitmap (.amp)
        b"AMC " => verify_am_composer(prefix, size), // Amiga A.M. Composer 1.2 Module (.amc)
        b"DMA\x00" => verify_amd_cpu_microcode(prefix, size), // AMD x86-64 Processor Microcode Update Container (.bin, DMA\\0)
        &[0x07, 0x14, 0x21, 0x28] => verify_amdgpu_microcode(prefix, size), // AMD Radeon / AMDGPU Graphics & Compute Microcode Binary (.bin)
        &[0xAD, 0x36, 0x22, 0x00] => verify_amg_archive(prefix, size), // Milen Georgiev AMG Compressed Archive (.amg)
        b"AHX\x00" // Amiga Abyss Highest Experience Chiptune Synthesizer Module (.ahx / .thx)
        | b"AHX\x01" // Amiga Abyss Highest Experience Chiptune Synthesizer Module (.ahx / .thx)
        | b"THX\x00" // Amiga Abyss Highest Experience Chiptune Synthesizer Module (.ahx / .thx)
        | b"THX\x01" => verify_amiga_ahx(prefix, size), // Amiga Abyss Highest Experience Chiptune Synthesizer Module (.ahx / .thx)
        b"Nu\x00\x00" => verify_amiga_cdtv_resident_tm(prefix, size), // Commodore Amiga CDTV Resident Trademark / Device Module (CDTV.TM, 1991) Commodore-Amiga
        b"DBM0" // Amiga DigiBooster / DigiBooster Pro Tracker Module (.dbm / .digi)
        | b"DIGI" => verify_amiga_dbm(prefix, size), // Amiga DigiBooster / DigiBooster Pro Tracker Module (.dbm / .digi)
        b"FC14" // Amiga Future Composer v1.0-v1.4 Synth Module (.fc / .fc13 / .fc14 / .smod)
        | b"SMOD" => verify_amiga_fc(prefix, size), // Amiga Future Composer v1.0-v1.4 Synth Module (.fc / .fc13 / .fc14 / .smod)
        b"ATN!" // Amiga File Imploder Compressed File (IMP!)
        | b"BDPI" // Amiga File Imploder Compressed File (IMP!)
        | b"CHFI" // Amiga File Imploder Compressed File (IMP!)
        | b"Dupa" // Amiga File Imploder Compressed File (IMP!)
        | b"EDAM" // Amiga File Imploder Compressed File (IMP!)
        | b"FLT!" // Amiga File Imploder Compressed File (IMP!)
        | b"IMP!" // Amiga File Imploder Compressed File (IMP!)
        | b"M.H." // Amiga File Imploder Compressed File (IMP!)
        | b"PARA" // Amiga File Imploder Compressed File (IMP!)
        | b"RDC9" => verify_amiga_imploder(prefix, size), // Amiga File Imploder Compressed File (IMP!)
        b"MCMD" => verify_amiga_mcmd_module(prefix, size), // Amiga MCMD Chiptune Module (.mcmd)
        b"OKTA" => verify_amiga_okt(prefix, size), // Amiga Oktalyzer 8-Channel Tracker Module (.okt, 1989) Armin Sander ("OKTASONG")
        b"PKD\x10" // Amiga PackDev Device Archive (.pkd / .dsq)
        | b"PKD\x11" // Amiga PackDev Device Archive (.pkd / .dsq)
        | b"PKD\x12" // Amiga PackDev Device Archive (.pkd / .dsq)
        | b"PKD\x13" // Amiga PackDev Device Archive (.pkd / .dsq)
        | b"PKD\x14" // Amiga PackDev Device Archive (.pkd / .dsq)
        | b"PKD\x15" // Amiga PackDev Device Archive (.pkd / .dsq)
        | b"PKD\x16" // Amiga PackDev Device Archive (.pkd / .dsq)
        | b"PKD\x17" // Amiga PackDev Device Archive (.pkd / .dsq)
        | b"PKD\x18" // Amiga PackDev Device Archive (.pkd / .dsq)
        | b"PKD\x19" => verify_amiga_packdev(prefix, size), // Amiga PackDev Device Archive (.pkd / .dsq)
        b"SQSH" => verify_amiga_squash(prefix, size), // Squash Amiga Data Compressor (.sqsh)
        b"ZOM5" // Amiga Zoom 5 Disk Archive (.zom, 1995) Olaf Barthel (CRC-32 header)
        | b"ZOOM" => verify_amiga_zoom(prefix, size), // Amiga Zoom 5 Disk Archive (.zom)
        b"MPT\x00" => verify_amlogic_partition_table(prefix, size), // Amlogic SoC eMMC Partition Table (MPT\0)
        b"TBTS" => verify_amplesound_umsk(prefix, size), // Ample Sound Guitar/Bass Keyswitch Articulation Map (.umsk) Ample Sound
        b"AMX " => verify_amx_synth(prefix, size), // And X Synth Module (.amx)
        &[0x18, 0x0A, 0x00, 0x02] // ANC Cruncher Amiga Data Stream (.anc)
        | &[0x18, 0x0A, 0x00, 0x18] // ANC Cruncher Amiga Data Stream (.anc)
        | b"FVL0" => verify_anc_cruncher(prefix, size), // ANC Cruncher Amiga Data Stream (.anc)
        b"mpl1" => verify_anders_oland(prefix, size), // Anders 0land Amiga Music Module (.hot)
        b"art\n" => verify_android_art(prefix, size), // Android Runtime (ART) Pre-Initialized Heap Image (.art, 2014) Google
        b"prm\x00" // Android Runtime (ART) Baseline Profile (.prof / .profm)
        | b"pro\x00" => verify_android_art_profile(prefix, size), // Android Runtime (ART) Baseline Profile (.prof / .profm)
        b"xV4\x12" => verify_android_bootloader_img(prefix, size), // Android Bootloader Signed Image Wrapper (BOOTAND)
        &[0xD7, 0xB7, 0xAB, 0x1E] => verify_android_dtbo(prefix, size), // Android Device Tree Blob Overlay Partition Image (.dtbo, 0x1EABB7D7)
        b"hy\xADb" => verify_android_hyphenation_dict(prefix, size), // Android Minikin Compiled Hyphenation Dictionary (.hyb)
        b"CrAU" => verify_android_ota(prefix, size), // Google ChromeOS / Android A/B Seamless OTA Update Payload (payload.bin, 2015) Google
        &[0x3A, 0xFF, 0x26, 0xED] => verify_android_sparse(prefix, size), // Android Sparse Ext4/EROFS System Image (.simg, 2010) Google AOSP (0xED26FF3A LE)
        b"AVB0" => verify_android_vbmeta(prefix, size), // Google Android Verified Boot 2.0 Metadata Image (vbmeta.img, 2016) Google AOSP
        b"vdex" => verify_android_vdex(prefix, size), // Android Runtime (ART) Verifier Dependencies DEX Container (.vdex, 2017) Google ("vdex006\0".."vdex027\0")
        b"\tA11" // Android AOSP Winscope WindowManager / SurfaceFlinger / ProtoLog Trace
        | b"\tIMC" // Android AOSP Winscope WindowManager / SurfaceFlinger / ProtoLog Trace
        | b"\tIMM" // Android AOSP Winscope WindowManager / SurfaceFlinger / ProtoLog Trace
        | b"\tIMS" // Android AOSP Winscope WindowManager / SurfaceFlinger / ProtoLog Trace
        | b"\tLYR" // Android AOSP Winscope WindowManager / SurfaceFlinger / ProtoLog Trace
        | b"\tPRO" // Android AOSP Winscope WindowManager / SurfaceFlinger / ProtoLog Trace
        | b"\tTNX" // Android AOSP Winscope WindowManager / SurfaceFlinger / ProtoLog Trace
        | b"\tTRN" // Android AOSP Winscope WindowManager / SurfaceFlinger / ProtoLog Trace
        | b"\tWIN" // Android AOSP Winscope WindowManager / SurfaceFlinger / ProtoLog Trace
        | b"\tWMS" => verify_android_winscope_trace(prefix, size), // Android AOSP Winscope WindowManager / SurfaceFlinger / ProtoLog Trace
        b"VOL1" => verify_ansi_labeled_tape(prefix, size), // raw (unframed) volume label stream
        b"JG\x03\x0E" // America Online Johnson-Grace ART Compressed Image (.art)
        | b"JG\x04\x0E" => verify_aol_art_image(prefix, size), // America Online Johnson-Grace ART Compressed Image (.art)
        b"AON-" => verify_aon_fm(prefix, size), // Bastian Spiegel Art Of Noise FM Synth Instrument (.fm)
        b"{U>P" => verify_apis_iq_ice(prefix, size), // APIS IQ-Software (IQ-FMEA / IQ-RM) Fridge Container (.ice / .cfg)
        b"8900" => verify_apple_8900(prefix, size), // Apple S5L8900 Secure Boot Firmware Container (89001.0, .dfu / .img2) Apple
        b"A2R1" // Applesauce A2R floppy flux capture
        | b"A2R2" // Applesauce A2R floppy flux capture
        | b"A2R3" => verify_apple_a2r(prefix, size), // Applesauce A2R floppy flux capture
        b"AEA1" => verify_apple_aea(prefix, size), // Apple Encrypted Archive / Signed Shortcut (.aea / .shortcut, 2021) Apple
        b"AA01" // Apple Archive (.aar / .yaa)
        | b"YAA1" => verify_apple_archive(prefix, size), // Apple Archive (.aar / .yaa)
        b"ASL " => verify_apple_asl(prefix, size), // Apple System Log Binary Database (.asl, "ASL DB\0\0\0\0\0\0")
        b"BOMS" => verify_apple_bom(prefix, size), // Apple / NeXTSTEP Bill of Materials (BOMStore)
        b"caff" => verify_apple_caf(prefix, size), // Apple Core Audio Format (.caf, 2005) Apple (Mac OS X 10.4+)
        b"kych" => verify_apple_keychain(prefix, size), // Apple macOS Keychain Encrypted Credential Store (.keychain / .keychain-db, 2001) Apple
        b"MTLB" => verify_apple_metallib(prefix, size), // Apple Metal Compiled GPU Shader Library (.metallib, 2014) Apple
        b"MKXT" => verify_apple_mkext(prefix, size), // Apple Darwin Multi-Kext Kernel Extension Archive (.mkext, 2000) Apple
        b"dImg" => verify_apple_ndif(prefix, size), // Apple DiskCopy 6.0 New Disk Image Format (.dmg / .img, 1997) Apple
        b"NIBA" => verify_apple_nibarchive(prefix, size), // Apple Compiled Interface Builder NIBArchive (.nib)
        &[0xFA, 0xDE, 0x07, 0x11] => verify_apple_securityd_blob(prefix, size), // Apple SecurityServer CommonBlob Key/Database Store (.db, 2000) Apple
        b"sprs" => verify_apple_sparse_image(prefix, size), // Apple macOS Sparse Disk Image (.sparseimage, sprs)
        b"\x00Pat" // Apple Spotlight Metadata Store Database (store.db)
        | b"8tsd" => verify_apple_spotlight_store(prefix, size), // Apple Spotlight Metadata Store Database (store.db)
        b"VMP4" => verify_apple_vmp4(prefix, size), // Apple Maps Vector Map Tile Container (VMP4, .vf) Apple
        b"Fasd" => verify_applescript_compiled(prefix, size), // Apple Compiled AppleScript Bytecode (.scpt)
        b"ADRV" => verify_aprosys_aps(prefix, size), // Petter Urstad AProSys Amiga Synth Module (.aps)
        b"APS1" => verify_aps_rom_patch(prefix, size), // Nintendo 64 APS 1.0 ROM Patch (.aps, APS10)
        b"CT\x02\x00" => verify_arc_castpart(prefix, size), // ARC CAST Airport Simulation Component (CT COBJ2, .CASTPart)
        &[0x30, 0x05, 0x11, 0x20] => verify_arcadyan_firmware(prefix, size), // Arcadyan Obfuscated LZMA Router Firmware Image (.bin)
        b"ARCH" => verify_archive_hash_rtm(prefix, size), // ARCHIVE# v2 Resource Package (.rtm)
        b"FSPA" => verify_arcsoft_psf(prefix, size), // PhotoStudio Native Layered Image (.psf, 1996) ArcSoft
        b"AUI\x00" => verify_arcsoft_ui_aui(prefix, size), // ArcSoft PhotoStudio UI Layout Definition (.aui, 1998) ArcSoft
        b"ASF\x00" => verify_argonaut_asf_audio(prefix, size), // Argonaut Games ASF ADPCM Audio Stream (.asf)
        b"BRPP" => verify_argonaut_brp(prefix, size), // Argonaut Games BRPP Full-Motion Video (.brp)
        &[0x13, 0xAB, 0xA1, 0x5C] => verify_arm_astc(prefix, size), // Adaptive Scalable Texture Compression (.astc, 2012) ARM & AMD / Khronos Group (0x5CA1AB13 LE)
        &[0x6E, 0x3A, 0xF1, 0xC3] => verify_arm_mali_csffw(prefix, size), // ARM Mali GPU Command Stream Frontend MCU Firmware (mali_csffw.bin)
        &[0x58, 0xF0, 0x9F, 0xE5] => verify_arm_vector_rom(prefix, size), // ARM32 Exception Vector Firmware ROM Image (.rom)
        b"AON4" // Bastian Spiegel Art Of Noise 4/8-Channel Amiga Tracker (.aon, AON4/AON8)
        | b"AON8" => verify_art_of_noise_module(prefix, size), // Bastian Spiegel Art Of Noise 4/8-Channel Amiga Tracker (.aon, AON4/AON8)
        b"#ASD" => verify_asdf(prefix, size), // Advanced Scientific Data Format (ASDF, 2015) Space Telescope Science Institute ("#ASDF ")
        b"URES" => verify_ashampoo_ashlang(prefix, size), // Ashampoo URESFILE Localization Package (.ashlang) Ashampoo
        b"askS" // askSam Free-Form Database (.ask, PRONOM fmt/2019)
        | b"askW" // askSam Free-Form Database (.ask, PRONOM fmt/2019)
        | b"askw" => verify_asksam(prefix, size), // askSam Free-Form Database (.ask, PRONOM fmt/2019)
        b"sc69" => verify_assetto_corsa_kn5(prefix, size), // Kunos Simulazioni Assetto Corsa KN5 3D Model (.kn5)
        b"ASTM" => verify_astm_e57_point_cloud(prefix, size), // ASTM E57 3D Imaging Lidar Point Cloud (.e57, ASTM-E57)
        b"ASzf" => verify_asz_package(prefix, size), // ASzf Zlib-Compressed Resource Package (.asz)
        b"\x01ATA" // Atari 7800 ProSystem Cartridge ROM (.a78)
        | b"ATAR" => verify_atari_7800_rom(prefix, size), // Atari 7800 ProSystem Cartridge ROM (.a78)
        b"CA\x00\x00" // Atari ST CrackArt Raster Image (.ca1 / .ca2 / .ca3)
        | b"CA\x00\x01" // Atari ST CrackArt Raster Image (.ca1 / .ca2 / .ca3)
        | b"CA\x00\x02" // Atari ST CrackArt Raster Image (.ca1 / .ca2 / .ca3)
        | b"CA\x01\x00" // Atari ST CrackArt Raster Image (.ca1 / .ca2 / .ca3)
        | b"CA\x01\x01" // Atari ST CrackArt Raster Image (.ca1 / .ca2 / .ca3)
        | b"CA\x01\x02" => verify_atari_crackart(prefix, size), // Atari ST CrackArt Raster Image (.ca1 / .ca2 / .ca3)
        b"\x00\x03GF" // GFA Systemtechnik Atari ST GFA-BASIC 3.x Program (.gfa, 1988) Frank Ostrowski / GFA Systemtechnik
        | b"\x00\x04GF" // GFA Systemtechnik Atari ST GFA-BASIC 3.x Program (.gfa, 1988) Frank Ostrowski / GFA Systemtechnik
        | b"\xFF\x03GF" // GFA Systemtechnik Atari ST GFA-BASIC 3.x Program (.gfa, 1988) Frank Ostrowski / GFA Systemtechnik
        | b"\xFF\x04GF" => verify_atari_gfa_basic(prefix, size), // GFA Systemtechnik Atari ST GFA-BASIC 3.x Program (.gfa, 1988) Frank Ostrowski / GFA Systemtechnik
        &[0x0E, 0x0F, 0x00, 0x09] // Atari ST Magic Shadow Archiver Compressed Floppy Image (.msa)
        | &[0x0E, 0x0F, 0x00, 0x0A] // Atari ST Magic Shadow Archiver Compressed Floppy Image (.msa)
        | &[0x0E, 0x0F, 0x00, 0x0B] // Atari ST Magic Shadow Archiver Compressed Floppy Image (.msa)
        | &[0x0E, 0x0F, 0x00, 0x12] => verify_atari_msa(prefix, size), // Atari ST Magic Shadow Archiver Compressed Floppy Image (.msa)
        b"RIP1" // Atari 8-bit Rocky Interlace Picture (.rip)
        | b"RIP2" => verify_atari_rocky_interlace(prefix, size), // Atari 8-bit Rocky Interlace Picture (.rip)
        b"SAP\n" // Atari 8-Bit Slight Atari Player POKEY Chiptune (.sap)
        | b"SAP\r" => verify_atari_sap(prefix, size), // Atari 8-Bit Slight Atari Player POKEY Chiptune (.sap)
        b"HDOC" => verify_atari_st_guide_hyp(prefix, size), // Atari ST-Guide Compiled Hypertext (.hyp, HDOC)
        b"RSY\x00" => verify_atari_stx(prefix, size), // Atari ST Pasti Copy-Protected Floppy Disk Image (.stx, 2004) Jorge Cwik (ijor)
        b"YM2!" // Atari ST / Amstrad CPC YM2149 Sound Chip Music (.ym)
        | b"YM3!" // Atari ST / Amstrad CPC YM2149 Sound Chip Music (.ym)
        | b"YM3b" // Atari ST / Amstrad CPC YM2149 Sound Chip Music (.ym)
        | b"YM4!" // Atari ST / Amstrad CPC YM2149 Sound Chip Music (.ym)
        | b"YM5!" // Atari ST / Amstrad CPC YM2149 Sound Chip Music (.ym)
        | b"YM6!" // Atari ST / Amstrad CPC YM2149 Sound Chip Music (.ym)
        | b"YMT1" // Atari ST / Amstrad CPC YM2149 Sound Chip Music (.ym)
        | b"YMT2" => verify_atari_ym(prefix, size), // Atari ST / Amstrad CPC YM2149 Sound Chip Music (.ym)
        b"YMST" => verify_atari_ymst_audio(prefix, size), // Atari ST YMST YM2149 Chiptune (.ymst)
        b"GFS0" => verify_atlus_gfs(prefix, size), // Atlus GFS0 3D Model / Texture Container (.gmd / .gfs, 2016) Atlus / Sega
        b"ATM5" // Atari ST Atomik Cruncher v3.5 Compressed File (ATM5)
        | b"ATOM" => verify_atomik_cruncher(prefix, size), // Atari ST Atomik Cruncher v3.5 Compressed File (ATM5)
        &[0x04, 0x05, 0x06, 0x07] => verify_atonce_data(prefix, size), // Vortex ATonce PC-AT hard disk image
        b"audf" => verify_audfprint(prefix, size), // Audfprint Audio Landmark Fingerprint Database (.afpt / .afpk, 2014) Dan Ellis ("audfprinthash" / "audfprintpeak")
        b"Audi" => verify_audio_sculpture_as(prefix, size), // Sync Audio Sculpture Synth Instrument Bank (.as)
        b"hcsd" => verify_aul_dsc(prefix, size), // Apple Unified Logging Shared-Cache Strings (.dsc)
        &[0xB0, 0xBB, 0x30, 0x00] => verify_aul_timesync(prefix, size), // Apple Unified Logging Timesync Database (.timesync, 0xBBB0 LE)
        b"\x99\x88wf" => verify_aul_uuidtext(prefix, size), // Apple Unified Logging UUIDText String Catalog (0x66778899 LE)
        b"Grai" // Aurora DSP Audio Plugin State Preset (.afx / .lpa) Aurora DSP
        | b"Mena" => verify_auroradsp_state(prefix, size), // Aurora DSP Audio Plugin State Preset (.afx / .lpa) Aurora DSP
        b"ECC0" => verify_autel_ecc(prefix, size), // Autel Diagnostic Encrypted Firmware Container (.ecc)
        b"VRTL" => verify_autocad_vlx(prefix, size), // Autodesk AutoCAD Visual LISP Compiled Application Package (.vlx, 1999) Autodesk
        &[0x0D, 0x0A, 0x20, 0x46] => verify_autolisp_fas(prefix, size), // Autodesk AutoCAD Visual LISP Compiled Fast-Load Bytecode (.fas, 1999) Autodesk
        b"ASWs" => verify_avast_vpu(prefix, size), // Avast Antivirus Setup & Definitions Package (.vpu / .vpx)
        b"2BIT" => verify_avr_audio(prefix, size), // Atari ST Audio Visual Research Sampled Sound (.avr, 2BIT)
        b"_GR_" => verify_award_bios_stf(prefix, size), // Award Software BIOS Setup Language Module (_XX_CODE.BIN, .stf)
        b"JB S" => verify_bahn_nt3(prefix, size), // JB Software BAHN 3.2 Railway Simulation Network Layout (.nt3, 1995) Jan Bochmann
        b"BAI\x01" => verify_bai(prefix, size), // SAMtools BAM Index (.bai, 2009) Heng Li / SAMtools
        b"NUS3" => verify_bandai_namco_nus3(prefix, size), // Bandai Namco NUS3 Audio Container (.nus3audio / .nus3bank)
        &[0xAB, 0xCD, 0x00, 0x05] => verify_bannermania_font(prefix, size), // Pixellite / Broderbund BannerMania Vector Font (.fnt, 1989) Pixellite Group & Broderbund
        b"\r\nBa" => verify_bartender_btw(prefix, size), // Seagull Scientific BarTender Barcode Label Document (.btw, 1992) Seagull Scientific
        b"sB\x13\x00" => verify_basis_universal(prefix, size), // Binomial Basis Universal Supercompressed GPU Texture (.basis)
        b"BCL1" => verify_bcl_compressed(prefix, size), // Marcus Geelnard Basic Compression Library File (.bfc)
        b"BWks" => verify_beagleworks_doc(prefix, size), // Beagle Bros BeagleWorks / WordPerfect Works Integrated Document (.bw / .wpw, BWks)
        b"gE\xB1b" => verify_beatmaker_inst_blob(prefix, size), // UJAM Beatmaker Instrument Sample Bank (.blob)
        b"+SNT" => verify_beaver_sweeper(prefix, size), // Beaver Sweeper Tracker Module (.gtk)
        b"BBSO" => verify_beepola_song(prefix, size), // ZX Spectrum Beepola Beeper Song (.bbsong)
        b"BRTF" => verify_berotracker_brt(prefix, size), // Benjamin Rosseaux BeRoTracker Module (.brt)
        b"BSA\x00" // Bethesda Game Studios Creation Engine Archive (.bsa / .ba2)
        | b"BTDX" => verify_bethesda_archive(prefix, size), // Bethesda Game Studios Creation Engine Archive (.bsa / .ba2)
        b"BGEM" // Bethesda Creation Engine Shader Material (.bgsm / .bgem)
        | b"BGSM" => verify_bethesda_bgsm(prefix, size), // Bethesda Creation Engine Shader Material (.bgsm / .bgem)
        b"FUZE" => verify_bethesda_fuz(prefix, size), // Bethesda Creation Engine Voice & Lip-Sync Audio (.fuz)
        &[0xDE, 0xC0, 0x57, 0xFA] // Bethesda Creation Engine Compiled Papyrus Script Bytecode (.pex, 2011) Bethesda Game Studios
        | &[0xFA, 0x57, 0xC0, 0xDE] => verify_bethesda_pex(prefix, size), // Bethesda Creation Engine Compiled Papyrus Script Bytecode (.pex, 2011) Bethesda Game Studios
        b"TES3" // Bethesda Creation Engine Plugin / Master (.esp / .esm / .esl)
        | b"TES4" => verify_bethesda_plugin_esp(prefix, size), // Bethesda Creation Engine Plugin (.esp / .esm)
        b"VID\x00" => verify_bethesda_vid(prefix, size), // Bethesda Softworks XnGine VID Video (.vid)
        b"FNHD" => verify_bethesda_xcar_font(prefix, size), // Bethesda Softworks XnGine FNHD/FNDT Bitmap Font (.fnt, 1995) Bethesda Softworks
        b"BRDF" => verify_beyondtrust_brdf_rdf(prefix, size), // BeyondTrust / Bomgar Remote Support Binary Resource Definition (BRDF, .rdf)
        &[0xB5, 0x00, 0x00, 0x00] => verify_beyondtrust_scc_ui(prefix, size), // BeyondTrust Remote Support UI RPC Cache (.cache-scc-ui)
        b"BF&I" => verify_bfi_video(prefix, size), // Blue Byte Brute Force & Ignorance Video (.bfi, BF&I)
        b"EN\xA1b" => verify_bigworld_packed_xml(prefix, size), // BigWorld Engine Packed Binary XML (.xml / .visual_processed, 2005) BigWorld / Wargaming
        b"BNTL" => verify_binja_bntl(prefix, size), // Vector 35 Binary Ninja Type Library (.bntl)
        b"BIFF" => verify_bioware_biff(prefix, size), // BioWare Infinity / Aurora Engine Binary Inclusion File Archive (.bif / .bifv, 1998) BioWare
        b"BIOE" => verify_bioware_fxm(prefix, size), // BioWare Eclipse / OC3 FaceFX Facial Animation (.fxm, 2009) BioWare Edmonton
        b"ARE " // BioWare Aurora / Odyssey / Eclipse Generic File Format (.gff / .uti / .utc / .dlg / .are, 2002) BioWare
        | b"DLG " // BioWare Aurora / Odyssey / Eclipse Generic File Format (.gff / .uti / .utc / .dlg / .are, 2002) BioWare
        | b"FAC " // BioWare Aurora / Odyssey / Eclipse Generic File Format (.gff / .uti / .utc / .dlg / .are, 2002) BioWare
        | b"GFF " // BioWare Aurora / Odyssey / Eclipse Generic File Format (.gff / .uti / .utc / .dlg / .are, 2002) BioWare
        | b"GIT " // BioWare Aurora / Odyssey / Eclipse Generic File Format (.gff / .uti / .utc / .dlg / .are, 2002) BioWare
        | b"IFO " // BioWare Aurora / Odyssey / Eclipse Generic File Format (.gff / .uti / .utc / .dlg / .are, 2002) BioWare
        | b"ITP " // BioWare Aurora / Odyssey / Eclipse Generic File Format (.gff / .uti / .utc / .dlg / .are, 2002) BioWare
        | b"JRL " // BioWare Aurora / Odyssey / Eclipse Generic File Format (.gff / .uti / .utc / .dlg / .are, 2002) BioWare
        | b"PTH " // BioWare Aurora / Odyssey / Eclipse Generic File Format (.gff / .uti / .utc / .dlg / .are, 2002) BioWare
        | b"UTC " // BioWare Aurora / Odyssey / Eclipse Generic File Format (.gff / .uti / .utc / .dlg / .are, 2002) BioWare
        | b"UTD " // BioWare Aurora / Odyssey / Eclipse Generic File Format (.gff / .uti / .utc / .dlg / .are, 2002) BioWare
        | b"UTE " // BioWare Aurora / Odyssey / Eclipse Generic File Format (.gff / .uti / .utc / .dlg / .are, 2002) BioWare
        | b"UTI " // BioWare Aurora / Odyssey / Eclipse Generic File Format (.gff / .uti / .utc / .dlg / .are, 2002) BioWare
        | b"UTM " // BioWare Aurora / Odyssey / Eclipse Generic File Format (.gff / .uti / .utc / .dlg / .are, 2002) BioWare
        | b"UTP " // BioWare Aurora / Odyssey / Eclipse Generic File Format (.gff / .uti / .utc / .dlg / .are, 2002) BioWare
        | b"UTS " // BioWare Aurora / Odyssey / Eclipse Generic File Format (.gff / .uti / .utc / .dlg / .are, 2002) BioWare
        | b"UTT " // BioWare Aurora / Odyssey / Eclipse Generic File Format (.gff / .uti / .utc / .dlg / .are, 2002) BioWare
        | b"UTW " => verify_bioware_gff(prefix, size), // BioWare Aurora / Odyssey / Eclipse Generic File Format (.gff / .uti / .utc / .dlg / .are, 2002) BioWare
        b"KEY " => verify_bioware_key(prefix, size), // BioWare Infinity / Aurora Engine Key Index (.key, 1998) BioWare
        b"NCS " => verify_bioware_ncs(prefix, size), // BioWare Aurora / Odyssey NWScript Compiled Bytecode (.ncs, 2002) BioWare
        b"E\x00R\x00" // BioWare Odyssey / Eclipse Engine Resource Image Archive (.rim / .erf)
        | b"ERF " // BioWare Odyssey / Eclipse Engine Resource Image Archive (.rim / .erf)
        | b"SAV " => verify_bioware_rim(prefix, size), // BioWare Odyssey / Eclipse Engine Resource Image Archive (.rim / .erf)
        b"TLK " // BioWare Aurora / Odyssey / Eclipse Talk Table (.tlk, 2002) BioWare
        | b"Tlk\x00" => verify_bioware_tlk(prefix, size), // BioWare Aurora / Odyssey / Eclipse Talk Table (.tlk, 2002) BioWare
        b"AVXS" => verify_bitdefender_avxs(prefix, size), // Bitdefender Antivirus Engine Threat Signature Segment (AVXS, .008 / .ivd)
        b"PFR0" // Bitstream TrueDoc Portable Font Resource (.pfr)
        | b"PFR1" => verify_bitstream_pfr(prefix, size), // Bitstream TrueDoc Portable Font Resource (.pfr)
        b"D1.0" // Bitstream Speedo Scalable Outline Font (.spd)
        | b"D1.1" => verify_bitstream_speedo(prefix, size), // Bitstream Speedo Scalable Outline Font (.spd)
        b"BTCo" => verify_bittorrent_btdb(prefix, size), // BitTorrent Speed / uTorrent Ledger Database (BTComm format 0, .btdb)
        b"BtWg" => verify_bitwig_studio_preset(prefix, size), // Bitwig Studio Preset / Device / Modulator (.bwpreset / .bwdevice / .bwmodulator)
        &[0xCB, 0xE0, 0x10, 0x39] => verify_biztree_btd(prefix, size), // BizTree Business-in-a-Box Encrypted Document Template (.btd)
        b"Blac" => verify_blackdragon_zl(prefix, size), // BlackDragon v3 Archive (.zl)
        b"BH\x05\x07" => verify_blackhole_archive(prefix, size), // ZipZag BlackHole Compressed Archive (.bh)
        b"33DM" // Blizzard Entertainment StarCraft II M3 3D Model (.m3)
        | b"43DM" => verify_blizzard_m3(prefix, size), // Blizzard Entertainment StarCraft II M3 3D Model (.m3)
        b"Woo!" => verify_blizzard_woo_tbl(prefix, size), // Blizzard Entertainment Diablo II String Table (.tbl, 2000) Blizzard Entertainment
        b"b2fr" // C-Blosc2 Contiguous / Sparse Compressed Frame (.b2frame)
        | b"b2sf" // C-Blosc2 Contiguous / Sparse Compressed Frame (.b2frame)
        | b"\x9B\xA8b2" // C-Blosc2 Contiguous / Sparse Compressed Frame (.b2frame)
        | b"\x9C\xA8b2" // C-Blosc2 Contiguous / Sparse Compressed Frame (.b2frame)
        | b"\x9D\xA8b2" // C-Blosc2 Contiguous / Sparse Compressed Frame (.b2frame)
        | b"\x9E\xA7b2" // C-Blosc2 Contiguous / Sparse Compressed Frame (.b2frame)
        | b"\x9E\xA8b2" // C-Blosc2 Contiguous / Sparse Compressed Frame (.b2frame)
        | b"\x9F\xA8b2" => verify_blosc_compressed(prefix, size), // C-Blosc2 Contiguous / Sparse Compressed Frame (.b2frame)
        b"TPWM" => verify_bluebyte_tpwm(prefix, size), // Blue Byte Turbo Packer Wolfgang Mayerhofer (.tpwm)
        b"\xFF\x11Bl" => verify_bluestripe_hb(prefix, size), // BlueStripe FactFinder / Microsoft Monitoring BlackBox Capture (.hb) BlueStripe Software
        b"HDMV" => verify_bluray_clpi(prefix, size), // Blu-ray Disc Clip Information (.clpi)
        b"MOBJ" => verify_bluray_movie_object(prefix, size), // Blu-ray Disc MovieObject Navigation Commands (MovieObject.bdmv)
        b"MPLS" => verify_bluray_mpls(prefix, size), // Blu-ray Disc Movie PlayList (.mpls)
        &[0xE1, 0xE6, 0xD5, 0x1A] => verify_bmf_font(prefix, size), // Tomaž Kac / CRS ByteMap Font (.bmf, 1998) Tomaž Kac
        b"\xFF\x0E80" => verify_bng(prefix, size), // Better than PNG Lossless Image (.bng, 2026) Sam H. Smith / Serenum ("BTPNGF1" in CLEAR encoding)
        b"CBMF" => verify_bobs_adlib_bam(prefix, size), // Bob's AdLib Music Module (1990) Robert A.ames / OHRRPGCE
        b"Boch" => verify_bochs_disk_image(prefix, size), // Bochs x86 Emulator Growing Virtual HD Image (Bochs Virtual HD Image)
        b"\x00DSO" => verify_bodyslide_osd(prefix, size), // BodySlide Outfit Studio Morph Diff Data (.osd)
        b"MLOD" // Bohemia Interactive Real Virtuality P3D 3D Model (.p3d, 2001) Bohemia Interactive
        | b"ODOL" => verify_bohemia_p3d(prefix, size), // Bohemia Interactive Real Virtuality P3D 3D Model (.p3d, 2001) Bohemia Interactive
        b"RTM_" => verify_bohemia_rtm(prefix, size), // Bohemia Interactive Real Virtuality Skeletal Animation (.rtm, 2001) Bohemia Interactive
        b"<CUD" => verify_boomtracker_module(prefix, size), // CUD BoomTracker 4.0 AdLib OPL2 Module (.cff, <CUD-FM-File>)
        b"BTLF" => verify_borland_bde_locale_btl(prefix, size), // Borland Database Engine (BDE) Locale Driver (.btl, 1994) Borland
        b"PK\x08\x08" => verify_borland_bgi_chr(prefix, size), // Borland Graphics Interface Stroked Vector Font (.chr)
        b"FBGD" // Borland Graphics Interface Video Driver (.bgi, 1987) Borland
        | b"pk\x08\x08" => verify_borland_bgi_driver(prefix, size), // Borland Graphics Interface Video Driver (.bgi, 1987) Borland
        b"YO\x06\x00" => verify_borland_dpmi_overlay_ovy(prefix, size), // Borland C++ / Pascal DPMI Protected-Mode Overlay (.ovy, 1991) Borland
        b"rz\x01\x00" => verify_borland_tb_tbc(prefix, size), // Borland Turbo Basic Chain Module (.tbc)
        b"TURB" => verify_borland_tb_tbh(prefix, size), // Borland Turbo Basic Help File (.tbh)
        b"FBOV" // Borland Turbo Pascal Overlay File (.ovr)
        | b"TPOV" => verify_borland_tp_ovr(prefix, size), // Borland Turbo Pascal Overlay File (.ovr)
        b"TPU0" // Borland Turbo Pascal Compiled Unit (.tpu / .tpp / .tpw, 1987) Borland
        | b"TPU5" // Borland Turbo Pascal Compiled Unit (.tpu / .tpp / .tpw, 1987) Borland
        | b"TPU6" // Borland Turbo Pascal Compiled Unit (.tpu / .tpp / .tpw, 1987) Borland
        | b"TPU9" // Borland Turbo Pascal Compiled Unit (.tpu / .tpp / .tpw, 1987) Borland
        | b"TPUQ" => verify_borland_tpu(prefix, size), // Borland Turbo Pascal Compiled Unit (.tpu / .tpp / .tpw, 1987) Borland
        b"86BF" => verify_box86_floppy_image(prefix, size), // 86Box Emulator Surface Floppy Disk Image (.86f, 86BF)
        b"as3t" => verify_br_as3trace(prefix, size), // B&R Industrial Automation Studio 3 Trace Capture (.as3trace) B&R Industrial Automation
        b"%BL!" => verify_broadcom_stb_bootloader(prefix, size), // Broadcom Set-Top Box Bootloader Image (%BL!)
        b"HDR0" => verify_broadcom_trx(prefix, size), // Broadcom TRX Router Firmware Image (2003) Broadcom
        b"#PES" => verify_brother_pes(prefix, size), // Brother / Babylock / Bernina PES Embroidery Design (.pes, 1995) Brother Industries ("#PES0001".."#PES0100")
        &[0x0A, 0x04, 0x42, 0xD2] => verify_brunsli_jpeg(prefix, size), // Google Brunsli Lossless Repacked JPEG (.brn)
        b"BSDF" // Colin Percival bsdiff / Chromium Courgette Binary Patch
        | b"BSDI" => verify_bsdiff(prefix, size), // Colin Percival bsdiff / Chromium Courgette Binary Patch
        b"IBSP" // Valve Source / Quake II / III Binary Space Partitioning Map (.bsp)
        | b"QBSP" // Valve Source / Quake II / III Binary Space Partitioning Map (.bsp)
        | b"RBSP" // Valve Source / Quake II / III Binary Space Partitioning Map (.bsp)
        | b"VBSP" => verify_bsp_map(prefix, size), // Valve Source / Quake II / III Binary Space Partitioning Map (.bsp)
        b"btrf" => verify_btrfs_send_stream(prefix, size), // Linux Btrfs Incremental Subvolume Send Stream (btrfs-stream\0)
        &[0x2B, 0x00, 0x00, 0x00] => verify_bun_pile(prefix, size), // Bun JavaScript Runtime Transpiler Cache (.pile)
        b"buz2" => verify_buzzic2(prefix, size), // Rebels Buzzic 2 Synthesizer Module (.buz2)
        b"BWD1" => verify_bwd1_doc(prefix, size), // BWD1 Structured Document (.bwd)
        b"GDM\xFE" => verify_bwsb_gdm(prefix, size), // BWSB General Digital Music Tracker Module (.gdm, 1994) Edward Schlunder
        b"MAXF" => verify_c3_maxfile(prefix, size), // TQ Digital C3 Engine 3D Physics Mesh (.c3, 2003) NetDragon / TQ Digital
        b"GOD0" // Commodore 64 GoDot Image / Clip (.4bt / .clp, GOD0/GOD1)
        | b"GOD1" => verify_c64_godot_image(prefix, size), // Commodore 64 GoDot Image / Clip (.4bt / .clp, GOD0/GOD1)
        b"C16-" // Commodore 64 / C16 CCS64 Raw Datasette Tape Pulse Image (.tap)
        | b"C64-" => verify_c64_tap(prefix, size), // Commodore 64 / C16 CCS64 Raw Datasette Tape Pulse Image (.tap)
        b"Caff" => verify_caffeine_cscheme(prefix, size), // Caffeine.Scheme Database Schema (.cscheme)
        b"CAKE" => verify_cakewalk_wrk(prefix, size), // Twelve Tone Systems Cakewalk Project (.wrk)
        b"CAF\x00" // Cal3D 3D Character Skeleton / Mesh / Animation / Material (.csf / .cmf / .caf / .crf)
        | b"CMF\x00" // Cal3D 3D Character Skeleton / Mesh / Animation / Material (.csf / .cmf / .caf / .crf)
        | b"CRF\x00" // Cal3D 3D Character Skeleton / Mesh / Animation / Material (.csf / .cmf / .caf / .crf)
        | b"CSF\x00" => verify_cal3d(prefix, size), // Cal3D 3D Character Skeleton / Mesh / Animation / Material (.csf / .cmf / .caf / .crf)
        b"CANO" // Canon Digital Photo Professional Recipe Data (.vrd / .dr4)
        | b"IIII" => verify_canon_vrd(prefix, size), // Canon Digital Photo Professional Recipe Data (.vrd / .dr4)
        b"GMD\x00" => verify_capcom_mt_gmd(prefix, size), // Capcom MT Framework Localized Text Message Database (.gmd, 2009) Capcom
        b"MRL\x00" => verify_capcom_mt_mrl(prefix, size), // Capcom MT Framework Material Library (.mrl, 2006) Capcom
        b"KPKA" => verify_capcom_re_pak(prefix, size), // Capcom RE Engine KPKA Game Archive (.pak, 2017) Capcom
        b"CAPS" => verify_caps_ipf(prefix, size), // Software Preservation Society (SPS / CAPS) Interchangeable Preservation Format Flux Image (.ipf, 2001) István Fábián
        b"CARV" => verify_carv_vrac(prefix, size), // CARV VRAC Metadata Container (.vrac)
        &[0xAA, 0xAC, 0xBD, 0xAF] => verify_casio_container(prefix, size), // Casio Graphing Calculator Bitwise-Inverted USBPower Container (.g1a / .g1m / .g3a / .g1e / .c2m)
        b".-= " => verify_caslog_dlog(prefix, size), // CASLOGv1 Binary Diagnostic / Usage Log (.dlog / .ulog)
        &[0x0D, 0x0A, 0x1A, 0x43] => verify_cazip(prefix, size), // Computer Associates CAZIP Compressed File (1990s) Computer Associates ("\r\n\x1ACAZIP")
        b"CBA\xF9" => verify_cba_module(prefix, size), // Chuck Biscuits / Black Artist Tracker Module (.cba)
        b"{{CD" => verify_cdes_encrypted_enc(prefix, size), // {{CDES}} Encrypted File Container (.enc)
        b"YACU" => verify_cdtree_cd(prefix, size), // cdTree Offline Media Catalog Database (.cd)
        &[0xE5, 0x91, 0xB5, 0x1F] => verify_cedict_unit_u8(prefix, size), // CC-CEDICT ASCII-0x1F Unit-Delimited Chinese Dictionary (.u8)
        &[0x01, 0xAA, 0xA9, 0xC5] => verify_cedit_ucview_his(prefix, size), // Chinese DOS CEdit / UCVIEW Revision History (.his)
        &[0x01, 0xB9, 0x73, 0xA1] => verify_cedit_ucview_hlp(prefix, size), // Chinese DOS CEdit / UCVIEW Help File (.hlp)
        b"root" => verify_cern_root(prefix, size), // CERN ROOT Binary Data File (.root)
        &[0x01, 0x23, 0xCD, 0xEF] // CERNLIB ZEBRA FZ Sequential Exchange File (.fz / .xsdst)
        | &[0xEF, 0xCD, 0x23, 0x01] => verify_cern_zebra_fz(prefix, size), // CERNLIB ZEBRA FZ Sequential Exchange File (.fz / .xsdst)
        b"b3dm" => verify_cesium_b3dm(prefix, size), // Cesium OGC 3D Tiles Batched 3D Model (.b3dm)
        b"geom" => verify_cesium_geom(prefix, size), // Cesium 3D Tiles Geometry Tile (.geom)
        b"CFL3" => verify_cfl3(prefix, size), // Jari Komppa Compressed File Library v3 (.cfl)
        b" TT " => verify_chaps_timetable_tt(prefix, size), // CHAPS TimeTable Version 004 Transit Schedule Database (.tt)
        b"MCom" => verify_chd(prefix, size), // MAME Compressed Hunks of Data (CHD v1..v5, 2002) Aaron Giles / MAMEdev ("MComprHD")
        b"VjCD" => verify_chemdraw_cdx(prefix, size), // ChemDraw Binary Chemical Structure Exchange (.cdx, 1996) CambridgeSoft / PerkinElmer ("VjCD0100")
        &[0x00, 0x0C, 0x00, 0x01] => verify_chessbase_tree_ctb(prefix, size), // ChessBase Opening Book Tree Table (.ctb)
        b"BDic" => verify_chrome_bdic(prefix, size), // Google Chrome / Chromium Compiled Hunspell Spellcheck Dictionary (.bdic, 2009) Google
        &[0xFE, 0x71, 0x0D, 0x60] => verify_chrome_safebrowsing_store(prefix, size), // Google Chrome Safe Browsing V4 Store (.store, 2016) Google (0x600D71FE LE)
        b"SNSS" => verify_chrome_snss(prefix, size), // Google Chrome / Chromium Session & Tab Restore Log (Current Session / Tabs, 2008) Google
        b"VLnk" => verify_chrome_visited_links(prefix, size), // Google Chrome / Chromium Visited Links History ("Visited Links", 2008) Google
        b"__FM" => verify_chromeos_fmap(prefix, size), // Google ChromeOS / Coreboot Flash Map (__FMAP__)
        b"Vb2B" // Google ChromeOS Verified Boot 2.1 Structure (Vb2B / Vb2F / Vb2I / Vb2K / Vb2P / Vb2S, 2014) ChromiumOS
        | b"Vb2F" // Google ChromeOS Verified Boot 2.1 Structure (Vb2B / Vb2F / Vb2I / Vb2K / Vb2P / Vb2S, 2014) ChromiumOS
        | b"Vb2I" // Google ChromeOS Verified Boot 2.1 Structure (Vb2B / Vb2F / Vb2I / Vb2K / Vb2P / Vb2S, 2014) ChromiumOS
        | b"Vb2K" // Google ChromeOS Verified Boot 2.1 Structure (Vb2B / Vb2F / Vb2I / Vb2K / Vb2P / Vb2S, 2014) ChromiumOS
        | b"Vb2P" // Google ChromeOS Verified Boot 2.1 Structure (Vb2B / Vb2F / Vb2I / Vb2K / Vb2P / Vb2S, 2014) ChromiumOS
        | b"Vb2S" => verify_chromeos_vb21(prefix, size), // Google ChromeOS Verified Boot 2.1 Structure (Vb2B / Vb2F / Vb2I / Vb2K / Vb2P / Vb2S, 2014) ChromiumOS
        b"CHRO" => verify_chromeos_vboot_keyblock(prefix, size), // Google ChromeOS Verified Boot Keyblock (CHROMEOS)
        &[0xC3, 0xCA, 0x03, 0xC1] // Google Chromium Disk Cache v2 Blockfile (data_0..data_3, 2008) Google
        | &[0xC3, 0xCA, 0x04, 0xC1] => verify_chromium_cache_blockfile(prefix, size), // Google Chromium Disk Cache v2 Blockfile (data_0..data_3, 2008) Google
        b"0\\r\xA7" // Google Chromium Simple Cache Stream Entry (_0.._9 / the-real-index, 2013) Google
        | b"oy r" => verify_chromium_simple_cache(prefix, size), // Google Chromium Simple Cache Stream Entry (_0.._9 / the-real-index, 2013) Google
        b"DFIA" => verify_chronomaster_dfa(prefix, size), // DreamForge Chronomaster DFIA Animation Video (.dfa)
        b"CHX\x00" => verify_chx_big_font(prefix, size), // CHX Big Font (.chx, 1995) ("CHX\0" + 256 u16 LE glyph offsets starting at 0x0205)
        b"DPK4" => verify_cigames_dpk4(prefix, size), // CI Games DPK4 Package Archive (.dpk)
        b"BICQ" // Firaxis Games Civilization III Editor Scenario (.bic / .bix / .biq, 2001) Firaxis Games
        | b"BICX" => verify_civ3_bic(prefix, size), // Firaxis Games Civilization III Editor Scenario (.bic / .bix / .biq, 2001) Firaxis Games
        b"CCOB" // Clang Offload Bundle (CCOB / __CLANG_OFFLOAD_BUNDLE__)
        | b"__CL" => verify_clang_offload_bundle(prefix, size), // Clang Offload Bundle (CCOB / __CLANG_OFFLOAD_BUNDLE__)
        b"BAT_" => verify_claris_resolve(prefix, size), // Claris Resolve Macintosh Spreadsheet (.rsv, BAT_BTSS)
        b"PAME" // Clickteam Multimedia Fusion / Fusion 2.5 Sub-Application (.ccn / .cca, 2006) Clickteam
        | b"PAMU" => verify_clickteam_ccn(prefix, size), // Clickteam Multimedia Fusion / Fusion 2.5 Sub-Application (.ccn / .cca, 2006) Clickteam
        b"MFU2" => verify_clickteam_mfa(prefix, size), // Clickteam Fusion / Multimedia Fusion 2 Unicode Project (.mfa, 2006) Clickteam
        b"CSFC" => verify_clip_studio_paint(prefix, size), // Celsys Clip Studio Paint Illustration Document (.clip)
        b"CMAP" => verify_cmap_cmp(prefix, size), // CMAP Zlib-Compressed Collision Tile Map (.cmp)
        b"CCZ!" // Cocos2d / TexturePacker Compressed PVR Container (.ccz)
        | b"CCZp" => verify_cocos2d_ccz(prefix, size), // Cocos2d / TexturePacker Compressed PVR Container (.ccz)
        b"IWff" => verify_cod_fastfile(prefix, size), // Infinity Ward / Activision Call of Duty FastFile Archive (.ff / .fp, 2007) Infinity Ward
        b"\x80GSC" => verify_cod_gscc(prefix, size), // Activision Call of Duty Compiled GSC Script Bytecode (.gscc)
        b"STK2" => verify_coktel_stk2(prefix, size), // Coktel Vision STK21 Archive (.stk / .itk)
        b"SMS-" => verify_combiloader_bcf(prefix, size), // SMS-Soft CombiLoader Binary Container File (.bcf) SMS-Soft
        b"CBM\x00" => verify_commodore_d81(prefix, size), // Commodore 1581 3.5-Inch 800 KB Floppy Disk Image (.d81)
        b"MNIB" => verify_commodore_nibtools(prefix, size), // Commodore 1541 Nibtools Raw/Compressed GCR Disk Image (.nib / .nbz)
        &[0x42, 0x81, 0xF5, 0x00] => verify_cons_dict_nx2(prefix, size), // CONS+ DICT(HUGE) Dictionary Database (.nx2)
        &[0xD2, 0x00, 0xD2, 0x00] => verify_consultantplus_dt5(prefix, size), // ConsultantPlus Information Bank Database (.dt5)
        b"COOL" => verify_cooledit(prefix, size), // Syntrillium Cool Edit / Adobe Audition Session (.ses, "COOLNESS")
        b"LARC" => verify_coreboot_cbfs(prefix, size), // coreboot CBFS ROM Component File (LARCHIVE)
        b"CMX3" => verify_corel_cmx3(prefix, size), // Corel Presentation Exchange Metafile (CMX)
        b"CDRC" => verify_coreldraw_cpx(prefix, size), // CorelDRAW Compressed Exchange Drawing (.cpx / .cmx, 1995) Corel
        b"BRCN" => verify_cosmigo_brc(prefix, size), // Cosmigo Pro Motion Brush Container (.brc, 2012) Cosmigo
        b"STMP" => verify_cosmigo_stm(prefix, size), // Cosmigo Pro Motion Simple Tile Map (.stm, 2017) Cosmigo
        b"0707" => verify_cpio(prefix, size), // Unix System V / SVR4 ASCII cpio Archive (1977 / 1989) AT&T Bell Labs ("070701" / "070702" / "070707")
        &[0x71, 0xC7, 0x00, 0x00] // CPIO archive
        | &[0xC7, 0x71, 0x00, 0x00] => verify_cpio_binary(prefix, size), // CPIO archive
        &[0x00, 0x20, 0x20, 0x20] => verify_cpm_lbr(prefix, size), // CP/M LU / LBR Library Archive (.lbr)
        &[0x28, 0xCD, 0x3D, 0x45] // Linux Compressed ROM File System Image (.cramfs)
        | &[0x45, 0x3D, 0xCD, 0x28] => verify_cramfs(prefix, size), // Linux Compressed ROM File System Image (.cramfs)
        b"CPAD" // Google Chrome Crashpad Database Settings (CPAD)
        | b"CPds" // Google Chrome Crashpad Database Settings (CPAD)
        | b"sdPC" => verify_crashpad_settings(prefix, size), // Google Chrome Crashpad Database Settings (CPAD)
        b"MXFL" // Creative Sound Blaster X-Fi / CA0132 DSP Microcode Container (ctefx.bin)
        | b"XFi\x00" => verify_creative_xfi_microcode(prefix, size), // Creative Sound Blaster X-Fi / CA0132 DSP Microcode Container (ctefx.bin)
        b"HCA\x00" // CRI Middleware High Compression Audio (.hca)
        | &[0xC8, 0xC3, 0xC1, 0x00] => verify_cri_hca(prefix, size), // CRI Middleware High Compression Audio (.hca)
        b"CRID" => verify_cri_usm(prefix, size), // CRI Middleware Sofdec2 USM Video Container (.usm, 2007) CRI Middleware
        b"AFS2" => verify_criware_afs2_awb(prefix, size), // CRI Middleware CRIWARE AFS2 / ADX2 Wave Bank Archive (.awb, 2009) CRI Middleware
        b"CPK " => verify_criware_cpk(prefix, size), // CRI Middleware CRIWARE FileMajik PRO Package (2006) CRI Middleware
        b"CrM!" // Thomas Schwarz Crunch-Mania Amiga Data Archive (.crm)
        | b"CrM2" // Thomas Schwarz Crunch-Mania Amiga Data Archive (.crm)
        | b"Crm!" // Thomas Schwarz Crunch-Mania Amiga Data Archive (.crm)
        | b"Crm2" => verify_crunch_mania(prefix, size), // Thomas Schwarz Crunch-Mania Amiga Data Archive (.crm)
        b"CRUS" => verify_crush(prefix, size), // CRUSH Archive / Index (.cru / .cri, 1990s) PocketWare ("CRUSH v")
        b"CrCh" => verify_cryengine_soc(prefix, size), // Crytek CryEngine CrChF Object Container (.soc / .cgf, 2004) Crytek
        b"CRYO" => verify_cryo_apc_audio(prefix, size), // Cryo Interactive IMA ADPCM Audio (.apc, CRYO_APC)
        b"BigF" => verify_cryo_bigfile(prefix, size), // Cryo Interactive BigFile Game Archive (.big, 1998) Cryo Interactive
        b"CLIB" => verify_cryo_clib_vox(prefix, size), // Adventure Game Studio CLIB Resource Archive (.ags / .vox, 1999) Chris Jones
        b"HNM4" // Cryo Interactive HNM4 Multimedia Video (.hnm)
        | b"HNM6" => verify_cryo_hnm4_video(prefix, size), // Cryo Interactive HNM4 Multimedia Video (.hnm)
        &[0x06, 0x02, 0x00, 0x00] // Microsoft CryptoAPI RSA key BLOB (PUBLICKEYBLOB / PRIVATEKEYBLOB, .snk)
        | &[0x07, 0x02, 0x00, 0x00] => verify_cryptoapi_keyblob(prefix, size), // Microsoft CryptoAPI RSA key BLOB (PUBLICKEYBLOB / PRIVATEKEYBLOB, .snk)
        &[0x00, 0x10, 0x41, 0x00] => verify_crystal_reports_v4_rpt(prefix, size), // Seagate Crystal Reports 4.x/5.x Report (.rpt)
        b"CS\x00\x00" // Amit / Edimax CSMan Router Configuration Data (.dat)
        | b"SC\x00\x00" => verify_csman_dat(prefix, size), // Amit / Edimax CSMan Router Configuration Data (.dat)
        b"CISO" // Compressed ISO (CSO/CISO) disc image
        | b"JISO" // Compressed ISO (CSO/CISO) disc image
        | b"ZISO" => verify_cso_container(prefix, size), // Compressed ISO (CSO/CISO) disc image
        b"CSR-" => verify_csr_dfu2(prefix, size), // Cambridge Silicon Radio CSR-dfu2 Bluetooth Firmware Image (.dfu)
        b"CTM\x02" => verify_ctm2_map(prefix, size), // CTM v2 Tile Map (.ctm)
        b"algo" => verify_ctrader_algo(prefix, size), // Spotware cTrader / cAlgo Compiled Trading Algorithm (.algo)
        b"MXM\x00" => verify_cubic_mxm(prefix, size), // Cubic Team Tiny XM Module (.mxm)
        b"P\xEDU\xBA" // NVIDIA CUDA Fatbinary GPU Code Container (.fatbin)
        | b"\xB1CbF" => verify_cuda_fatbin(prefix, size), // NVIDIA CUDA Fatbinary GPU Code Container (.fatbin)
        &[0x2A, 0x49, 0x0D, 0x0A] => verify_cxl_help(prefix, size), // C eXtended Library (CXL) Compiled Help Database (.hlp, 1988) Mike Smedley / Innovative Data Concepts
        b"MHWK" => verify_cyan_mohawk_mhk(prefix, size), // Cyan Worlds Mohawk Resource Archive (.mhk, 1996) Cyan Worlds
        &[0x1B, 0x58, 0x00, 0x07] => verify_cyberlink_kc(prefix, size), // CyberLink Compiled UI / Script Resource (.kc)
        &[0xFE, 0xDB, 0x00, 0x00] // Atari ST Antic Cyber Paint Sequence Animation (.seq)
        | &[0xFE, 0xDC, 0x00, 0x00] => verify_cyberpaint_seq(prefix, size), // Atari ST Antic Cyber Paint Sequence Animation (.seq)
        b"\x00\x04NN" => verify_cybertracker_c64(prefix, size), // Commodore 64 Noname Cybertracker SID Module (.ct)
        b"BLOB" => verify_cypress_clm_blob(prefix, size), // Cypress / Broadcom Wi-Fi Country Locale Matrix Regulatory Blob (.clm_blob)
        b"CZ_H" => verify_cz_help(prefix, size), // CZ Help File (.hlp, 1990s) ("CZ_HELP!")
        b"DxP2" => verify_dagor_dxp(prefix, size), // Gaijin Dagor Engine Texture Pack Archive (.dxp.bin, 2012) Gaijin Entertainment
        b"GRP2" // Gaijin Dagor Engine Game Resource Pack Archive (.grp, 2012) Gaijin Entertainment
        | b"GRP3" => verify_dagor_grp(prefix, size), // Gaijin Dagor Engine Game Resource Pack Archive (.grp, 2012) Gaijin Entertainment
        b"VRFs" // Gaijin Dagor Engine Virtual ROM Filesystem Archive (.vromfs.bin, 2012) Gaijin Entertainment
        | b"VRFx" => verify_dagor_vromfs(prefix, size), // Gaijin Dagor Engine Virtual ROM Filesystem Archive (.vromfs.bin, 2012) Gaijin Entertainment
        b"DAIS" // Roy Shiels Daisy-Dot II / III Near Letter Quality Font (.nlq)
        | b"DOT_" => verify_daisydot_nlq_font(prefix, size), // Roy Shiels Daisy-Dot II / III Near Letter Quality Font (.nlq)
        b"DAN1" => verify_dan1_archive(prefix, size), // DAN1 Character Animation Package (.dan)
        b" PWD" => verify_darkwave_dwp(prefix, size), // ExperimentalScene DarkWave Studio Project (.dwp)
        b"\x7FCGC" => verify_darpa_cgc(prefix, size), // DARPA Cyber Grand Challenge DECREE Executable (.cgc, 2014) DARPA
        b"Dart" => verify_dart_dsk(prefix, size), // Dart Desktop State File (.dsk)
        b"DBN\x01" // Databento Binary Encoding Market Data Stream (.dbn)
        | b"DBN\x02" // Databento Binary Encoding Market Data Stream (.dbn)
        | b"DBN\x03" => verify_databento_dbn_v1(prefix, size), // Databento Binary Encoding Market Data Stream (.dbn)
        &[0x00, 0x04, 0x00, 0x48] => verify_dave_lowe_dln(prefix, size), // Dave Lowe New Uncle Art Amiga Module (.dln)
        b"DSNG" => verify_david_hanney_dh(prefix, size), // David Hanney Amiga Chiptune Module (.dh)
        &[0x48, 0xE7, 0xF0, 0xF0] // David Whittaker Amiga 68000 Music Module (.dw)
        | &[0x48, 0xE7, 0xF1, 0xFE] => verify_david_whittaker_module(prefix, size), // David Whittaker Amiga 68000 Music Module (.dw)
        b"Abys" => verify_dawesome_abyss2(prefix, size), // Tracktion / Dawesome Abyss 2 Synthesizer Preset (.abyss2) Peter V / Dawesome
        b"Kult" => verify_dawesome_kult(prefix, size), // Tracktion / Dawesome KULT Synthesizer Preset (.kult) Peter V / Dawesome
        b"dBAS" => verify_dbase_lbl(prefix, size), // Ashton-Tate dBASE IV Label / Report Design File (.lbl)
        b"DI\x00\x00" // Visual C++ Standalone COFF/CodeView Debug Symbols (.dbg, 1993) Microsoft
        | b"DI\x00\x80" // Visual C++ Standalone COFF/CodeView Debug Symbols (.dbg, 1993) Microsoft
        | b"DI\x01\x00" // Visual C++ Standalone COFF/CodeView Debug Symbols (.dbg, 1993) Microsoft
        | b"DI\x01\x80" => verify_dbg(prefix, size), // Visual C++ Standalone COFF/CodeView Debug Symbols (.dbg, 1993) Microsoft
        b"EDM\x06" // DCS World / Eagle Dynamics 3D Model (.edm)
        | b"EDM\x07" // DCS World / Eagle Dynamics 3D Model (.edm)
        | b"EDM\x08" // DCS World / Eagle Dynamics 3D Model (.edm)
        | b"EDM\t" // DCS World / Eagle Dynamics 3D Model (.edm)
        | b"EDM\n" // DCS World / Eagle Dynamics 3D Model (.edm)
        | b"EDM\x0B" // DCS World / Eagle Dynamics 3D Model (.edm)
        | b"EDM\x0C" // DCS World / Eagle Dynamics 3D Model (.edm)
        | b"EDM\r" // DCS World / Eagle Dynamics 3D Model (.edm)
        | b"EDM\x0E" // DCS World / Eagle Dynamics 3D Model (.edm)
        | b"EDM\x0F" // DCS World / Eagle Dynamics 3D Model (.edm)
        | b"EDM\x10" => verify_dcs_world_edm(prefix, size), // DCS World / Eagle Dynamics 3D Model (.edm)
        b"DDI1" => verify_ddi1_bgra(prefix, size), // DDI1 Uncompressed 32-Bit BGRA Bitmap (.bgra)
        b"HB\xD1\x07" // DEC WPS-Plus Word Processing Document & DX Transfer (.wpl / .dx)
        | &[0x74, 0x00, 0x5F, 0x20] => verify_dec_wpsplus(prefix, size), // DEC WPS-Plus Word Processing Document & DX Transfer (.wpl / .dx)
        &[0x14, 0x1E, 0x01, 0x00] => verify_definicon_dsi32_e32(prefix, size), // Definicon DSI-32 NS32032 Executable (.e32)
        b"DeFy" => verify_defy_tracker_module(prefix, size), // DeFy AdLib Tracker Module (.dfy, DeFy DT )
        &[0x7F, 0x18, 0x00, 0x00] => verify_dell_diagnostics_mlm(prefix, size), // Dell Modular System Diagnostics Module (.mlm, 1994) Dell
        b"LCP\x01" // Borland Delphi TCardDocument / Desktop Card Publisher (.pub, 1995)
        | b"LCP\x02" => verify_delphi_card_publisher_lcp(prefix, size), // Borland Delphi TCardDocument / Desktop Card Publisher (.pub, 1995)
        &[0x96, 0x19, 0xE0, 0xBD] => verify_delphi_cds(prefix, size), // Borland / Embarcadero Delphi ClientDataSet Binary Packet (.cds / .rds, MIDAS)
        &[0x0D, 0x00, 0x00, 0x11] // Borland / Embarcadero Delphi Compiled Unit (.dcu, 1996) Borland
        | &[0x23, 0x00, 0x00, 0x12] // Borland / Embarcadero Delphi Compiled Unit (.dcu, 1996) Borland
        | &[0x39, 0x00, 0x00, 0x14] // Borland / Embarcadero Delphi Compiled Unit (.dcu, 1996) Borland
        | b"A\x86QD" // Borland / Embarcadero Delphi Compiled Unit (.dcu, 1996) Borland
        | &[0x45, 0x00, 0x00, 0x15] // Borland / Embarcadero Delphi Compiled Unit (.dcu, 1996) Borland
        | b"HSPP" // Borland / Embarcadero Delphi Compiled Unit (.dcu, 1996) Borland
        | &[0x8B, 0x14, 0x1F, 0xF2] // Borland / Embarcadero Delphi Compiled Unit (.dcu, 1996) Borland
        | &[0x8C, 0x14, 0x1F, 0xF2] // Borland / Embarcadero Delphi Compiled Unit (.dcu, 1996) Borland
        | b"\xD8\xA6hG" // Borland / Embarcadero Delphi Compiled Unit (.dcu, 1996) Borland
        | &[0xDD, 0x00, 0x00, 0x0E] // Borland / Embarcadero Delphi Compiled Unit (.dcu, 1996) Borland
        | &[0xDF, 0x00, 0x00, 0x0F] => verify_delphi_dcu(prefix, size), // Borland / Embarcadero Delphi Compiled Unit (.dcu, 1996) Borland
        b"DLT3" // Red Hat / SUSE Delta RPM Package (.drpm, drpm / DLT3)
        | b"drpm" => verify_delta_rpm(prefix, size), // Red Hat / SUSE Delta RPM Package (.drpm, drpm / DLT3)
        &[0x03, 0x19, 0x02, 0x01] // Atari ST / DOS DemoMaker Film Animation (.flm / .seq, 1990) ("FLM\0\0" / A9 D1 6A 30)
        | b"FLM\x00" // Atari ST / DOS DemoMaker Film Animation (.flm / .seq, 1990) ("FLM\0\0" / A9 D1 6A 30)
        | b"\xA9\xD1j0" => verify_demomaker_flm(prefix, size), // Atari ST / DOS DemoMaker Film Animation (.flm / .seq, 1990) ("FLM\0\0" / A9 D1 6A 30)
        b"DHF0" // Parallax Software Descent HOG Archive (.hog)
        | b"DHF1" // Parallax Software Descent HOG Archive (.hog)
        | b"DHF2" // Parallax Software Descent HOG Archive (.hog)
        | b"DHF3" // Parallax Software Descent HOG Archive (.hog)
        | b"DHF4" // Parallax Software Descent HOG Archive (.hog)
        | b"DHF5" // Parallax Software Descent HOG Archive (.hog)
        | b"DHF6" // Parallax Software Descent HOG Archive (.hog)
        | b"DHF7" // Parallax Software Descent HOG Archive (.hog)
        | b"DHF8" // Parallax Software Descent HOG Archive (.hog)
        | b"DHF9" // Parallax Software Descent HOG Archive (.hog)
        | b"DHFA" // Parallax Software Descent HOG Archive (.hog)
        | b"DHFB" // Parallax Software Descent HOG Archive (.hog)
        | b"DHFC" // Parallax Software Descent HOG Archive (.hog)
        | b"DHFD" // Parallax Software Descent HOG Archive (.hog)
        | b"DHFE" // Parallax Software Descent HOG Archive (.hog)
        | b"DHFF" // Parallax Software Descent HOG Archive (.hog)
        | b"DHFG" // Parallax Software Descent HOG Archive (.hog)
        | b"DHFH" // Parallax Software Descent HOG Archive (.hog)
        | b"DHFI" // Parallax Software Descent HOG Archive (.hog)
        | b"DHFJ" // Parallax Software Descent HOG Archive (.hog)
        | b"DHFK" // Parallax Software Descent HOG Archive (.hog)
        | b"DHFL" // Parallax Software Descent HOG Archive (.hog)
        | b"DHFM" // Parallax Software Descent HOG Archive (.hog)
        | b"DHFN" // Parallax Software Descent HOG Archive (.hog)
        | b"DHFO" // Parallax Software Descent HOG Archive (.hog)
        | b"DHFP" // Parallax Software Descent HOG Archive (.hog)
        | b"DHFQ" // Parallax Software Descent HOG Archive (.hog)
        | b"DHFR" // Parallax Software Descent HOG Archive (.hog)
        | b"DHFS" // Parallax Software Descent HOG Archive (.hog)
        | b"DHFT" // Parallax Software Descent HOG Archive (.hog)
        | b"DHFU" // Parallax Software Descent HOG Archive (.hog)
        | b"DHFV" // Parallax Software Descent HOG Archive (.hog)
        | b"DHFW" // Parallax Software Descent HOG Archive (.hog)
        | b"DHFX" // Parallax Software Descent HOG Archive (.hog)
        | b"DHFY" // Parallax Software Descent HOG Archive (.hog)
        | b"DHFZ" // Parallax Software Descent HOG Archive (.hog)
        | b"DHF_" // Parallax Software Descent HOG Archive (.hog)
        | b"DHFa" // Parallax Software Descent HOG Archive (.hog)
        | b"DHFb" // Parallax Software Descent HOG Archive (.hog)
        | b"DHFc" // Parallax Software Descent HOG Archive (.hog)
        | b"DHFd" // Parallax Software Descent HOG Archive (.hog)
        | b"DHFe" // Parallax Software Descent HOG Archive (.hog)
        | b"DHFf" // Parallax Software Descent HOG Archive (.hog)
        | b"DHFg" // Parallax Software Descent HOG Archive (.hog)
        | b"DHFh" // Parallax Software Descent HOG Archive (.hog)
        | b"DHFi" // Parallax Software Descent HOG Archive (.hog)
        | b"DHFj" // Parallax Software Descent HOG Archive (.hog)
        | b"DHFk" // Parallax Software Descent HOG Archive (.hog)
        | b"DHFl" // Parallax Software Descent HOG Archive (.hog)
        | b"DHFm" // Parallax Software Descent HOG Archive (.hog)
        | b"DHFn" // Parallax Software Descent HOG Archive (.hog)
        | b"DHFo" // Parallax Software Descent HOG Archive (.hog)
        | b"DHFp" // Parallax Software Descent HOG Archive (.hog)
        | b"DHFq" // Parallax Software Descent HOG Archive (.hog)
        | b"DHFr" // Parallax Software Descent HOG Archive (.hog)
        | b"DHFs" // Parallax Software Descent HOG Archive (.hog)
        | b"DHFt" // Parallax Software Descent HOG Archive (.hog)
        | b"DHFu" // Parallax Software Descent HOG Archive (.hog)
        | b"DHFv" // Parallax Software Descent HOG Archive (.hog)
        | b"DHFw" // Parallax Software Descent HOG Archive (.hog)
        | b"DHFx" // Parallax Software Descent HOG Archive (.hog)
        | b"DHFy" // Parallax Software Descent HOG Archive (.hog)
        | b"DHFz" => verify_descent_hog(prefix, size), // Parallax Software Descent HOG Archive (.hog)
        b"DESI" => verify_desi_ddf(prefix, size), // DESI Telephone Label Definition File (.ddf) DESI Telephone Labels
        b"DskT" => verify_desktop_tracker(prefix, size), // Equivalence Acorn Archimedes Desktop Tracker Module (.dtt)
        &[0x23, 0x4C, 0x00, 0x00] => verify_desqview_hlp(prefix, size), // Quarterdeck DESQview / DESQview/X Help File (.hlp)
        b"U\xAAU\xAA" => verify_diablo2_save(prefix, size), // Blizzard North Diablo II Character Save (.d2s, 2000) Blizzard Entertainment
        b"gai\x00" => verify_diesel_gai(prefix, size), // Grin / Overkill Diesel Engine Skeletal Animation (.gai)
        b"DFS " => verify_difstream_svf(prefix, size), // DifStream Smart Version Patch (DFS File, .svf, Gentee / DifStream)
        b"DFM\x1A" => verify_digital_fm(prefix, size), // Roel Vanhout Digital-FM AdLib Module (.dfm)
        b" MUG" => verify_digital_mugician_module(prefix, size), // Softeyes / Thalamus Amiga Digital Mugician Module (.dmu / .mug)
        &[0x02, 0x01, 0x13, 0x13] => verify_digital_symphony(prefix, size), // Acorn Archimedes Digital Symphony Tracker Module (.dsym, 1993) Bluemoon Software
        b"D.T." => verify_digital_tracker_dtm(prefix, size), // Softjee Atari Falcon Digital Tracker Module (.dtm)
        b"DMDL" => verify_digitrakker_mdl(prefix, size), // Prodatron Digitrakker Module (.mdl)
        b"#EMB" => verify_dimps_emb(prefix, size), // Dimps #EMB Texture/Resource Container (.emb, 2008) Dimps
        b"DwPr" => verify_directwave_dwp(prefix, size), // Image-Line DirectWave Sampler Program (.dwp)
        b"MSMP" => verify_dirk_bialluch_smp(prefix, size), // Dirk Bialluch Amiga Sample Bank (.smp)
        b"MSNG" => verify_dirk_bialluch_tpu(prefix, size), // Dirk Bialluch Amiga Music Module (.tpu)
        b"IM\x00\x00" // Micro System Designs DiskDupe Floppy Disk Image (.ddi)
        | b"MSD " => verify_diskdupe_ddi(prefix, size), // Micro System Designs DiskDupe Floppy Disk Image (.ddi)
        &[0xCA, 0xFE, 0x00, 0x09] => verify_diskovery_scan(prefix, size), // Diskovery Disk Inventory Snapshot (.diskovery)
        b"PLM\x1A" => verify_disorder_plm_module(prefix, size), // Cubic Team / Statix DisorderTracker 2 Module (.plm)
        &[0x00, 0x05, 0xE1, 0x03] => verify_displaywrite_dca(prefix, size), // Document Content Architecture Revisable-Form-Text (DCA-RFT / DisplayWrite, 1981) IBM
        b"AIH0" => verify_dlink_aih0(prefix, size), // D-Link AIH0 Router Firmware Image (.bin)
        &[0xDE, 0xAF, 0xBE, 0xAD] => verify_dlink_deafbead(prefix, size), // D-Link DEAFBEAD Firmware Archive Container (.deafbead)
        b"CPKG" // D-Link FPKG / CPKG NetDefend Firewall Firmware Package (.pkg)
        | b"FPKG" => verify_dlink_fpkg(prefix, size), // D-Link FPKG / CPKG NetDefend Firewall Firmware Package (.pkg)
        b"MH01" => verify_dlink_mh01(prefix, size), // D-Link MH01 Encrypted Router Firmware Image (.bin)
        b"SHRS" => verify_dlink_shrs(prefix, size), // D-Link SHRS Encrypted Router Firmware Image (.bin)
        &[0x64, 0x80, 0x19, 0x40] => verify_dlink_tlv(prefix, size), // D-Link TLV Board-Tagged Router Firmware Image (.bin)
        b"DMS!" => verify_dms(prefix, size), // Commodore Amiga Disk Masher System Floppy Archive (.dms, 1989) SDS Software / ParCon Software
        b"\x12D\xD2d" // DMTF DSP0267 PLDM Firmware Update Package (.pldm, 2014) DMTF
        | &[0x31, 0x19, 0xCE, 0x2F] // DMTF DSP0267 PLDM Firmware Update Package (.pldm, 2014) DMTF
        | &[0x7B, 0x29, 0x1C, 0x99] // DMTF DSP0267 PLDM Firmware Update Package (.pldm, 2014) DMTF
        | &[0xF0, 0x18, 0x87, 0x8C] => verify_dmtf_pldm_firmware(prefix, size), // DMTF DSP0267 PLDM Firmware Update Package (.pldm, 2014) DMTF
        b"Disk" => verify_dos_dim_archive(prefix, size), // Ray Arachelian Disk IMage Floppy Archive (.dim)
        &[0xC5, 0xD0, 0xD3, 0xC6] => verify_dos_eps(prefix, size), // MS-DOS Binary Encapsulated PostScript Header (.eps, 1989) Adobe (0xC6D3D0C5 LE)
        &[0x5E, 0x4D, 0x00, 0x00] => verify_dos_hayes_mdm(prefix, size), // DOS Hayes AT Command Modem Definition File (.mdm)
        &[0x40, 0x00, 0x00, 0x01] => verify_dotnet_jit_profile(prefix, size), // .NET Runtime Multicore JIT Startup Profile (.profile)
        b"DBDT" => verify_dragonbones_dbbin(prefix, size), // Egret Technology DragonBones Binary Skeleton (.dbbin)
        b"DSFm" => verify_dreamstation_dss(prefix, size), // Audio Simulation DreamStation Synthesizer Song (.dss)
        b"DTL\x00" => verify_drumtraker_dtl(prefix, size), // AdLib DrumTraker Rhythm Module (.dtl)
        b"\x00K\x001" => verify_drumware_k1_bank(prefix, size), // Drumware Kawai K1 Synthesizer Sound Bank (.k1i, 1988) Drumware
        b"DSD " => verify_dsd_stream_file(prefix, size), // Sony Direct Stream Digital Stream File (.dsf)
        b"FRM8" => verify_dsdiff_audio(prefix, size), // Philips / Sony Direct Stream Digital Interchange File Format (DSDIFF)
        b"DSM\x10" => verify_dsik_dsm_module(prefix, size), // Carlos Hasan Digital Sound Interface Kit Module (.dsm)
        b"AMF\x08" // Otto Chrons DSMI Advanced Module Format (.amf)
        | b"AMF\t" // Otto Chrons DSMI Advanced Module Format (.amf)
        | b"AMF\n" // Otto Chrons DSMI Advanced Module Format (.amf)
        | b"AMF\x0B" // Otto Chrons DSMI Advanced Module Format (.amf)
        | b"AMF\x0C" // Otto Chrons DSMI Advanced Module Format (.amf)
        | b"AMF\r" // Otto Chrons DSMI Advanced Module Format (.amf)
        | b"AMF\x0E" // Otto Chrons DSMI Advanced Module Format (.amf)
        | b"DMF\x0E" => verify_dsmi_amf_module(prefix, size), // Otto Chrons DSMI Advanced Module Format (.amf)
        &[0xD0, 0x0D, 0xFE, 0xED] => verify_dtb(prefix, size), // Flattened Device Tree Blob (.dtb / FIT uImage, 2005) David Gibson / Linux Kernel & Devicetree.org (0xD00DFEED BE)
        b"DTCH" => verify_dtch_chart(prefix, size), // DTCH Song Chart Database (.dtc)
        b"DKIF" => verify_duck_ivf(prefix, size), // Duck TrueMotion / On2 / AOMedia IVF Video Container (.ivf, 1999) Duck / On2 Technologies
        &[0x1F, 0x07, 0x00, 0x3F] // IEC 61834 / SMPTE 314M Digital Video DIF Stream (.dv / .dif, 1995)
        | &[0x1F, 0x07, 0x00, 0xBF] => verify_dv_video(prefix, size), // IEC 61834 / SMPTE 314M Digital Video DIF Stream (.dv / .dif, 1995)
        b"DVDV" => verify_dvd_video_ifo(prefix, size), // DVD-Video VMGI / VTSI Navigation Information (.ifo / .bup)
        b"DGUS" => verify_dwin_dgus3_icl(prefix, size), // DWIN Technology DGUS II/III T5L HMI Icon Library (.icl, 2017) DWIN Technology
        b"DEXA" => verify_dxa(prefix, size), // DXA Compressed Video (.dxa, 2003) Adventure Soft / ScummVM
        b"DXTK" => verify_dxtk_spritefont(prefix, size), // Microsoft DirectXTK MakeSpriteFont Binary (.spritefont, 2012) Microsoft ("DXTKfont")
        b"DSm\x1A" => verify_dynamic_studio_dsm(prefix, size), // Dynamic Studio Professional Tracker Module (.dsm)
        b"ABKC" => verify_ea_abk(prefix, size), // Electronic Arts Sports Audio Bank (.abk, 2004) EA Sports
        b"BNKb" // Electronic Arts Audio Sample Bank (.bnk, BNKl/BNKb)
        | b"BNKl" => verify_ea_bnk_audio_bank(prefix, size), // Electronic Arts Audio Sample Bank (.bnk, BNKl/BNKb)
        b"EBO\x00" => verify_ea_ebo(prefix, size), // Electronic Arts Sports EBO 3D Stadium/Player Mesh (.ebo, 2002) EA Sports
        b"FNTF" // Electronic Arts FNTF / FNTS / FNTI Game Font (.ffn / .fnt)
        | b"FNTI" // Electronic Arts FNTF / FNTS / FNTI Game Font (.ffn / .fnt)
        | b"FNTS" // Electronic Arts FNTF / FNTS / FNTI Game Font (.ffn / .fnt)
        | b"FntF" => verify_ea_font(prefix, size), // Electronic Arts FNTF / FNTS / FNTI Game Font (.ffn / .fnt)
        b"SHPI" // Electronic Arts FSH / SHPI Shape Image (.fsh / .ssh)
        | b"SHPM" // Electronic Arts FSH / SHPI Shape Image (.fsh / .ssh)
        | b"SHPP" // Electronic Arts FSH / SHPI Shape Image (.fsh / .ssh)
        | b"SHPS" => verify_ea_fsh_image(prefix, size), // Electronic Arts FSH / SHPI Shape Image (.fsh / .ssh)
        b"SCHl" // Electronic Arts SCHl / SHEN Multimedia Audio/Video Stream (.asf / .exa)
        | b"SEAD" // Electronic Arts SCHl / SHEN Multimedia Audio/Video Stream (.asf / .exa)
        | b"SHEN" // Electronic Arts SCHl / SHEN Multimedia Audio/Video Stream (.asf / .exa)
        | b"kVGT" => verify_ea_schl_multimedia(prefix, size), // Electronic Arts SCHl / SHEN Multimedia Audio/Video Stream (.asf / .exa)
        b"ViV4" => verify_ea_viv4_bh(prefix, size), // Electronic Arts ViV4 BIG Archive Index Header (.bh, 2005) EA Sports
        b"EASO" => verify_earache_ea(prefix, size), // Casper Amiga EarAche Module (.ea)
        b"EGLL" => verify_ease_gll(prefix, size), // AFMG EASE Generic Loudspeaker Library (.gll)
        b"EASY" => verify_easytrax_module(prefix, size), // EasyTrax Module (.etx)
        b"ECM\x00" => verify_ecm(prefix, size), // Error Code Modeler CD-ROM Sector Image (.ecm, 2002) Neill Corlett
        &[0x65, 0x02, 0x01, 0x02] => verify_ecw_wavelet(prefix, size), // Enhanced Compression Wavelet Geospatial Image (.ecw, 1998) Stuart Nixon / Earth Resource Mapping & Hexagon
        b"EDF\x00" => verify_edf_mesh(prefix, size), // PiBoSo Engine 3D Model (.edf)
        b"EDIL" => verify_edi_lzss(prefix, size), // EDI Install / EDI LZSSLib LZSS Packed File (1990s) Eschalon Development ("EDILZSS1" / "EDILZSS2")
        b"JCH&" => verify_edlib_d00(prefix, size), // Vibrants (Jens-Christian Huus) EdLib AdLib FM Audio (.d00)
        &[0x00, 0x06, 0xFE, 0xFD] => verify_edlib_edl(prefix, size), // Jens-Christian Huus EdLib Packed OPL2 Module (.edl)
        b"EGGA" => verify_egg_archive(prefix, size), // ESTsoft EGG compressed archive
        b"\n(de" // GNU Emacs Byte-Compiled Lisp (.elc)
        | b";ELC" => verify_emacs_elc(prefix, size), // GNU Emacs Byte-Compiled Lisp (.elc)
        b"E.M." => verify_ems_v6(prefix, size), // Amiga Electronic Music System v6 Module (.ems)
        b"DOT\x00" => verify_emtex_dot_driver(prefix, size), // emTeX Dot-Matrix Printer Driver Table (.dot, 1990) Eberhard Mattes
        b"FLIB" => verify_emtex_font_library(prefix, size), // emTeX Font Library (.fli, 1990) Eberhard Mattes
        b"tcp\x02" => verify_emtex_tcp_codepage(prefix, size), // emTeX TeX Codepage Translation Table (.tcp, 1991) Eberhard Mattes
        b"EVF\t" // EnCase / Expert Witness Forensic Disk Image (.E01 / .Ex01)
        | b"EVF2" => verify_encase_e01(prefix, size), // EnCase / Expert Witness Forensic Disk Image (.E01 / .Ex01)
        b"SCO5" // Passport Designs Encore Musical Notation Score (.enc)
        | b"SCOW" => verify_encore_enc(prefix, size), // Passport Designs Encore Musical Notation Score (.enc)
        b"NDNE" => verify_endnote_enf(prefix, size), // Clarivate / Thomson Reuters EndNote Connection/Filter File (.enf / .enz) ISI ResearchSoft
        b"Copy" => verify_ensemble_drs(prefix, size), // Ensemble Studios Age of Empires DRS Resource Archive (.drs, 1997) Ensemble Studios
        b" paf" // Ensoniq PARIS Digital Audio File (.paf / .fap)
        | b"fap " => verify_ensoniq_paris_audio(prefix, size), // Ensoniq PARIS Digital Audio File (.paf / .fap)
        b"Enti" => verify_entis_mio(prefix, size), // Entis MIO Music Interleaved and Orthogonal Transformed Audio (.mio, 2001) Entis / Leshade Entis
        b"EnVi" => verify_envision_publisher(prefix, size), // EnVision Publisher DTP Document (1990s) Mark Rapacz ("EnVision Publisher DTP document file.")
        &[0x44, 0x83, 0xF9, 0xF8] => verify_envision_publisher_font(prefix, size), // Envision Publisher Scalable Vector Font (.svf, 1991) Software Publishing / Envision Solutions
        &[0x0C, 0x00, 0x00, 0x00] // Essential Objects EO.WebEngine IPC Cache (.0_1 / .0_1_x64) Essential Objects
        | &[0x28, 0x03, 0x00, 0x00] => verify_eo_webengine(prefix, size), // Essential Objects EO.WebEngine IPC Cache (.0_1 / .0_1_x64) Essential Objects
        b"PSM " // Epic MegaGames MASI ProTracker Studio Module (.psm)
        | b"PSM\xFE" => verify_epic_psm_module(prefix, size), // Epic MegaGames MASI ProTracker Studio Module (.psm)
        &[0xFE, 0x53, 0x02, 0x02] => verify_eplan_eox_db(prefix, size), // EPLAN Electric P8 / EEC Engineering Object Database (.eox)
        b"EPLY" => verify_eplybnds_model(prefix, size), // EPLYBNDS 3D Polygon Bounding Model (EPLYBNDS)
        b"PKM " => verify_ericsson_pkm(prefix, size), // Ericsson Texture Compression ETC1/ETC2 PKM Container (.pkm, 2005) Jacob Ström & Tomas Akenine-Möller / Ericsson Research ("PKM 10" / "PKM 20")
        b"ARCV" => verify_eschalon_arcv(prefix, size), // Eschalon Setup / EDI Install ARCV Compressed Container (.arv, 1990s) Eschalon Development
        &[0xAA, 0x50, 0x00, 0x00] // Espressif ESP-IDF Flash Partition Table Binary
        | &[0xAA, 0x50, 0x00, 0x10] // Espressif ESP-IDF Flash Partition Table Binary
        | &[0xAA, 0x50, 0x01, 0x00] // Espressif ESP-IDF Flash Partition Table Binary
        | &[0xAA, 0x50, 0x01, 0x01] // Espressif ESP-IDF Flash Partition Table Binary
        | &[0xAA, 0x50, 0x01, 0x02] => verify_espressif_partition_table(prefix, size), // Espressif ESP-IDF Flash Partition Table Binary
        b"e2\x00\x00" => verify_ethereum_era1(prefix, size), // Ethereum Era1 / Era / E2Store Historical Block Archive (.era1 / .era / .e2s)
        &[0xFF, 0xFE, 0x45, 0x00] => verify_eurekalog_el(prefix, size), // Neos Eureka S.r.l. EurekaLog Delphi/C++Builder Exception Log (.el, 2011) EurekaLog
        b"EXBN" => verify_exbn_ebn(prefix, size), // EXBN Binary Resource Container (.ebn)
        b"\xFF\x01Ex" => verify_exiv2_exv(prefix, size), // Exiv2 Metadata Sidecar Container (.exv, 2004) Andreas Huggel
        b"SECO" => verify_exodus_seco(prefix, size), // Exodus Wallet SECO Encrypted Container (.seco)
        b"XAD!" => verify_exotic_xad(prefix, size), // Riven the Mage eXotic AdLib Module (.xad)
        &[0xCA, 0xFE, 0xBE, 0xEF] => verify_express_publisher(prefix, size), // Power Up Express Publisher Document (.epd, 0xCAFEBEEF)
        b"EMOD" => verify_extended_mod_emd(prefix, size), // ABT Extended MOD Module (.emd)
        b"ofTA" => verify_extra_simple_music_xsm(prefix, size), // Davey Taylor eXtra Simple Music AdLib Module (.xsm, ofTAZ!)
        b"Extr" => verify_extremes_tracker_ams(prefix, size), // Velvet Development Extreme's Tracker Module (.ams)
        b"EZA_" => verify_ezart(prefix, size), // EZ-Art Professional Graphic (.eza, 1990s) ("EZA_2.0\0")
        &[0x08, 0x00, 0x01, 0x00] => verify_eznec_ez(prefix, size), // EZNEC Antenna Modeling Description (.ez, Roy Lewallen W7EL)
        b"FABY" => verify_fabylon(prefix, size), // Fabylon Educational/Dictionary Data File (1992) Fabylon Software ("FABYLON ")
        b"FTMN" => verify_face_the_music_module(prefix, size), // Jörg Schmidt / Maxon Amiga Face The Music 8-Channel Module (.ftm)
        b"FRTR" => verify_facegen_tri(prefix, size), // Singular Inversions FaceGen / Creation Engine TRI Head Morph Mesh (.tri, 2003) Singular Inversions
        b"FGUI" => verify_fairygui_fui(prefix, size), // FairyGUI Binary UI Asset Package (.fui)
        b"JSON" => verify_falcom_mi(prefix, size), // Nihon Falcom Trails / Kuro Engine Binary JSON Model Info (.mi)
        b"Fami" => verify_famitracker_module(prefix, size), // NES FamiTracker Module (.ftm)
        &[0x80, 0x00, 0x00, 0x00] => verify_farbrausch_v2m(prefix, size), // Farbrausch V2 Synthesizer Module (.v2m)
        b"P\x00\x00P" => verify_fastgraph_ppr(prefix, size), // Fastgraph Packed Pixel Run / Young Picasso Bitmap (.yp / .ppr, 1991) Ted Gruber Software / Blueview Software
        b"F\x00A\x00" => verify_fastgraph_spr(prefix, size), // Fastgraph Standard Pixel Run Bitmap (.prf / .spr, 1991) Ted Gruber Software
        b"Exte" => verify_fasttracker2_xm(prefix, size), // FastTracker II Extended Module Audio (.xm)
        b"FMC!" => verify_faust_music_creator(prefix, size), // Faust Music Creator AdLib Module (.sng)
        b"FB1 " => verify_fba_fs(prefix, size), // FinalBurn Alpha / FBNeo Emulator Savestate (.fs, 2002) Team FBAlpha
        b"FBCH" => verify_fbchunks_ngp(prefix, size), // FBCHUNKS Chunked Package (.ngp)
        &[0xFF, 0xA0, 0xFF, 0xA2] // FBI Wavelet Scalar Quantization Grayscale Fingerprint Image (.wsq)
        | &[0xFF, 0xA0, 0xFF, 0xA4] // FBI Wavelet Scalar Quantization Grayscale Fingerprint Image (.wsq)
        | &[0xFF, 0xA0, 0xFF, 0xA8] => verify_fbi_wsq(prefix, size), // FBI Wavelet Scalar Quantization Grayscale Fingerprint Image (.wsq)
        &[0x31, 0x01, 0x01, 0x01] => verify_fbs_timetable_fpl(prefix, size), // iRFP FBS Railway Timetable (.fpl)
        b"ADLI" => verify_fernandez_adlib(prefix, size), // Martin Fernandez AdLib Music Module (1990) Martin Fernandez ("ADLIB\x01")
        b"Fiel" => verify_fieldlog_fl(prefix, size), // NetManila / Deventec FieldLog Binary Diagnostic Trace (.fl)
        b"FETM" => verify_fifa_fifamod(prefix, size), // FIFA Editor Tool Mod Package (.fifamod)
        b"KGTG" => verify_fighter_maker_kgt(prefix, size), // ASCII / Enterbrain 2D Fighter Maker 95 Game Data (.kgt / .bg, 1999) Enterbrain
        b"fig-" => verify_figma_fig(prefix, size), // Figma Binary Kiwi Canvas Document (.fig)
        b"SREV" => verify_filament_material(prefix, size), // Google Filament Compiled Material Package (.filamat, "SREV_TAM")
        b"FLhd" => verify_fl_studio_project(prefix, size), // Image-Line FruityLoops / FL Studio Project (.flp, FLhd)
        b"flat" // Flatpak Single-File Application Bundle (.flatpak)
        | b"xdg-" => verify_flatpak_bundle(prefix, size), // Flatpak Single-File Application Bundle (.flatpak)
        b"FPF " => verify_flir_fpf(prefix, size), // Teledyne FLIR Public Image Format Thermal Radiometric Image (.fpf)
        b"RMOd" => verify_flmobile_flmrack(prefix, size), // Image-Line FL Studio Mobile Effect/Instrument Rack (.flmrack) Image-Line
        b"FLOG" => verify_flogiciels(prefix, size), // Flogiciels Atari ST Compressed Raster Image (1989) Flogiciels
        b"5\n2\n" => verify_fluent_interp_ip(prefix, size), // ANSYS Fluent Interpolation Field Data (.ip)
        b"\x02\x01af" => verify_fm_fmf(prefix, size), // Sports Interactive Football Manager Data Archive (.fmf, Sports Interactive)
        b"FMK!" => verify_fm_kingtracker(prefix, size), // FM-Kingtracker AdLib Module (.fmk)
        b"FMTr" => verify_fm_tracker(prefix, size), // FM Tracker AdLib Module (.fmt)
        b"FEV1" => verify_fmod_designer_fev(prefix, size), // Firelight Technologies FMOD Designer Compiled Event Project (.fev, 2006) Firelight Technologies
        b"MUSI" => verify_fmp_m(prefix, size), // Guu PC-98 FMP Music Driver Module (.m)
        &[0x04, 0xFC, 0x02, 0xFC] // Freedesktop.org Fontconfig Binary Font Cache (.cache-*)
        | &[0x05, 0xFC, 0x02, 0xFC] // Freedesktop.org Fontconfig Binary Font Cache (.cache-*)
        | &[0xFC, 0x02, 0xFC, 0x04] // Freedesktop.org Fontconfig Binary Font Cache (.cache-*)
        | &[0xFC, 0x02, 0xFC, 0x05] => verify_fontconfig_cache(prefix, size), // Freedesktop.org Fontconfig Binary Font Cache (.cache-*)
        b"\x1AWLF" => verify_fontlab_vfb(prefix, size), // FontLab Studio Vector Font Binary (.vfb, 1993) Yuri Yarmola / FontLab ("\x1AWLF10")
        b"Cff1" => verify_fontographer_fog(prefix, size), // Altsys / Macromedia Fontographer Database (.fog, 1986 / 1995) Altsys / Macromedia
        b"4DH4" // JP Software 4DOS Compressed Help Database (.hlp, 1991) JP Software
        | b"4NH4" // JP Software 4DOS Compressed Help Database (.hlp, 1991) JP Software
        | b"4OH4" => verify_fourdos_help(prefix, size), // JP Software 4DOS Compressed Help Database (.hlp, 1991) JP Software
        b"FIF\x01" => verify_fractal_image(prefix, size), // Fractal Image Format (.fif, 1992) Iterated Systems (Michael Barnsley & Alan Sloan)
        b"ArC\x01" => verify_freearc(prefix, size), // FreeArc Archive (2006) Bulat Ziganshin
        b"FBHF" // Borland Turbo Vision / FreeVision Compiled Help File (.hlp / .hf, 1990) Borland
        | b"FBRW" => verify_freebasic_hf(prefix, size), // Borland Turbo Vision / FreeVision Compiled Help File (.hlp / .hf, 1990) Borland
        &[0xAD, 0xDB, 0xDE, 0xFA] => verify_freedom_force_save(prefix, size), // Irrational Games Freedom Force Save Game (.dat)
        b"BHD5" // FromSoftware Binder Archive / Header (.bnd / .bhf / .tpf, 2009) FromSoftware
        | b"BHF3" // FromSoftware Binder Archive / Header (.bnd / .bhf / .tpf, 2009) FromSoftware
        | b"BHF4" // FromSoftware Binder Archive / Header (.bnd / .bhf / .tpf, 2009) FromSoftware
        | b"BND3" // FromSoftware Binder Archive / Header (.bnd / .bhf / .tpf, 2009) FromSoftware
        | b"BND4" // FromSoftware Binder Archive / Header (.bnd / .bhf / .tpf, 2009) FromSoftware
        | b"TPF\x00" => verify_fromsoft_binder(prefix, size), // FromSoftware Binder Archive / Header (.bnd / .bhf / .tpf, 2009) FromSoftware
        b"DCX\x00" => verify_fromsoft_dcx(prefix, size), // FromSoftware DCX Compressed Asset Container (.dcx, 2009) FromSoftware
        b"FLVE" => verify_fromsoft_flver(prefix, size), // FromSoftware FLVER 3D Model (.flver, 2009) FromSoftware
        b"TAE " => verify_fromsoft_tae(prefix, size), // FromSoftware TimeAct Animation Event Table (.tae, 2009) FromSoftware
        b"FROS" => verify_frosty_fbmod(prefix, size), // Frosty Toolsuite EA Frostbite Engine Mod Package (.fbmod)
        b"FS02" => verify_fsecure_fs02_mf(prefix, size), // F-Secure Signed Update Manifest (FS02, .mf)
        b"Funk" => verify_funktracker(prefix, size), // Jason Nunn Funktracker Module (.fnk)
        b"FUCO" => verify_future_composer_bsi(prefix, size), // Superions Future Composer BSI Amiga Module (.bsi)
        b"MUSE" => verify_galaxy_music_system(prefix, size), // Carlo Vogelsang Galaxy Music System Audio (MUSE)
        b"GBRF" => verify_gameboy_gbr(prefix, size), // Game Boy Ripped Sound Format (.gbr)
        b"GBS\x01" => verify_gameboy_gbs(prefix, size), // Nintendo Game Boy Sound System Chiptune Rip (.gbs, 2000) Scott Worley
        b"Game" // NetImmerse / Gamebryo 3D Model (.nif)
        | b"NetI" => verify_gamebryo_nif(prefix, size), // NetImmerse / Gamebryo 3D Model (.nif)
        &[0x91, 0xD5, 0x12, 0x00] => verify_gamemaker_gmk(prefix, size), // Mark Overmars / YoYo Games GameMaker Legacy Project (.gm6 / .gmk / .gm81, 2004) YoYo Games
        b"GMAD" => verify_garrysmod_gma(prefix, size), // Facepunch Garry's Mod Addon Archive v3 (2012) Garry Newman / Facepunch Studios
        b"PinE" => verify_gba_gbfs(prefix, size), // Damian Yerrick Pineight Game Boy Advance ROM File System (.gbfs)
        b"DGCA" // GCA / DGCA compressed archive (.gca/.dgc)
        | b"GCA\x00" // GCA / DGCA compressed archive (.gca/.dgc)
        | b"GCA0" => verify_gca_archive(prefix, size), // GCA / DGCA compressed archive (.gca/.dgc)
        b"GEA\x00" => verify_gea_archive(prefix, size), // GEA Installer Uninstall Package (.gea)
        b"star" => verify_gecko_startupcache(prefix, size), // Mozilla Firefox / Gecko Startup Cache (startupcache)
        &[0xFF, 0xFF, 0x00, 0x0E] // Digital Research GEM VDI Metafile (.gem)
        | &[0xFF, 0xFF, 0x00, 0x18] // Digital Research GEM VDI Metafile (.gem)
        | &[0xFF, 0xFF, 0x0E, 0x00] // Digital Research GEM VDI Metafile (.gem)
        | &[0xFF, 0xFF, 0x18, 0x00] => verify_gem_vdi_metafile(prefix, size), // Digital Research GEM VDI Metafile (.gem)
        b"GEO\x00" => verify_geo_gtx(prefix, size), // GEO 3D Voxel Chunk (.gtx)
        b"GEOX" => verify_geosoft_gx(prefix, size), // Geosoft Oasis montaj Compiled GX Executable (.gx)
        b"\xC7E\xC1S" // PC/GEOS Geoworks GeoFile Executable / Document (.geo / .vm)
        | b"\xC7E\xCFS" => verify_geoworks_geos(prefix, size), // PC/GEOS Geoworks GeoFile Executable / Document (.geo / .vm)
        &[0x95, 0x04, 0x12, 0xDE] // GNU gettext Machine Object Compiled Translation Catalog (.mo)
        | &[0xDE, 0x12, 0x04, 0x95] => verify_gettext_mo(prefix, size), // GNU gettext Machine Object Compiled Translation Catalog (.mo)
        &[0x01, 0xFA, 0xCE, 0x32] // GHC Team Glasgow Haskell Compiler Interface File (.hi / .dyn_hi / .p_hi)
        | &[0x01, 0xFA, 0xCE, 0x64] => verify_ghc_hi(prefix, size), // GHC Team Glasgow Haskell Compiler Interface File (.hi / .dyn_hi / .p_hi)
        b"/01*" // NSA Ghidra Unpacked Indexed BufferFile Database (.bf, /01*49,+)
        | b"/01," => verify_ghidra_buffer_file(prefix, size), // NSA Ghidra Unpacked Indexed BufferFile Database (.bf, /01*49,+)
        b"!MDF" // GIANTS Engine Compressed Foliage Density Map (.gdm, 2014) GIANTS Software
        | b"\"MDF" => verify_giants_gdm(prefix, size), // GIANTS Engine Compressed Foliage Density Map (.gdm, 2014) GIANTS Software
        b"GRLE" => verify_giants_grle(prefix, size), // GIANTS Engine Run-Length Encoded Foliage / Terrain Map (.grle, 2008) GIANTS Software
        b"DIRC" => verify_git_index(prefix, size), // Git Working Tree Staging Index (.git/index, DIRC)
        b"\xFFtOc" => verify_git_pack_index(prefix, size), // Git Packfile v2 Fanout Index (.idx, \\xFFtOc)
        b"RIDX" => verify_git_pack_rev(prefix, size), // Git Packfile Reverse Index (.rev, 2021) Git Project
        b"glib" => verify_glibc_ld_so_cache(prefix, size), // GNU C Library Dynamic Linker Cache (ld.so.cache, glibc-ld.so.cache1.1)
        b"3df " => verify_glide_3dfx_texture(prefix, size), // 3dfx Glide Texture File (.3df)
        b"GLUE" => verify_gluemon(prefix, size), // GlueMon Amiga Chiptune Module (.glue)
        b"Glul" => verify_glulx_story(prefix, size), // Glulx Virtual Machine Story File (.ulx)
        &[0xC8, 0xCF, 0xD2, 0xD8] => verify_gmotor_c8cf_mas(prefix, size), // ISI gMotor 2 MAS Game Archive (.mas)
        b"Gnom" => verify_gnome_keyring(prefix, size), // GNOME Keyring Binary Keystore (.keyring, 2003, "GnomeKeyring\n\r\0\n")
        b"\x00LOC" => verify_gnu_locate_db(prefix, size), // GNU findutils locate File Name Database (LOCATE02)
        b"\x89GND" => verify_gnunet_directory(prefix, size), // GNUnet Directory Listing (.gnd, \\x89GND\\r\\n\\x1A\\n)
        b"\x01gpg" => verify_gnupg_trustdb(prefix, size), // GNU Privacy Guard Key Trust Database (trustdb.gpg, 1998) Werner Koch
        b"# te" => verify_go_telemetry_count(prefix, size), // Go Runtime Telemetry Counter File (.count, 2023) Go Authors
        b"GTI!" // Commodore 64 GoatTracker Chiptune Song / Instrument (.sng / .ins)
        | b"GTI2" // Commodore 64 GoatTracker Chiptune Song / Instrument (.sng / .ins)
        | b"GTI3" // Commodore 64 GoatTracker Chiptune Song / Instrument (.sng / .ins)
        | b"GTI4" // Commodore 64 GoatTracker Chiptune Song / Instrument (.sng / .ins)
        | b"GTI5" // Commodore 64 GoatTracker Chiptune Song / Instrument (.sng / .ins)
        | b"GTS!" // Commodore 64 GoatTracker Chiptune Song / Instrument (.sng / .ins)
        | b"GTS2" // Commodore 64 GoatTracker Chiptune Song / Instrument (.sng / .ins)
        | b"GTS3" // Commodore 64 GoatTracker Chiptune Song / Instrument (.sng / .ins)
        | b"GTS4" // Commodore 64 GoatTracker Chiptune Song / Instrument (.sng / .ins)
        | b"GTS5" => verify_goattracker(prefix, size), // Commodore 64 GoatTracker Chiptune Song / Instrument (.sng / .ins)
        b"GOBJ" => verify_gobject_typelib(prefix, size), // GNOME GObject Introspection Compiled Type Library (.typelib, 2008) GNOME
        b"GDEC" => verify_godot_gdec(prefix, size), // Godot Engine Encrypted File (GDEC, FileAccessEncrypted)
        b"GDPC" => verify_godot_pck(prefix, size), // Godot Engine Resource Pack Archive (2014) Juan Linietsky & Ariel Manzur / Godot Engine
        b"RSCC" // Godot Engine Binary Resource / Scene (.res / .scn, 2014) Godot Engine
        | b"RSRC" => verify_godot_rsrc(prefix, size), // Godot Engine Binary Resource / Scene (.res / .scn)
        &[0x03, 0x0F, 0x00, 0x00] // IBM z/OS Generalized Object File Format (GOFF)
        | &[0x03, 0xF0, 0x00, 0x00] // IBM z/OS Generalized Object File Format (GOFF)
        | &[0x03, 0xF1, 0x00, 0x00] => verify_goff_object(prefix, size), // IBM z/OS Generalized Object File Format (GOFF)
        b"WAD3" => verify_goldsrc_wad3(prefix, size), // Valve GoldSrc / Half-Life Texture WAD3 Archive (.wad, 1998) Valve
        b"PKM\x00" => verify_grafx2_pkm(prefix, size), // Sunset Design GrafX2 PKM Paletted Bitmap (.pkm)
        b"GT2\x01" // Laurent de Soras Graoumf Tracker 1/2 Module (.gtk / .gt2)
        | b"GT2\x02" // Laurent de Soras Graoumf Tracker 1/2 Module (.gtk / .gt2)
        | b"GT2\x03" // Laurent de Soras Graoumf Tracker 1/2 Module (.gtk / .gt2)
        | b"GT2\x04" // Laurent de Soras Graoumf Tracker 1/2 Module (.gtk / .gt2)
        | b"GT2\x05" // Laurent de Soras Graoumf Tracker 1/2 Module (.gtk / .gt2)
        | b"GT2\x06" // Laurent de Soras Graoumf Tracker 1/2 Module (.gtk / .gt2)
        | b"GT2\x07" // Laurent de Soras Graoumf Tracker 1/2 Module (.gtk / .gt2)
        | b"GT2\x08" // Laurent de Soras Graoumf Tracker 1/2 Module (.gtk / .gt2)
        | b"GT2\t" // Laurent de Soras Graoumf Tracker 1/2 Module (.gtk / .gt2)
        | b"GTK\x01" // Laurent de Soras Graoumf Tracker 1/2 Module (.gtk / .gt2)
        | b"GTK\x02" // Laurent de Soras Graoumf Tracker 1/2 Module (.gtk / .gt2)
        | b"GTK\x03" // Laurent de Soras Graoumf Tracker 1/2 Module (.gtk / .gt2)
        | b"GTK\x04" // Laurent de Soras Graoumf Tracker 1/2 Module (.gtk / .gt2)
        | b"GTK\x05" // Laurent de Soras Graoumf Tracker 1/2 Module (.gtk / .gt2)
        | b"GTK\x06" // Laurent de Soras Graoumf Tracker 1/2 Module (.gtk / .gt2)
        | b"GTK\x07" // Laurent de Soras Graoumf Tracker 1/2 Module (.gtk / .gt2)
        | b"GTK\x08" // Laurent de Soras Graoumf Tracker 1/2 Module (.gtk / .gt2)
        | b"GTK\t" => verify_graoumf_tracker_module(prefix, size), // Laurent de Soras Graoumf Tracker 1/2 Module (.gtk / .gt2)
        b"PReg" => verify_group_policy_registry_pol(prefix, size), // Windows Group Policy Administrative Template Policy (registry.pol, PReg)
        b"GSDD" => verify_gsdd_gdd(prefix, size), // GSDD Display Descriptor Database (.gdd)
        b"ANCT" // Rockstar Games GTA RenderWare Animation Package (.ifp, 2001) Rockstar Games
        | b"ANP3" // Rockstar Games GTA RenderWare Animation Package (.ifp, 2001) Rockstar Games
        | b"ANPK" => verify_gta_ifp(prefix, size), // Rockstar Games GTA RenderWare Animation Package (.ifp, 2001) Rockstar Games
        b"\xC7U\xC5m" => verify_guemap(prefix, size), // GUEmap Interactive Fiction Map (.gmp)
        b"BCFS" // Arobas Music Guitar Pro 6 Tablature Container (.gpx)
        | b"BCFZ" => verify_guitar_pro_gpx(prefix, size), // Arobas Music Guitar Pro 6 Tablature Container (.gpx)
        b"GF1P" => verify_gus_patch(prefix, size), // Advanced Gravis Ultrasound GF1 Instrument Patch (.pat, 1992) Advanced Gravis Computer Technology & Forte Technologies ("GF1PATCH110\0ID#000002\0" / "GF1PATCH100\0ID#000002\0")
        &[0xFA, 0x04, 0x30, 0xFF] // Brightbill-Roberts Show Partner GX1 Graphics Image (.gx1)
        | &[0xFA, 0x04, 0x3E, 0xFF] // Brightbill-Roberts Show Partner GX1 Graphics Image (.gx1)
        | &[0xFA, 0x04, 0x3F, 0xFF] => verify_gx1_image(prefix, size), // Brightbill-Roberts Show Partner GX1 Graphics Image (.gx1)
        b"_blf" => verify_halo_blf(prefix, size), // Bungie Halo BLF Container / Forge Variant (.blf / .map, 2007) Bungie
        b" HAL" => verify_halpst_audio(prefix, size), // HAL Laboratory GameCube HALPST Audio Stream (.hps)
        b"HYLS" // HanMac Word-J / Word-K Macintosh Japanese/Korean Document (.hj / .hk, YLYS / HYLS)
        | b"YLYS" => verify_hanmac_word_doc(prefix, size), // HanMac Word-J / Word-K Macintosh Japanese/Korean Document (.hj / .hk, YLYS / HYLS)
        b"Falc" // Software Publishing Harvard Graphics Presentation / Chart (.ch3 / .sh3)
        | b"HGB1" // Software Publishing Harvard Graphics Presentation / Chart (.ch3 / .sh3)
        | b"HGB3" => verify_harvard_graphics(prefix, size), // Software Publishing Harvard Graphics Presentation / Chart (.ch3 / .sh3)
        &[0x1E, 0x0D, 0xB0, 0xCA] // Havok Physics / Animation Binary Tagfile & Packfile (.hkx / .hkt, 2004) Havok
        | b"W\xE0\xE0W" => verify_havok_hkx(prefix, size), // Havok Physics / Animation Binary Tagfile & Packfile (.hkx / .hkt, 2004) Havok
        b"HDIF" => verify_hdiffpatch(prefix, size), // HDiffPatch Binary Differential Patch (2013–2021) HouSisong ("HDIFF13&" / "HDIFF19&" / "HDIFF20&")
        b"HELI" => verify_helios_save(prefix, size), // HELIOS Save State Container (.save)
        b" Hel" => verify_helppc_index(prefix, size), // David Jurgens HelpPC Quick Reference Index (.ndx, 1991)
        b"\x89HTA" => verify_hemera_hta(prefix, size), // Hemera Photo-Objects Thumbnails Archive (.hta, 1990s) Hemera Technologies ("\x89HTA\r\n\x1A")
        b"CKD_" // Hercules IBM System/370 & z/Architecture CKD / FBA DASD Image (CKD_P370 / CKD_C370)
        | b"FBA_" => verify_hercules_dasd_image(prefix, size), // Hercules IBM System/370 & z/Architecture CKD / FBA DASD Image (CKD_P370 / CKD_C370)
        b"ES\x01\x00" => verify_hercules_wof(prefix, size), // Hercules Write On! RamFont Bitmap Font (.wof, 1987) Hercules Computer Technology
        &[0xC6, 0x1F, 0xBC, 0x03] => verify_hermes_bytecode(prefix, size), // Hermes JavaScript Engine AOT Bytecode (.hbc, 2019) Meta Platforms (0x1F1903C103BC1FC6 LE)
        b"HXCH" // HxC Floppy Emulator disk image (.hfe)
        | b"HXCP" // HxC Floppy Emulator disk image (.hfe)
        | b"HXCS" => verify_hfe(prefix, size), // HxC Floppy Emulator disk image (.hfe)
        b"HFST" => verify_hfst_ol_transducer(prefix, size), // Helsinki Finite-State Transducer Optimized Lookup (.hfstol)
        &[0x91, 0x58, 0x27, 0xFD] => verify_hhd_lpack(prefix, size), // HHD Software Localization Pack (.lpack)
        b"Hiew" => verify_hiew_external_info(prefix, size), // SEN Hiew External Information / Names / Save File
        b"\x89FCP" => verify_highlogic_fcp(prefix, size), // High-Logic FontCreator Font Project (.fcp, 2013) High-Logic
        b"HPDT" => verify_highway_pursuit_hpdt(prefix, size), // Adam Dawson Highway Pursuit Game Data Archive (.h*d, 2003) Retrospec
        &[0x8A, 0xFF, 0xF7, 0xB6] // Hikvision DVR/NVR/IPC Firmware Package (.dav, 2012) Hikvision
        | &[0xE9, 0x9A, 0xF7, 0xB6] => verify_hikvision_firmware(prefix, size), // Hikvision DVR/NVR/IPC Firmware Package (.dav, 2012) Hikvision
        b"7V\x00\x00" => verify_hippel_7v_module(prefix, size), // Jochen Hippel Amiga 68000 / 7-Voice Module (.hip / .hip7)
        b"COSO" => verify_hippel_coso_module(prefix, size), // Jochen Hippel Amiga/Atari ST COSO Compressed Module (.hipc / .soc)
        b"CADC" => verify_hitachi_hrf(prefix, size), // Hitachi CADCore / Tracer Raster Format (.hrf)
        b"HVL\x00" // Amiga / Cross-Platform HivelyTracker Chiptune Module (.hvl)
        | b"HVL\x01" => verify_hively_tracker(prefix, size), // Amiga / Cross-Platform HivelyTracker Chiptune Module (.hvl)
        b"ARP." => verify_holy_noise_module(prefix, size), // The Holy Noise Amiga Audio Module (.thn)
        b"Will" => verify_homeworld_lif_texture(prefix, size), // Relic Entertainment Homeworld Library Image Format Texture (.lif)
        b"mann" => verify_homeworld_mex_data(prefix, size), // Relic Entertainment Homeworld Mesh Extension Data (.mex)
        b";; H" => verify_hoops_hsf(prefix, size), // Tech Soft 3D HOOPS Stream File / SolidWorks eDrawings (.hsf / .easm)
        b"HPHP" => verify_hp48_binary(prefix, size), // HP 48 / HP 49 Calculator Binary Object (.hp / .grb, 1990) Hewlett-Packard ("HPHP48-" / "HPHP49-")
        b"ibdl" => verify_hp_bdl(prefix, size), // HP FutureSmart Printer Firmware Bundle (.bdl)
        b"ipkg" => verify_hp_ipkg(prefix, size), // HP LaserJet IPKG Firmware Package (.ipkg)
        b"HS\x00\x00" => verify_hp_palmtop_exm(prefix, size), // HP 95LX / 100LX / 200LX System Manager Executable (.exm)
        &[0x02, 0x0B, 0x01, 0x06] // HP-UX PA-RISC / M68k System Object Model (SOM) Executable & Object (.sl / .o, 1985) Hewlett-Packard
        | &[0x02, 0x0B, 0x01, 0x07] // HP-UX PA-RISC / M68k System Object Model (SOM) Executable & Object (.sl / .o, 1985) Hewlett-Packard
        | &[0x02, 0x0B, 0x01, 0x08] // HP-UX PA-RISC / M68k System Object Model (SOM) Executable & Object (.sl / .o, 1985) Hewlett-Packard
        | &[0x02, 0x0B, 0x01, 0x0B] // HP-UX PA-RISC / M68k System Object Model (SOM) Executable & Object (.sl / .o, 1985) Hewlett-Packard
        | &[0x02, 0x0B, 0x01, 0x0D] // HP-UX PA-RISC / M68k System Object Model (SOM) Executable & Object (.sl / .o, 1985) Hewlett-Packard
        | &[0x02, 0x0B, 0x01, 0x0E] // HP-UX PA-RISC / M68k System Object Model (SOM) Executable & Object (.sl / .o, 1985) Hewlett-Packard
        | &[0x02, 0x0C, 0x01, 0x06] // HP-UX PA-RISC / M68k System Object Model (SOM) Executable & Object (.sl / .o, 1985) Hewlett-Packard
        | &[0x02, 0x0C, 0x01, 0x07] // HP-UX PA-RISC / M68k System Object Model (SOM) Executable & Object (.sl / .o, 1985) Hewlett-Packard
        | &[0x02, 0x0C, 0x01, 0x08] // HP-UX PA-RISC / M68k System Object Model (SOM) Executable & Object (.sl / .o, 1985) Hewlett-Packard
        | &[0x02, 0x0C, 0x01, 0x0B] // HP-UX PA-RISC / M68k System Object Model (SOM) Executable & Object (.sl / .o, 1985) Hewlett-Packard
        | &[0x02, 0x0C, 0x01, 0x0D] // HP-UX PA-RISC / M68k System Object Model (SOM) Executable & Object (.sl / .o, 1985) Hewlett-Packard
        | &[0x02, 0x0C, 0x01, 0x0E] // HP-UX PA-RISC / M68k System Object Model (SOM) Executable & Object (.sl / .o, 1985) Hewlett-Packard
        | &[0x02, 0x10, 0x01, 0x06] // HP-UX PA-RISC / M68k System Object Model (SOM) Executable & Object (.sl / .o, 1985) Hewlett-Packard
        | &[0x02, 0x10, 0x01, 0x07] // HP-UX PA-RISC / M68k System Object Model (SOM) Executable & Object (.sl / .o, 1985) Hewlett-Packard
        | &[0x02, 0x10, 0x01, 0x08] // HP-UX PA-RISC / M68k System Object Model (SOM) Executable & Object (.sl / .o, 1985) Hewlett-Packard
        | &[0x02, 0x10, 0x01, 0x0B] // HP-UX PA-RISC / M68k System Object Model (SOM) Executable & Object (.sl / .o, 1985) Hewlett-Packard
        | &[0x02, 0x10, 0x01, 0x0D] // HP-UX PA-RISC / M68k System Object Model (SOM) Executable & Object (.sl / .o, 1985) Hewlett-Packard
        | &[0x02, 0x10, 0x01, 0x0E] // HP-UX PA-RISC / M68k System Object Model (SOM) Executable & Object (.sl / .o, 1985) Hewlett-Packard
        | &[0x02, 0x14, 0x01, 0x06] // HP-UX PA-RISC / M68k System Object Model (SOM) Executable & Object (.sl / .o, 1985) Hewlett-Packard
        | &[0x02, 0x14, 0x01, 0x07] // HP-UX PA-RISC / M68k System Object Model (SOM) Executable & Object (.sl / .o, 1985) Hewlett-Packard
        | &[0x02, 0x14, 0x01, 0x08] // HP-UX PA-RISC / M68k System Object Model (SOM) Executable & Object (.sl / .o, 1985) Hewlett-Packard
        | &[0x02, 0x14, 0x01, 0x0B] // HP-UX PA-RISC / M68k System Object Model (SOM) Executable & Object (.sl / .o, 1985) Hewlett-Packard
        | &[0x02, 0x14, 0x01, 0x0D] // HP-UX PA-RISC / M68k System Object Model (SOM) Executable & Object (.sl / .o, 1985) Hewlett-Packard
        | &[0x02, 0x14, 0x01, 0x0E] => verify_hpux_som(prefix, size), // HP-UX PA-RISC / M68k System Object Model (SOM) Executable & Object (.sl / .o, 1985) Hewlett-Packard
        b"hsi1" => verify_hsi_jpeg(prefix, size), // Handmade Software Image Alchemy HSI JPEG (.jpg / .hsi)
        b"mhwa" => verify_hsi_raw(prefix, size), // Handmade Software Image Alchemy HSI Raw Bitmap (.raw)
        b"HSXG" => verify_hsxg_06xd_bls(prefix, size), // Blizzard World of Warcraft Compiled Shader Bundle (.bls)
        b"HXNF" => verify_hxnf_troubleshoot(prefix, size), // HXNF Troubleshoot Diagnostic Dump (.troubleshoot)
        b"\x1AHP%" // Hyper Compressed Archive (.hyp)
        | b"\x1AHP&" // Hyper Compressed Archive (.hyp)
        | b"\x1AHP'" // Hyper Compressed Archive (.hyp)
        | b"\x1AST%" // Hyper Compressed Archive (.hyp)
        | b"\x1AST&" // Hyper Compressed Archive (.hyp)
        | b"\x1AST'" => verify_hyper_archive(prefix, size), // Hyper Compressed Archive (.hyp)
        b"Hype" => verify_hyperwriter_doc(prefix, size), // Ntergaid HyperWriter! Hypertext Document (.hw3 / .hw4)
        b"Hyte" => verify_hytera_fudx(prefix, size), // Hytera DMR Radio Firmware Upgrade Package (.fudx) Hytera Communications
        &[0x80, 0x00, 0x00, 0x09] => verify_ibm_displaywrite(prefix, size), // IBM DisplayWrite 2/3/4/5 Document (1984) IBM (PRONOM x-fmt/288)
        b"MRNZ" => verify_ibm_mrnz(prefix, size), // IBM PC-DOS 6.3 / OS/2 MRNZ Obfuscated Installer File (.co_ / .ex_ / .sy_, 1994) IBM ("MRNZ\x88\xF0\x27\x33")
        b"ED\x00\x00" => verify_ibm_os2_extended_attr_ea(prefix, size), // IBM OS/2 Extended Attributes Data File (EA DATA. SF, 1989) IBM
        b"\xFFMKM" => verify_ibm_os2_mkmsgf_msg(prefix, size), // IBM OS/2 & PC DOS MKMSGF Compiled Message Catalog (.msg, 1987) IBM
        &[0xAA, 0x58, 0xF0, 0x00] // IBM OS/2 SaveDskF / LoadDskF Floppy Disk Image (.dsk / .1dk / .2dk)
        | &[0xAA, 0x59, 0xF0, 0x00] // IBM OS/2 SaveDskF / LoadDskF Floppy Disk Image (.dsk / .1dk / .2dk)
        | &[0xAA, 0x5A, 0xF0, 0x00] => verify_ibm_savedskf(prefix, size), // IBM OS/2 SaveDskF / LoadDskF Floppy Disk Image (.dsk / .1dk / .2dk)
        b"\xAE1m\xC6" => verify_ibm_savf(prefix, size), // IBM i / OS/400 Save File (SAVF)
        &[0x39, 0xC8, 0x3B, 0x07] => verify_ibm_stap_ctl(prefix, size), // IBM Guardium Windows S-TAP Circular Trace Log (.ctl)
        &[0x01, 0x89, 0x69, 0xA5] // IBM z/OS AMATERSE / TRSMAIN Native Non-PDS Tersed Data Set (.terse)
        | &[0x05, 0x01, 0x01, 0x00] // IBM VM/CMS TERSE Data Set (.terse)
        | &[0x07, 0x89, 0x69, 0xA5] => verify_ibm_terse(prefix, size), // IBM z/OS AMATERSE / TRSMAIN Native PDS Tersed Data Set (.terse)
        b"Over" => verify_ibmpc_ovl(prefix, size), // IBM PC Compiler Runtime Overlay (.ovl)
        b"IBus" => verify_ibus_dict(prefix, size), // Red Hat IBus GVariant Emoji Dictionary (.dict)
        b"ITR1" => verify_icy_tower_itr(prefix, size), // Free Lunch Design Icy Tower Replay Recording (ITR140, .itr)
        &[0x84, 0x10, 0xFF, 0xFF] => verify_id_roq_video(prefix, size), // id Software Quake III / Doom 3 RoQ Video (.roq)
        b"TED5" => verify_id_ted5(prefix, size), // id Software TED5 Tile Editor Game Maps Header (.wl1 / .wl6 / .map)
        b"IDA0" // Hex-Rays IDA Pro Disassembly Database (.idb / .i64)
        | b"IDA1" // Hex-Rays IDA Pro Disassembly Database (.idb / .i64)
        | b"IDA2" => verify_ida_database(prefix, size), // Hex-Rays IDA Pro Disassembly Database (.idb / .i64)
        b"IDAT" => verify_ida_til(prefix, size), // IDA Pro Type Information Library (.til, 1998) Ilfak Guilfanov / DataRescue & Hex-Rays ("IDATIL")
        b"IDD " // Interactive Delphi Reconstructor Knowledge Base (.kb, 2009) crypto2011
        | b"IDR " => verify_idr_kb(prefix, size), // Interactive Delphi Reconstructor Knowledge Base (.kb, 2009) crypto2011
        b"IDSP" => verify_idsp_spr(prefix, size), // id Software Quake / Valve Half-Life IDSP 2D/3D Sprite (.spr, 1996) id Software / Valve
        &[0x00, 0xCA, 0x6D, 0x0F] // IBM z/OS IEBCOPY Unloaded Partitioned Data Set (COPYR1/COPYR2)
        | &[0x80, 0xCA, 0x6D, 0x0F] => verify_iebcopy_unloaded_pds(prefix, size), // IBM z/OS IEBCOPY Unloaded Partitioned Data Set (COPYR1/COPYR2)
        &[0xCD, 0xAB, 0x0A, 0x00] => verify_illusion_i3d(prefix, size), // Illusion Softworks LS3D Engine 3D Model (.i3d, 2002) Illusion Softworks
        b"IMD " => verify_imd(prefix, size), // ImageDisk Preservation Floppy Sector Image (.imd, 2005) Dave Dunfield ("IMD 1.xx: ")
        b"IMP\n" => verify_imp_archive(prefix, size), // Technelysium IMP Compressed Archive (.imp)
        b"IMPM" => verify_impulse_tracker(prefix, size), // Impulse Tracker Music Module (.it, 1995) Jeffrey Lim
        b"MCMP" => verify_imuse_mcmp_audio(prefix, size), // LucasArts iMUSE MCMP Compressed Audio (.imc)
        b"SDMA" => verify_imx_sdma_firmware(prefix, size), // NXP i.MX Smart Direct Memory Access Controller Firmware (SDMA)
        b"IS20" // Hans Wessels In Stereo! 1.0/2.0 Amiga Synthesizer Module (ISM!V1.2 / IS20DF10)
        | b"ISM!" => verify_in_stereo_module(prefix, size), // Hans Wessels In Stereo! 1.0/2.0 Amiga Synthesizer Module (ISM!V1.2 / IS20DF10)
        &[0x06, 0x06, 0xED, 0xF5] => verify_indesign(prefix, size), // Adobe InDesign Document / Book / Library (.indd / .indb / .indl)
        b"ICS " => verify_infocrypt_icsl(prefix, size), // Infocrypt ICS Encrypted Audit Log (.icsl)
        &[0xC5, 0x04, 0x48, 0x00] => verify_infotrieve_tpx(prefix, size), // INFOTrieve Technologies TPX Record (.tpx, INFOTrieve Technologies)
        b"ACF " // Inmagic DB/TextWorks Database File (.dbs / .btx / .tba / .ixl)
        | b"BTX " // Inmagic DB/TextWorks Database File (.dbs / .btx / .tba / .ixl)
        | b"DBO " // Inmagic DB/TextWorks Database File (.dbs / .btx / .tba / .ixl)
        | b"DBR " // Inmagic DB/TextWorks Database File (.dbs / .btx / .tba / .ixl)
        | b"DBS " // Inmagic DB/TextWorks Database File (.dbs / .btx / .tba / .ixl)
        | b"IXL " // Inmagic DB/TextWorks Database File (.dbs / .btx / .tba / .ixl)
        | b"OCC " // Inmagic DB/TextWorks Database File (.dbs / .btx / .tba / .ixl)
        | b"SDO " // Inmagic DB/TextWorks Database File (.dbs / .btx / .tba / .ixl)
        | b"TBA " // Inmagic DB/TextWorks Database File (.dbs / .btx / .tba / .ixl)
        | b"TBU " => verify_inmagic_dbtextworks(prefix, size), // Inmagic DB/TextWorks Database File (.dbs / .btx / .tba / .ixl)
        b"Inno" // Jordan Russell Inno Setup Installer Data / Log / Messages (.dat / .msg / .bin)
        | b"My I" // Jordan Russell Inno Setup Installer Data / Log / Messages (.dat / .msg / .bin)
        | b"i1.2" // Jordan Russell Inno Setup Installer Data / Log / Messages (.dat / .msg / .bin)
        | b"idsk" // Jordan Russell Inno Setup Installer Data / Log / Messages (.dat / .msg / .bin)
        | b"zlb\x1A" => verify_inno_setup(prefix, size), // Jordan Russell Inno Setup Installer Data / Log / Messages (.dat / .msg / .bin)
        b"ESIV" => verify_installer_vise_xyz(prefix, size), // MindVision Installer VISE Metadata Container (ESIVCSIM, miscdata.xyz)
        b"ISc(" => verify_installshield_cab(prefix, size), // InstallShield Cabinet Archive (.cab / .hdr, 1997) Stirling Technologies / Flexera
        b"setu" => verify_installshield_ibt(prefix, size), // InstallShield IBT Multi-Member SZDD Bundle (setup.ibt, 1998) Stirling / InstallShield ("setup.dl_\0setup.dll\0")
        &[0xB8, 0xC9, 0x0C, 0x00] // InstallShield Compiled Script (.ins)
        | &[0xFF, 0xFF, 0x0C, 0x00] => verify_installshield_ins(prefix, size), // InstallShield Compiled Script (.ins)
        b"aLuZ" // InstallShield Compiled Rules Script (.inx / .ins)
        | &[0x74, 0xC4, 0x2C, 0x84] => verify_installshield_inx(prefix, size), // InstallShield Compiled Rules Script (.inx / .ins)
        &[0x71, 0xA8, 0x00, 0x00] => verify_installshield_isu(prefix, size), // InstallShield Uninstall Script / Log (.isu)
        &[0x2A, 0xAB, 0x79, 0xD8] => verify_installshield_v3(prefix, size), // InstallShield v3 Compressed Installer Archive (_INST32I.EX_, 1995) Stirling Technologies
        &[0x13, 0x5D, 0x65, 0x8C] // InstallShield Z / TSComp compressed archive
        | &[0x65, 0x5D, 0x13, 0x8C] => verify_installshield_z(prefix, size), // InstallShield Z / TSComp compressed archive
        b"BNEG" => verify_instar_bneg(prefix, size), // INSTAR IP Camera BNEG Firmware Container (.pkg)
        b"$CPD" => verify_intel_cpd_firmware(prefix, size), // Intel Code Partition Directory Firmware Container ($CPD)
        b"IVDV" => verify_intel_dvi_avss(prefix, size), // Intel / IBM DVI ActionMedia AVSS Multimedia File (.dvi, 1989) Intel (VSTD_HDR_ID 0x56445649 LE)
        &[0xE5, 0x1F, 0x94, 0x00] => verify_intel_esif_dv(prefix, size), // Intel Dynamic Tuning Technology (DPTF) ESIF DataVault (.dv)
        b"ISCU" => verify_intel_iscu_firmware(prefix, size), // Intel C600 Storage Controller Unit Cable Management OEM Firmware (ISCUOEMB)
        b"SCVF" => verify_intel_ivsc_firmware(prefix, size), // Intel Vision Sensing Controller Firmware (SCVF / SFWI)
        b"$FPT" => verify_intel_me_fpt(prefix, size), // Intel Management Engine Flash Partition Table ($FPT, .bin) Intel
        &[0x97, 0x28, 0x03, 0x01] => verify_interplay_acm(prefix, size), // Interplay Entertainment ACM Compressed Audio (.acm)
        b"PML " => verify_ion_pml_dgm(prefix, size), // Power Measurement / Schneider Electric ION Enterprise PML User Diagram (.dgm) Power Measurement
        b"IRLO" => verify_irlog1_log(prefix, size), // IRLOG1 ECDSA-Signed Incident Response Log (.irlog)
        b"FRAC" => verify_isi_frac_car(prefix, size), // Image Space Incorporated FRAC Encrypted Vehicle Archive (.car, 2005) ISI
        b"CUBE" // ISI gMotor / rFactor MAS Game Data Archive (.mas / .gtl, GMOTORMAS10)
        | b"GMOT" => verify_isi_gmotor_mas(prefix, size), // ISI gMotor / rFactor MAS Game Data Archive (.mas / .gtl, GMOTORMAS10)
        b"ISWL" => verify_iswlog_swl(prefix, size), // ISWLOG Encrypted Diagnostic Trace (.swl)
        b"IW4X" => verify_iw4x_heat(prefix, size), // IW4x Call of Duty: Modern Warfare 2 Multiplayer Heatmap (.iw4heat)
        b"*WES" // Intel PRO/Wireless & Wi-Fi 6/7 Microcode Image (.ucode, IWL\\n)
        | &[0x3E, 0x00, 0x00, 0x00] // Intel PRO/Wireless & Wi-Fi 6/7 Microcode Image (.ucode, IWL\\n)
        | &[0x40, 0x00, 0x00, 0x00] => verify_iwlwifi_ucode(prefix, size), // Intel PRO/Wireless & Wi-Fi 6/7 Microcode Image (.ucode, IWL\\n)
        b"IX\x01\x01" => verify_ix_index(prefix, size), // IX Binary Database Index (.ix)
        b"IXS!" => verify_ixalance_ixs(prefix, size), // Shortcut Ixalance Impulse Tracker Synth Module (.ixs)
        b"JHMF" => verify_jackhammer_jmf(prefix, size), // J.A.C.K. / Jackhammer Binary Map (.jmf, 2006) Jackhammer team / Chain Studios
        b"LSD!" // Atari ST JAM Packer Compressed Data (LSD! / LZH! / LZW!)
        | b"LZH!" // Atari ST JAM Packer Compressed Data (LSD! / LZH! / LZW!)
        | b"LZW!" => verify_jam_packer(prefix, size), // Atari ST JAM Packer Compressed Data (LSD! / LZH! / LZW!)
        b"BeEp" => verify_jamcracker_module(prefix, size), // M. Gemmel JamCracker Pro Amiga Tracker Module (.jam, BeEp)
        &[0xCE, 0xCE, 0xCE, 0xCE] // Java KeyStore (JKS/JCEKS)
        | &[0xFE, 0xED, 0xFE, 0xED] => verify_java_keystore(prefix, size), // Java KeyStore (JKS/JCEKS)
        &[0x80, 0x0D, 0x00, 0x00] // Johannes Bjerregaard JayTrax Synth Module (.jxs)
        | &[0x81, 0x0D, 0x00, 0x00] => verify_jaytrax_jxs(prefix, size), // Johannes Bjerregaard JayTrax Synth Module (.jxs)
        b"\x97JB2" => verify_jbig2(prefix, size), // Joint Bi-level Image Experts Group JBIG2 (.jb2 / .jbig2, 2000) ITU-T T.88 / ISO/IEC 14492 ("\x97JB2\r\n\x1A\n")
        &[0x24, 0x21, 0x03, 0x02] => verify_jboot_sch2(prefix, size), // JBOOT SCH2 Second-Stage Kernel Header Image (.bin)
        b"JIF9" => verify_jeffs_image_format(prefix, size), // Jeff's Image Format (.jif, JIF99a)
        b"Buzz" => verify_jeskola_buzz_module(prefix, size), // Oskari Tammelin Jeskola Buzz Modular Song (.bmx / .bmw)
        b"/\x00He" => verify_jetbrains_phm_at(prefix, size), // JetBrains IntelliJ / Kotlin PersistentHashMap Value Storage (.values.at)
        &[0x19, 0x85, 0x20, 0x03] // Red Hat JFFS2 Flash Filesystem Image (.jffs2)
        | &[0x19, 0x85, 0xE0, 0x01] // Red Hat JFFS2 Flash Filesystem Image (.jffs2)
        | &[0x19, 0x85, 0xE0, 0x02] // Red Hat JFFS2 Flash Filesystem Image (.jffs2)
        | &[0x85, 0x19, 0x01, 0xE0] // Red Hat JFFS2 Flash Filesystem Image (.jffs2)
        | &[0x85, 0x19, 0x02, 0xE0] // Red Hat JFFS2 Flash Filesystem Image (.jffs2)
        | &[0x85, 0x19, 0x03, 0x20] => verify_jffs2(prefix, size), // Red Hat JFFS2 Flash Filesystem Image (.jffs2)
        b"JFX1" => verify_jfx1(prefix, size), // GMM Entertainment JFX1 / JPK1 Compressed Archive (.jpk, 1990s) GMM Entertainment
        b"JGXP" => verify_jgxp_package(prefix, size), // JGXP Game Resource Package (.gxp)
        b"PG L" // ISO/IEC 15444-4 JPEG 2000 PGX Verification Component Image (.pgx)
        | b"PG M" => verify_jp2_pgx(prefix, size), // ISO/IEC 15444-4 JPEG 2000 PGX Verification Component Image (.pgx)
        &[0x1C, 0x9A, 0x4A, 0x92] => verify_jphp_module(prefix, size), // JPHP (DevelNext) Compiled PHP-on-JVM Module (.phb, 2014) jphp-group (`ModuleDumper.DUMP_STAMP` 479873682 BE + `DUMP_VERSION` 20141011)
        b"JSOS" => verify_jsos_md8(prefix, size), // JSOS SOS Standard Filesystem 1.x (.md8, MatchWare)
        b"PROP" => verify_juce_prop(prefix, size), // JUCE PropertySet Binary Configuration (.settings, 2004) Raw Material Software
        b"VC2!" => verify_juce_vc2_preset(prefix, size), // JUCE AudioProcessorValueTreeState Binary XML Preset (VC2!, 2015) Raw Material Software / ROLI
        b"\xFBjli" => verify_julia_ji(prefix, size), // Julia Project Precompiled Module Image (.ji)
        b"V6\x00\x00" // JVC Everio Camcorder Media Object Information Sidecar (.moi)
        | b"V7\x00\x00" => verify_jvc_everio_moi(prefix, size), // JVC Everio Camcorder Media Object Information Sidecar (.moi)
        b"0\x00li" => verify_jwlaunch_p2(prefix, size), // Java Web Start Standalone Launcher Bundle (.p2)
        &[0xFF, 0x10, 0xFF, 0x50] => verify_jxs_codestream(prefix, size), // JPEG XS Low-Latency Lightweight Image Codestream (.jxs, 2019) ISO/IEC 21122 (SOC=0xFF10 + CAP=0xFF50)
        b"[Mea" => verify_kapel_mdf(prefix, size), // Lumex Kapel Capillary Electrophoresis Measurement (.mdf)
        b"UPDS" => verify_kaspersky_upds(prefix, size), // Kaspersky Anti-Virus UPDS Gzip Signature Database (.kdc, 2006) Kaspersky Lab
        b"RR\x01)" => verify_kd_red(prefix, size), // RED Compressed Installer File (1990s) Knowledge Dynamics
        b"KWAL" => verify_kde_kwallet(prefix, size), // KDE KWallet Binary Keystore (.kwl, 2003, "KWALLET\n\r\0\r\n")
        &[0x03, 0xD9, 0xA2, 0x9A] => verify_keepass(prefix, size), // KeePass Password Safe v1/v2 Encrypted Database (.kdb / .kdbx, 2003 / 2007) Dominik Reichl (0x9AA2D903 LE)
        b"LJ-V" // Keyence LJ-X8000 / LJ-V7000 Series Laser Profiler Settings (.ljhp)
        | b"LJ-X" => verify_keyence_ljx_ljhp(prefix, size), // Keyence LJ-X8000 / LJ-V7000 Series Laser Profiler Settings (.ljhp)
        b"KGB_" => verify_kgb(prefix, size), // KGB Archiver Compressed Archive (.kgb, "KGB_arch -")
        b"DGKP" => verify_kill_la_kill_dgkp(prefix, size), // APLUS / Arc System Works Kill la Kill: IF DGKP Asset Archive (.pac / .lyt)
        b"CONT" => verify_kindle_kfx(prefix, size), // Amazon Kindle Format 10 Container (.kfx, CONT)
        b"\xEADRM" => verify_kindle_kfx_drm(prefix, size), // Amazon Kindle KFX DRM-Protected Ion Voucher (.kfx / .azw8, \\xEADRMION\\xEE)
        b"XP3\r" => verify_kirikiri_xp3(prefix, size), // KiriKiri2 / KiriKiriZ Visual Novel Archive (2002) W.Dee ("XP3\r\n \n\x1A\x8B\x67\x01")
        b"KTEX" => verify_klei_ktex(prefix, size), // Klei Entertainment KTEX Game Texture (.tex, 2013) Klei Entertainment
        b"cyd!" => verify_klystrack_module(prefix, size), // kometbomb Klystrack Chiptune Song (.kt)
        &[0x80, 0x2A, 0x5F, 0xD7] // Kodak Cineon Digital Film Image (.cin)
        | &[0xD7, 0x5F, 0x2A, 0x80] => verify_kodak_cineon(prefix, size), // Kodak Cineon Digital Film Image (.cin)
        b"TSRP" => verify_kodak_dc20_preset_pst(prefix, size), // Eastman Kodak DC20 Digital Camera Preset (.pst, 1996) Kodak
        b"PHOT" => verify_kodak_photo_cd_info_pcd(prefix, size), // Eastman Kodak Photo CD Disc Information (INFO.PCD, 1992) Kodak
        b"XBTF" => verify_kodi_xbt(prefix, size), // XBMC / Kodi XBTF Texture Bundle (.xbt)
        b"LS11" => verify_koei_ls11_s11(prefix, size), // KOEI Romance of the Three Kingdoms XI LS11 Data Archive (.s11)
        b"G1TG" // Koei Tecmo G1T Texture Archive (.g1t)
        | b"GT1G" => verify_koei_tecmo_g1t(prefix, size), // Koei Tecmo G1T Texture Archive (.g1t)
        b"XWSF" => verify_koei_tecmo_xws(prefix, size), // Koei Tecmo XWS Wave Stream Audio (.xws)
        b"KA A" => verify_kohan_snd(prefix, size), // TimeGate Studios Kohan KA Sound Archive (.snd, 2001) TimeGate Studios
        b"KF\x1A\x00" // ASCON Kompas-3D CAD Drawing / Fragment (.cdw / .frw, 1999) ASCON
        | b"KF\\\x00" => verify_kompas_3d(prefix, size), // ASCON Kompas-3D CAD Drawing / Fragment (.cdw / .frw, 1999) ASCON
        b"xph`" => verify_konami_scc_musixx(prefix, size), // MSX Konami SCC-Musixx Song (.sng)
        &[0x01, 0xA8, 0x9E, 0xD6] => verify_kontakt_ncw(prefix, size), // Native Instruments Kontakt Lossless Compressed Wave (.ncw)
        &[0x0D, 0x0A, 0x20, 0x20] => verify_kontext_lat(prefix, size), // KONTEXT DOS Word Processor Printer/Keyboard Driver (.lat)
        b"VERS" => verify_krita_tile_data(prefix, size), // Krita Layer Tile Data (.kra layer, 2009) KDE / Krita Foundation
        b"KROM" // KROM / MORK 64-Bit Resource Table (.1 / .10d0)
        | b"MORK" => verify_krom_mork_res(prefix, size), // KROM / MORK 64-Bit Resource Table (.1 / .10d0)
        &[0xFF, 0x2A, 0x01, 0x00] => verify_ksp_mu(prefix, size), // Squad Kerbal Space Program Part 3D Model (.mu)
        b"NG\x00\x00" => verify_kudo_catalog_db(prefix, size), // Imspace Kudo Image Browser Catalog Database (.kdb, 1992) Imspace Systems
        b"FASH" => verify_kudo_hash_index(prefix, size), // Imspace Kudo Image Browser Hash Index (.hsh, 1992) Imspace Systems
        &[0x05, 0x28, 0xBC, 0x96] // KuGou Music Encrypted Audio Container (.kgm / .vpr)
        | &[0x7C, 0xD5, 0x32, 0xEB] => verify_kugou_music_kgm(prefix, size), // KuGou Music Encrypted Audio Container (.kgm / .vpr)
        b"KWAJ" => verify_kwaj(prefix, size), // MS-DOS COMPRESS.EXE KWAJ Multi-Method Single-File Archive (1993) Microsoft ("KWAJ\x88\xF0\x27\xD1")
        b"Kwik" => verify_kwikdraw(prefix, size), // KwikDraw Object-Oriented Drawing (.kwk, 1990s) Kwikstart Software ("KwikDraw - ")
        b"LA02" // Michael Bevin La Lossless Audio (.la, 2002) Michael Bevin
        | b"LA03" // Michael Bevin La Lossless Audio (.la, 2002) Michael Bevin
        | b"LA04" => verify_la_lossless_audio(prefix, size), // Michael Bevin La Lossless Audio (.la, 2002) Michael Bevin
        b"IPX\x00" => verify_labgruppen_ipxpreset(prefix, size), // Lab.gruppen IPX Series Amplifier DSP Preset (.ipxpreset) Lab.gruppen
        b"LSOF" => verify_larian_lsof(prefix, size), // Larian Studios LSOF Binary Resource (.lsf, 2014) Larian Studios
        b"LSPK" => verify_larian_lspk(prefix, size), // Larian Studios Divinity / Baldur's Gate 3 LSPK Package (.pak / .lsv, 2014) Larian Studios
        b"LME\x00" => verify_leggless_lme(prefix, size), // Leggless / Silents UK Amiga Music Editor Module (.lme)
        b"ALP " => verify_lego_alp_audio(prefix, size), // High Voltage Software / LEGO Racers ALP Audio (.tun)
        b"LENG" => verify_leica_apsc_firmware(prefix, size), // Leica Camera APS-C ASCII-Headered Firmware Update (.upd / .tuf / .lfu, 2009) Leica Camera
        &[0x1E, 0x1C, 0xAF, 0x1E] // Leica Camera Maestro / Fujitsu Firmware Container (.fw / .s2, 2009) Leica Camera
        | &[0x1E, 0x1C, 0xAF, 0x2E] // Leica Camera Maestro / Fujitsu Firmware Container (.fw / .s2, 2009) Leica Camera
        | &[0x3E, 0xAF, 0x1C, 0x1E] => verify_leica_camera_firmware(prefix, size), // Leica Camera Maestro / Fujitsu Firmware Container (.fw / .s2, 2009) Leica Camera
        b"LEIC" => verify_leica_lens_db(prefix, size), // Leica Camera Lens Correction & Calibration Database (2010) Leica Camera
        b"LFM2" => verify_lfm2_lfmat(prefix, size), // LFM2 3D Material Definition (.lfmat)
        b"{s\x07v" => verify_libgdx_g3db(prefix, size), // libGDX UBJSON Binary 3D Model (.g3db)
        b"LZG\x00" => verify_liblzg_compressed(prefix, size), // Marcus Geelnard liblzg Compressed Stream (.lzg)
        b"SDS2" => verify_lifetech_sds(prefix, size), // Sequence Detection System qPCR Data (.sds, 2000) Applied Biosystems / Life Technologies
        b"L\x00i\x00" => verify_lineage2_ver111_usx(prefix, size), // NCSoft Lineage II Encrypted Unreal Engine Package (Lineage2Ver, .usx / .ukx / .utx)
        b"Dict" => verify_lingea_lld(prefix, size), // Lingea Lexicon Bilingual Dictionary Database (.lld, 2002) Lingea s.r.o.
        &[0xB0, 0x07, 0xF4, 0x57] => verify_linux_kmod_index(prefix, size), // Linux kmod Binary Module Index (modules.dep.bin, 2009) Alan Jenkins
        b"2ELI" // Linux Kernel perf.data Performance Profile (PERFILE2)
        | b"PERF" => verify_linux_perf_data(prefix, size), // Linux Kernel perf.data Performance Profile (PERFILE2)
        &[0x36, 0x04, 0x00, 0x08] // Linux Console PC Screen Font (PSF1 / PSF2)
        | &[0x36, 0x04, 0x00, 0x0E] // Linux Console PC Screen Font (PSF1 / PSF2)
        | &[0x36, 0x04, 0x00, 0x10] // Linux Console PC Screen Font (PSF1 / PSF2)
        | &[0x36, 0x04, 0x02, 0x08] // Linux Console PC Screen Font (PSF1 / PSF2)
        | &[0x36, 0x04, 0x02, 0x0E] // Linux Console PC Screen Font (PSF1 / PSF2)
        | &[0x36, 0x04, 0x02, 0x10] => verify_linux_psf1(prefix, size), // Linux Console PC Screen Font (PSF1 / PSF2)
        b"r\xB5J\x86" => verify_linux_psf2(prefix, size), // Linux Console PC Screen Font v2 (.psf / .psfu, 1999) Andries Brouwer (0x864AB572 LE)
        b"LIQ " // Nir Oren Liquid Tracker Module (.liq)
        | b"Liqu" // Nir Oren Liquid Tracker Module (.liq)
        | b"NO\x00\x00" => verify_liquid_tracker_liq(prefix, size), // Nir Oren Liquid Tracker Module (.liq)
        b"\r\nRe" // Monolith LithTech Engine RezMgr Resource Archive (.rez, 1998) Monolith Productions
        | b"\r\nWi" => verify_lithtech_rez(prefix, size), // Monolith LithTech Engine RezMgr Resource Archive (.rez, 1998) Monolith Productions
        b"CAFF" => verify_live2d_cmo3(prefix, size), // Live2D Cubism 3+ Editor Model Project (.cmo3 / .can3, 2017) Live2D
        b"REVO" // Runtime Revolution / LiveCode Stack (.rev / .livecode)
        | b"REVS" => verify_livecode(prefix, size), // Runtime Revolution / LiveCode Stack (.rev / .livecode)
        b"GSYM" // LLVM GSYM Debug Symbol File
        | b"MYSG" => verify_llvm_gsym(prefix, size), // LLVM GSYM Debug Symbol File
        b"\x80mfo" // LLVM InstrProf Indexed / Raw Coverage Profile Data (.profdata / .profraw)
        | b"\x81Rfo" // LLVM InstrProf Indexed / Raw Coverage Profile Data (.profdata / .profraw)
        | b"\x81ifo" // LLVM InstrProf Indexed / Raw Coverage Profile Data (.profdata / .profraw)
        | b"\x81rfo" // LLVM InstrProf Indexed / Raw Coverage Profile Data (.profdata / .profraw)
        | &[0x82, 0xE4, 0xD0, 0xB1] // LLVM InstrProf Indexed / Raw Coverage Profile Data (.profdata / .profraw)
        | &[0x84, 0xE4, 0xD0, 0xB1] // LLVM InstrProf Indexed / Raw Coverage Profile Data (.profdata / .profraw)
        | b"\xFFlpr" => verify_llvm_profile_data(prefix, size), // LLVM InstrProf Indexed / Raw Coverage Profile Data (.profdata / .profraw)
        b"\x01LOB" // LOB's File Compressor / Multipak Amiga Data Stream (.lob)
        | b"\x02LOB" // LOB's File Compressor / Multipak Amiga Data Stream (.lob)
        | b"\x03LOB" => verify_lob_multipak(prefix, size), // LOB's File Compressor / Multipak Amiga Data Stream (.lob)
        b"JOY\x01" // Locomotive Software LocoScript Document (.loc)
        | b"JOY!" => verify_locoscript_doc(prefix, size), // Locomotive Software LocoScript Document (.loc)
        b"BNKS" => verify_loriciel_bnks(prefix, size), // Loriciel Amiga Sample Bank (.bnk)
        b"Clay" => verify_lotus_cmz(prefix, size), // Lotus Ami Pro CMZ Compressed Archive (.cmz)
        b"*R1\r" // Lotus Manuscript Technical Document (.doc)
        | b"*R2\r" => verify_lotus_manuscript(prefix, size), // Lotus Manuscript Technical Document (.doc)
        b"\x80SCM" => verify_lotus_screencam(prefix, size), // Lotus ScreenCam Screen Recording Movie (.scm, 1994) Lotus
        b"LPAC" => verify_lpac(prefix, size), // Tilman Liebchen LPAC Lossless Predictive Audio Compressor (.pac, 1999) TU Berlin
        b"LSST" => verify_lsstat_stat(prefix, size), // LSSTAT License/System Status Record (.stat)
        b"GOB\n" // LucasArts Dark Forces GOB Archive (.gob)
        | b"GOB " // LucasArts Dark Forces GOB Archive (.gob)
        | b"LABN" => verify_lucasarts_gob(prefix, size), // LucasArts Dark Forces GOB Archive (.gob)
        b"ADL " // LucasArts AdLib Audio Music (.laa)
        | &[0xF3, 0x01, 0x00, 0x00] // LucasArts AdLib Audio Music (.laa)
        | &[0xF3, 0x02, 0x00, 0x00] // LucasArts AdLib Audio Music (.laa)
        | &[0xF3, 0x03, 0x00, 0x00] // LucasArts AdLib Audio Music (.laa)
        | &[0xF3, 0x04, 0x00, 0x00] // LucasArts AdLib Audio Music (.laa)
        | &[0xF3, 0x05, 0x00, 0x00] // LucasArts AdLib Audio Music (.laa)
        | &[0xF3, 0x06, 0x00, 0x00] // LucasArts AdLib Audio Music (.laa)
        | &[0xF3, 0x07, 0x00, 0x00] // LucasArts AdLib Audio Music (.laa)
        | &[0xF3, 0x08, 0x00, 0x00] // LucasArts AdLib Audio Music (.laa)
        | &[0xF3, 0x09, 0x00, 0x00] => verify_lucasarts_laa(prefix, size), // LucasArts AdLib Audio Music (.laa)
        b"SOU " // LucasArts iMUSE / Monster.SOU VCTL Voice Audio (.sou)
        | b"VCTL" => verify_lucasarts_sou(prefix, size), // LucasArts iMUSE / Monster.SOU VCTL Voice Audio (.sou)
        &[0x3F, 0xD7, 0x6C, 0x17] => verify_lucene_si(prefix, size), // Apache Lucene Codec Segment Index File (.si / .cfe / .cfs)
        b"WV\x02\x00" // LuraTech LuraWave Wavelet Bitmap (.lwf)
        | b"WV\xF2\x00" => verify_lurawave_lwf(prefix, size), // LuraTech LuraWave Wavelet Bitmap (.lwf)
        b"LUXO" => verify_luxonix_purity_pth(prefix, size), // Luxonix / Sonic Cat Purity Rompler Synthesizer Patch (.pth, 2005) Luxonix
        b"\x89LFP" => verify_lytro_lfp(prefix, size), // Lytro Light Field Picture Container (.lfp / .lfr)
        b"ZV\x00\x00" // LZF compressed stream
        | b"ZV\x01\x00" => verify_lzf(prefix, size), // LZF compressed stream
        b"bvx-" // Apple LZFSE / LZVN compressed stream
        | b"bvx1" // Apple LZFSE / LZVN compressed stream
        | b"bvx2" // Apple LZFSE / LZVN compressed stream
        | b"bvxn" => verify_lzfse(prefix, size), // Apple LZFSE / LZVN compressed stream
        b"LZM1" => verify_lzm_compressed(prefix, size), // LZM1 Chunked Compressed Stream (.lzm)
        b"\x89LZO" => verify_lzop(prefix, size), // lzop LZO compressed file (.lzo)
        b"LZP1" => verify_lzp1_compressed(prefix, size), // Charles Bloom LZP1 Compressed Stream (.lzp)
        &[0xFE, 0x32, 0x00, 0x00] // Microsoft Word for Macintosh Classic Document (.mcw, v1-v5)
        | &[0xFE, 0x32, 0x00, 0x1C] // Microsoft Word for Macintosh Classic Document (.mcw, v1-v5)
        | &[0xFE, 0x34, 0x00, 0x00] // Microsoft Word for Macintosh Classic Document (.mcw, v1-v5)
        | &[0xFE, 0x34, 0x00, 0x1C] // Microsoft Word for Macintosh Classic Document (.mcw, v1-v5)
        | &[0xFE, 0x37, 0x00, 0x1C] // Microsoft Word for Macintosh Classic Document (.mcw, v1-v5)
        | &[0xFE, 0x37, 0x00, 0x23] => verify_mac_word_doc(prefix, size), // Microsoft Word for Macintosh Classic Document (.mcw, v1-v5)
        b"1SLD" // Apple macOS FSEvents File System Event Log (1SLD / 2SLD / 3SLD)
        | b"2SLD" // Apple macOS FSEvents File System Event Log (1SLD / 2SLD / 3SLD)
        | b"3SLD" => verify_macos_fsevents(prefix, size), // Apple macOS FSEvents File System Event Log (1SLD / 2SLD / 3SLD)
        b"utmp" => verify_macos_utmpx(prefix, size), // Apple macOS utmpx User Accounting Log ("utmpx-1.00\0")
        b"ME\xA0\x0F" => verify_macro_express_mex(prefix, size), // Insight Software Solutions Macro Express Macro File (.mex)
        &[0x00, 0x2E, 0x00, 0x2E] => verify_macwrite2(prefix, size), // Claris MacWrite II Document (.mw2)
        b"MT20" => verify_madtracker2_module(prefix, size), // Yannick Delwiche MadTracker 2 Module (.mt2)
        b"VOX " => verify_magicavoxel(prefix, size), // ephtracy MagicaVoxel 3D Voxel Model (.vox, VOX )
        b"id=I" // ImageMagick Machine Independent File Format (.miff)
        | &[0x7B, 0x0A, 0x20, 0x20] => verify_magick_miff(prefix, size), // ImageMagick Machine Independent File Format (.miff)
        b"MaSc" => verify_magnetic_scrolls_game(prefix, size), // Magnetic Scrolls Game Data (.mag)
        b"MaP2" // Magnetic Scrolls Picture Container (.gfx)
        | b"MaP4" // Magnetic Scrolls Picture Container (.gfx)
        | b"MaPi" => verify_magnetic_scrolls_gfx(prefix, size), // Magnetic Scrolls Picture Container (.gfx)
        b"!MCC" // Malcat Constant & Kesakode Signature Database (.csig)
        | b"MCST" => verify_malcat_csig(prefix, size), // Malcat Constant & Kesakode Signature Database (.csig)
        b"MAME" => verify_mame_mfi(prefix, size), // MAME Floppy Image (.mfi, "MAMEFLOP")
        &[0x2C, 0x00, 0x00, 0x00] => verify_mandelbulb3d_m3p(prefix, size), // Mandelbulb 3D Fractal Parameter Preset (.m3p)
        b"FFFF" // Mariner Write Document (v1.6-3.5)
        | b"MRNW" => verify_mariner_write_doc(prefix, size), // Mariner Write Classic Mac Document (.mwd)
        &[0x48, 0xE7, 0x00, 0xF0] => verify_mark2_music_system(prefix, size), // Cachet Software Amiga Mark II Music System Module (.mk2)
        &[0xD0, 0x40, 0xD0, 0x40] => verify_mark_cooksey(prefix, size), // Mark Cooksey Amiga Music Module (.mc)
        b"DTRZ" => verify_marmalade_dz(prefix, size), // Ideaworks3D Marmalade SDK Derbh Compressed Archive (.dz)
        b"CPT0" => verify_marvell_cpt_microcode(prefix, size), // Marvell OCTEON TX2 / CN10K Cryptographic Acceleration Engine Microcode (CPT01)
        b"SCOR" => verify_master_tracks_mts(prefix, size), // Passport Designs Master Tracks Pro Sequence (.mts)
        &[0x78, 0x00, 0x00, 0x00] => verify_mastercam_v9_mc9(prefix, size), // CNC Software Mastercam v9 CAD/CAM Part (.mc9)
        b"MTRA" => verify_mastertracker_mtr(prefix, size), // Kane Wood / Arkha Master Tracker Module (.mtr)
        &[0x1E, 0xF1, 0xEE, 0x1B] => verify_matter_ota(prefix, size), // Connectivity Standards Alliance Matter OTA Software Update Image (.ota)
        b"DBPF" => verify_maxis_dbpf(prefix, size), // Maxis / Electronic Arts Database Packed File (.package / .dat, 2003) Maxis
        b"MXTX" => verify_maxtrax_module(prefix, size), // Taliesin Amiga MaxTrax Module (.mxtx)
        b"\x89MCA" => verify_mcap(prefix, size), // Foxglove Robotics MCAP Container (.mcap)
        b"MCFT" => verify_mcft_cft(prefix, size), // MCFT Bitmap/Vector Font Resource (.cft)
        b"\x18MCS" => verify_mcs_stimulus_stm(prefix, size), // Multi Channel Systems MC_Stimulus II Waveform Definition (.stm) Multi Channel Systems
        &[0x3D, 0xB8, 0xF3, 0x96] => verify_mcuboot_image(prefix, size), // MCUboot / Apache Mynewt Secure Bootloader Firmware Image (.bin)
        b"MDCD" // Dietrich / MDCD Compressed Archive (.mdcd)
        | b"MDmd" => verify_mdcd_archive(prefix, size), // Dietrich / MDCD Compressed Archive (.mdcd)
        b"_MDX" => verify_mdxelm_g2p(prefix, size), // MDXELM DTPS 3D Part Element (.g2p)
        b"BRLY" => verify_mediatek_brlyt(prefix, size), // MediaTek BootROM Layout Descriptor Header (.bin)
        &[0x88, 0x16, 0x88, 0x58] => verify_mediatek_image(prefix, size), // MediaTek Partition Image (0x58881688, .img / .bin) MediaTek
        b"EMMC" // MediaTek EMMC_BOOT / SF_BOOT Preloader Image (.bin) MediaTek
        | b"SF_B" => verify_mediatek_preloader(prefix, size), // MediaTek EMMC_BOOT / SF_BOOT Preloader Image (.bin) MediaTek
        b"BFBF" // MediaTek Signed Partition Image (BFBF / SSSS)
        | b"SSSS" => verify_mediatek_signed_image(prefix, size), // MediaTek Signed Partition Image (BFBF / SSSS)
        b"MSOB" => verify_medley_mso(prefix, size), // Medley Sound Amiga Module (.mso)
        b"SNGM" => verify_megastation_msm(prefix, size), // MegaStation Compressed MIDI Module (.msm)
        b"\x00MGT" // Simplet / Fatal Design Atari Falcon MegaTracker Module (.mgt)
        | b"MGT\x11" => verify_megatracker_mgt(prefix, size), // Simplet / Fatal Design Atari Falcon MegaTracker Module (.mgt)
        b"{U\x03r" => verify_melee_slippi(prefix, size), // Project Slippi Super Smash Bros. Melee Replay (.slp, 2018) Project Slippi
        b"CFF-" => verify_mercedes_cff(prefix, size), // Mercedes-Benz Caesar CFF ECU Flash Firmware (.cff) Mercedes-Benz
        b"JL\x1A\x00" // Metakit v4 Database / Tcl Starkit (.mk4 / .kit, JL/LJ)
        | b"JL\x1A\x80" // Metakit v4 Database / Tcl Starkit (.mk4 / .kit, JL/LJ)
        | b"LJ\x1A\x00" // Metakit v4 Database / Tcl Starkit (.mk4 / .kit, JL/LJ)
        | b"LJ\x1A\x80" => verify_metakit(prefix, size), // Metakit v4 Database / Tcl Starkit (.mk4 / .kit, JL/LJ)
        b"EX4\x00" => verify_metatrader_ex4(prefix, size), // MetaQuotes MetaTrader 4 Compiled MQL4 Expert Advisor Bytecode (.ex4, 2005) MetaQuotes
        b"EX5\x02" => verify_metatrader_ex5(prefix, size), // MetaQuotes MetaTrader 5 Compiled MQL5 Expert Advisor Bytecode (.ex5, 2010) MetaQuotes
        &[0xF6, 0x01, 0x00, 0x00] => verify_metatrader_hc(prefix, size), // MetaQuotes MetaTrader 5 History Cache (.hc)
        b"Alpa" => verify_metatrader_srv(prefix, size), // MetaQuotes MetaTrader Broker Server Definition (.srv)
        b"MPFS" => verify_microchip_mpfs(prefix, size), // Microchip MPFS2 Embedded HTTP Read-Only Filesystem (.mpfs)
        &[0x39, 0x03, 0x03, 0x00] => verify_microfocus_gnt(prefix, size), // Micro Focus COBOL Generated Native Code (.gnt, Micro Focus)
        b"#Pro" => verify_microhouse_prosec_bin(prefix, size), // Micro House Technical Library Pro-Sec Hardware Database (.bin, 1994) Micro House
        b"^vMI" => verify_micromeritics_subset(prefix, size), // Micromeritics Instrument Subset Database (.smp / .mth / .anc) Micromeritics
        &[0x57, 0x8F, 0x1C, 0xD0] => verify_microsoft_prague_binary(prefix, size), // Microsoft Fluid Framework / Loop Prague Binary Serialization (.loop / .fluid)
        b"vhdx" => verify_microsoft_vhdx(prefix, size), // Microsoft Hyper-V Virtual Hard Disk v2 (.vhdx, 2012) Microsoft ("vhdxfile")
        b"UN04" // Jean-Paul Mikkers MikMod UniMod Module (.uni)
        | b"UN05" // Jean-Paul Mikkers MikMod UniMod Module (.uni)
        | b"UN06" => verify_mikmod_uni_module(prefix, size), // Jean-Paul Mikkers MikMod UniMod Module (.uni)
        &[0x1E, 0xF1, 0xD0, 0xBA] => verify_mikrotik_npk(prefix, size), // MikroTik RouterOS NPK Firmware Package (.npk)
        &[0x0D, 0xF0, 0x1D, 0xC0] => verify_mikrotik_winbox_cdb(prefix, size), // MikroTik WinBox Address Book & RouterOS Configuration Database (.cdb, 2002) MikroTik
        b"MVB0" // KIDASA Software Milestones Professional Schedule (.mlb / .mlq) KIDASA Software
        | b"MVQ0" => verify_milestones_mlb(prefix, size), // KIDASA Software Milestones Professional Schedule (.mlb / .mlq) KIDASA Software
        b"MENG" => verify_mindware_mfs_archive(prefix, size), // Mindware Studios Meng Engine Archive (.mfs)
        &[0x0A, 0x90, 0x03, 0x0A] => verify_minitaur_log_proto(prefix, size), // Minitaur Robotics Latency Telemetry Protobuf Log (.proto)
        b"MKJa" => verify_mkjamz_mkj(prefix, size), // MKJamz AdLib FM Module (.mkj)
        b"MAD+" => verify_mlat_mad(prefix, size), // Søren Lund MLAT AdLib Tracker Module (.mad)
        b"\x00mlo" => verify_mlocate_db(prefix, size), // Linux mlocate Database ("\0mlocate")
        b"GEOV" => verify_mm_geov1_r565(prefix, size), // Magneti Marelli RT6/SMEG Automotive Navigation GEOV1 RGB565 Image (.r565) Magneti Marelli
        b"ziRC" => verify_mmcmp(prefix, size), // Emmanuel Giasson Music Module Compressor (.mmcmp / .zig)
        b"Pmd\x00" => verify_mmd_pmd(prefix, size), // MikuMikuDance Polygon Model Data (.pmd, 2008) Yu Higuchi
        b"PMX " => verify_mmd_pmx(prefix, size), // MikuMikuDance Polygon Model eXtended (.pmx, 2011) Yu Higuchi
        b"Voca" => verify_mmd_vmd(prefix, size), // MikuMikuDance Vocaloid Motion Data (.vmd, 2008) Yu Higuchi
        b"MO3\x00" // Un4seen Developments MO3 Compressed Tracker Module (.mo3)
        | b"MO3\x01" // Un4seen Developments MO3 Compressed Tracker Module (.mo3)
        | b"MO3\x02" // Un4seen Developments MO3 Compressed Tracker Module (.mo3)
        | b"MO3\x03" // Un4seen Developments MO3 Compressed Tracker Module (.mo3)
        | b"MO3\x04" // Un4seen Developments MO3 Compressed Tracker Module (.mo3)
        | b"MO3\x05" => verify_mo3(prefix, size), // Un4seen Developments MO3 Compressed Tracker Module (.mo3)
        b"MCLF" => verify_mobicore_mclf(prefix, size), // Trustonic Kinibi / G&D MobiCore TEE Loadable Binary (MCLF)
        b"XMF_" => verify_mobile_xmf(prefix, size), // Extensible Music Format / Mobile XMF (2001) MIDI Manufacturers Association
        b"Mast" => verify_mom_grf(prefix, size), // Simtex Master of Magic Graphics Resource (.grf, 1994) Simtex / MicroProse
        b"\x08MON" => verify_monotone_module(prefix, size), // Trixter MONOTONE PC Speaker / OPL Tracker Module (.mon)
        b"MOOF" => verify_moof(prefix, size), // Applesauce Macintosh MOOF Flux / Bitstream Floppy Image (.moof, 2021) John Morris ("MOOF\xFF\n\r\n")
        b"\x036.0" => verify_moshidraw_yz(prefix, size), // MoshiDraw / WINYZ Laser Engraving Design v6.0 (.yz)
        b"Moto" => verify_motorola_motologo(prefix, size), // Motorola MotoLogo Boot Logo Container (.bin)
        b"MOTO" => verify_motorola_piv_image(prefix, size), // Motorola PIV Signed Partition Image (MOT_PIV_FULL256)
        b"\x04GM0" // Roger Wagner MouseWrite Apple IIe/IIc/IIGS Word Processor Document (.mw)
        | b"MW\x01\x00" // Roger Wagner MouseWrite Apple IIe/IIc/IIGS Word Processor Document (.mw)
        | b"MW\x04\x00" => verify_mousewrite_doc(prefix, size), // Roger Wagner MouseWrite Apple IIe/IIc/IIGS Word Processor Document (.mw)
        b"*FRM" => verify_moxa_frm(prefix, size), // Moxa Industrial Device Firmware Image (.frm)
        b"mozL" => verify_mozlz4(prefix, size), // Mozilla Firefox LZ4-Compressed JSON Container (.jsonlz4 / .mozlz4, 2014) Mozilla Foundation ("mozLz40\0")
        b"MPQ\x1A" // Blizzard MoPaQ Archive (.mpq)
        | b"MPQ\x1B" => verify_mpq(prefix, size), // Blizzard MoPaQ Archive (.mpq)
        b"mpu4" => verify_mpu401_mtk(prefix, size), // MPU-401 Tracker Compressed Module (.mtk)
        b"MSID" // LizardTech MrSID Multiresolution Seamless Image (.sid)
        | b"msid" => verify_mrsid(prefix, size), // LizardTech MrSID Multiresolution Seamless Image (.sid)
        b"RITE" => verify_mruby_rite(prefix, size), // mruby RiteVM Compiled Bytecode (.mrb)
        b"LN\x02\x00" => verify_ms_advisor_help(prefix, size), // Microsoft Advisor MS-DOS Help File (.hlp, 1990) Microsoft
        &[0x8A, 0x01, 0x09, 0x00] // Microsoft Office Answer Wizard Database (.aw)
        | &[0x8A, 0x01, 0x0A, 0x00] => verify_ms_answer_wizard_aw(prefix, size), // Microsoft Office Answer Wizard Database (.aw)
        b"N\x00P\x00" => verify_ms_netmon_npb(prefix, size), // Microsoft Network Monitor 3 Compiled Parser (.npb)
        b"XT\x01\x00" => verify_ms_publisher_borderart_bdr(prefix, size), // Microsoft Publisher BorderArt File (.bdr, 1993) Microsoft
        b"POCP" => verify_ms_publisher_pagewizard_poc(prefix, size), // Microsoft Publisher PageWizard Design Assistant (.poc, 1995) Microsoft
        &[0x89, 0x6C, 0xA2, 0x00] => verify_ms_publisher_preview_prv(prefix, size), // Microsoft Publisher Template Preview (.prv, 1995) Microsoft
        b"QQ\x00\x00" => verify_ms_setup_tdf(prefix, size), // Microsoft Setup Font/Typeface Definition (SETUP.TDF, 1991) Microsoft
        b"LOGO" => verify_ms_windows_boot_logo_lgo(prefix, size), // Microsoft Windows 3.x EGA/VGA Startup Logo Code Stub (.lgo, 1990) Microsoft
        &[0x12, 0x00, 0x00, 0x00] => verify_ms_windows_postscript_wpd(prefix, size), // Microsoft Windows 3.1 PostScript Printer Description (.wpd, 1992) Microsoft
        &[0x01, 0xFE, 0x00, 0x00] // Microsoft Works for MS-DOS Word Processor Document (.wps)
        | &[0x01, 0xFE, 0x01, 0x00] => verify_ms_works_dos_wps(prefix, size), // Microsoft Works for MS-DOS Word Processor Document (.wps)
        b"DCD\x01" // Microsoft Windows Update Delta Compression Patch (PA30 / PA19)
        | b"DCM\x01" // Microsoft Windows Update Delta Compression Patch (PA30 / PA19)
        | b"PA19" // Microsoft Windows Update Delta Compression Patch (PA30 / PA19)
        | b"PA30" // Microsoft Windows Update Delta Compression Patch (PA30 / PA19)
        | b"PA31" => verify_msdelta_patch(prefix, size), // Microsoft Windows Update Delta Compression Patch (PA30 / PA19)
        b"\xFFCOU" => verify_msdos_country_sys(prefix, size), // MS-DOS / PC DOS Country Information Table (COUNTRY.SYS, 1987) Microsoft / IBM
        b"\xFFKEY" => verify_msdos_keyboard_sys(prefix, size), // MS-DOS / PC DOS Keyboard Layout Table (KEYBOARD.SYS, 1987) Microsoft / IBM
        &[0x01, 0x02, 0x92, 0x19] => verify_msfs_bgl(prefix, size), // Microsoft Flight Simulator BGL Scenery Database (.bgl, 2003) Microsoft
        &[0xAC, 0xEB, 0x14, 0x00] => verify_msfs_spb(prefix, size), // Microsoft Flight Simulator SimProp Binary XML (.spb, 2006) Microsoft
        b"Clie" => verify_msie_cache_index_dat(prefix, size), // Microsoft Internet Explorer WinINet URL Cache (index.dat)
        b"DanM" // Microsoft Paint v1/v2 bitmap (.msp)
        | b"LINS" // Microsoft Paint v1/v2 bitmap (.msp)
        | b"LinS" => verify_msp(prefix, size), // Microsoft Paint v1/v2 bitmap (.msp)
        b"msto" => verify_mstor_ibx(prefix, size), // mstor Indexed Mailbox Archive (.ibx)
        b"JINX" // Microsoft Train Simulator Kuju SIMIS Binary Data (.s / .ace, SIMISA@F)
        | b"SIMI" => verify_msts_simis_binary(prefix, size), // Microsoft Train Simulator Kuju SIMIS Binary Data (.s / .ace, SIMISA@F)
        b"TQE\x1A" => verify_msvc_ifc(prefix, size), // Microsoft Visual C++ C++20 Built Module Interface (.ifc)
        b"H.Q\x7F" => verify_msvc_ipdb(prefix, size), // Microsoft Visual C++ Incremental Linker Program Database (.ipdb)
        &[0x20, 0x54, 0x02, 0x00] => verify_msworks_wdb(prefix, size), // Microsoft Works Database v1-2 (.wdb / .bdb, 1987) Microsoft
        &[0x1F, 0xA6, 0xDE, 0xBA] => verify_msx_cas(prefix, size), // MSX Emulator Cassette Tape Image (.cas)
        &[0xFE, 0x00, 0x80, 0xFF] => verify_msx_fac_soundtracker(prefix, size), // MSX FAC SoundTracker Music Module (.mus / .sm1 / .sm2)
        b"KSCC" // MSX / Sega Master System KSS Chiptune Audio (KSCC / KSSX)
        | b"KSSX" => verify_msx_kss_audio(prefix, size), // MSX / Sega Master System KSS Chiptune Audio (KSCC / KSSX)
        b"MBMS" => verify_msx_mbms_audio(prefix, size), // MSX MoonBlaster for MoonSound OPL4 Music (.mbm)
        b"MGS0" // MSX MGSDRV Chiptune Music Data (.mgs)
        | b"MGS1" // MSX MGSDRV Chiptune Music Data (.mgs)
        | b"MGS2" // MSX MGSDRV Chiptune Music Data (.mgs)
        | b"MGS3" // MSX MGSDRV Chiptune Music Data (.mgs)
        | b"MGSA" // MSX MGSDRV Chiptune Music Data (.mgs)
        | b"MGSB" => verify_msx_mgsdrv_audio(prefix, size), // MSX MGSDRV Chiptune Music Data (.mgs)
        b"MPK0" // MSX K-kaz MPK Chiptune Music Data (.mpk)
        | b"MPK1" // MSX K-kaz MPK Chiptune Music Data (.mpk)
        | b"MPK2" => verify_msx_mpk_audio(prefix, size), // MSX K-kaz MPK Chiptune Music Data (.mpk)
        b"OPXA" // MSX / DOS OPLLDRV FM Chiptune (.opx)
        | b"TSW1" => verify_msx_oplldrv_audio(prefix, size), // MSX / DOS OPLLDRV FM Chiptune (.opx)
        b"MTC1" => verify_mtc_chiptune(prefix, size), // MTC Chiptune Module (.mtc)
        b"D$  " => verify_multimate(prefix, size), // MultiMate Advantage II Word Processor Document (.doc, 1987) Will Jones / Multimate International & Ashton-Tate
        &[0x08, 0xE7, 0x00, 0x00] // Microsoft Multiplan Spreadsheet (.mod / .mp, 1982) Microsoft
        | &[0x0A, 0xEC, 0x00, 0x00] // Microsoft Multiplan Spreadsheet (.mod / .mp, 1982) Microsoft
        | &[0x0C, 0xE7, 0x00, 0x00] // Microsoft Multiplan Spreadsheet (.mod / .mp, 1982) Microsoft
        | &[0x0C, 0xE9, 0x00, 0x00] // Microsoft Multiplan Spreadsheet (.mod / .mp, 1982) Microsoft
        | &[0x0C, 0xEC, 0x00, 0x00] // Microsoft Multiplan Spreadsheet (.mod / .mp, 1982) Microsoft
        | &[0x0C, 0xED, 0x00, 0x00] // Microsoft Multiplan Spreadsheet (.mod / .mp, 1982) Microsoft
        | b"\x0C\xEFMP" // Microsoft Multiplan Spreadsheet (.mod / .mp, 1982) Microsoft
        | &[0x11, 0xAB, 0x00, 0x00] => verify_multiplan(prefix, size), // Microsoft Multiplan Spreadsheet (.mod / .mp, 1982) Microsoft
        b"\xCDMst" => verify_music_studio_sng(prefix, size), // Activision The Music Studio Song (.sng)
        b"MLED" => verify_musicline_module(prefix, size), // Conny & Christian Cyrus Musicline Editor Amiga Module (.ml, MLEDMODL)
        &[0x02, 0xC5, 0xE2, 0xC4] // IBM System/370 & z/OS MVS Object Deck (ESD/TXT/RLD/END)
        | &[0x02, 0xE2, 0xE8, 0xD4] => verify_mvs_object_deck(prefix, size), // IBM System/370 & z/OS MVS Object Deck (ESD/TXT/RLD/END)
        b"MVSM" => verify_mvs_tracker(prefix, size), // MVS Tracker Module (.mus)
        b"MVM\x00" => verify_mvx_mvm(prefix, size), // MVX Tracker Module (.mvm)
        &[0x01, 0xF9, 0xEC, 0x01] // Nucleotide Archival Format (.naf)
        | &[0x01, 0xF9, 0xEC, 0x02] => verify_naf_bio(prefix, size), // Nucleotide Archival Format (.naf)
        b"HBSS" => verify_namco_nu_binary(prefix, size), // Bandai Namco Smash Bros. Ultimate NuLibrary Binary Resource (.nuanmb / .numatb / .numshb / .numdlb)
        b"NAMX" => verify_namx_profile(prefix, size), // NAMX Neural Amp Modeler Extended Profile (.namx)
        b"RDOF" => verify_nasm_rdoff(prefix, size), // NASM Relocatable Dynamic Object File Format (.rdf, RDOFF2)
        b"# VE" => verify_nbi_legacy(prefix, size), // NBI Legacy Word Processing Document (.leg, "# VE # 4.00")
        b"NCBI" => verify_ncbi_sra(prefix, size), // NCBI Sequence Read Archive KAR Container (.sra, 2007) NCBI / NIH ("NCBI.sra")
        b"BTX0" => verify_nds_nsbtx(prefix, size), // Nintendo DS Nitro SDK 3D Binary Texture (.nsbtx, 2004) Nintendo
        b" LIC" // SNK Neo Geo Pocket / Pocket Color Cartridge ROM (.ngp / .ngc)
        | b"COPY" => verify_neo_geo_pocket_rom(prefix, size), // SNK Neo Geo Pocket / Pocket Color Cartridge ROM (.ngp / .ngc)
        b"NED:" => verify_nerdtracker2_module(prefix, size), // Bananmos NES NerdTracker II Module (.ned)
        b"NESM" // Nintendo Entertainment System Sound Format (.nsf / .nsfe)
        | b"NSFE" => verify_nes_nsf(prefix, size), // Nintendo Entertainment System Sound Format (.nsf / .nsfe)
        b"*#$^" => verify_netgear_chk(prefix, size), // NETGEAR CHK Router Firmware Image (.chk)
        b"Firm" => verify_netron_nfw(prefix, size), // Obsidian Control Systems Netron DMX Node Firmware Image (.nfw) Obsidian Control Systems
        b"Neww" => verify_newword_ovr(prefix, size), // NewStar NewWord 2 Messages Overlay (.ovr)
        &[0xAA, 0x47, 0x3C, 0x03] => verify_nexon_kartrider_1s(prefix, size), // Nexon Crazyracing KartRider 3D Model (.1s)
        &[0x69, 0x02, 0xFC, 0xAE] => verify_nextstep_wmd(prefix, size), // NeXTSTEP Workspace Manager Window Attributes (.wmd, 1989) NeXT
        b"IMGD" => verify_nexus_imgdata(prefix, size), // Google Nexus IMGDATA! Boot Splash Container (.img)
        &[0x0C, 0xA5, 0x01, 0x00] // nfdump NetFlow / IPFIX Binary Capture (nfcapd, 0xA50C)
        | &[0x0C, 0xA5, 0x02, 0x00] => verify_nfdump_flow_data(prefix, size), // nfdump NetFlow / IPFIX Binary Capture (nfcapd, 0xA50C)
        &[0x37, 0x80, 0x40, 0x12] // Nintendo 64 ROM Image (.z64 / .n64 / .v64)
        | &[0x40, 0x12, 0x37, 0x80] // Nintendo 64 ROM Image (.z64 / .n64 / .v64)
        | &[0x80, 0x37, 0x12, 0x40] => verify_nintendo64_rom(prefix, size), // Nintendo 64 ROM Image (.z64 / .n64 / .v64)
        b"AAMP" => verify_nintendo_aamp(prefix, size), // Nintendo EPD Binary Parameter Archive (.bxml / .baatarc / .aamp, 2015) Nintendo
        b"AIB " => verify_nintendo_ainb(prefix, size), // Nintendo AI Node Binary Graph (.ainb, 2022) Nintendo
        b"STRM" => verify_nintendo_ast(prefix, size), // Nintendo GameCube / Wii AST Audio Stream (.ast)
        b"CGFX" => verify_nintendo_bcres(prefix, size), // Nintendo 3DS NW4C Binary CTR Resource Package (2011) Nintendo EAD
        b"FRES" => verify_nintendo_bfres(prefix, size), // Nintendo Wii U / Switch NW4F Binary Cafe Resource Package (2012 / 2017) Nintendo EAD / EPD
        b"GSEM" // Nintendo JSystem Binary Message Group (.bmg, 2001) Nintendo
        | b"MESG" => verify_nintendo_bmg(prefix, size), // Nintendo JSystem Binary Message Group (.bmg, 2001) Nintendo
        b"BNTX" => verify_nintendo_bntx_texture(prefix, size), // Nintendo Switch Binary Texture Container (.bntx)
        b"RSAR" => verify_nintendo_brsar(prefix, size), // Nintendo GameCube / Wii Sound Archive (.brsar, 2006) Nintendo
        b"RSTM" => verify_nintendo_brstm(prefix, size), // Nintendo Wii Binary Revolution Stream Audio (.brstm)
        b"RWAV" => verify_nintendo_brwav(prefix, size), // Nintendo Wii Binary Revolution Wave Audio (.brwav)
        b"BWAV" => verify_nintendo_bwav(prefix, size), // Nintendo Switch / Wii U Binary Wave Audio (.bwav)
        b"\x01*NI" // Nintendo Famicom Disk System Disk Image (.fds)
        | b"FDS\x1A" => verify_nintendo_fds_disk(prefix, size), // Nintendo Famicom Disk System Disk Image (.fds)
        b"CFNT" // Nintendo NW4R / NW4C Binary Font Resource (RFNT / CFNT / FFNT)
        | b"FFNT" // Nintendo NW4R / NW4C Binary Font Resource (RFNT / CFNT / FFNT)
        | b"RFNT" // Nintendo NW4R / NW4C Binary Font Resource (RFNT / CFNT / FFNT)
        | b"RTFN" => verify_nintendo_font(prefix, size), // Nintendo NW4R / NW4C Binary Font Resource (RFNT / CFNT / FFNT)
        b"J3D1" // Nintendo GameCube / Wii JSystem 3D Model / Animation (.bmd / .bdl / .bck / .btk / .brk)
        | b"J3D2" => verify_nintendo_j3d(prefix, size), // Nintendo GameCube / Wii JSystem 3D Model / Animation (.bmd / .bdl / .bck / .btk / .brk)
        b"MsgF" // Nintendo LMS Binary Message Table / Flow / Project (.msbt / .msbf / .msbp, 2010) Nintendo
        | b"MsgP" // Nintendo LMS Binary Message Table / Flow / Project (.msbt / .msbf / .msbp, 2010) Nintendo
        | b"MsgS" => verify_nintendo_msbt(prefix, size), // Nintendo LMS Binary Message Table / Flow / Project (.msbt / .msbf / .msbp, 2010) Nintendo
        b"NARC" => verify_nintendo_narc_archive(prefix, size), // Nintendo DS Nitro Archive (.narc / .carc, NARC)
        b"NES\x1A" => verify_nintendo_nes_rom(prefix, size), // Nintendo Entertainment System / Famicom ROM (.nes / iNES / NES 2.0, 1997) Marat Fayzullin
        b"CRAR" // Nintendo GameCube / Wii JKR Archive (.rarc / .arc)
        | b"RARC" => verify_nintendo_rarc(prefix, size), // Nintendo GameCube / Wii JKR Archive (.rarc / .arc)
        b"SARC" => verify_nintendo_sarc(prefix, size), // Nintendo Wii U / 3DS / Switch SARC Resource Archive (.sarc)
        b"U\xAA8-" => verify_nintendo_u8_archive(prefix, size), // Nintendo Wii / Wii U U8 Content Archive (.arc / .szs, U\\xAA8-)
        b"Gfx2" => verify_nintendo_wiiu_gtx(prefix, size), // Nintendo Wii U GX2 Texture Container (.gtx)
        b"Yay0" => verify_nintendo_yay0(prefix, size), // Nintendo Yay0 SZP Compressed Stream (.yay0 / .szp)
        b"Yaz0" => verify_nintendo_yaz0(prefix, size), // Nintendo Yaz0 Run-Length Compressed Stream (.szs / .yaz0)
        b"NIST" => verify_nist_sphere_audio(prefix, size), // NIST SPHERE Speech Header Resources Audio (.sph / .wav)
        b"NPA\x01" => verify_nitroplus_npa(prefix, size), // Nitroplus Visual Novel Engine NPA Archive (.npa, 2004) Nitroplus
        &[0xCC, 0xCC, 0xCC, 0xCC] => verify_nms_mbin(prefix, size), // Hello Games No Man's Sky Compiled EXML Binary (.mbin)
        b"NGG\x00" => verify_nokia_ngg(prefix, size), // Nokia Group Graphics Bitmap (.ngg)
        b"NLM " => verify_nokia_nlm(prefix, size), // Nokia Logo Manager Bitmap (.nlm)
        b"NPM\x00" => verify_nokia_npm(prefix, size), // Nokia Picture Message Bitmap (.npm)
        b"Abor" => verify_norton_commander_msg(prefix, size), // Peter Norton Computing Norton Commander Message File (.msg)
        b"EH\x00\x01" // Peter Norton Computing Norton Guides Help Database (.ng / .hlp, 1987) Peter Norton Computing
        | b"NG\x00\x01" => verify_norton_guides_hlp(prefix, size), // Peter Norton Computing Norton Guides Help Database (.ng / .hlp, 1987) Peter Norton Computing
        b"NOS_" // Netopsystems FEAD Optimizer (NOS_PO) Archive
        | &[0xD5, 0xD9, 0xE9, 0x1D] => verify_nos_installer_archive(prefix, size), // Netopsystems FEAD Optimizer (NOS_PO) Archive
        &[0x3B, 0x9E, 0x00, 0x00] // C-Lab Notator / Creator MIDI song arrangement
        | &[0x3B, 0x9E, 0x00, 0x03] => verify_notator_song(prefix, size), // C-Lab Notator / Creator MIDI song arrangement
        &[0x32, 0x5E, 0x10, 0x10] // Novell / Tumbleweed Envoy Document (.evy)
        | &[0xB2, 0x97, 0xE1, 0x69] => verify_novell_envoy(prefix, size), // Novell / Tumbleweed Envoy Document (.evy)
        b"DOS " // Novell NetWare / Message Librarian Message Catalog (.msg, 1989) Novell
        | b"NetW" => verify_novell_netware_message_msg(prefix, size), // Novell NetWare Message Catalog (.msg)
        b"SYS$" => verify_novell_sys_help_hep(prefix, size), // Novell NetWare SYS$HELP Help File (.hep, 1989) Novell
        &[0xED, 0xBE, 0xAD, 0xDE] // Nullsoft Scriptable Install System (NSIS) data
        | &[0xEF, 0xBE, 0xAD, 0xDE] => verify_nsis_at_deadbeef(prefix, size), // Nullsoft Scriptable Install System (NSIS) data
        b"\x00ntf" => verify_ntfsclone_image(prefix, size), // Linux-NTFS ntfsclone Sparse Partition Backup Image (\0ntfsclone-image)
        b"SPNT" => verify_ntsp_system(prefix, size), // Amiga NTSP-system Audio Module (.ntsp)
        b"\x00NuF" // NuFX / ShrinkIt compressed archive
        | b"NuFX" // NuFX / ShrinkIt compressed archive
        | b"NuFi" // NuFX / ShrinkIt compressed archive
        | b"N\xF5F\xD8" // NuFX / ShrinkIt compressed archive
        | b"N\xF5F\xE9" => verify_nufx(prefix, size), // NuFX / ShrinkIt compressed archive
        b"Myth" // NuppelVideo / MythTV RTjpeg+LZO Television Capture (.nuv)
        | b"Nupp" => verify_nuppel_video(prefix, size), // NuppelVideo / MythTV RTjpeg+LZO Television Capture (.nuv)
        b"nut/" => verify_nut_container(prefix, size), // FFmpeg / MPlayer NUT Open Multimedia Container (.nut)
        b"OBIS" => verify_obarski_osp(prefix, size), // Karsten Obarski Synth Pack Bank (.osp)
        b"CDOo" => verify_oberon_odc(prefix, size), // BlackBox Component Builder / Oberon-2 Compound Document (.odc, 1997) Oberon microsystems
        b"OGL_" => verify_objectgraphics_ogl(prefix, size), // ObjectGraphics OGL Drawing (.ogl, 1990s) ("OGL_FILE\0")
        b"Caml" => verify_ocaml_compiler_artifact(prefix, size), // OCaml Compiled Interface & Bytecode Object (.cmi / .cmo / .cma / .cmx / .cmt, 1996) INRIA
        &[0x00, 0x10, 0x5B, 0x4D] => verify_ogre_3d_mesh(prefix, size), // OGRE 3D Engine Binary Mesh (.mesh, [MeshSerializer_v1.)
        b"CORK" => verify_omega_logic_cork(prefix, size), // Omega Logic CORK Compressed Installer File (.dl$ / .ex$)
        b"EBFG" => verify_onedrive_odl(prefix, size), // Microsoft OneDrive Obfuscated Diagnostic Log (.odl)
        &[0xA1, 0x2F, 0xFF, 0x43] => verify_onenote_toc2(prefix, size), // Microsoft OneNote Table of Contents v2 (.onetoc2)
        b"Onyx" => verify_onyx_omf(prefix, size), // Onyx Music File (.omf)
        b"v/1\x01" => verify_openexr(prefix, size), // OpenEXR High Dynamic Range Image (1999) Industrial Light & Magic (0x762F3101 LE)
        b"2\x19\xCCn" // OpenFst Weighted Finite-State Transducer / Archive (.fst / .far)
        | b"2%\xCCn" // OpenFst Weighted Finite-State Transducer / Archive (.fst / .far)
        | b"Jk\xD9~" // OpenFst Weighted Finite-State Transducer / Archive (.fst / .far)
        | b"Jm\xD9~" // OpenFst Weighted Finite-State Transducer / Archive (.fst / .far)
        | b"\\QV\x00" // OpenFst Weighted Finite-State Transducer / Archive (.fst / .far)
        | &[0x5C, 0xF3, 0xB2, 0x7E] // OpenFst Weighted Finite-State Transducer / Archive (.fst / .far)
        | &[0x8F, 0x30, 0x91, 0xB2] // OpenFst Weighted Finite-State Transducer / Archive (.fst / .far)
        | &[0xD6, 0xFD, 0xB2, 0x7E] => verify_openfst_transducer(prefix, size), // OpenFst Weighted Finite-State Transducer / Archive (.fst / .far)
        b"PARK" => verify_openrct2_park(prefix, size), // OpenRCT2 Park Save / Scenario Container (.park, 2021) OpenRCT2
        b"SSHK" => verify_openssh_krl(prefix, size), // OpenSSH Key Revocation List (.krl, SSHKRL\\n\\x00)
        b"KPG\x01" => verify_optiks_pcr_font(prefix, size), // Keith P. Graham Optiks PCR Font (.pcr, 1988) Keith P. Graham ("KPG\x01\x00\x20")
        b"OFR " // OptimFROG Lossless Audio (.ofr / .ofs)
        | b"OFRX" => verify_optimfrog(prefix, size), // OptimFROG Lossless Audio (.ofr / .ofs)
        b"Org-" => verify_organya_audio(prefix, size), // Studio Pixel Cave Story Organya Music (.org)
        b"MFM_" // Oric Atmos / Telestrat MFM Disk Image (.dsk)
        | b"ORIC" => verify_oric_mfm_disk(prefix, size), // Oric Atmos / Telestrat MFM Disk Image (.dsk)
        &[0xA5, 0x96, 0xFD, 0xFF] => verify_os2_ftcomp(prefix, size), // IBM OS/2 2.0+ PACK2 FTCOMP Installation Archive (.ba_ / .ex_ / .dl_, 1992) IBM
        b"HSP\x01" // IBM OS/2 Presentation Manager IPF Hypertext Help / Book (.hlp / .inf)
        | b"HSP\x10" => verify_os2_help_inf(prefix, size), // IBM OS/2 Presentation Manager IPF Hypertext Help / Book (.hlp / .inf)
        &[0xA5, 0x96, 0x00, 0x14] // IBM OS/2 PACK Compressed Installation File (.ex$ / .dl$ / .sy$)
        | &[0xA5, 0x96, 0x14, 0x0A] // IBM OS/2 PACK Compressed Installation File (.ex$ / .dl$ / .sy$)
        | &[0xA5, 0x96, 0xFE, 0xFF] // IBM OS/2 PACK Compressed Installation File (.ex$ / .dl$ / .sy$)
        | &[0xA5, 0x96, 0xFF, 0xFF] => verify_os2_pack(prefix, size), // IBM OS/2 PACK Compressed Installation File (.ex$ / .dl$ / .sy$)
        &[0xFE, 0xFF, 0xFF, 0xFF] => verify_os2_pm_font(prefix, size), // OS/2 Presentation Manager Bitmap Font (.fnt / .fmf, 1988) IBM & Microsoft ("OS/2 FONT" at @8)
        &[0x83, 0xFF, 0xFF, 0xFF] // OSASK / Haribote OS Compressed Application Binary (.hrb, OSASKCMP)
        | &[0x85, 0xFF, 0xFF, 0xFF] // OSASK / Haribote OS Compressed Application Binary (.hrb, OSASKCMP)
        | &[0x89, 0xFF, 0xFF, 0xFF] => verify_osask_hrb_exec(prefix, size), // OSASK / Haribote OS Compressed Application Binary (.hrb, OSASKCMP)
        b"OSO_" => verify_osl_oso_binary(prefix, size), // Compiled Binary OSO Shader (.oso)
        &[0x0D, 0xF0, 0xAD, 0xBA] => verify_outlook_nk2(prefix, size), // Microsoft Outlook AutoComplete Cache (.nk2, 0xBAADF00D LE)
        b"PABL" => verify_pablopaint(prefix, size), // Atari ST PabloPaint Packed Picture (.pa3 / .ppp, 1980s) Groupe CDND ("PABLO PACKED PICTURE")
        b"ICE!" // Atari ST Pack-Ice Compressed File (ICE! / Ice!)
        | b"Ice!" => verify_pack_ice(prefix, size), // Atari ST Pack-Ice Compressed File (ICE! / Ice!)
        &[0x00, 0xFF, 0x27, 0x20] => verify_pads_pcb(prefix, size), // Mentor Graphics PADS Layout Binary PCB Design (.pcb)
        b"4D P" => verify_paint4d(prefix, size), // 4D Paint 3D Texture Painting Project (.4dp, 1990s) 4D Vision ("4D Paint ")
        b"PLX\x00" => verify_palladix_pcs(prefix, size), // Palladix Amiga Audio Module (.pcs)
        b"ISL_" => verify_panasonic_s3d(prefix, size), // Panasonic ISL_PANA_3DMAP 3D Calibration Map (.s3d) Panasonic
        b"UPD\x00" // Panasonic / Leica Camera UPD Firmware Update Container (.bin / .quf / .lfu, 2008) Panasonic / Leica
        | &[0xAA, 0xAF, 0xBB, 0xFF] => verify_panasonic_upd_firmware(prefix, size), // Panasonic / Leica Camera UPD Firmware Update Container (.bin / .quf / .lfu, 2008) Panasonic / Leica
        b"Pang" => verify_pangolin_prr(prefix, size), // Pangolin Laser Systems Request Package (.prr) Pangolin Laser Systems
        b"pQ9\x01" // Matt Mahoney PAQ Context-Mixing Archive (.paq6 / .paq7 / .paq8 / .paq9a)
        | b"paq6" // Matt Mahoney PAQ Context-Mixing Archive (.paq6 / .paq7 / .paq8 / .paq9a)
        | b"paq7" // Matt Mahoney PAQ Context-Mixing Archive (.paq6 / .paq7 / .paq8 / .paq9a)
        | b"paq8" // Matt Mahoney PAQ Context-Mixing Archive (.paq6 / .paq7 / .paq8 / .paq9a)
        | b"paq9" => verify_paq_archive(prefix, size), // Matt Mahoney PAQ Context-Mixing Archive (.paq6 / .paq7 / .paq8 / .paq9a)
        b"@@b@" => verify_paradox_mesh(prefix, size), // Paradox Clausewitz / Jomini Engine Binary 3D Mesh (.mesh / .anim, 2013) Paradox Interactive
        b"With" => verify_parallels_hdd_image(prefix, size), // Parallels Desktop Virtual Hard Disk Image (.hdd, WithoutFreeSpace)
        b"PAR\x00" // Parchive Parity Volume Set (.par / .par2 / .par3)
        | b"PAR2" // Parchive Parity Volume Set (.par / .par2 / .par3)
        | b"PAR3" => verify_parchive(prefix, size), // Parchive Parity Volume Set (.par / .par2 / .par3)
        b"PARS" => verify_parsons_wrk(prefix, size), // Parsons Technology Resource Data (.wrk)
        b"PTRE" => verify_passmark_pt(prefix, size), // PassMark PerformanceTest Baseline Results (PTRESFILE, .pt)
        b"PWS3" => verify_passwordsafe(prefix, size), // Password Safe v3 Encrypted Password Database (.psafe3, 2006) Bruce Schneier & Rony Shapiro
        &[0x00, 0x04, 0x00, 0x01] => verify_paul_robotham_dat(prefix, size), // Paul Robotham Amiga Music Module (.dat)
        b"BOX1" => verify_pazligsaw_pzl(prefix, size), // Pazligsaw Jigsaw Puzzle Package (BOX1, .pzl)
        b"PC-8" => verify_pc88_t88(prefix, size), // NEC PC-8801 T88 Cassette Tape Image (.t88, 1999)
        b"PVI1" // NEC PC-98 FMP ADPCM Sample Bank (.pvi)
        | b"PVI2" => verify_pc98_pvi_sample_bank(prefix, size), // NEC PC-98 FMP ADPCM Sample Bank (.pvi)
        b"S980" // NEC PC-88 / PC-98 S98 FM Sound Chip Register Log (.s98)
        | b"S981" // NEC PC-88 / PC-98 S98 FM Sound Chip Register Log (.s98)
        | b"S982" // NEC PC-88 / PC-98 S98 FM Sound Chip Register Log (.s98)
        | b"S983" => verify_pc98_s98_audio(prefix, size), // NEC PC-88 / PC-98 S98 FM Sound Chip Register Log (.s98)
        b"VFD1" => verify_pc98_vfd_fdd(prefix, size), // NEC PC-98 Virtual98 Floppy Disk Image (.fdd)
        b"HESM" => verify_pc_engine_hes_audio(prefix, size), // PC Engine / TurboGrafx-16 Hudson Entertainment System Sound Format (.hes)
        b"PCBo" => verify_pcboard_ppe(prefix, size), // Clark Development PCBoard PPL Executable (.ppe, 1993)
        b"Desk" => verify_pctools_msg(prefix, size), // Central Point PC Tools Desktop Help / Message File (.msg / .hlp)
        b"PCVI" => verify_pcvic_snapshot(prefix, size), // PCVIC Commodore VIC-20 Emulator System Snapshot (.vic, 1996) ("PCVIC system snapshot\0")
        b"pcxL" => verify_pcx_library(prefix, size), // PCX Programmer's Toolkit Image Library (.pcl, 1988) Genus Microprogramming ("pcxLib\0")
        b"SILF" => verify_pdfannotator_sil(prefix, size), // GRAHL Software Design PDF Annotator Stamp/Item Library (.sil) GRAHL Software Design
        b"0|PD" => verify_pdi2_dictionary(prefix, size), // PDI2 Bilingual Dictionary Database (.pdi)
        b"PDN3" => verify_pdn3(prefix, size), // Paint.NET Layered Raster Image v3.0+ (2007) Rick Brewster / dotPDN
        &[0xC9, 0xC5, 0xE6, 0xD7] => verify_pdse_program_object(prefix, size), // IBM z/OS PDSE Program Object (IEWPLMH)
        b"PEA\x01" // PeaZip PEA (Pack Encrypt Authenticate, 2006) Giorgio Tani (v1.x ASCII header)
        | &[0xEA, 0x01, 0x00, 0x00] // PeaZip PEA archive (.pea)
        | &[0xEA, 0x01, 0x00, 0x01] => verify_pea_archive(prefix, size), // PeaZip PEA archive (.pea)
        b"PDI1" => verify_pepakura_pdi1(prefix, size), // Tama Software Pepakura Designer Legacy v1 3D Papercraft Model (.pdo, 2002) Tama Software
        b"DTiJ" => verify_perf_jitdump(prefix, size), // Linux Kernel perf JIT Dump Binary Log (.dump)
        b"PF\n " // Portable FloatMap HDR Raster Image (.pfm)
        | b"PF\n#" // Portable FloatMap HDR Raster Image (.pfm)
        | b"PF\n1" // Portable FloatMap HDR Raster Image (.pfm)
        | b"PF\n2" // Portable FloatMap HDR Raster Image (.pfm)
        | b"PF\n3" // Portable FloatMap HDR Raster Image (.pfm)
        | b"PF\n4" // Portable FloatMap HDR Raster Image (.pfm)
        | b"PF\n5" // Portable FloatMap HDR Raster Image (.pfm)
        | b"PF\n6" // Portable FloatMap HDR Raster Image (.pfm)
        | b"PF\n7" // Portable FloatMap HDR Raster Image (.pfm)
        | b"PF\n8" // Portable FloatMap HDR Raster Image (.pfm)
        | b"PF\n9" // Portable FloatMap HDR Raster Image (.pfm)
        | b"PF\r\n" // Portable FloatMap HDR Raster Image (.pfm)
        | b"Pf\n#" // Portable FloatMap HDR Raster Image (.pfm)
        | b"Pf\n1" // Portable FloatMap HDR Raster Image (.pfm)
        | b"Pf\n2" // Portable FloatMap HDR Raster Image (.pfm)
        | b"Pf\n3" // Portable FloatMap HDR Raster Image (.pfm)
        | b"Pf\n4" // Portable FloatMap HDR Raster Image (.pfm)
        | b"Pf\n5" // Portable FloatMap HDR Raster Image (.pfm)
        | b"Pf\n6" // Portable FloatMap HDR Raster Image (.pfm)
        | b"Pf\n7" // Portable FloatMap HDR Raster Image (.pfm)
        | b"Pf\n8" // Portable FloatMap HDR Raster Image (.pfm)
        | b"Pf\n9" // Portable FloatMap HDR Raster Image (.pfm)
        | b"Pf\r\n" => verify_pfm(prefix, size), // Portable FloatMap HDR Raster Image (.pfm)
        b"PFPA" => verify_pfpart3(prefix, size), // PFPART3 Partial Download Container (.pfpart)
        b"PGPl" => verify_pgp_locstr(prefix, size), // PGP Desktop Compiled Localization String Table (.str, 2005) PGP
        b"FUEL" => verify_pharo_fuel(prefix, size), // Pharo Smalltalk FUEL Object Serializer Stream (.fuel, 2011) RMoD INRIA
        b"PH\n " // Portable HalfMap 16-Bit Float HDR Image (.phm)
        | b"PH\n#" // Portable HalfMap 16-Bit Float HDR Image (.phm)
        | b"PH\n1" // Portable HalfMap 16-Bit Float HDR Image (.phm)
        | b"PH\n2" // Portable HalfMap 16-Bit Float HDR Image (.phm)
        | b"PH\n3" // Portable HalfMap 16-Bit Float HDR Image (.phm)
        | b"PH\n4" // Portable HalfMap 16-Bit Float HDR Image (.phm)
        | b"PH\n5" // Portable HalfMap 16-Bit Float HDR Image (.phm)
        | b"PH\n6" // Portable HalfMap 16-Bit Float HDR Image (.phm)
        | b"PH\n7" // Portable HalfMap 16-Bit Float HDR Image (.phm)
        | b"PH\n8" // Portable HalfMap 16-Bit Float HDR Image (.phm)
        | b"PH\n9" // Portable HalfMap 16-Bit Float HDR Image (.phm)
        | b"PH\r\n" // Portable HalfMap 16-Bit Float HDR Image (.phm)
        | b"Ph\n " // Portable HalfMap 16-Bit Float HDR Image (.phm)
        | b"Ph\n#" // Portable HalfMap 16-Bit Float HDR Image (.phm)
        | b"Ph\n1" // Portable HalfMap 16-Bit Float HDR Image (.phm)
        | b"Ph\n2" // Portable HalfMap 16-Bit Float HDR Image (.phm)
        | b"Ph\n3" // Portable HalfMap 16-Bit Float HDR Image (.phm)
        | b"Ph\n4" // Portable HalfMap 16-Bit Float HDR Image (.phm)
        | b"Ph\n5" // Portable HalfMap 16-Bit Float HDR Image (.phm)
        | b"Ph\n6" // Portable HalfMap 16-Bit Float HDR Image (.phm)
        | b"Ph\n7" // Portable HalfMap 16-Bit Float HDR Image (.phm)
        | b"Ph\n8" // Portable HalfMap 16-Bit Float HDR Image (.phm)
        | b"Ph\n9" // Portable HalfMap 16-Bit Float HDR Image (.phm)
        | b"Ph\r\n" => verify_phm(prefix, size), // Portable HalfMap 16-Bit Float HDR Image (.phm)
        &[0x00, 0x06, 0x00, 0x01] // Adobe Photoshop Brush Preset (.abr)
        | &[0x00, 0x06, 0x00, 0x02] // Adobe Photoshop Brush Library (.abr, v6.2)
        | &[0x00, 0x07, 0x00, 0x01] // Adobe Photoshop Brush Preset (.abr)
        | &[0x00, 0x07, 0x00, 0x02] // Adobe Photoshop Brush Preset (.abr)
        | &[0x00, 0x08, 0x00, 0x01] // Adobe Photoshop Brush Preset (.abr)
        | &[0x00, 0x08, 0x00, 0x02] // Adobe Photoshop Brush Preset (.abr)
        | &[0x00, 0x09, 0x00, 0x01] // Adobe Photoshop Brush Preset (.abr)
        | &[0x00, 0x09, 0x00, 0x02] // Adobe Photoshop Brush Preset (.abr)
        | &[0x00, 0x0A, 0x00, 0x01] // Adobe Photoshop Brush Preset (.abr)
        | &[0x00, 0x0A, 0x00, 0x02] => verify_photoshop_abr(prefix, size), // Adobe Photoshop Brush Preset (.abr)
        b"NTDB" => verify_piapro_ntdb(prefix, size), // Crypton Future Media Piapro Studio NT Voice Database (.ntdb, 2020) Crypton Future Media
        b"ci%p" => verify_pismo_compact_iso(prefix, size), // Pismo File Mount Compact ISO Archive (.cfs / .ciso)
        b"XISF" => verify_pixinsight_xisf(prefix, size), // Pleiades Astrophoto PixInsight Extensible Image Serialization Format (.xisf)
        b"MADG" // Antoine Rosset Macintosh PlayerPRO Module (.madg / .madk)
        | b"MADI" // Antoine Rosset Macintosh PlayerPRO Module (.madg / .madk)
        | b"MADK" => verify_playerpro_madg(prefix, size), // Antoine Rosset Macintosh PlayerPRO Module (.madg / .madk)
        b"\x7FPKG" => verify_playstation_3_pkg(prefix, size), // Sony PlayStation 3 / PSP Package (.pkg, 2006) Sony
        b"\x7FCNT" => verify_playstation_4_pkg(prefix, size), // Sony PlayStation 4 / PS Vita Content Package (.pkg, 2011) Sony
        b"\x00PBP" => verify_playstation_psp_pbp(prefix, size), // Sony PlayStation Portable Executable Package (.pbp / EBOOT.PBP, 2004) Sony
        b"\x00PSF" => verify_playstation_sfo(prefix, size), // Sony PlayStation System File Object (PARAM.SFO, 2004) Sony
        b"VAGi" // Sony PlayStation SPU VAG Audio (.vag)
        | b"VAGp" => verify_playstation_vag(prefix, size), // Sony PlayStation SPU VAG Audio (.vag)
        b"XVAG" => verify_playstation_xvag(prefix, size), // Sony PlayStation 3 / PS Vita / PS4 XVAG Audio Stream (.xvag)
        b"\xFFtxf" => verify_plib_txf(prefix, size), // PLIB / OpenGL Texture Mapped Font (.txf, 1997) Mark J. Kilgard & Steve Baker ("\xFFtxf")
        b"VIEW" // PM Picture (VIEW, .pm)
        | b"WEIV" => verify_pm_picture(prefix, size), // PM Picture (VIEW, .pm)
        b"SFCD" // Stephan Fuhrmann PowerPlayer Music Cruncher (.pmc)
        | b"SFHD" => verify_pmc_cruncher(prefix, size), // Stephan Fuhrmann PowerPlayer Music Cruncher (.pmc)
        b"\x8BPOD" => verify_pod5(prefix, size), // Oxford Nanopore POD5 Raw Signal Container (.pod5, 2022) Oxford Nanopore Technologies ("\x8BPOD\r\n\x1A\n")
        b"PODW" => verify_podbot_pwf(prefix, size), // Counter-Strike POD-Bot Waypoint File (.pwf, 2001) Count Floyd
        b"CRAG" => verify_pokemon_garc(prefix, size), // Game Freak Nintendo 3DS GARC Archive (.garc, 2013) Game Freak
        b"GFLX" => verify_pokemon_gfpak(prefix, size), // Game Freak Nintendo Switch GFPAK Archive (.gfpak, 2018) Game Freak
        b"PAM!" => verify_popcap_pam(prefix, size), // PopCap Games PAM Sprite Animation (.pam) PopCap Games
        &[0x41, 0x00, 0xA0, 0x00] => verify_popstarter_vcd(prefix, size), // Sony PlayStation POPStarter Virtual CD-ROM Image (.vcd)
        b"HDR*" => verify_powerbuilder_pbd(prefix, size), // Sybase / Appeon PowerBuilder Dynamic Library (.pbd / .pbl, 1991) Powersoft / Sybase
        &[0x41, 0x00, 0x00, 0x00] // Sybase / Appeon PowerBuilder Standalone DataWindow Report (.psr / .dw)
        | &[0x82, 0x00, 0x00, 0x00] => verify_powerbuilder_psr(prefix, size), // Sybase / Appeon PowerBuilder Standalone DataWindow Report (.psr / .dw)
        b"DAA\x00" // PowerISO Direct-Access-Archive Disc Image (.daa)
        | b"DAAC" => verify_poweriso_daa(prefix, size), // PowerISO Direct-Access-Archive Disc Image (.daa)
        b"PP11" // Amiga PowerPacker Compressed File (PP20)
        | b"PP20" // Amiga PowerPacker Compressed File (PP20)
        | b"PPbk" // Amiga PowerPacker Compressed File (PP20)
        | b"PX20" => verify_powerpacker(prefix, size), // Amiga PowerPacker Compressed File (PP20)
        b"\x03RVP" // Imagination Technologies PowerVR Texture (.pvr)
        | &[0x34, 0x00, 0x00, 0x00] // Imagination Technologies PowerVR Texture (.pvr)
        | b"PVR\x03" // Imagination Technologies PowerVR Texture (.pvr)
        | b"PVRT" => verify_powervr_pvr(prefix, size), // Imagination Technologies PowerVR Texture (.pvr)
        b"PPF1" // Icarus / Paradox PlayStation Patch File (.ppf)
        | b"PPF2" // Icarus / Paradox PlayStation Patch File (.ppf)
        | b"PPF3" => verify_ppf(prefix, size), // Icarus / Paradox PlayStation Patch File (.ppf)
        b"***P" => verify_ppm_mpd(prefix, size), // Bauer+Kirch PPM-Optimierung Data (.mpd, Bauer+Kirch Monschau)
        &[0xFF, 0x63, 0xFF, 0x29] => verify_premiere_prmi(prefix, size), // Adobe Premiere Pro Media Intelligence Sidecar Index (.prmi)
        b"pres" => verify_preset_sap(prefix, size), // Sound Particles / Audio Plugin Preset Data Tree (.sap / .afx)
        b"PRT\x18" // Pink / Abyss preTracker Amiga Chiptune Module (.prt)
        | b"PRT\x19" // Pink / Abyss preTracker Amiga Chiptune Module (.prt)
        | b"PRT\x1A" // Pink / Abyss preTracker Amiga Chiptune Module (.prt)
        | b"PRT\x1B" // Pink / Abyss preTracker Amiga Chiptune Module (.prt)
        | b"PRT\x1C" // Pink / Abyss preTracker Amiga Chiptune Module (.prt)
        | b"PRT\x1D" // Pink / Abyss preTracker Amiga Chiptune Module (.prt)
        | b"PRT\x1E" // Pink / Abyss preTracker Amiga Chiptune Module (.prt)
        | b"PRT\x1F" // Pink / Abyss preTracker Amiga Chiptune Module (.prt)
        | b"PRT " // Pink / Abyss preTracker Amiga Chiptune Module (.prt)
        | b"PRT!" // Pink / Abyss preTracker Amiga Chiptune Module (.prt)
        | b"PRT\"" => verify_pretracker_module(prefix, size), // Pink / Abyss preTracker Amiga Chiptune Module (.prt)
        b"POP_" => verify_princed_pop_plv(prefix, size), // Princed Prince of Persia Level (.plv, 2003) Enrique Calot / Princed Development Team
        b"SMD " => verify_priston_tale_smd_model(prefix, size), // Triglow Pictures Priston Tale 3D Model / Motion (.smd / .smb)
        b"PSC " => verify_pro_sound_creator_psc(prefix, size), // ZX Spectrum Pro Sound Creator Module (.psc)
        b"PGF\x01" // libPGF Progressive Graphics File Wavelet Image (.pgf)
        | b"PGF\x02" // libPGF Progressive Graphics File Wavelet Image (.pgf)
        | b"PGF\x04" // libPGF Progressive Graphics File Wavelet Image (.pgf)
        | b"PGF\x05" // libPGF Progressive Graphics File Wavelet Image (.pgf)
        | b"PGF\x06" // libPGF Progressive Graphics File Wavelet Image (.pgf)
        | b"PGF\x07" // libPGF Progressive Graphics File Wavelet Image (.pgf)
        | b"PGF\x16" // libPGF Progressive Graphics File Wavelet Image (.pgf)
        | b"PGF&" // libPGF Progressive Graphics File Wavelet Image (.pgf)
        | b"PGF1" // libPGF Progressive Graphics File Wavelet Image (.pgf)
        | b"PGF2" // libPGF Progressive Graphics File Wavelet Image (.pgf)
        | b"PGF3" // libPGF Progressive Graphics File Wavelet Image (.pgf)
        | b"PGF4" // libPGF Progressive Graphics File Wavelet Image (.pgf)
        | b"PGF5" // libPGF Progressive Graphics File Wavelet Image (.pgf)
        | b"PGF6" // libPGF Progressive Graphics File Wavelet Image (.pgf)
        | b"PGF7" => verify_progressive_graphics_file(prefix, size), // libPGF Progressive Graphics File Wavelet Image (.pgf)
        &[0xC8, 0xA6, 0xD8, 0x23] => verify_project64_pj(prefix, size), // Project64 Nintendo 64 Emulator Save State (.pj)
        b"ASEF" => verify_pronom_adobe_swatch_exchange_ase(prefix, size), // Adobe Swatch Exchange (PRONOM fmt/1816)
        &[0x3C, 0x42, 0x3F, 0xC9] => verify_pronom_adrift_taf(prefix, size), // ADRIFT Text Adventure File (.taf, PRONOM fmt/1584)
        b"AFF1" => verify_pronom_advanced_forensic_format_aff(prefix, size), // Advanced Forensic Format (PRONOM fmt/844)
        b"ASYL" => verify_pronom_asylum_music_format_amf(prefix, size), // Asylum Music Format (PRONOM fmt/1622)
        b"MPP\x00" // Atari ST Multi Palette Picture File (.mpp, PRONOM fmt/1471)
        | b"MPP\x01" // Atari ST Multi Palette Picture File (.mpp, PRONOM fmt/1471)
        | b"MPP\x02" // Atari ST Multi Palette Picture File (.mpp, PRONOM fmt/1471)
        | b"MPP\x03" => verify_pronom_atari_mpp(prefix, size), // Atari ST Multi Palette Picture File (.mpp, PRONOM fmt/1471)
        b"MENU" => verify_pronom_autorun_maestro_menu_file_mnu(prefix, size), // Autorun Maestro Menu File (PRONOM fmt/1461)
        b"TIDX" => verify_pronom_avchd_thumbnail_index_file_tid(prefix, size), // AVCHD Thumbnail Index File (PRONOM fmt/1077)
        &[0x00, 0x1F, 0x01, 0x34] => verify_pronom_bim_metadata_file(prefix, size), // BIM Metadata File (PRONOM fmt/1722)
        b"\xFFBIO" => verify_pronom_biosemi_24_bit_bdf(prefix, size), // BioSemi 24-Bit BDF Electrophysiology (PRONOM fmt/2102)
        b"BB3D" => verify_pronom_blitz3d_file_format_b3d(prefix, size), // Blitz3D File Format (PRONOM fmt/1182)
        b"V5_C" => verify_pronom_catia_drawing(prefix, size), // CATIA Drawing (PRONOM fmt/1615)
        &[0x01, 0x32, 0x00, 0x00] => verify_pronom_chemstation_gas_chromatography_mass_ms(prefix, size), // ChemStation Gas Chromatography/Mass Spectrum (GC/MS) Data File (PRONOM fmt/2101)
        &[0x42, 0xBD, 0x0A, 0x00] => verify_pronom_common_instrument_ci1(prefix, size), // Common Instrument File Format (.ci1 / .ci2, PRONOM fmt/1887)
        &[0x40, 0xBD, 0x0A, 0x00] => verify_pronom_common_loudspeaker_cf1(prefix, size), // Common Loudspeaker Format Binary (.cf1 / .cf2, PRONOM fmt/1944)
        b"RAW1" // Bruker AXS DIFFRAC.suite / DIFFRACplus X-Ray Diffraction Raw Data (.raw, PRONOM fmt/1120)
        | b"RAW2" // Bruker AXS DIFFRAC.suite / DIFFRACplus X-Ray Diffraction Raw Data (.raw, PRONOM fmt/1120)
        | b"RAW4" => verify_pronom_diffracplus_raw_data(prefix, size), // Bruker AXS DIFFRAC.suite / DIFFRACplus X-Ray Diffraction Raw Data (.raw, PRONOM fmt/1120)
        b".scf" => verify_pronom_dna_sequence_chromatogram_file_scf(prefix, size), // DNA Sequence Chromatogram File (PRONOM fmt/1554)
        b"FINC" // FinePrint Saved Output (.fp, PRONOM fmt/1396)
        | b"FINE" => verify_pronom_fineprint_fp(prefix, size), // FinePrint Saved Output (.fp, PRONOM fmt/1396)
        b"G9B\x00" // G9B MSX2+ V9990 Graphics Format Bitmap (.g9b, PRONOM fmt/1787)
        | b"G9B\x0B" => verify_pronom_g9b_bitmap(prefix, size), // G9B MSX2+ V9990 Graphics Format Bitmap (.g9b, PRONOM fmt/1787)
        b"<@gA" => verify_pronom_garmin_gmn(prefix, size), // Garmin Track Log File (.gmn, PRONOM fmt/1679)
        &[0xFF, 0x00, 0x00, 0x04] // GSSI SIR-10 RADAN Ground Penetrating Radar Data (.dzt, PRONOM fmt/362)
        | &[0xFF, 0x07, 0x00, 0x04] => verify_pronom_gssi_dzt(prefix, size), // GSSI SIR-10 RADAN Ground Penetrating Radar Data (.dzt, PRONOM fmt/362)
        b"GX2\x01" => verify_pronom_gx2_bitmap(prefix, size), // Show Partner GX2 Graphics File (.gx2, PRONOM fmt/1789)
        b"ncif" => verify_pronom_haiku_vector_icon_format_hvif(prefix, size), // Haiku Vector Icon Format (PRONOM fmt/1791)
        b"7224" => verify_pronom_infofile_flr(prefix, size), // InfoFile FLR Database (.flr, PRONOM fmt/1785)
        b"AIXF" => verify_pronom_interleaved_adx_audio_aix(prefix, size), // Interleaved ADX Audio Format (AIX) (PRONOM fmt/841)
        b"LEAD" // LEADTOOLS Compressed Image (.cmp, PRONOM fmt/1337)
        | b"Lead" => verify_pronom_leadtools_cmp(prefix, size), // LEADTOOLS Compressed Image (.cmp, PRONOM fmt/1337)
        b"Leo " => verify_pronom_leonardo_image_format_leo(prefix, size), // Leonardo Image Format (PRONOM fmt/1217)
        b"maps" => verify_pronom_mapsforge_binary_vector_map(prefix, size), // Mapsforge Binary Vector Map (PRONOM fmt/915)
        b"MAR\x80" => verify_pronom_mar_archive_mac(prefix, size), // Mar Archive (PRONOM fmt/1483)
        &[0x01, 0xFF, 0x00, 0xFF] => verify_pronom_plextalk_project_file_imdn(prefix, size), // Shinano Kenshi PLEXTALK Recording Project (.imdn, PRONOM fmt/2036)
        b"!<<P" // QuickVerse / Parsons Technology Palm OS Portable Database (.pdb, PRONOM fmt/1064)
        | b"!<><" => verify_pronom_portable_database_pdb(prefix, size), // QuickVerse / Parsons Technology Palm OS Portable Database (.pdb, PRONOM fmt/1064)
        b"PFF\x00" => verify_pronom_portable_form_file_pff(prefix, size), // Portable Form File (PRONOM fmt/322)
        &[0xE8, 0x03, 0x00, 0x00] => verify_pronom_powervr_object_data_pod(prefix, size), // PowerVR Object Data (PRONOM fmt/782)
        b"TBWP" => verify_pronom_q_a_word_processor(prefix, size), // Q&A Word Processor Document (PRONOM fmt/1045)
        b"QW V" => verify_pronom_quicken_3_database_file_qst(prefix, size), // Quicken 3 Database File (PRONOM fmt/1861)
        b"#ROS" => verify_pronom_ros_bag_format(prefix, size), // ROS Bag Format (PRONOM fmt/2093)
        &[0xCE, 0x19, 0xCE, 0x19] => verify_pronom_serif_pageplus_publication_ppp(prefix, size), // Serif PagePlus Publication (PRONOM fmt/1537)
        b"EA3\x00" // Sony OpenMG ATRAC3 Audio (.oma / .aa3, PRONOM fmt/2018)
        | b"EA3\x01" // Sony OpenMG ATRAC3 Audio (.oma / .aa3, PRONOM fmt/2018)
        | b"EA3\x02" // Sony OpenMG ATRAC3 Audio (.oma / .aa3, PRONOM fmt/2018)
        | b"EA3\x03" // Sony OpenMG ATRAC3 Audio (.oma / .aa3, PRONOM fmt/2018)
        | b"ea3\x02" // Sony OpenMG ATRAC3 Audio (.oma / .aa3, PRONOM fmt/2018)
        | b"ea3\x03" // Sony OpenMG ATRAC3 Audio (.oma / .aa3, PRONOM fmt/2018)
        | b"ea3\x04" => verify_pronom_sony_oma(prefix, size), // Sony OpenMG ATRAC3 Audio (.oma / .aa3, PRONOM fmt/2018)
        b"SPD\x01" // SpritePad C64 Sprite Image (.spd, PRONOM fmt/1561)
        | b"SPD\x02" // SpritePad C64 Sprite Image (.spd, PRONOM fmt/1561)
        | b"SPD\x03" // SpritePad C64 Sprite Image (.spd, PRONOM fmt/1561)
        | b"SPD\x04" // SpritePad C64 Sprite Image (.spd, PRONOM fmt/1561)
        | b"SPD\x05" => verify_pronom_spritepad_spd(prefix, size), // SpritePad C64 Sprite Image (.spd, PRONOM fmt/1561)
        b"SVR\x08" => verify_pronom_superscape_virtual_reality_format_svr(prefix, size), // Superscape Virtual Reality Format (PRONOM fmt/734)
        b"tru?" => verify_pronom_true_colour_picture_spooky_trp(prefix, size), // True Colour Picture [Spooky Sprites] (Atari Falcon) (PRONOM fmt/1607)
        b"\x18TUN" => verify_pronom_tundra_tnd(prefix, size), // TUNDRA (PRONOM fmt/1603)
        &[0x08, 0xF2, 0xA6, 0xB6] => verify_pronom_vips_image_v(prefix, size), // Vips Image (PRONOM fmt/1811)
        &[0x9C, 0xCB, 0xCB, 0x8D] => verify_pronom_windows_address_book_wab(prefix, size), // Windows Address Book (PRONOM fmt/1255)
        b"XARA" => verify_pronom_xar_image_format(prefix, size), // Xar Image Format (PRONOM fmt/922)
        b"XLPM" => verify_pronom_xl_paint_max(prefix, size), // XL-Paint MaX (PRONOM fmt/1658)
        b"XLPB" => verify_pronom_xl_paint_raw(prefix, size), // XL-Paint (PRONOM fmt/1659)
        b"P7 3" => verify_pronom_xv_thumbnail_p7(prefix, size), // John Bradley XV Visual Schnauzer Thumbnail (.p7, PRONOM fmt/1497)
        b"zbex" => verify_pronom_zoombrowser_ex_thumbnail_cache_info(prefix, size), // ZoomBrowser Ex Thumbnail Cache (PRONOM fmt/1496)
        b"ProS" => verify_proshape(prefix, size), // ProShape CAD Drawing (.psp, 1990s) ("ProShape ... Drawing\r\n\x1A")
        b"ISIS" => verify_proteus_isis_dsn(prefix, size), // Labcenter Proteus ISIS Circuit Schematic (.dsn, 1990) Labcenter Electronics
        b"Prot" => verify_proteus_pdspart(prefix, size), // Labcenter Proteus CAD Part Descriptor (.pdspart) Labcenter Electronics
        b"\x03001" // Digidesign / Avid Pro Tools Session (.ptf / .pts)
        | b"Ptf " // Digidesign / Avid Pro Tools Session (.ptf / .pts)
        | b"ptf " => verify_protools(prefix, size), // Digidesign / Avid Pro Tools Session (.ptf / .pts)
        b"PROT" // Franck Charlet ProTrekkr / NoiseTrekkr Module (.ptk, PROTREKK / TWNN)
        | b"TWNN" => verify_protrekkr_module(prefix, size), // Franck Charlet ProTrekkr / NoiseTrekkr Module (.ptk, PROTREKK / TWNN)
        b"PROX" => verify_prox_usb_xha(prefix, size), // Pro-face PROX_USB_XS HMI Project Archive (.xha) Pro-face / Schneider Electric
        &[0x0D, 0xA4, 0x16, 0xDF] // Proxmox Backup Server Indexed Archive / Blob (.pxar / .fidx / .didx / .blob)
        | &[0x1F, 0xC3, 0x87, 0x21] // Proxmox Backup Server Indexed Archive / Blob (.pxar / .fidx / .didx / .blob)
        | b"B\xAB8\x07" // Proxmox Backup Server Indexed Archive / Blob (.pxar / .fidx / .didx / .blob)
        | b"by\x95'" // Proxmox Backup Server Indexed Archive / Blob (.pxar / .fidx / .didx / .blob)
        | &[0xD5, 0x95, 0xE9, 0x65] // Proxmox Backup Server Indexed Archive / Blob (.pxar / .fidx / .didx / .blob)
        | &[0xEF, 0xAC, 0x88, 0xE5] => verify_proxmox_backup_archive(prefix, size), // Proxmox Backup Server Indexed Archive / Blob (.pxar / .fidx / .didx / .blob)
        b"PRVW" => verify_prvw_prw(prefix, size), // PRVW Preview Image Container (.prw)
        b"MC\x00\x00" => verify_ps1_memory_card(prefix, size), // Sony PlayStation 1 Memory Card Image (.mcr / .mcd, 1994) Sony ("MC\0\0")
        b"IECS" => verify_ps2_hd(prefix, size), // Sony PlayStation 2 SCEI SoundBank Header (.hd)
        b"ipum" => verify_ps2_ipu_video(prefix, size), // Sony PlayStation 2 IPU MPEG-2 Intra-Frame Video Stream (.ipu, ipum)
        b"Sony" => verify_ps2_memory_card(prefix, size), // Sony PlayStation 2 Memory Card Image (.ps2 / .vmc, 2000) Sony ("Sony PS2 Memory Card Format ")
        b"PSA\x00" => verify_psa_module(prefix, size), // Professional Sound Artists Amiga Module (.psa)
        &[0x50, 0x80, 0xFE, 0x00] => verify_psg_sleep(prefix, size), // Polysomnography (PSG) Sleep Study Recording (.psg)
        b"PSIO" => verify_psion3_word(prefix, size), // Psion Series 3 Word Document (.wrd, 1991) Psion PLC ("PSIONWPDATAFILE\0")
        b"OPLO" => verify_psion_opl(prefix, size), // Psion Organiser II / Series 3 OPL Translated Bytecode (.opo / .opa)
        b"Imag" => verify_psion_sibo_app(prefix, size), // Psion Series 3 SIBO IMG / APP / DYL Executable (.img / .app / .dyl)
        b"DAX\x00" => verify_psp_dax(prefix, size), // Dark_AleX PlayStation Portable DAX Deflate Compressed ISO (2006) Dark_AleX
        b"PS-X" // Sony PlayStation 1 PS-X EXE Executable
        | b"SCE " => verify_psx_exe(prefix, size), // Sony PlayStation 1 PS-X EXE Executable
        b"SPU\x00" // Sony PlayStation SPU Register Dump (.spu)
        | b"SPU1" => verify_psx_spu_dump(prefix, size), // Sony PlayStation SPU Register Dump (.spu)
        b"PSY2" // Psycle Modular Music Creation Studio Song (.psy)
        | b"PSY3" => verify_psycle_module(prefix, size), // Psycle Modular Music Creation Studio Song (.psy)
        b"Ptex" => verify_ptex(prefix, size), // Ptex Per-Face Texture Mapping (.ptx, 2010) Walt Disney Animation Studios
        b"PUF1" => verify_puffin_patch(prefix, size), // Google ChromeOS / Android Puffin Deterministic Deflate Diff Patch (2017) Google
        b"PXHG" => verify_pxhg_package(prefix, size), // PXHG Multilingual String Package (.hg)
        b"PTCO" // Studio Pixel PxTone / Piston Collage Noise or Voice (.ptnoise / .ptvoice)
        | b"PTNO" // Studio Pixel PxTone / Piston Collage Noise or Voice (.ptnoise / .ptvoice)
        | b"PTTU" // Studio Pixel PxTone / Piston Collage Noise or Voice (.ptnoise / .ptvoice)
        | b"PTVO" => verify_pxtone(prefix, size), // Studio Pixel PxTone / Piston Collage Noise or Voice (.ptnoise / .ptvoice)
        b"QCA-" // Qualcomm Atheros WLAN / WiGig Firmware & Board Data (QCA-ATH10K/11K/12K/6KL)
        | b"_wmi" => verify_qca_ath_firmware(prefix, size), // Qualcomm Atheros WLAN / WiGig Firmware & Board Data (QCA-ATH10K/11K/12K/6KL)
        b"QFI\xFB" => verify_qcow2(prefix, size), // QEMU Copy-On-Write Virtual Disk Image (.qcow2)
        b"QED\x00" => verify_qemu_qed_image(prefix, size), // QEMU Enhanced Disk Image (.qed, QED\0)
        b"iwly" => verify_qnx_deflate(prefix, size), // BlackBerry QNX deflate Block-Compressed Filesystem Stream (.deflated)
        &[0x00, 0xFF, 0x7E, 0xEB] // BlackBerry QNX Neutrino Image Filesystem Boot Image (.ifs)
        | &[0xEB, 0x7E, 0xFF, 0x00] => verify_qnx_ifs(prefix, size), // BlackBerry QNX Neutrino Image Filesystem Boot Image (.ifs)
        b"IDPO" => verify_quake1_mdl(prefix, size), // id Software Quake 1 3D Model (.mdl, 1996) id Software
        b"IDP2" => verify_quake2_md2(prefix, size), // id Software Quake II 3D Model (.md2, 1997) id Software
        b"D\x14r\x12" // id Tech 3 Quake III Arena Virtual Machine Bytecode (.qvm, 1999) id Software
        | b"E\x14r\x12" => verify_quake3_qvm(prefix, size), // id Tech 3 Quake III Arena Virtual Machine Bytecode (.qvm, 1999) id Software
        b"QCMS" => verify_qualcomm_acdb(prefix, size), // Qualcomm Audio Calibration Database (.acdb)
        b"FBPK" // Qualcomm Fastboot Firmware Pack (FBPK)
        | b"FBPT" => verify_qualcomm_fbpk(prefix, size), // Qualcomm Fastboot Firmware Pack (FBPK)
        &[0x3C, 0xD6, 0x1A, 0xCE] => verify_qualcomm_meta_image(prefix, size), // Qualcomm Android Bootloader Meta Image (meta_image)
        &[0x9A, 0x1B, 0x7D, 0xAA] => verify_qualcomm_mibib(prefix, size), // Qualcomm MSM MIBIB Flash Partition Table (.bin)
        b"QCDT" => verify_qualcomm_qcdt(prefix, size), // Qualcomm Bootloader Device Tree Table Image (QCDT)
        &[0xD1, 0xDC, 0x4B, 0x84] => verify_qualcomm_sbl1(prefix, size), // Qualcomm Secondary Bootloader SBL1 / NorCodeWord Image (.mbn)
        b"SPLA" => verify_qualcomm_splash(prefix, size), // Qualcomm Snapdragon Boot Splash Screen Image (.img)
        b"WWBB" => verify_quattro_pro_wb1(prefix, size), // Borland Quattro Pro for Windows Encrypted/Packed Notebook (.wb1) Borland
        b"QCGF" => verify_quest_cas(prefix, size), // Quest 3/4 Compiled Game (.cas)
        b"QLII" => verify_quicklink_qfx(prefix, size), // Smith Micro QuickLink II Fax Image (.qfx)
        b"DOF1" => verify_racer_dof1(prefix, size), // Racer Simulation Dynamic Object Format 3D Model (.dof, 2001) Ruud van Gaal
        b"BIKb" // RAD Game Tools Bink Video Container (.bik / .bk2)
        | b"BIKd" // RAD Game Tools Bink Video Container (.bik / .bk2)
        | b"BIKf" // RAD Game Tools Bink Video Container (.bik / .bk2)
        | b"BIKg" // RAD Game Tools Bink Video Container (.bik / .bk2)
        | b"BIKh" // RAD Game Tools Bink Video Container (.bik / .bk2)
        | b"BIKi" // RAD Game Tools Bink Video Container (.bik / .bk2)
        | b"BIKk" // RAD Game Tools Bink Video Container (.bik / .bk2)
        | b"KB2a" // RAD Game Tools Bink Video Container (.bik / .bk2)
        | b"KB2d" // RAD Game Tools Bink Video Container (.bik / .bk2)
        | b"KB2f" // RAD Game Tools Bink Video Container (.bik / .bk2)
        | b"KB2g" // RAD Game Tools Bink Video Container (.bik / .bk2)
        | b"KB2h" // RAD Game Tools Bink Video Container (.bik / .bk2)
        | b"KB2i" // RAD Game Tools Bink Video Container (.bik / .bk2)
        | b"KB2j" // RAD Game Tools Bink Video Container (.bik / .bk2)
        | b"KB2k" // RAD Game Tools Bink Video Container (.bik / .bk2)
        | b"KB2m" // RAD Game Tools Bink Video Container (.bik / .bk2)
        | b"KB2n" => verify_rad_bink(prefix, size), // RAD Game Tools Bink Video Container (.bik / .bk2)
        b"1FCB" => verify_rad_bink_audio(prefix, size), // RAD Game Tools Bink Audio Stream (.binka)
        b"SMK2" // RAD Game Tools Smacker Video Container (.smk)
        | b"SMK4" => verify_rad_smacker(prefix, size), // RAD Game Tools Smacker Video Container (.smk)
        b"radd" => verify_raddbg_rdi(prefix, size), // RAD Debugger Debug Information (.rdi, 2024) Ryan Fleury / Epic Games & RAD Game Tools ("raddbg\0\0")
        b"#?RA" // Radiance RGBE HDR image (.hdr/.pic)
        | b"#?RG" => verify_radiance_hdr(prefix, size), // Radiance RGBE HDR image (.hdr/.pic)
        b"ATG " // Radical Entertainment Cement Library Archive (.rcf)
        | b"RADC" => verify_radical_rcf(prefix, size), // Radical Entertainment Cement Library Archive (.rcf)
        b"RSD2" // Radical Entertainment RSD Audio Stream (.rsd)
        | b"RSD3" // Radical Entertainment RSD Audio Stream (.rsd)
        | b"RSD4" // Radical Entertainment RSD Audio Stream (.rsd)
        | b"RSD6" => verify_radical_rsd(prefix, size), // Radical Entertainment RSD Audio Stream (.rsd)
        b"#Arq" => verify_radiocef_rcf(prefix, size), // Radiocef Studio Dental X-Ray Cephalometry (.rcf)
        b"ADAT" => verify_rage_awc(prefix, size), // Rockstar Games RAGE Audio Wave Container (.awc)
        b"RSC7" => verify_rage_rsc7(prefix, size), // Rockstar Games RAGE Engine Resource Container v7 (.ydr / .ytd / .yft / .ydd / .ymap / .ytyp / .ybn / .ycd)
        b"C#+D" => verify_ragtime_doc(prefix, size), // RagTime Business Publishing Document (.rag / .rtd)
        b")2==" => verify_railworks_dav(prefix, size), // RailWorks / Train Simulator XOR-Obfuscated WAV Audio (.dav)
        b"SERZ" => verify_railworks_serz(prefix, size), // Kuju Entertainment RailWorks / Train Simulator SERZ Binary XML (.tgpcdx / .geopcdx / .bin)
        b"TRK0" => verify_ramtracker_trk(prefix, size), // RamTracker Module (.trk)
        b"GLIT" => verify_ran_online_ims(prefix, size), // Min Communications Ran Online GLITEM_MIX Item Crafting Table (.ims)
        &[0x02, 0x08, 0x03, 0x00] // Rapiscan Dual-Energy X-Ray Scan Archive (.rcf, 0xFFFF1111)
        | &[0x11, 0x11, 0xFF, 0xFF] => verify_rapiscan_rcf(prefix, size), // Rapiscan Dual-Energy X-Ray Scan Archive (.rcf, 0xFFFF1111)
        b"XBNF" => verify_raven_xbnf_bnx(prefix, size), // Raven Software Engine XBNF Binary Font / Text Resource (.bnx)
        b"RCBo" => verify_rcbot_rcw(prefix, size), // Half-Life RCBot Waypoint File (.rcw, 2004) Cheeseh
        &[0x35, 0x00, 0x00, 0x00] => verify_re_engine_chain(prefix, size), // Capcom RE Engine Physics Chain Descriptor (.chain.*)
        b"MDF\x00" => verify_re_engine_mdf2(prefix, size), // Capcom RE Engine Material Definition v2 (.mdf2)
        b"RTMM" => verify_real_tracker_rtm(prefix, size), // RealTech Real Tracker Module (.rtm)
        b"RbBF" => verify_realbasic_rbbf_rbo(prefix, size), // REALbasic / Xojo Binary Project or Object (RbBF, .rbp / .rbo)
        b".RMF" // RealNetworks RealMedia / RealAudio Stream (.rm / .rmvb / .ra)
        | b".ra\xFD" => verify_realmedia(prefix, size), // RealNetworks RealMedia / RealAudio Stream (.rm / .rmvb / .ra)
        b"Real" // Realtek Bluetooth Controller Firmware Patch & Config (Realtech)
        | &[0x55, 0xAB, 0x23, 0x87] => verify_realtek_bt_firmware(prefix, size), // Realtek Bluetooth Controller Firmware Patch & Config (Realtech)
        b"CSYS" => verify_realtek_csys(prefix, size), // Realtek AP / Router CSYS Firmware Image (.bin)
        b"RTK0" => verify_realtek_rtk0(prefix, size), // Realtek RTK0 Bluetooth / Wi-Fi SoC Firmware Patch (.bin)
        b"Asur" => verify_rebellion_asura(prefix, size), // Rebellion Developments Asura Engine Resource Archive (.asr / .pc, 2005) Rebellion Developments
        b"RCM-" => verify_recomposer_rcp_music(prefix, size), // Come On Music Recomposer PC-98 / X68000 MIDI Sequencer (.rcp / .r36)
        b"REDT" => verify_red_thumb(prefix, size), // RED Digital Cinema Camera Thumbnail Preview Image (2007) RED Digital Cinema
        b"redb" => verify_redb(prefix, size), // Christopher Berner redb Embedded Rust Key-Value Database (.redb / .vectre)
        b"CR2W" => verify_redengine_cr2w(prefix, size), // CD Projekt RED REDengine CR2W Resource (.w2ent / .w2mesh / .w2scene / .xbm, 2011) CD Projekt RED
        b"RDAR" => verify_redengine_rdar(prefix, size), // CD Projekt RED REDengine 4 Archive (.archive, 2020) CD Projekt RED
        &[0x10, 0xFB, 0x00, 0x71] => verify_refpack_big(prefix, size), // Electronic Arts RefPack-Compressed BIGF Archive (.big, Electronic Arts)
        b"REKO" => verify_reko_cardset(prefix, size), // Amiga / PC REKO Klondike Cardset (.reko)
        b"_ARC" => verify_relic_archive(prefix, size), // Relic Entertainment SGA / Big Archive (.sga / .big, 2003) Relic Entertainment
        b"PIFF" => verify_relic_rbh(prefix, size), // Relic Entertainment PIFF RBHF RigidBody Header (.rbh, 2004) Relic Entertainment
        b"RNS0" => verify_renoise_old_rns(prefix, size), // Renoise Legacy Binary Song (.rns)
        b"RPA-" => verify_renpy_rpa(prefix, size), // Ren'Py Visual Novel Archive (2004) Tom "PyTom" Rothamel ("RPA-3.0 " / "RPA-2.0 ")
        b"RENP" // Ren'Py Compiled Script AST Bytecode (.rpyc)
        | b"RPYC" => verify_renpy_rpyc(prefix, size), // Ren'Py Compiled Script AST Bytecode (.rpyc)
        b"RESC" => verify_rescseg_cmf(prefix, size), // RESCSEG Segmented Resource Package (.cmf)
        b"ResF" => verify_resfile_rsc(prefix, size), // ResFile Zlib-Compressed Resource Container (.rsc)
        b"RFZP" => verify_rfzpack_rfz(prefix, size), // RFZPACK Resource Archive (.rfz)
        b"RGSS" => verify_rgssad(prefix, size), // Enterbrain RPG Maker XP / VX / VX Ace RGSSEncryptedArchive (2004) Enterbrain ("RGSSAD\0")
        &[0x00, 0x17, 0x5F, 0x5F] // Robert Hoehne RHIDE DJGPP IDE Project / Desktop File (.gpr / .gdt, 1996) Robert Hoehne
        | &[0x17, 0x5F, 0x5F, 0x52] => verify_rhide_gdt(prefix, size), // Robert Hoehne RHIDE DJGPP IDE Project / Desktop File (.gpr / .gdt, 1996) Robert Hoehne
        b"RJP0" // Richard Joseph / Vectordean Amiga Audio (.rjp / .sng / .ins)
        | b"RJP1" // Richard Joseph / Vectordean Amiga Audio (.rjp / .sng / .ins)
        | b"RJP2" => verify_richard_joseph_audio(prefix, size), // Richard Joseph / Vectordean Amiga Audio (.rjp / .sng / .ins)
        &[0x83, 0xAF, 0x70, 0xD1] => verify_riegeli(prefix, size), // Google Riegeli Record Stream File (2017) Google (0xD170AF83 LE signature chunk header)
        &[0xFF, 0xFE, 0xFF, 0x00] // Rillsoft Project Management Schedule (.rpj / .mbk) Rillsoft
        | &[0xFF, 0xFE, 0xFF, 0x0C] => verify_rillsoft_rpj(prefix, size), // Rillsoft Project Management Schedule (.rpj / .mbk) Rillsoft
        b"FONT" => verify_riscos_outlines(prefix, size), // Acorn RISC OS Outline Font (Outlines, 1987) Acorn ("FONT\0")
        b"RIVE" => verify_rive_riv(prefix, size), // Rive Interactive Vector Animation (.riv)
        b"RKA7" => verify_rk_audio(prefix, size), // Malcolm Taylor RK Audio Lossless Compressor (.rka, RKA7)
        b"RWLa" => verify_rli(prefix, size), // RealWorld Paint Layered Image (.rli, 2000s) RealWorld Graphics ("RWLayeredImage_1")
        b"RNC\x00" // Rob Northen Computing ProPack Compressed File (RNC)
        | b"RNC\x01" // Rob Northen Computing ProPack Compressed File (RNC)
        | b"RNC\x02" // Rob Northen Computing ProPack Compressed File (RNC)
        | b"RNCA" => verify_rnc(prefix, size), // Rob Northen Computing ProPack Compressed File (RNC)
        &[0x00, 0x40, 0x7F, 0x40] => verify_rob_hubbard_rho(prefix, size), // Rob Hubbard Atari ST Music Module (.rho)
        b"ROBO" => verify_roboplay_ply(prefix, size), // ROBOPLAY AdLib OPL2 Chiptune Module (.ply)
        b"RKAF" => verify_rockchip_rkaf(prefix, size), // Rockchip RKAF Update Archive Image (.img)
        b"RKFW" => verify_rockchip_rkfw(prefix, size), // Rockchip RKFW Bootable Firmware Package (.img)
        b"RSCE" => verify_rockchip_rsce(prefix, size), // Rockchip RSCE Android Resource Image (.img)
        b"7FPR" // Rockstar Games RAGE Package File Archive (.rpf, 2008) Rockstar Games
        | b"8FPR" // Rockstar Games RAGE Package File Archive (.rpf, 2008) Rockstar Games
        | b"RPF0" // Rockstar Games RAGE Package File Archive (.rpf, 2008) Rockstar Games
        | b"RPF2" // Rockstar Games RAGE Package File Archive (.rpf, 2008) Rockstar Games
        | b"RPF3" // Rockstar Games RAGE Package File Archive (.rpf, 2008) Rockstar Games
        | b"RPF4" // Rockstar Games RAGE Package File Archive (.rpf, 2008) Rockstar Games
        | b"RPF6" // Rockstar Games RAGE Package File Archive (.rpf, 2008) Rockstar Games
        | b"RPF7" // Rockstar Games RAGE Package File Archive (.rpf, 2008) Rockstar Games
        | b"RPF8" => verify_rockstar_rpf(prefix, size), // Rockstar Games RAGE Package File Archive (.rpf, 2008) Rockstar Games
        b"-rom" => verify_romfs(prefix, size), // Linux ROMFS Read-Only Filesystem (.romfs, 1997) Janos Farkas ("-rom1fs-")
        b"WLK5" // TP-Link / Realtek WLK Encrypted Router Configuration (.bin) TP-Link / Realtek
        | b"WLK6" // TP-Link / Realtek WLK Encrypted Router Configuration (.bin) TP-Link / Realtek
        | b"WLK7" // TP-Link / Realtek WLK Encrypted Router Configuration (.bin) TP-Link / Realtek
        | b"WLK8" => verify_router_wlk_config(prefix, size), // TP-Link / Realtek WLK Encrypted Router Configuration (.bin) TP-Link / Realtek
        b"DX\x08\x00" => verify_rpg_wolf(prefix, size), // WOLF RPG Editor DXLib Encrypted Archive (.wolf, 2008) SmokingWOLF
        b"\nLcf" // ASCII / Enterbrain RPG Maker 2000 / 2003 LCF Database / Map / Save (.ldb / .lmt / .lmu / .lsd)
        | b"\x0BLcf" => verify_rpgmaker_lcf(prefix, size), // ASCII / Enterbrain RPG Maker 2000 / 2003 LCF Database / Map / Save (.ldb / .lmt / .lmu / .lsd)
        b"XYZ1" => verify_rpgmaker_xyz1(prefix, size), // ASCII / Enterbrain RPG Maker 2000 & 2003 XYZ Zlib-Paletted Raster Image (2000) ASCII
        b"RPGM" => verify_rpgmv(prefix, size), // Kadokawa RPG Maker MV / MZ Encrypted Asset (2015) Kadokawa ("RPGMV\0\0\0")
        b"RTST" => verify_rtstrike_font(prefix, size), // RTSTRK01 Zlib-Compressed Font Strike (.rtstrike)
        &[0x04, 0x08, 0x55, 0x3A] => verify_ruby_rdoc_ri(prefix, size), // Ruby RDoc RI Marshal Documentation Entry (.ri, 2003)
        b"YARB" => verify_ruby_yarb(prefix, size), // CRuby YARV Compiled Instruction Sequence Bytecode (2016) Koichi Sasada / Ruby Core Team (Ruby 2.3+ RubyVM::InstructionSequence)
        b"NINJ" => verify_rup_rom_patch(prefix, size), // errick NINJA 2.0 Raw Ultima ROM Patch (.rup, NINJA2)
        b"SAB " => verify_sabdu(prefix, size), // SAB Diskette Utility Floppy Image (.sdu, 1990s) ("SAB Diskette Utility")
        b"cook" => verify_safari_binarycookies(prefix, size), // Apple Safari Binary Cookies (.binarycookies)
        b"ASCR" => verify_sakura_wars_sbx(prefix, size), // SEGA / Overworks Sakura Wars 3 Compiled Script (ASCR, .sbx)
        b"v\x984\x12" => verify_samsung_odin_pit(prefix, size), // Samsung Odin Partition Information Table (.pit)
        b"PWFI" => verify_sankom_ozd(prefix, size), // Sankom Audytor OZC/CO Heating & HVAC Engineering Project (.ozd) Sankom
        b"CAR " => verify_sar_car(prefix, size), // SAR / CAR v2.0x Compressed Archive (.sar, 1990s) ("CAR 2.00" / "CAR 2.01")
        b"S\x00A\x00" => verify_sawgrass_dm(prefix, size), // Sawgrass SG400/SG800 RPCS-R Printer Profile (.dm, Sawgrass Technologies)
        b"PACG" => verify_sbstudio_pac(prefix, size), // Henning Sauer SBStudio PAC Module (.pac)
        b"SC68" => verify_sc68_music(prefix, size), // Benjamin Gerard sc68 Atari ST / Amiga YM2149 & Paula Music (.sc68)
        &[0xC2, 0x19, 0xC2, 0x93] => verify_sc_rep(prefix, size), // Blizzard StarCraft / StarCraft: Remastered Replay (.rep, 1998) Blizzard Entertainment
        b"CFX\x06" // Autodesk Scaleform GFx Movie (.gfx)
        | b"CFX\x07" // Autodesk Scaleform GFx Movie (.gfx)
        | b"CFX\x08" // Autodesk Scaleform GFx Movie (.gfx)
        | b"CFX\t" // Autodesk Scaleform GFx Movie (.gfx)
        | b"CFX\n" // Autodesk Scaleform GFx Movie (.gfx)
        | b"CFX\x0B" // Autodesk Scaleform GFx Movie (.gfx)
        | b"CFX\x0C" // Autodesk Scaleform GFx Movie (.gfx)
        | b"CFX\r" // Autodesk Scaleform GFx Movie (.gfx)
        | b"CFX\x0E" // Autodesk Scaleform GFx Movie (.gfx)
        | b"CFX\x0F" // Autodesk Scaleform GFx Movie (.gfx)
        | b"CFX\x10" // Autodesk Scaleform GFx Movie (.gfx)
        | b"CFX\x11" // Autodesk Scaleform GFx Movie (.gfx)
        | b"CFX\x12" // Autodesk Scaleform GFx Movie (.gfx)
        | b"CFX\x13" // Autodesk Scaleform GFx Movie (.gfx)
        | b"CFX\x14" // Autodesk Scaleform GFx Movie (.gfx)
        | b"GFX\x06" // Autodesk Scaleform GFx Movie (.gfx)
        | b"GFX\x07" // Autodesk Scaleform GFx Movie (.gfx)
        | b"GFX\x08" // Autodesk Scaleform GFx Movie (.gfx)
        | b"GFX\t" // Autodesk Scaleform GFx Movie (.gfx)
        | b"GFX\n" // Autodesk Scaleform GFx Movie (.gfx)
        | b"GFX\x0B" // Autodesk Scaleform GFx Movie (.gfx)
        | b"GFX\x0C" // Autodesk Scaleform GFx Movie (.gfx)
        | b"GFX\r" // Autodesk Scaleform GFx Movie (.gfx)
        | b"GFX\x0E" // Autodesk Scaleform GFx Movie (.gfx)
        | b"GFX\x0F" // Autodesk Scaleform GFx Movie (.gfx)
        | b"GFX\x10" // Autodesk Scaleform GFx Movie (.gfx)
        | b"GFX\x11" // Autodesk Scaleform GFx Movie (.gfx)
        | b"GFX\x12" // Autodesk Scaleform GFx Movie (.gfx)
        | b"GFX\x13" // Autodesk Scaleform GFx Movie (.gfx)
        | b"GFX\x14" => verify_scaleform_gfx_v8(prefix, size), // Autodesk Scaleform GFx Movie (.gfx)
        b"SZ\n\x04" => verify_schindler_szip(prefix, size), // Michael Schindler SZIP Block-Sorted Stream (.sz)
        b"SCP\x00" // SuperCard Pro magnetic flux capture (.scp)
        | b"SCP\x10" // SuperCard Pro magnetic flux capture (.scp)
        | b"SCP\x18" // SuperCard Pro magnetic flux capture (.scp)
        | b"SCP\x19" // SuperCard Pro magnetic flux capture (.scp)
        | b"SCP " // SuperCard Pro magnetic flux capture (.scp)
        | b"SCP$" => verify_scp(prefix, size), // SuperCard Pro magnetic flux capture (.scp)
        b"Scra" => verify_scratch_project_v1(prefix, size), // MIT Scratch 1.x Squeak Binary Project (.sb, 2007) MIT Media Lab
        b"SCRI" // Movie Magic Screenwriter Screenplay (.scw / .mms)
        | b"SCWX" => verify_screenwriter(prefix, size), // Movie Magic Screenwriter Screenplay (.scw / .mms)
        b"SCS#" => verify_scs_hashfs(prefix, size), // SCS Software HashFS Game Archive (.scs, 2012) SCS Software
        b"\x15gmP" => verify_scs_pmg(prefix, size), // SCS Software Prism3D Engine 3D Model Geometry (.pmg, 2012) SCS Software
        b"BSII" // SCS Software Prism3D Encrypted SII Configuration / Save (.sii, 2012) SCS Software
        | b"ScsC" => verify_scs_sii_encrypted(prefix, size), // SCS Software Prism3D Encrypted SII Configuration / Save (.sii, 2012) SCS Software
        &[0x01, 0x0A, 0xB1, 0x70] => verify_scs_tobj(prefix, size), // SCS Software Prism3D Engine Texture Object Descriptor (.tobj, 2012) SCS Software
        b"SDC " => verify_sdc_gdb(prefix, size), // Esri Smart Data Compression (SDC FORMAT 1.00) Geospatial Database (.gdb, Esri)
        b"SDW\x02" => verify_sdw2_snp(prefix, size), // SDW v2 System Snapshot (.snp)
        &[0x17, 0xA4, 0xA3, 0x5E] // Alpha Networks SEAMA Firmware Container (.seama)
        | &[0x5E, 0xA3, 0xA4, 0x17] => verify_seama_firmware(prefix, size), // Alpha Networks SEAMA Firmware Container (.seama)
        &[0x0F, 0x1C, 0x0F, 0x0E] // Sean Conran Amiga Music Module (.scr)
        | &[0x0F, 0xFF, 0x0F, 0xE2] => verify_sean_conran_scr(prefix, size), // Sean Conran Amiga Music Module (.scr)
        b"SBLI" => verify_security_box_encrypted(prefix, size), // Stormshield / Arkoon Security BOX Encrypted Container (.box)
        b"AFS\x00" => verify_sega_afs_archive(prefix, size), // Sega Dreamcast / PS2 AFS File System Archive (.afs)
        b"GSGM" => verify_sega_dragon_engine_gmd(prefix, size), // Sega Ryu Ga Gotoku Dragon Engine 3D Model (.gmd, 2016) Sega
        b"FILM" => verify_sega_film(prefix, size), // Sega Saturn FILM / Cinepak CPK Multimedia Container (.cpk / .cak, 1994) Sega
        b"GYMX" => verify_sega_genesis_gym(prefix, size), // Sega Genesis YM2612 Audio Log (.gym, 1999) YMAMP / DGen
        b"PVMH" // Sega Dreamcast PVM Texture Archive (PVMH)
        | b"XVMH" => verify_sega_pvm(prefix, size), // Sega Dreamcast PVM Texture Archive (PVMH)
        b"SGC\x1A" => verify_sega_sgc_audio(prefix, size), // Sega Master System / Game Gear / ColecoVision SGC Audio (.sgc)
        &[0xF3, 0xED, 0x56, 0x18] // Sega Master System / Game Gear Cartridge ROM (.sms / .gg)
        | b"\xF3\xEDV1" // Sega Master System / Game Gear Cartridge ROM (.sms / .gg)
        | &[0xF3, 0xED, 0x56, 0xC3] => verify_sega_sms_rom_bootstrap(prefix, size), // Sega Master System / Game Gear Cartridge ROM (.sms / .gg)
        &[0x8C, 0xFF, 0x7C, 0xF9] // NSA / Linux Kernel SELinux Binary Security Policy (0xF97CFF8C / 0xF97CFF8D / 0xF97CFF8E LE)
        | &[0x8D, 0xFF, 0x7C, 0xF9] // NSA / Linux Kernel SELinux Binary Security Policy (0xF97CFF8C / 0xF97CFF8D / 0xF97CFF8E LE)
        | &[0x8E, 0xFF, 0x7C, 0xF9] => verify_selinux_policy(prefix, size), // NSA / Linux Kernel SELinux Binary Security Policy (0xF97CFF8C / 0xF97CFF8D / 0xF97CFF8E LE)
        &[0x8F, 0xFF, 0x7C, 0xF9] // NSA SELinux Compiled Policy Module Package (.pp, 0x8FFF7CF9)
        | &[0xF9, 0x7C, 0xFF, 0x8F] => verify_selinux_policy_package(prefix, size), // NSA SELinux Compiled Policy Module Package (.pp, 0x8FFF7CF9)
        b"%SEM" => verify_semi_oasis_stream(prefix, size), // SEMI P39 OASIS Open Artwork System Interchange Standard IC Layout (.oas)
        b"SENJ" => verify_senjutsu_s7(prefix, size), // Senjutsu Tactical Campaign Map Data (SENJUTSUMAPDATA, .s7)
        b"sErC" => verify_sercomm_firmware(prefix, size), // Sercomm sErCoMm Router Firmware Image (.bin)
        b"\x00mos" => verify_serpent_stone_package(prefix, size), // Serpent OS / AerynOS Stone Binary Software Package (.stone, \\x00mos)
        b"TGCF" => verify_setup_specialist_tgcf(prefix, size), // TG-Soft Setup Specialist Multi-Part Archive Volume (.2 / .3, 1999) Bernd Steinhauser
        b".sff" => verify_sff(prefix, size), // Roche 454 Standard Flowgram Format (.sff, 2006) 454 Life Sciences / NCBI / EBI
        b"MOVI" => verify_sgi_movie(prefix, size), // IRIX Movie Multimedia Container (.mv / .movie, 1992) Silicon Graphics
        &[0x01, 0xDA, 0x00, 0x01] // Silicon Graphics IRIS RGB Image (.sgi / .rgb / .bw)
        | &[0x01, 0xDA, 0x00, 0x02] // Silicon Graphics IRIS RGB Image (.sgi / .rgb / .bw)
        | &[0x01, 0xDA, 0x01, 0x01] // Silicon Graphics IRIS RGB Image (.sgi / .rgb / .bw)
        | &[0x01, 0xDA, 0x01, 0x02] => verify_sgi_rgb(prefix, size), // Silicon Graphics IRIS RGB Image (.sgi / .rgb / .bw)
        b"SH3\x00" => verify_sh3_smart(prefix, size), // SH3 R02.04 Smart Configuration Profile (.smart)
        b"IFX\x00" => verify_shockwave_w3d(prefix, size), // Intel / Macromedia Shockwave 3D Scene (IFX, .w3d, 2001) Macromedia / Adobe
        b"ajkg" => verify_shorten(prefix, size), // Shorten Lossless Audio (.shn, 1992) Tony Robinson / SoftSound
        b"Show" => verify_showpartner_hlp(prefix, size), // Brightbill-Roberts Show Partner Animator Help File (.hlp)
        b"shro" => verify_shroom_module(prefix, size), // Shroom Player Module (.sho)
        b"\x16\x00SO" => verify_sierra_rbt(prefix, size), // Sierra On-Line SCI Robot Video Animation (.rbt)
        &[0x81, 0x00, 0xFE, 0x02] => verify_sierra_sci32_pic(prefix, size), // Sierra On-Line SCI1.1 / SCI32 Background Picture Resource (.p56, 1992) Sierra On-Line
        &[0x80, 0x80, 0x00, 0x00] => verify_sierra_sci32_view(prefix, size), // Sierra On-Line SCI32 Sprite View Resource (.v56, 1993) Sierra On-Line
        b"\r\x0BSO" // Sierra On-Line SOL Audio (.aud / .sfx)
        | b"\r\x0CSO" // Sierra On-Line SOL Audio (.aud / .sfx)
        | b"\r\rSO" // Sierra On-Line SOL Audio (.aud / .sfx)
        | b"\r\x0ESO" // Sierra On-Line SOL Audio (.aud / .sfx)
        | b"\x8D\x0BSO" // Sierra On-Line SOL Audio (.aud / .sfx)
        | b"\x8D\x0CSO" // Sierra On-Line SOL Audio (.aud / .sfx)
        | b"\x8D\rSO" // Sierra On-Line SOL Audio (.aud / .sfx)
        | b"\x8D\x0ESO" => verify_sierra_sol(prefix, size), // Sierra On-Line SOL Audio (.aud / .sfx)
        b"FOVb" => verify_sigma_x3f_raw(prefix, size), // Sigma Foveon X3 Camera RAW Image (.x3f, FOVb)
        b"sdoc" => verify_signum_doc(prefix, size), // Application Systems Heidelberg Signum! 1/2 Atari ST Document (.sdo)
        b"eset" // Application Systems Heidelberg Signum! Atari ST Font (.e24 / .l30 / .p24 / .p9)
        | b"ls30" // Application Systems Heidelberg Signum! Atari ST Font (.e24 / .l30 / .p24 / .p9)
        | b"ps09" // Application Systems Heidelberg Signum! Atari ST Font (.e24 / .l30 / .p24 / .p9)
        | b"ps24" => verify_signum_font(prefix, size), // Application Systems Heidelberg Signum! Atari ST Font (.e24 / .l30 / .p24 / .p9)
        &[0xEB, 0x17, 0xA6, 0x03] => verify_silabs_gbl(prefix, size), // Silicon Labs Gecko Bootloader Upgrade Image (.gbl) Silicon Labs
        b"KVAG" => verify_simon_schuster_kvag(prefix, size), // Simon & Schuster Interactive KVAG ADPCM Audio (.vag)
        b"\x81SF3" => verify_simple_file_format_3(prefix, size), // Shinmera Simple File Format 3 Container (.sf3, \\x81SF3\\x00\\xE0\\xD0\\r\\n\\n)
        b"IFF " => verify_sims1_iff(prefix, size), // Maxis The Sims 1 IFF 2.5 Resource Container (.iff / .spf / .flr / .wll, 2000) Maxis / Electronic Arts
        b"QL5A" // Sinclair QL QDOS Filesystem Disk Image (QL5A / QL5B)
        | b"QL5B" => verify_sinclair_qdos(prefix, size), // Sinclair QL QDOS Filesystem Disk Image (QL5A / QL5B)
        b"SRS\x01" // sing-box Binary Rule-Set Container (SRS v1–v5, .srs)
        | b"SRS\x02" // sing-box Binary Rule-Set Container (SRS v1–v5, .srs)
        | b"SRS\x03" // sing-box Binary Rule-Set Container (SRS v1–v5, .srs)
        | b"SRS\x04" // sing-box Binary Rule-Set Container (SRS v1–v5, .srs)
        | b"SRS\x05" => verify_singbox_srs(prefix, size), // sing-box Binary Rule-Set Container (SRS v1–v5, .srs)
        b"\xCD sL" => verify_slim_archive(prefix, size), // MS-DOS Dominic Herity SLIM Compressed File (.slm)
        b"IPSM" => verify_smoothmove_pan(prefix, size), // Infinite Pictures SmoothMove Pan Viewer Panorama (.pan)
        b"SDPX" // SMPTE 268M Digital Picture Exchange Image (.dpx)
        | b"XPDS" => verify_smpte_dpx(prefix, size), // SMPTE 268M Digital Picture Exchange Image (.dpx)
        b"SNES" => verify_snes_spc700(prefix, size), // Nintendo Super NES SPC700 Sound Chip State Dump (.spc, 1998) ZSNES / Snes9x ("SNES-SPC700 Sound File Data")
        b"Snev" => verify_sneventracker_snm(prefix, size), // HertzDevil SnevenTracker Chiptune Module (.snm)
        b"ObsM" => verify_sngplay_obsm(prefix, size), // ObsM SNGPlay AdLib Chiptune (.sng)
        b"TRSN" => verify_sniffer_dos(prefix, size), // Sniffer DOS / Sniffer Basic Network Capture (1986) Harry Saal & Leonard Shustek / Network General ("TRSNIFF data    ")
        b"snoo" => verify_snoop(prefix, size), // Solaris Snoop v2 Network Packet Capture (1995) Brent Callaghan & Rob Gilligan / Sun Microsystems (IETF RFC 1761, "snoop\0\0\0")
        b"$AE1" // Sound Open Firmware Audio DSP Image & Extended Manifest (.ri / .ldc)
        | b"Logs" // Sound Open Firmware Audio DSP Image & Extended Manifest (.ri / .ldc)
        | b"XMan" => verify_sof_firmware(prefix, size), // Sound Open Firmware Audio DSP Image & Extended Manifest (.ri / .ldc)
        b"SLIB" => verify_softlib_archive(prefix, size), // SoftLib Compressed Library Archive (.slb)
        b"(c) " => verify_softmaker_dwr(prefix, size), // SoftMaker Office UI & Dialog Resource Archive (.dwr)
        b"hc`\xA2" => verify_softplan_spv(prefix, size), // SoftPlan Architectural Design reView Project (.spv) SoftPlan Systems
        b"# Pa" => verify_solaris_pkg(prefix, size), // Sun Solaris / SVR4 Package Datastream (.pkg)
        b"Sona" => verify_sonarc(prefix, size), // Sonarc Lossless Compressed Creative Voice Audio (.wv / .vc, 1990s) Speech Compression ("Sonarc-squeezed VOC")
        b"MIDI" => verify_sonix_instrument(prefix, size), // Aegis Sonix Instrument Descriptor (.instr)
        b"PTM " => verify_sony_picturegear_pts(prefix, size), // Sony PictureGear / VAIO Theme Package (PTM fnIA, .pts)
        b".GIM" // Sony PlayStation Portable GIM Graphics Image (.gim, MIG.00.1PSP)
        | b"MIG." => verify_sony_psp_gim(prefix, size), // Sony PlayStation Portable GIM Graphics Image (.gim, MIG.00.1PSP)
        b"OMG." => verify_sony_psp_gmo(prefix, size), // Sony PlayStation Portable GMO 3D Model (.gmo)
        b"SGXD" => verify_sony_sgxd(prefix, size), // Sony SCEI SGXD Sound Group Audio Container (.sgd / .sgb, 2006) Sony Computer Entertainment
        b"riff" => verify_sony_wave64(prefix, size), // Sony Wave64 64-Bit Audio Container (.w64, 1998) Sonic Foundry / Sony Pictures Digital (GUID "riff")
        b"SOPH" => verify_sophos_ide(prefix, size), // Sophos Anti-Virus Virus Identity Definition (.ide, 1999) Sophos
        b"SNGs" => verify_sound_club_module(prefix, size), // Bluemoon Sound Club Module (.sn / .sn2)
        b"IAN9" // Apple IIgs Huibert Aalbers SoundSmith Module (.sng)
        | b"SONG" => verify_soundsmith_module(prefix, size), // Apple IIgs SoundSmith Module (.sng)
        b".SoX" // Sound eXchange (SoX) Native Audio Format (.sox)
        | b"XoS." => verify_sox_native_audio(prefix, size), // Sound eXchange (SoX) Native Audio Format (.sox)
        b"gi\x00\x00" => verify_space_rangers_gi(prefix, size), // Elemental Games Space Rangers Graphics Image (.gi)
        b"SPIS" => verify_spis_lh5(prefix, size), // SPIS LH5-Compressed Container (Delphi TCompress, .lng)
        b"B\x00P\x00" => verify_spreadtrum_pac(prefix, size), // Spreadtrum / Unisoc PAC Firmware Package (.pac)
        b"DHTB" => verify_spreadtrum_signed_image(prefix, size), // Spreadtrum / Unisoc DHTB Signed Partition Image (DHTB)
        b"AKB " => verify_sqex_akb(prefix, size), // Square Enix AKB Audio Bank Container (.akb, 2010) Square Enix
        b"** T" => verify_sqlite2(prefix, size), // SQLite 2.1 Relational Database (2001) D. Richard Hipp ("** This file contains an SQLite 2.1 database **")
        &[0xD9, 0xD5, 0x05, 0xF9] => verify_sqlite_journal(prefix, size), // SQLite rollback journal
        &[0x37, 0x7F, 0x06, 0x82] // SQLite 3 Write-Ahead Log
        | &[0x37, 0x7F, 0x06, 0x83] // SQLite 3 Write-Ahead Log
        | &[0x37, 0x7F, 0x06, 0x84] // SQLite 3 Write-Ahead Log
        | &[0x37, 0x7F, 0x06, 0x85] => verify_sqlite_wal(prefix, size), // SQLite 3 Write-Ahead Log
        b"SEDB" => verify_square_enix_scd(prefix, size), // Square Enix Sound Container Data (.scd)
        b"SeWa" => verify_square_enix_spw(prefix, size), // Square Enix SeWave Audio Stream (.spw)
        b"\xFA\xFARI" // Squirrel VM Compiled Closure Bytecode (.cnut / .nut)
        | b"\xFA\xFASQ" => verify_squirrel_cnut(prefix, size), // Squirrel VM Compiled Closure Bytecode (.cnut / .nut)
        b"HLSQ" => verify_sqz_archive(prefix, size), // Jonas I. Hammarberg Squeeze It (SQZ) Archive (.sqz, 1990)
        b"para" => verify_ssbu_paracobn_prc(prefix, size), // Nintendo Super Smash Bros. Ultimate Binary Parameter (paracobn, .prc)
        b"GFFI" => verify_ssi_gff_archive(prefix, size), // Strategic Simulations General File Format Archive (.gff)
        b"DfuS" => verify_st_dfuse(prefix, size), // STMicroelectronics DfuSe USB Device Firmware Upgrade Image (2007) STMicroelectronics ("DfuSe\x01")
        b"STAC" => verify_stacker(prefix, size), // Stacker Compressed Disk Volume (.dsk, 1990) Stac Electronics ("STACKER  version  1.")
        b"pM85" // Atari ST STAD High-Resolution Compressed Graphic (.pac)
        | b"pM86" => verify_stad_pac(prefix, size), // Atari ST STAD High-Resolution Compressed Graphic (.pac)
        &[0x10, 0x08, 0x00, 0x00] => verify_stalker_xray_thm(prefix, size), // GSC Game World X-Ray Engine Texture Descriptor (.thm, 0x0810)
        b"SBAs" => verify_starbound_pak(prefix, size), // Chucklefish Starbound SBAsset6 Game Archive (.pak, 2013) Chucklefish
        b"STK1" => verify_starkos_song(prefix, size), // Targhan / Arkos Amstrad CPC Starkos Song (.sks)
        b"ST1." => verify_startrekker_nt(prefix, size), // Fairlight Startrekker AM Synth Instrument Bank (.nt)
        b".\\\\\\" => verify_starwriter_legacy_vor(prefix, size), // Star Division StarWriter DOS Legacy Document Template (.vor, 1991) Star Division
        &[0x6F, 0xD9, 0xFF, 0xFF] => verify_statistica_scr(prefix, size), // StatSoft Statistica Scrollsheet (.scr)
        b"CSS " => verify_statistica_sta(prefix, size), // StatSoft Statistica Spreadsheet (.sta)
        &[0x05, 0x04, 0x03, 0x06] => verify_statistica_stg(prefix, size), // StatSoft Statistica Graph (.stg)
        b"STLR" => verify_steelmc_stlr_srg(prefix, size), // SteelMC Rust Minecraft Server Region Container (STLR, .srg)
        &[0x20, 0x2F, 0xF3, 0x7A] => verify_stockfish_nnue(prefix, size), // Stockfish Developers NNUE Chess Neural Network (.nnue)
        b"SPM\x02" => verify_stonetracker_module(prefix, size), // Stone Amiga Stonetracker Module (.spm)
        b"SPS\x02" => verify_stonetracker_sps(prefix, size), // Amiga Stonetracker Sample Bank (.sps)
        b"50.1" => verify_storm_gm(prefix, size), // Akella Storm Engine 2.x 3D Geometry Model (.gm, 2003) Akella
        b"AmBs" // Atari ST STOS / Amiga AMOS BASIC Memory Bank (.mbk / .abk)
        | b"Lion" => verify_stos_amos_bank(prefix, size), // Atari ST STOS / Amiga AMOS BASIC Memory Bank (.mbk / .abk)
        b"\x1ATLC" => verify_student_writing_center(prefix, size), // The Learning Company Student Writing Center Document (\\x1ATLC\\x01FF\\x00)
        b"Sun\xFE" => verify_sun_xrs(prefix, size), // Sun Microsystems XRS Resource Archive (.xrs) Sun Microsystems
        b"Y\xA6j\x95" => verify_sunras(prefix, size), // Sun Raster image
        b"SVOX" => verify_sunvox_module(prefix, size), // WarmPlace SunVox Modular Synthesizer Project (.sunvox)
        b"sunx" => verify_sunxi_package(prefix, size), // Allwinner sunxi-package Boot Firmware Container (.fex, 2012) Allwinner Technology
        b"PFS0" => verify_switch_pfs0(prefix, size), // Nintendo Switch Partition FS0 / NSP Submission Package (2017) Nintendo
        &[0x7A, 0x1A, 0x20, 0x10] => verify_symbian_sisx(prefix, size), // Symbian OS v9 SISX Installation Package (.sis / .sisx, 2005) Symbian (UID1 0x10201A7A)
        b"EMBE" => verify_symbolics_carry(prefix, size), // Symbolics Lisp Machine Carry / Distribution Tape (.carry, 1982) Symbolics
        b"SymM" => verify_symphonie_module(prefix, size), // Patrick Meng Amiga Symphonie Pro Module (.symmod)
        b"SYNB" // Synder SNG-Player Chiptune (.sng)
        | b"SYNC" // Synder SNG-Player Chiptune (.sng)
        | b"SYNF" // Synder SNG-Player Chiptune (.sng)
        | b"SYNH" => verify_synder_sng_module(prefix, size), // Synder SNG-Player Chiptune (.sng)
        &[0x51, 0x10, 0xA5, 0xAB] => verify_synopse_synlz(prefix, size), // Synopse mORMot SynLZ Compressed Stream (.synlz, Arnaud Bouchez / Synopse)
        b"RHHH" => verify_systemd_catalog(prefix, size), // Linux systemd Binary Journal Message Catalog (.catalog, 2012) Lennart Poettering
        b"LPKS" => verify_systemd_journal(prefix, size), // Linux systemd Structured Binary Journal Log (.journal, 2012) Lennart Poettering & Kay Sievers / Red Hat ("LPKSHHRH")
        b"T98F" => verify_t98next_nfd_image(prefix, size), // NEC PC-9801 T98-Next Floppy Disk Image (.nfd, T98FDDIMAGE.R0/R1)
        b"T3-i" => verify_tads3_image(prefix, size), // TADS 3 T3 VM Image (.t3)
        b"\x06t.a" => verify_tag_adventure(prefix, size), // T.A.G. Text-Adventure-Generator Compiled Game (.tag)
        b"\x89SDB" => verify_tally_compiled_tdl(prefix, size), // Tally.ERP 9 Compiled Tally Definition Language (.tcp)
        b"\x04V1." => verify_taxprep_tfd(prefix, size), // Wolters Kluwer Taxprep / Canadian CRA Tax Form Definition (.tfd)
        b"\x81u\xE49" => verify_tdbd_database(prefix, size), // TDBD Binary Database (.tdbd)
        b"FB09" // Borland Turbo Debugger 16/32-Bit Standalone Symbol Table (.tds, 1988–1997) Borland
        | b"FB0A" // Borland Turbo Debugger 16/32-Bit Standalone Symbol Table (.tds, 1988–1997) Borland
        | b"FB0B" // Borland Turbo Debugger 16/32-Bit Standalone Symbol Table (.tds, 1988–1997) Borland
        | &[0xFB, 0x52, 0x00, 0x00] => verify_tds(prefix, size), // Borland Turbo Debugger 16/32-Bit Standalone Symbol Table (.tds, 1988–1997) Borland
        &[0x0F, 0x0F, 0x3A, 0x57] // Tektronix Oscilloscope Reference Waveform (.wfm, :WFM#001/002/003)
        | &[0xF0, 0xF0, 0x3A, 0x57] => verify_tektronix_wfm(prefix, size), // Tektronix Oscilloscope Reference Waveform (.wfm, :WFM#001/002/003)
        b"TD\x00\x00" // Sydex TeleDisk floppy image (.td0)
        | b"td\x00\x00" => verify_teledisk(prefix, size), // Sydex TeleDisk floppy image (.td0)
        b"TDF$" => verify_telegram_tdf(prefix, size), // Telegram Desktop Encrypted Data File (TDF$)
        &[0x0B, 0x1D, 0x2B, 0x1A] => verify_telix_salt_slc(prefix, size), // Telix SALT Compiled Script (.slc, 1988, Exis / deltaComm)
        b"TAP0" => verify_tencent_tap_image(prefix, size), // Tencent TAP Animated Image Container (.tap)
        &[0x0C, 0xDB, 0x12, 0x34] // Google TensorStore OCDBT B+Tree Node / Manifest (.ocdbt)
        | &[0x0C, 0xDB, 0x20, 0xDE] // Google TensorStore OCDBT B+Tree Node / Manifest (.ocdbt)
        | &[0x0C, 0xDB, 0x3A, 0x2A] => verify_tensorstore_ocdbt(prefix, size), // Google TensorStore OCDBT B+Tree Node / Manifest (.ocdbt)
        b"TMOD" => verify_terraria_tmod(prefix, size), // tModLoader Terraria Compiled Mod Archive (.tmod, 2015) tModLoader
        b"TFMf" => verify_tfm_music_maker(prefix, size), // Shiru TFM Music Maker YM2203/YM2612 Module (.tfe)
        b"TFMX" // Chris Huelsbeck TFMX Amiga Module (.mdat)
        | b"tfmx" => verify_tfmx_module(prefix, size), // Chris Huelsbeck TFMX (The Final Musicsystem eXtended) Amiga Module (.mdat)
        b"ZZ\x12\xAB" => verify_thebat_srb(prefix, size), // Ritlabs The Bat! Sorting Office Filter Rules (ACCOUNT.SRB)
        b"\x13The" => verify_thedraw_font(prefix, size), // TheDraw ANSI/ASCII Block Font (.tdf, 1986) Ian E. Davis / TheSoft Programming Services ("\x13TheDraw FONTS file\x1A")
        b"G15:" => verify_thilmera7_zpac_t7p(prefix, size), // thilmera7 System Monitor ZPAC Resource Package (.t7p)
        b"TIPA" => verify_ti_ais(prefix, size), // Texas Instruments Application Image Script Boot Image (.ais) Texas Instruments
        b"BTSB" => verify_ti_bts_script(prefix, size), // Texas Instruments WiLink Bluetooth Init Script (.bts, BTSB)
        b"MSTR" => verify_ti_multicore(prefix, size), // Texas Instruments Multicore Meta Header Application Image (.appimage) Texas Instruments
        b"CC13" // Texas Instruments SimpleLink Over-the-Air Download Firmware Image (.bin) Texas Instruments
        | b"CC23" // Texas Instruments SimpleLink Over-the-Air Download Firmware Image (.bin) Texas Instruments
        | b"CC26" // Texas Instruments SimpleLink Over-the-Air Download Firmware Image (.bin) Texas Instruments
        | b"OAD " => verify_ti_oad(prefix, size), // Texas Instruments SimpleLink Over-the-Air Download Firmware Image (.bin) Texas Instruments
        b"RPRC" => verify_ti_rprc(prefix, size), // Texas Instruments Remote Processor Resource Core Firmware Image (.rprc) Texas Instruments
        &[0x2A, 0x9F, 0x00, 0x00] // Texas Instruments TAS2781 / TAS2563 Smart Amplifier DSP Firmware (.bin)
        | b"5552" => verify_ti_smartamp_firmware(prefix, size), // Texas Instruments TAS2781 / TAS2563 Smart Amplifier DSP Firmware (.bin)
        b"M2B1" => verify_ti_tps_upg(prefix, size), // Dell / TI TPS65987 USB-C Monitor Firmware Upgrade (.upg, Texas Instruments)
        b"The " => verify_time_of_war_d3e(prefix, size), // The Time of War Engine Data File v4.00 (.d3e)
        &[0x78, 0x9F, 0x3E, 0x22] => verify_tnef(prefix, size), // Microsoft Transport Neutral Encapsulation Format (winmail.dat / .tnef, 1992) Microsoft
        b"velK" => verify_tomofon_topatch(prefix, size), // Klevgränd Tomofon Audio Patch (velKqdiP, .topatch)
        &[0x21, 0x00, 0x00, 0x00] => verify_torquescript_dso(prefix, size), // GarageGames Torque Game Engine Compiled TorqueScript (.dso, 2001) GarageGames
        b"PFH0" // Creative Assembly Total War PackFile Archive (.pack, PFH0..PFH5)
        | b"PFH2" // Creative Assembly Total War PackFile Archive (.pack, PFH0..PFH5)
        | b"PFH3" // Creative Assembly Total War PackFile Archive (.pack, PFH0..PFH5)
        | b"PFH4" // Creative Assembly Total War PackFile Archive (.pack, PFH0..PFH5)
        | b"PFH5" => verify_total_war_pack_pfh(prefix, size), // Creative Assembly Total War PackFile Archive (.pack, PFH0..PFH5)
        b"\x00CAL" => verify_toyota_cuw(prefix, size), // Toyota / Lexus Techstream Calibration Update Wizard Package (.cuw) Toyota
        b"MINI" => verify_tplink_minifs(prefix, size), // TP-LINK MiniFS Embedded Read-Only Filesystem (.minifs)
        b"CPLX" // Crazy Crack TrackerPacker Compressed Amiga Module (MEXX_TP / CPLX_TP3)
        | b"MEXX" => verify_trackerpacker_module(prefix, size), // Crazy Crack TrackerPacker Compressed Amiga Module (MEXX_TP / CPLX_TP3)
        b"GBX\x03" // Nadeo GameBox Binary Asset & TrackMania Map (.gbx, 2003) Nadeo / Ubisoft
        | b"GBX\x04" // Nadeo GameBox Binary Asset & TrackMania Map (.gbx, 2003) Nadeo / Ubisoft
        | b"GBX\x05" // Nadeo GameBox Binary Asset & TrackMania Map (.gbx, 2003) Nadeo / Ubisoft
        | b"GBX\x06" => verify_trackmania_gbx(prefix, size), // Nadeo GameBox Binary Asset & TrackMania Map (.gbx, 2003) Nadeo / Ubisoft
        b"ACS$" => verify_trainz_cdp(prefix, size), // Auran / N3V Games Trainz Content Dispatcher Pack (.cdp, 2001) N3V Games
        b"JIRF" => verify_trainz_im(prefix, size), // Auran Trainz Jet Indexed Mesh (.im)
        &[0xFF, 0x64, 0x80, 0xFF] => verify_tresorit_profile_backup(prefix, size), // Tresorit Encrypted Storage Profile Backup (.backup)
        b"TROC" => verify_troc_fs(prefix, size), // Toshiba TROC Read-Only Embedded Filesystem (.troc)
        b"TRPH" => verify_trph_xpd(prefix, size), // TRPH Mobile Printer Configuration (.xpd)
        b"T'So" => verify_tsoundsystem(prefix, size), // T'SoundSystem Tracker Module (.tsd)
        b"TCW4" => verify_ttc_xcfg(prefix, size), // TCW40 Controller Configuration (.xcfg)
        b"T\x00U\x00" => verify_tuneup_tun(prefix, size), // TuneUp Utilities Installation/Undo Log (.tun, 2004) TuneUp Software
        b"INFO" => verify_turbopower_pophelp(prefix, size), // TurboPower PopHelp / Object Professional Help Database (.hlp, 1990)
        b"RMP1" => verify_turbopower_tphelp(prefix, size), // TurboPower Turbo Professional Reference Manual / TPHelp Database (.hlp, 1989)
        b"TPH2" => verify_turbopower_tphelp2(prefix, size), // TurboPower Object Professional Compiled Help (.hlp, 1991) TurboPower Software
        b"TVRW" => verify_turvision_rwp(prefix, size), // Turbo Vision Resource Workshop Project (.rwp, 1993) BitSoft / Borland
        b"TWIN" => verify_twinvq_audio(prefix, size), // NTT Human Interface Labs TwinVQ Transform-Domain Audio (.vqf)
        b"TZif" => verify_tzif(prefix, size), // IANA Time Zone Information Binary Format (TZif, 1986) Arthur David Olson / IETF RFC 8536
        b"TZ\x00\x00" => verify_tzip_compressed(prefix, size), // TZIP Compressed Stream (.tz)
        b"U\x00A\x00" => verify_ua_resume1(prefix, size), // UA Developments Patcher Download Resume State (UA-RESUME-1, .resume)
        b"UBF9" => verify_ubf_font(prefix, size), // Symantec / Peter Norton Universal BitMap Font (.ubf, 1991) Symantec ("UBF91a\r\n")
        &[0xFA, 0x32, 0x00, 0x80] => verify_ubicom_firmware(prefix, size), // Ubicom IP3K / IP5K / IP7K Network Processor Firmware Image (.bin)
        &[0x31, 0x18, 0x10, 0x06] // Linux Unsorted Block Image / UBIFS Flash Filesystem (.ubi / .ubifs)
        | b"UBI!" // Linux Unsorted Block Image / UBIFS Flash Filesystem (.ubi / .ubifs)
        | b"UBI#" => verify_ubifs(prefix, size), // Linux Unsorted Block Image / UBIFS Flash Filesystem (.ubi / .ubifs)
        b"UBNT" => verify_ubiquiti_ubnt(prefix, size), // Ubiquiti airOS / EdgeOS Firmware Image (.bin)
        b"3\xAA\xFBW" => verify_ubisoft_anvil_acv(prefix, size), // Ubisoft Anvil Engine Save Game Container (.acv)
        b"BIG\x00" => verify_ubisoft_jade_bf(prefix, size), // Ubisoft Jade Engine Bigfile Archive (.bf, 2003) Ubisoft Montpellier
        b"TBX\x00" => verify_ubisoft_xbt(prefix, size), // Ubisoft Dunia Engine XBT Texture (.xbt, 2008) Ubisoft Montreal
        &[0x27, 0x05, 0x19, 0x56] => verify_uboot_uimage(prefix, size), // Das U-Boot Legacy uImage Firmware/Kernel Image (2000) Wolfgang Denk / DENX Software Engineering (0x27051956 BE)
        b"bFLT" => verify_uclinux_bflt(prefix, size), // uClinux Binary Flat Format No-MMU Executable (1998) Jeff Dionne & Kenneth Albanowski / uClinux Project
        &[0x01, 0x4D, 0x8B, 0x3B] // UEFI Update Capsule / AMI Aptio Signed Firmware Capsule (.cap / .bin) UEFI Forum
        | &[0x8B, 0xA6, 0x3C, 0x4A] // UEFI Update Capsule / AMI Aptio Signed Firmware Capsule (.cap / .bin) UEFI Forum
        | &[0xB9, 0x82, 0x91, 0x53] // UEFI Update Capsule / AMI Aptio Signed Firmware Capsule (.cap / .bin) UEFI Forum
        | &[0xBD, 0x86, 0x66, 0x3B] // UEFI Update Capsule / AMI Aptio Signed Firmware Capsule (.cap / .bin) UEFI Forum
        | &[0xC9, 0x3E, 0xB1, 0x14] => verify_uefi_capsule(prefix, size), // UEFI Update Capsule / AMI Aptio Signed Firmware Capsule (.cap / .bin) UEFI Forum
        b"VZL\x01" // UEFI Terse Executable (TE) firmware image
        | b"VZd\x86" // UEFI Terse Executable (TE) firmware image
        | b"VZd\xAA" // UEFI Terse Executable (TE) firmware image
        | b"VZ\xC2\x01" => verify_uefi_te(prefix, size), // UEFI Terse Executable (TE) firmware image
        b"UHA\x01" // UHARC high-compression archive (.uha)
        | b"UHA\x02" // UHARC high-compression archive (.uha)
        | b"UHA\x03" // UHARC high-compression archive (.uha)
        | b"UHA\x04" // UHARC high-compression archive (.uha)
        | b"UHA\x05" // UHARC high-compression archive (.uha)
        | b"UHA\x06" // UHARC high-compression archive (.uha)
        | b"UHA\x07" => verify_uharc(prefix, size), // UHARC high-compression archive (.uha)
        b"TWFB" => verify_ultimate_ride_text(prefix, size), // Disney / Walt Disney Imagineering Ultimate Ride Text Strings (.tsf)
        b"MAS_" => verify_ultratracker_module(prefix, size), // MAS UltraTracker Module (.ult)
        b"S\x8FNG" => verify_unique_dev_uds(prefix, size), // Unique Development Amiga Music Module (.uds)
        &[0x42, 0x89, 0xE3, 0x0D] => verify_unity_addressables_catalog(prefix, size), // Unity Engine Addressables Binary Content Catalog (catalog.bin, 0x0DE38942 LE, 2023) Unity Technologies
        b"tQ\x10\xAD" => verify_unity_enlighten_caw(prefix, size), // Unity Engine Enlighten Precomputed GI Cluster/Albedo Workspace (.caw / .sse)
        &[0xAF, 0x1B, 0xB1, 0xFA] => verify_unity_il2cpp_metadata(prefix, size), // Unity Engine IL2CPP Global Metadata Binary (global-metadata.dat, 0xFAB11BAF LE, 2015) Unity Technologies
        b"Prev" => verify_unity_preview_info(prefix, size), // Unity Editor Library Asset Preview Descriptor (PreviewAssetData, .info)
        b"Tuan" // Tuanjie Engine WebGL Data Package ("TuanjieWebData1.0\0", 2023) Unity China
        | b"Unit" => verify_unity_webdata(prefix, size), // Unity WebGL Data Package (UnityWebData1.0)
        &[0xCE, 0xFA, 0xAD, 0x1B] => verify_unixware_bfs(prefix, size), // SCO UnixWare Boot File System Image (.bfs)
        &[0x0C, 0xC0, 0xBE, 0x44] => verify_unreal_buildpatch_manifest(prefix, size), // Epic Games Unreal Engine BuildPatch Binary Manifest
        b"GVAS" => verify_unreal_gvas(prefix, size), // Epic Games Unreal Engine GVAS SaveGame (.sav)
        b"O\xEEL\xA1" => verify_unreal_locmeta(prefix, size), // Epic Games Unreal Engine Localization Metadata (.locmeta)
        b"\x0E\x14tu" => verify_unreal_locres(prefix, size), // Epic Games Unreal Engine Localization Resource (.locres)
        &[0x9E, 0x2A, 0x83, 0xC1] // Epic Games Unreal Engine Package / Asset (.uasset / .umap / .upk / .u)
        | &[0xC1, 0x83, 0x2A, 0x9E] => verify_unreal_package(prefix, size), // Epic Games Unreal Engine Package / Asset (.uasset / .umap / .upk / .u)
        b"HCAC" => verify_unreal_pipeline_cache(prefix, size), // Epic Games Unreal Engine Pipeline State Object Cache (.upipelinecache)
        b"-==-" => verify_unreal_utoc(prefix, size), // Unreal Engine 4.26 / 5 IoStore Table of Contents (.utoc, 2020) Epic Games ("-==--==--==--==-")
        b"2CRT" // Epic Games Unreal Insights Trace Capture (.utrace)
        | b"ECRT" // Epic Games Unreal Insights Trace Capture (.utrace)
        | b"TRC2" // Epic Games Unreal Insights Trace Capture (.utrace)
        | b"TRCE" => verify_unreal_utrace(prefix, size), // Epic Games Unreal Insights Trace Capture (.utrace)
        b"FREQ" => verify_utau_frq(prefix, size), // UTAU Singing Synthesizer Fundamental Frequency Table (.frq, 2008) Ameya/Ayame
        b"V129" => verify_v129_rdt(prefix, size), // DMR Radio Codeplug Data (V129 / V100, .rdt)
        &[0x50, 0x05, 0xDE, 0xC0] // Google V8 JavaScript Engine Cached Bytecode (.jsc / .bin)
        | &[0xEC, 0x05, 0xDE, 0xC0] => verify_v8_bytecode(prefix, size), // Google V8 JavaScript Engine Cached Bytecode (.jsc / .bin)
        b"IDAG" => verify_valve_ani(prefix, size), // Valve Source Engine StudioMDL External Animation Data (.ani, 2004) Valve
        b"IDST" => verify_valve_studio_mdl(prefix, size), // Valve GoldSrc / Source Studio 3D Model (.mdl, 1998) Valve
        &[0xE8, 0xAC, 0xCC, 0xC4] // Valve Source 2 Tools Asset Info Index (tools_asset_info.bin)
        | &[0xE9, 0xAC, 0xCC, 0xC4] => verify_valve_tools_asset_info(prefix, size), // Valve Source 2 Tools Asset Info Index (tools_asset_info.bin)
        b"4\x12\xAAU" => verify_valve_vpk(prefix, size), // Valve Pak Source Engine Archive (2008) Valve (0x55AA1234 LE)
        b"VTF\x00" => verify_valve_vtf(prefix, size), // Valve Source Engine Texture Format (.vtf, 2004) Valve
        b"IDSV" => verify_valve_vvd(prefix, size), // Valve Source Engine StudioMDL Vertex Data (2004) Valve
        b"\x0CDyn" => verify_vam_vab(prefix, size), // Virt-A-Mate DynamicStore Binary Mesh (.vab, 2018) MeshedVR
        b"VCLM" => verify_vcl_metafile(prefix, size), // Borland VCL Metafile / StarView SVM (.svm / .vcl, "VCLMTF")
        b"vctr" => verify_vctr_vector(prefix, size), // vctr Binary-Framed JSON Vector (.vctr)
        b"LOGG" => verify_vector_blf(prefix, size), // Vector Informatik Binary Logging Format CAN/LIN/FlexRay Bus Log (.blf)
        b"\nPan" => verify_vector_fwk(prefix, size), // VecTor Panel PV19 Finite Element Workspace (.fwk)
        b"Vgm " => verify_vgm(prefix, size), // Video Game Music Chiptune Register Log (.vgm, 2000) Dave / Maxim / ValleyBell / SMS Power!
        b"GENH" => verify_vgmstream_genh(prefix, size), // vgmstream Generic Headered Audio Stream (.genh)
        b"Vic2" => verify_vic2_huffman(prefix, size), // Atari ST Vic2 Huffman Compressed Data (Vic2)
        b"VIMs" => verify_vim_spell_spl(prefix, size), // Vim Compiled Spell Checking Dictionary (.spl, 2006) Bram Moolenaar
        b"b0VI" => verify_vim_swap(prefix, size), // Vim Swap File (.swp, 1991) Bram Moolenaar ("b0VIM ")
        b"VIO\x02" => verify_vio2_score(prefix, size), // VIO v2 Music Score (.vio)
        b"VDJ\x00" => verify_virtualdj_vdjsample(prefix, size), // Atomix VirtualDJ Audio/Video Sampler Clip (.vdjsample, 2014) Atomix Productions
        b"COWD" // VMware Virtual Disk Image (.vmdk)
        | b"KDMV" => verify_vmdk(prefix, size), // VMware Virtual Disk Image (.vmdk)
        b"MRVN" => verify_vmware_nvram(prefix, size), // VMware Virtual Machine BIOS/EFI Non-Volatile RAM State (.nvram, 1999) VMware
        b"HERE" => verify_voodoo_vss(prefix, size), // Thomas Winischhofer Voodoo Supreme Synthesizer Module (.vss)
        b"Kvxl" => verify_voxlap_kv6(prefix, size), // Ken Silverman Voxlap Engine KV6 Voxel Sprite (.kv6, 2000) Ken Silverman
        b"CcnK" => verify_vst2_fxp(prefix, size), // Steinberg VST 2 Plug-In Preset / Bank (.fxp / .fxb)
        b"VST3" => verify_vst3_vstpreset(prefix, size), // Steinberg VST 3 Plug-In Preset (.vstpreset)
        b"[Hea" => verify_vtech_vrp_log(prefix, size), // VTech Multi-Section Install Log Container (.vrp, [HeadLen])
        b"SGML" => verify_vw_sgo(prefix, size), // Volkswagen / Audi ODIS / VAS-PC SGML Object ECU Flash Container (.sgo) Volkswagen
        b"\x00aot" => verify_wamr_aot(prefix, size), // Bytecode Alliance WebAssembly Micro Runtime AOT Module (.aot)
        b"HM3W" => verify_warcraft3_map(prefix, size), // Blizzard Entertainment Warcraft III World Editor Map (.w3m / .w3x, 2002) Blizzard Entertainment
        b"MDLX" => verify_warcraft3_mdx(prefix, size), // Blizzard Entertainment Warcraft III Binary 3D Model (.mdx, 2002) Blizzard Entertainment
        b"Warc" => verify_warcraft3_replay(prefix, size), // Blizzard Entertainment Warcraft III Recorded Game Replay (.w3g, 2002) Blizzard Entertainment
        b"N\xC6g\x18" => verify_warframe_toc(prefix, size), // Digital Extremes Evolution Engine (Warframe) Cache TOC Index (.toc)
        &[0x03, 0x24, 0x01, 0x01] // Watcom Installer WPK Archive (.wpk)
        | b"_WPK" => verify_watcom_wpk(prefix, size), // Watcom Installer WPK Archive (.wpk)
        &[0x83, 0x48, 0xF0, 0x9F] // W3C / Google Chrome WebBundle & Isolated Web App (.wbn / .swbn)
        | &[0x84, 0x48, 0xF0, 0x9F] // W3C / Google Chrome WebBundle & Isolated Web App (.wbn / .swbn)
        | &[0x85, 0x48, 0xF0, 0x9F] // W3C / Google Chrome WebBundle & Isolated Web App (.wbn / .swbn)
        | &[0x86, 0x48, 0xF0, 0x9F] => verify_webbundle(prefix, size), // W3C / Google Chrome WebBundle & Isolated Web App (.wbn / .swbn)
        b"\x89wbp" => verify_weltenbauer_wbp(prefix, size), // Weltenbauer / Astragon Construction Simulator Persistence Save (.wbp)
        b" FSC" => verify_westwood_csf(prefix, size), // Westwood Studios / EA SAGE Engine Compiled String Table (.csf, 2000) Westwood Studios
        b"Voxe" => verify_westwood_vxl(prefix, size), // Westwood Studios Command & Conquer Voxel Animation (.vxl, 1999) Westwood Studios
        b"WGT " => verify_wgt_font(prefix, size), // WordUp Graphics Toolkit Bitmap Font (.wfn, 1990s) Egerter Software ("WGT Font File  ")
        &[0x01, 0x03, 0xFC, 0xFF] => verify_whitewater_actor_ima(prefix, size), // Whitewater Group Actor Object-Oriented Image (.ima, 1987) Whitewater Group
        &[0x00, 0xD0, 0xC0, 0xDE] => verify_wii_gecko_gct(prefix, size), // Nintendo Wii Gecko / Ocarina Cheat Code Table (.gct)
        b"GHSH" => verify_wildcat_wcx(prefix, size), // Wildcat! BBS v5 Compiled WCX Application/Screen (1996) Santronics / Mustang Software
        &[0x65, 0x00, 0x00, 0x00] // Microsoft Windows System Restore Point Log (rp.log)
        | &[0x66, 0x00, 0x00, 0x00] // Microsoft Windows System Restore Point Log (rp.log)
        | &[0x67, 0x00, 0x00, 0x00] => verify_win_restore_rp_log(prefix, size), // Microsoft Windows System Restore Point Log (rp.log)
        b"MEM0" // Microsoft Windows SuperFetch / ReadyBoot Trace Database (AgGl*.db)
        | b"MEMO" => verify_win_superfetch(prefix, size), // Microsoft Windows SuperFetch / ReadyBoot Trace Database (AgGl*.db)
        &[0x00, 0x04, 0x01, 0x00] // Microsoft Windows Task Scheduler Job Object (.job, 1997) Microsoft
        | &[0x00, 0x05, 0x01, 0x00] // Microsoft Windows Task Scheduler Job Object (.job, 1997) Microsoft
        | &[0x00, 0x06, 0x01, 0x00] // Microsoft Windows Task Scheduler Job Object (.job, 1997) Microsoft
        | &[0x00, 0x0A, 0x01, 0x00] // Microsoft Windows Task Scheduler Job Object (.job, 1997) Microsoft
        | &[0x01, 0x05, 0x01, 0x00] // Microsoft Windows Task Scheduler Job Object (.job, 1997) Microsoft
        | &[0x01, 0x06, 0x01, 0x00] // Microsoft Windows Task Scheduler Job Object (.job, 1997) Microsoft
        | &[0x02, 0x05, 0x01, 0x00] // Microsoft Windows Task Scheduler Job Object (.job, 1997) Microsoft
        | &[0x02, 0x06, 0x01, 0x00] // Microsoft Windows Task Scheduler Job Object (.job, 1997) Microsoft
        | &[0x03, 0x06, 0x01, 0x00] => verify_win_task_job(prefix, size), // Microsoft Windows Task Scheduler Job Object (.job, 1997) Microsoft
        b"B000" => verify_wince_b000ff(prefix, size), // Microsoft Windows CE ROM Binary Image Data Format (.nb0 / .bin)
        b"MSCE" => verify_wince_cab_header(prefix, size), // Microsoft Windows CE Installation Cabinet Header (MSCE)
        b"SHCC" => verify_windows_31_registry(prefix, size), // Microsoft Windows 3.1 Registration Info Database (REG.DAT, SHCC3.10)
        &[0x00, 0xAC, 0xAC, 0x00] => verify_windows_caches_db(prefix, size), // Microsoft Windows Property / App Cache Database (.db)
        &[0xB5, 0xA2, 0xB0, 0xB3] => verify_windows_calendar(prefix, size), // Microsoft Windows 1.x–3.1 Calendar (1985) Microsoft (8-byte palindromic "CALENDAR"+"radnelac" sum)
        b"PcmH" => verify_windows_cdf_ms(prefix, size), // Microsoft Windows Component-Based Servicing Compiled Form (.cdf-ms)
        &[0x30, 0x00, 0x00, 0x00] => verify_windows_evt(prefix, size), // Windows NT / 2000 / XP / Server 2003 Event Log (.evt, 1993) Microsoft (48-byte HeaderSize + "LfLe" at @4..8)
        b"ElfF" => verify_windows_evtx(prefix, size), // Windows Vista+ XML Event Log (.evtx, 2006) Microsoft ("ElfFile\0")
        b"PMCC" => verify_windows_grp(prefix, size), // Microsoft Windows 3.x Program Manager Group (1990) Microsoft
        b"9\x005\x00" => verify_windows_ime_mb(prefix, size), // Microsoft Windows Input Method Editor Compiled Dictionary Table (.mb)
        &[0x02, 0x09, 0x00, 0x40] => verify_windows_installer_script(prefix, size), // Microsoft Windows Installer Binary Execution / Rollback Script (.ibs / .rbs)
        b"PAGE" => verify_windows_kernel_dump(prefix, size), // Windows NT Kernel Memory Crash Dump (1993) Microsoft ("PAGEDUMP" 32-bit / "PAGEDU64" 64-bit / "PAGEHDMP" triage)
        &[0x04, 0x03, 0x02, 0x00] => verify_windows_pnf(prefix, size), // Microsoft Windows Precompiled Setup Information (.pnf, 2001) Microsoft
        b"mrm_" => verify_windows_pri(prefix, size), // Microsoft Windows Package Resource Index (.pri)
        b"HMGR" // Microsoft Windows Game Explorer Rich Saved Game (.sav / .pcsav)
        | b"RGMH" => verify_windows_rich_saved_game(prefix, size), // Microsoft Windows Game Explorer Rich Saved Game (.sav / .pcsav)
        b"$SDI" => verify_windows_sdi_image(prefix, size), // Microsoft Windows System Deployment Image (.sdi, $SDI0001)
        b"RPKW" => verify_wink_presentation(prefix, size), // DebugMode Wink Screen Capture Presentation (.wnk)
        b"WTP " // DebugMode Wink Presentation Template (.wtp)
        | b"WTP2" // DebugMode Wink Presentation Template (.wtp)
        | b"WTP3" // DebugMode Wink Presentation Template (.wtp)
        | b"WTP4" => verify_wink_template(prefix, size), // DebugMode Wink Presentation Template (.wtp)
        &[0x0B, 0x00, 0x00, 0x00] // WinOLS ECU Project / Map Pack (.ols / .kp)
        | b"WinO" => verify_winols_kp(prefix, size), // EVC Electronic WinOLS ECU Map Pack (.kp) EVC electronic
        b"RGDB" => verify_wireless_regdb(prefix, size), // Linux CRDA / cfg80211 Wireless Regulatory Database (regulatory.db, RGDB)
        b"momv" => verify_wiresync_momv2_omo(prefix, size), // momv2$[] WireSyncEngine Obfuscated Model (.omo)
        b"DZIP" => verify_witcher2_dzip(prefix, size), // CD Projekt RED REDengine 2 / The Witcher 2 DZIP Archive (.dzip, 2011) CD Projekt RED
        b"POTA" => verify_witcher3_bundle(prefix, size), // CD Projekt RED REDengine 3 / The Witcher 3 Asset Bundle (.bundle, 2015) CD Projekt RED
        b"SNFH" => verify_witcher3_sav(prefix, size), // CD Projekt RED The Witcher 3 Saved Game (.sav, 2015) CD Projekt RED
        b"CPSW" // CD Projekt RED REDengine 3 / The Witcher 3 Speech Container (.w3speech / .meta, 2015) CD Projekt RED
        | b"TEMS" => verify_witcher3_w3speech(prefix, size), // CD Projekt RED REDengine 3 / The Witcher 3 Speech Container (.w3speech / .meta, 2015) CD Projekt RED
        b"RTSW" => verify_witcher3_w3strings(prefix, size), // CD Projekt RED REDengine 3 / The Witcher 3 Localization Strings (.w3strings, 2015) CD Projekt RED
        b"cwc2" => verify_woodworks_cwb(prefix, size), // WoodWorks Sizer / Connections Project (.cwb) Canadian Wood Council
        b"\x1DWS2" // MicroPro WordStar 2000 Document (.ws2)
        | b"\x7F WS" => verify_wordstar_2000_doc(prefix, size), // MicroPro WordStar 2000 Document (.ws2)
        b"Worl" => verify_worldcraft_cmdseq(prefix, size), // Valve Hammer / Worldcraft Compile Command Sequences (CmdSeq.wc, 1997) Valve
        b"WDB2" // Blizzard World of Warcraft Client Database (.dbc / .db2, 2004) Blizzard Entertainment
        | b"WDB5" // Blizzard World of Warcraft Client Database (.dbc / .db2, 2004) Blizzard Entertainment
        | b"WDB6" // Blizzard World of Warcraft Client Database (.dbc / .db2, 2004) Blizzard Entertainment
        | b"WDBC" // Blizzard World of Warcraft Client Database (.dbc / .db2, 2004) Blizzard Entertainment
        | b"WDC1" // Blizzard World of Warcraft Client Database (.dbc / .db2, 2004) Blizzard Entertainment
        | b"WDC2" // Blizzard World of Warcraft Client Database (.dbc / .db2, 2004) Blizzard Entertainment
        | b"WDC3" // Blizzard World of Warcraft Client Database (.dbc / .db2, 2004) Blizzard Entertainment
        | b"WDC4" // Blizzard World of Warcraft Client Database (.dbc / .db2, 2004) Blizzard Entertainment
        | b"WDC5" => verify_wow_dbc(prefix, size), // Blizzard World of Warcraft Client Database (.dbc / .db2, 2004) Blizzard Entertainment
        b"MD20" // Blizzard World of Warcraft M2 3D Model (.m2, 2004) Blizzard Entertainment
        | b"MD21" => verify_wow_m2(prefix, size), // Blizzard World of Warcraft M2 3D Model (.m2, 2004) Blizzard Entertainment
        b"WOZ1" // Applesauce WOZ floppy bitstream image
        | b"WOZ2" => verify_woz(prefix, size), // Applesauce WOZ floppy bitstream image
        &[0xB7, 0xD8, 0x00, 0x20] => verify_wtv(prefix, size), // Windows Media Center Recorded TV Show (.wtv, 2008) Microsoft
        b"AKPK" // Audiokinetic Wwise SoundBank / File Package (.bnk / .pck)
        | b"BKHD" => verify_wwise_bnk(prefix, size), // Audiokinetic Wwise SoundBank / File Package (.bnk / .pck)
        b"\x01fcp" => verify_x11_pcf(prefix, size), // X11 / Linux Portable Compiled Font (.pcf / .pmf, 1991) Keith Packard / MIT X Consortium ("\x01fcp", PRONOM fmt/1720)
        b"TRAC" => verify_x64dbg_trace(prefix, size), // x64dbg Binary Execution Trace (.trace32 / .trace64)
        b"FSGX" // Microsoft XACT Global Settings (.xgs)
        | b"XGSF" => verify_xact_global_settings(prefix, size), // Microsoft XACT Global Settings (.xgs)
        b"KBDS" // Microsoft XACT Sound Bank (.xsb)
        | b"SDBK" => verify_xact_sound_bank(prefix, size), // Microsoft XACT Sound Bank (.xsb)
        b"DNBW" // Microsoft DirectX XACT Wave Bank (.xwb)
        | b"WBND" => verify_xact_wave_bank(prefix, size), // Microsoft DirectX XACT Wave Bank (.xwb)
        b"XALZ" => verify_xamarin_xalz(prefix, size), // Xamarin.Android / .NET for Android LZ4-Compressed Assembly (XALZ)
        b"XBIN" => verify_xbin(prefix, size), // XBIN Extended Binary Text-Mode Image (.xb, 1996) Tasmaniac / ACiD Productions
        b"BDES" => verify_xbox_bdes_xzp(prefix, size), // BDES Encrypted Package (.xzp)
        b"SDNS" => verify_xbox_sdns_audio(prefix, size), // Xbox SDNS ADPCM Audio Stream (.sdns)
        b"XBEH" => verify_xbox_xbe(prefix, size), // Microsoft Original Xbox Executable (.xbe, 2001) Microsoft
        b"xcr " // XCR Archive (.xcr)
        | b"xcr\xBC" => verify_xcr_archive(prefix, size), // XCR Archive (.xcr)
        b"XFSB" => verify_xfs(prefix, size), // Silicon Graphics XFS High-Performance Filesystem
        b"HDR1" // Xiaomi MiWiFi HDR1 / HDR2 Router Firmware Image (.bin)
        | b"HDR2" => verify_xiaomi_hdr(prefix, size), // Xiaomi MiWiFi HDR1 / HDR2 Router Firmware Image (.bin)
        b"DERF" => verify_xilam_derf_audio(prefix, size), // Xilam / Stupid Invaders DERF DPCM Audio (.adp)
        &[0x00, 0x09, 0x0F, 0xF0] => verify_xilinx_bitstream(prefix, size), // Xilinx FPGA Configuration Bitstream (.bit, 1990) Xilinx
        b"xclb" => verify_xilinx_xclbin(prefix, size), // AMD / Xilinx XRT FPGA Accelerator Binary Container (.xclbin, xclbin2\0)
        b"XNBa" // Microsoft XNA Game Studio / MonoGame Compiled Binary Asset (.xnb)
        | b"XNBi" // Microsoft XNA Game Studio / MonoGame Compiled Binary Asset (.xnb)
        | b"XNBm" // Microsoft XNA Game Studio / MonoGame Compiled Binary Asset (.xnb)
        | b"XNBw" // Microsoft XNA Game Studio / MonoGame Compiled Binary Asset (.xnb)
        | b"XNBx" => verify_xna_xnb(prefix, size), // Microsoft XNA Game Studio / MonoGame Compiled Binary Asset (.xnb)
        b"XPKF" => verify_xpk(prefix, size), // Amiga eXternal Package Compressed File (XPK, 1991) Urban Dominik Mueller & Bryan Ford
        &[0x61, 0x00, 0x00, 0x02] => verify_xplane_acf(prefix, size), // Laminar Research X-Plane Binary Aircraft Model (.acf, v7)
        b"XPLN" => verify_xplane_dsf_scenery(prefix, size), // Laminar Research X-Plane Distribution Scenery Format (.dsf)
        b"XPLA" => verify_xplapp_package(prefix, size), // XPLAPP Application Package (.xplapp)
        b"DDMF" => verify_xtracker_dmf_module(prefix, size), // D-Lusion X-Tracker Delusion Digital Music File (.dmf)
        b"yGLF" => verify_yglf_file(prefix, size), // yGLF Graphics/Glyph Format (.glf)
        b"YPF\x00" => verify_yuris_ypf(prefix, size), // YU-RIS Visual Novel Engine YPF Archive (.ypf, 2005) Firstia / YU-RIS
        b"yz01" // DeepFreezer YZ1 Archive (1999) Yamazaki ("yz1\x01" / "yz010600" / "yz020...")
        | b"yz02" // DeepFreezer YZ1 Archive (1999) Yamazaki ("yz1\x01" / "yz010600" / "yz020...")
        | b"yz1\x01" => verify_yz1_archive(prefix, size), // DeepFreezer YZ1 Archive (1999) Yamazaki ("yz1\x01" / "yz010600" / "yz020...")
        b"ZBR1" => verify_zbr1bgjb_prn(prefix, size), // Zebra ZBR1BGJB XML-Headered Card Print Job (.prn)
        b"\x00ZCK" => verify_zchunk_container(prefix, size), // Fedora / RPM zchunk Delta-Compressed File (.zck, \0ZCK1)
        b"TPD\x03" => verify_zebra_tpd(prefix, size), // Zebra Technologies Printer Driver Profile (.tpd) Zebra Technologies
        b"Libr" => verify_zedal_library_lv(prefix, size), // ZEDAL Library Timestamped Binary Export (.lv)
        &[0x5D, 0x76, 0x00, 0x00] => verify_zeroplus_als(prefix, size), // Zeroplus Logic Analyzer (LAP-C) Project (.als)
        &[0x1E, 0xF1, 0xEE, 0x0B] => verify_zigbee_ota(prefix, size), // Zigbee Alliance Cluster Library Over-the-Air Upgrade Image (.ota / .zigbee) Connectivity Standards Alliance
        b"ZIM\x04" => verify_zim_archive(prefix, size), // openZIM Offline Wiki Content Archive (.zim, ZIM\\x04)
        b"7\xE4S\x96" => verify_zisofs_compressed(prefix, size), // H. Peter Anvin zisofs Compressed File (.zf)
        &[0x5C, 0x00, 0x00, 0x00] => verify_zoner_zmf(prefix, size), // Zoner Callisto / Zoner Draw Vector Metafile (.zmf, 1990s) Zoner Software (92-byte header + 0xD4015ADD + 0x12345678)
        b"ZOO " => verify_zoo(prefix, size), // ZOO LZW/LH5 Archive (.zoo, 1986) Rahul Dhesi ("ZOO x.xx Archive.\x1A")
        b"OL\x81\x99" // IBM z/OS USS EBCDIC Archiver Library (!<arch>)
        | b"ZL\x81\x99" => verify_zos_ebcdic_ar(prefix, size), // IBM z/OS USS EBCDIC Archiver Library (!<arch>)
        b"zp1\x00" => verify_zp1_archive(prefix, size), // zp1 Compressed Data Package (.zp)
        b"7kSt" // ZPAQ journaling archive (.zpaq)
        | b"zPQ\x01" // ZPAQ journaling archive (.zpaq)
        | b"zPQ\x02" // ZPAQ journaling archive (.zpaq)
        | b"zPaQ" => verify_zpaq(prefix, size), // ZPAQ journaling archive (.zpaq)
        b"Zuc\x00" // Google Chrome Zucchini Executable Differential Patch (Zuc\\0)
        | b"Zucc" => verify_zucchini_patch(prefix, size), // Google Chrome Zucchini Executable Differential Patch (Zuc\\0)
        b"CHIP" => verify_zx_chip_tracker(prefix, size), // ZX Spectrum Exolon Chip Tracker Module (.chi)
        b"Modu" => verify_zx_fast_tracker(prefix, size), // ZX Spectrum Digital Reality Fast Tracker Module (.ftc)
        b"FXSM" => verify_zx_fuxoft_fxm(prefix, size), // ZX Spectrum Fuxoft AY Music Module (.fxm)
        b"ZXTa" => verify_zx_tzx(prefix, size), // ZX Spectrum / Amstrad CPC Digital Tape Image (.tzx / .cdt, 1997) Tomaz Kac & Martijn van der Heide ("ZXTape!\x1A")
        &[0x83, 0x80, 0x00, 0x00] => verify_zyxel_bix(prefix, size), // ZyXEL BIX Router Firmware Image (.bix)
        _ => None,
    }
}
