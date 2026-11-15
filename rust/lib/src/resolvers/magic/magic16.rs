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

use super::*;
use crate::ContentType as Label;

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

        // --- Additional Standalone Binary Formats ---
        b"JG" => verify_aol_art_image(prefix, size), // America Online Johnson-Grace ART Compressed Image (.art, 1995) Johnson-Grace / AOL (PRONOM fmt/666)
        &[0x0A, 0x47] => verify_apple_binary_ii(prefix, size), // Apple II Binary II Archive (.bny / .bqy)
        &[0x3D, 0x02] => verify_atari_cad3d2(prefix, size), // Antic Software Atari ST Cyber Studio CAD-3D 2.0 3D Object (.3d2, 1987) Tom Hudson / Antic Publishing
        &[0x0E, 0x0F] => verify_atari_msa(prefix, size), // Atari ST Magic Shadow Archiver Compressed Floppy Image (.msa, 1990) (0x0E0F BE = 0E 0F)
        b"sB" => verify_basis_universal(prefix, size), // Binomial Basis Universal Supercompressed GPU Texture (.basis, 2019) Binomial (0x4273 LE)
        &[0x01, 0x1E] // Berkeley Unix / SunOS vfont Bitmap Font (.vfont, 1979) UC Berkeley CSRG / Sun Microsystems
        | &[0x1E, 0x01] => verify_berkeley_vfont(prefix, size), // Berkeley Unix / SunOS vfont Bitmap Font (.vfont, 1979) UC Berkeley CSRG / Sun Microsystems
        &[0x4C, 0xFC] => verify_broadcom_hcd(prefix, size), // Broadcom / Cypress Bluetooth HCI RAM Patch Download Image (.hcd, 2005) Broadcom (0xFC4C LE Write_RAM opcode)
        &[0xE0, 0x49] => verify_clarion_hlp(prefix, size), // Barrington Systems Clarion for DOS Help File (.hlp)
        b"M3" => verify_clarion_mem(prefix, size), // Clarion Memory Variable File (.mem)
        &[0x06, 0x00] // Nantucket / CA-Clipper NTX Database Index (.ntx, 1987) Nantucket / Computer Associates
        | &[0x06, 0x01] // Nantucket / CA-Clipper NTX Database Index (.ntx, 1987) Nantucket / Computer Associates
        | &[0x26, 0x00] => verify_clipper_ntx(prefix, size), // Nantucket / CA-Clipper NTX Database Index (.ntx, 1987) Nantucket / Computer Associates
        &[0x1F, 0x4D] => verify_commodore_nibtools(prefix, size), // Commodore 1541 Nibtools Raw/Compressed GCR Disk Image (.nib / .nbz)
        b"gf" => verify_composer_667(prefix, size), // Future Crew Composer 667 OPL2 Module (.667)
        &[0x71, 0xC7] // CPIO archive
        | &[0xC7, 0x71] => verify_cpio_binary(prefix, size), // CPIO archive
        b"GW" => verify_ctos_run(prefix, size), // Convergent CTOS / Burroughs BTOS Executable (.run / .sys, 1981) Convergent Technologies
        &[0x01, 0x01] // CUPS Internet Printing Protocol Control Job (c#####)
        | &[0x02, 0x00] // CUPS Internet Printing Protocol Job (IPP 2.0)
        | &[0x02, 0x01] => verify_cups_ipp(prefix, size), // CUPS Internet Printing Protocol Control Job (c#####)
        &[0x33, 0x9B] => verify_daisydot_nlq3_font(prefix, size), // Atari 8-Bit Daisy-Dot III NLQ Font (.nlq, 1992) Roy E. Myers ("3\x9B" + variable-stride glyph chain)
        b"DI" => verify_dbg(prefix, size), // Windows Separate Debug file (.dbg)
        &[0x2B, 0xD1] // IBM DisplayWrite DCA RFT/FFT (.dca, PRONOM x-fmt/148)
        | &[0x2B, 0xD2] => verify_displaywrite_dca_fft(prefix, size), // IBM DisplayWrite DCA RFT/FFT (.dca, PRONOM x-fmt/148)
        b"BW" => verify_dos16m_bw(prefix, size), // Rational Systems / Tenberry DOS/16M Protected-Mode BW Executable (.exp / .exe)
        &[0x01, 0x22] // Milanerny Electronic Music System v1 Amiga Module (.ems)
        | &[0x01, 0x25] => verify_ems_v1(prefix, size), // Milanerny Electronic Music System v1 Amiga Module (.ems)
        &[0xAA, 0x50] => verify_espressif_partition_table(prefix, size), // Espressif ESP-IDF Flash Partition Table Binary
        &[0xFF, 0xA0] => verify_fbi_wsq(prefix, size), // FBI Wavelet Scalar Quantization Grayscale Fingerprint Image (.wsq)
        &[0xFF, 0xFF] => verify_gem_vdi_metafile(prefix, size), // Digital Research GEM VDI Vector Metafile (.gem, 1985) Digital Research (0xFFFF header sentinel, PRONOM fmt/542)
        b"HA" => verify_ha_archive(prefix, size), // Harri Hirvola HA Compressed Archive (.ha)
        &[0x60, 0x02] // Jochen Hippel Amiga 68000 / 7-Voice Module (.hip / .hip7)
        | &[0x60, 0x04] // Jochen Hippel Amiga 68000 / 7-Voice Module (.hip / .hip7)
        | &[0x60, 0x06] // Jochen Hippel Amiga 68000 / 7-Voice Module (.hip / .hip7)
        | &[0x60, 0x08] // Jochen Hippel Amiga 68000 / 7-Voice Module (.hip / .hip7)
        | &[0x60, 0x0A] // Jochen Hippel Amiga 68000 / 7-Voice Module (.hip / .hip7)
        | &[0x60, 0x0C] // Jochen Hippel Amiga 68000 / 7-Voice Module (.hip / .hip7)
        | &[0x60, 0x0E] // Jochen Hippel Amiga 68000 / 7-Voice Module (.hip / .hip7)
        | &[0x60, 0x10] // Jochen Hippel Amiga 68000 / 7-Voice Module (.hip / .hip7)
        | &[0x60, 0x12] // Jochen Hippel Amiga 68000 / 7-Voice Module (.hip / .hip7)
        | &[0x60, 0x14] // Jochen Hippel Amiga 68000 / 7-Voice Module (.hip / .hip7)
        | &[0x60, 0x16] // Jochen Hippel Amiga 68000 / 7-Voice Module (.hip / .hip7)
        | &[0x60, 0x18] // Jochen Hippel Amiga 68000 / 7-Voice Module (.hip / .hip7)
        | &[0x60, 0x1C] // Jochen Hippel Amiga 68000 / 7-Voice Module (.hip / .hip7)
        | &[0x60, 0x1E] // Jochen Hippel Amiga 68000 / 7-Voice Module (.hip / .hip7)
        | b"` " // Jochen Hippel Amiga 68000 / 7-Voice Module (.hip / .hip7)
        | b"`\"" // Jochen Hippel Amiga 68000 / 7-Voice Module (.hip / .hip7)
        | b"`$" // Jochen Hippel Amiga 68000 / 7-Voice Module (.hip / .hip7)
        | b"`&" // Jochen Hippel Amiga 68000 / 7-Voice Module (.hip / .hip7)
        | b"`(" // Jochen Hippel Amiga 68000 / 7-Voice Module (.hip / .hip7)
        | b"`*" // Jochen Hippel Amiga 68000 / 7-Voice Module (.hip / .hip7)
        | b"`," // Jochen Hippel Amiga 68000 / 7-Voice Module (.hip / .hip7)
        | b"`." => verify_hippel_7v_module(prefix, size), // Jochen Hippel Amiga 68000 / 7-Voice Module (.hip / .hip7)
        b"DL" => verify_hp_palmtop_exm(prefix, size), // HP 95LX / 100LX / 200LX System Manager Executable (.exm)
        &[0x1B, 0x29] => verify_hp_pcl_softfont(prefix, size), // HP PCL / PaintJet Downloadable Soft Font (.sfp / .sfl / .pjf, 1985) Hewlett-Packard
        &[0xAA, 0x58] // IBM OS/2 SaveDskF / LoadDskF Floppy Disk Image (.dsk / .1dk / .2dk)
        | &[0xAA, 0x59] // IBM OS/2 SaveDskF / LoadDskF Floppy Disk Image (.dsk / .1dk / .2dk)
        | &[0xAA, 0x5A] => verify_ibm_savedskf(prefix, size), // IBM OS/2 SaveDskF / LoadDskF Floppy Disk Image (.dsk / .1dk / .2dk)
        &[0x4A, 0xA3] => verify_installshield_pkg(prefix, size), // InstallShield Package Directory (setup.pkg)
        b"+|" => verify_jason_page_old(prefix, size), // Jason Page Old Amiga Music Module (.jpo)
        &[0x19, 0x84] // Red Hat JFFS2 Flash Filesystem Image (.jffs2)
        | &[0x19, 0x85] // Red Hat JFFS2 Flash Filesystem Image (.jffs2)
        | &[0x84, 0x19] // Red Hat JFFS2 Flash Filesystem Image (.jffs2)
        | &[0x85, 0x19] => verify_jffs2(prefix, size), // Red Hat JFFS2 Flash Filesystem Image (.jffs2)
        &[0x50, 0x00] => verify_joda_timezone_info(prefix, size), // Joda-Time Binary ZoneInfo Map (ZoneInfoMap)
        &[0x76, 0x81] // Kerberos v5 KRB-CRED Credential Ticket (.kirbi)
        | &[0x76, 0x82] // Kerberos v5 KRB-CRED Credential Ticket (.kirbi)
        | &[0x76, 0x84] => verify_kerberos_kirbi(prefix, size), // Kerberos v5 KRB-CRED Credential Ticket (.kirbi)
        b"KF" => verify_kompas_3d(prefix, size), // ASCON Kompas-3D CAD Drawing / Fragment (.cdw / .frw, 1999) ASCON ("KF\x5C\x00" / "KF\x1A\x00")
        &[0x0D, 0x01] // Software Preservation Society KryoFlux Raw Flux Stream (.raw)
        | &[0x0D, 0x02] // Software Preservation Society KryoFlux Raw Flux Stream (.raw)
        | &[0x0D, 0x03] // Software Preservation Society KryoFlux Raw Flux Stream (.raw)
        | &[0x0D, 0x04] => verify_kryoflux_stream(prefix, size), // Software Preservation Society KryoFlux Raw Flux Stream (.raw)
        &[0x36, 0x04] => verify_linux_psf1(prefix, size), // Linux Console PC Screen Font v1 (PSF1, 1994) H. Peter Anvin (PSF1_MAGIC = 0x0436 LE)
        &[0xDA, 0xFA] => verify_logitech_compress(prefix, size), // Logitech LGEXPAND Compressed Installer File (.ex_ / .dl_ / .cu_)
        &[0xA9, 0x09] => verify_lotus_123_install_dat(prefix, size), // Lotus 1-2-3 Release 3 Installation Driver Database (LOTUS.DAT, 1989) Lotus
        b"ZV" => verify_lzf(prefix, size), // LibLZF framed compressed stream ("ZV\x00" uncompressed / "ZV\x01" compressed, 2000) Marc Alexander Lehmann
        &[0x7B, 0x9E] => verify_lzsa_compressed(prefix, size), // Emmanuel Marty LZSA Compressed Stream (.lzsa)
        &[0x00, 0xFE] // Microsoft Works for MS-DOS Word Processor Document (.wps)
        | &[0x01, 0xFE] => verify_ms_works_dos_wps(prefix, size), // Microsoft Works for MS-DOS Word Processor Document (.wps)
        b"BY" // Nintendo Binary YAML (.byml / .byaml / .bgyml, 2015) Nintendo
        | b"YB" => verify_nintendo_byml(prefix, size), // Nintendo Binary YAML (.byml / .byaml / .bgyml, 2015) Nintendo
        b"PK" => verify_nos_installer_archive(prefix, size), // Netopsystems FEAD Optimizer (NOS_PO) Archive
        &[0x3B, 0x9E] => verify_notator_song(prefix, size), // Notator SL / Creator MIDI sequence .son (1987) Gerhard Lengeling & Chris Adam / C-Lab
        &[0xFA, 0xE9] => verify_novell_netware_rpl_rom(prefix, size), // Novell NetWare Remote Program Load (RPL) Boot ROM (.rom / .rpl, 1989) Novell
        b"S." => verify_palladix_pcs(prefix, size), // Palladix Amiga Audio Module (.pcs)
        &[0xEA, 0x01] => verify_pea_archive(prefix, size), // PeaZip PEA v1.x binary archive header (2006) Giorgio Tani
        &[0x00, 0x03] // Three Rivers PERQ POS Executable Segment (.seg)
        | &[0x01, 0x03] // Three Rivers PERQ POS / CMU Accent Q-Code Segment (.seg, 1980) Three Rivers Computer
        | &[0x02, 0x03] // Three Rivers PERQ POS / CMU Accent Q-Code Segment (.seg, 1980) Three Rivers Computer
        | &[0x03, 0x03] // Three Rivers PERQ POS / CMU Accent Q-Code Segment (.seg, 1980) Three Rivers Computer
        | &[0x16, 0x04] // Three Rivers PERQ POS / CMU Accent Q-Code Segment (.seg, 1980) Three Rivers Computer
        | &[0x17, 0x04] => verify_perq_pos_seg(prefix, size), // Three Rivers PERQ POS / CMU Accent Q-Code Segment (.seg, 1980) Three Rivers Computer
        b"MC" => verify_ps1_memory_card(prefix, size), // Sony PlayStation 1 Memory Card Image (.mcr / .mcd, 1994) Sony
        b"SC" => verify_scp(prefix, size), // SuperCard Pro Magnetic Flux Preservation Image (.scp, 2013) Jim Drew / CBMSTUFF ("SCP")
        &[0xF3, 0xED] => verify_sega_sms_rom_bootstrap(prefix, size), // Sega Master System / Game Gear Z80 Bootstrap ROM (.sms / .gg, 1985) Sega (`F3 ED 56` = `DI; IM 1`)
        &[0x01, 0xDA] => verify_sgi_rgb(prefix, size), // Silicon Graphics IRIS RGB Image (.sgi / .rgb / .bw / .rgba, 1988) Paul Haeberli / Silicon Graphics (IMAGIC = 0x01DA BE)
        &[0x89, 0x00] => verify_sierra_sci_patch(prefix, size), // Sierra On-Line SCI0 AdLib Patch Bank (.003)
        b"SN" => verify_sound_club_module(prefix, size), // Bluemoon Sound Club Module (.sn / .sn2)
        &[0xFA, 0xFA] => verify_squirrel_cnut(prefix, size), // Squirrel VM Compiled Closure Bytecode (.cnut / .nut)
        &[0xFB, 0x52] => verify_tds(prefix, size), // Borland Turbo Debugger Symbols (.tds)
        b"TD" // Sydex TeleDisk floppy image (.td0)
        | b"td" => verify_teledisk(prefix, size), // Sydex TeleDisk floppy image (.td0)
        b"VZ" => verify_uefi_te(prefix, size), // UEFI Terse Executable (TE) firmware image
        &[0xC4, 0x30] => verify_unreal_usmap(prefix, size), // Unreal Engine 4/5 Unversioned Property Schema Mapping (.usmap, 2020) Epic Games / CUE4Parse
        &[0xDF, 0x05] // Voyetra PatchMaster / Sideman MIDI Synthesizer Bank (.b*, .c*, 1987) Voyetra Technologies
        | &[0xE0, 0x05] => verify_voyetra_patch_bank(prefix, size), // Voyetra PatchMaster / Sideman MIDI Synthesizer Bank (.b*, .c*, 1987) Voyetra Technologies
        &[0x0D, 0x00] => verify_windows_codepage_nls(prefix, size), // Microsoft Windows National Language Support Codepage Table (C_*.NLS, 1993) Microsoft
        b"ay" // ZX Spectrum Vortex Project AY/YM Chiptune (.vtx)
        | b"ym" => verify_zx_vortex_vtx(prefix, size), // ZX Spectrum Vortex Project AY/YM Chiptune (.vtx)
        _ => None,
    }
}
