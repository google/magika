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

        _ => None,
    }
}
