/*
$File: COPYING,v 1.2 2018/09/09 20:33:28 christos Exp $
Copyright (c) Ian F. Darwin 1986, 1987, 1989, 1990, 1991, 1992, 1994, 1995.
Software written by Ian F. Darwin and others;
maintained 1994- Christos Zoulas.

This software is not subject to any export provision of the United States
Department of Commerce, and may be exported to any country or planet.

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions
are met:
1. Redistributions of source code must retain the above copyright
   notice immediately at the beginning of the file, without modification,
   this list of conditions, and the following disclaimer.
2. Redistributions in binary form must reproduce the above copyright
   notice, this list of conditions and the following disclaimer in the
   documentation and/or other materials provided with the distribution.

THIS SOFTWARE IS PROVIDED BY THE AUTHOR AND CONTRIBUTORS ``AS IS'' AND
ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
ARE DISCLAIMED. IN NO EVENT SHALL THE AUTHOR OR CONTRIBUTORS BE LIABLE FOR
ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS
OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION)
HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT
LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY
OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF
SUCH DAMAGE.
*/

// Format predicates adapted from the pinned sources listed in rules/LICENSES.
// Rules are grouped by observed development evidence; defaults remain off.
// Evidence corpus: parquet-v56, 29,523 whole files; independent qualification remains pending.

rule taxonomy_3dsm
{
	meta:
        source_refs = "pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:1447; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:319"
		label = "3dsm"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    // 3DS root chunk and child discriminator; reject impossible small main-chunk lengths and TIFF collisions.
    strings:
        $root = { 4D 4D }
        $version = { 02 00 0A 00 00 00 }
        $editor = { 3D 3D }

    condition:
        prefix_size >= 16 and $root at 0 and uint32(2) >= 16
        and (($version at 6 and uint32(12) >= 3 and uint32(12) <= 4)
             or ($editor at 6 and uint32(8) >= 6))
}

rule taxonomy_3dsx
{
	meta:
        source_refs = "libmagic:magic/Magdir/console:libmagic_2e701145f75e467f36f5_line_1183"
		label = "3dsx"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    strings:
        $base = { 33 44 53 58 20 00 08 00 00 00 00 00 00 00 00 00 }
        $extended = { 33 44 53 58 2C 00 08 00 00 00 00 00 00 00 00 00 }
    condition:
        (($base at 0 and prefix_size >= 56) or ($extended at 0 and prefix_size >= 68)) and
        uint32(16) >= 4 and uint32(16) % 4 == 0 and uint32(20) % 4 == 0 and
        uint32(24) % 4 == 0 and uint32(28) % 4 == 0
}

rule taxonomy_3gp
{
	meta:
        source_refs = "libmagic:magic/Magdir/animation:libmagic_551bb9d95fd0095e0a3a_line_56; libmagic:magic/Magdir/animation:libmagic_551bb9d95fd0095e0a3a_line_63; libmagic:magic/Magdir/animation:libmagic_551bb9d95fd0095e0a3a_line_67; libmagic:magic/Magdir/animation:libmagic_551bb9d95fd0095e0a3a_line_70; libmagic:magic/Magdir/animation:libmagic_551bb9d95fd0095e0a3a_line_73; libmagic:magic/Magdir/animation:libmagic_551bb9d95fd0095e0a3a_line_82; libmagic:magic/Magdir/animation:libmagic_551bb9d95fd0095e0a3a_line_86; libmagic:magic/Magdir/animation:libmagic_551bb9d95fd0095e0a3a_line_91; puremagic:puremagic/magic_data.json:headers[838]; puremagic:puremagic/magic_data.json:headers[839]; puremagic:puremagic/magic_data.json:headers[840]; puremagic:puremagic/magic_data.json:headers[841]; puremagic:puremagic/magic_data.json:headers[964]; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[1605]/magic[0]"
		label = "3gp"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    // Ordinary 32-bit ftyp: complete major/minor version fields and aligned brand table.
    // Payload completeness is not format identity; the brand table may exceed the scan prefix.
    strings:
        $header = { 66 74 79 70 33 67 (65 (36 | 37 | 39) | 67 (36 | 39) | 68 39 | 6D (39 | 41) | 70 (31 | 32 | 33 | 34 | 35 | 36 | 37 | 38 | 39) | 72 (36 | 39) | 73 (36 | 37 | 39) | 74 (38 | 39 | 76)) }

    condition:
        prefix_size >= 16 and $header at 4 and
        uint32be(0) >= 16 and uint32be(0) % 4 == 0
}

rule taxonomy_ace
{
	meta:
        source_refs = "libmagic:magic/Magdir/archive:libmagic_245541f7bb881bcd2ec1_line_2407; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:3438; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:3439"
		label = "ace"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

	strings:
		$p0_0 = "**ACE**"
		$p1_0 = /(\x2a){2}\x41\x43\x45(\x2a){2}(\x14){2}/
		$p2_0 = /(\x2a){2}\x41\x43\x45(\x2a){2}[\x0a-\x0d](\x0a|\x0b|\x0c|\x0d|\x14)/

	condition:
		prefix_size >= 8 and (((prefix_size >= 14 and original_size >= 14 and $p0_0 at 7) or ($p1_0 at 7) or ($p2_0 at 7)))
}

rule taxonomy_applebplist
{
	meta:
        source_refs = "libmagic:magic/Magdir/apple:libmagic_cdfe968d7ca88b25ceba_line_439; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[894]/magic[0]; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[894]/magic[1]"
		label = "applebplist"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    // Classic readers accept bplist0?; other known serializations have distinct version bytes.
    strings:
        $classic = /bplist0[\x00-\xff]/
        $modern = /bplist1[0-9]/
        $binary_version = { 62 70 6C 69 73 74 (00 (00 | 01) | 40 00) }
    condition:
        prefix_size >= 8 and
        ((prefix_size >= 41 and $classic at 0) or $modern at 0 or $binary_version at 0)
}

rule taxonomy_appledouble
{
	meta:
        source_refs = "tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[1465]/magic[0]"
		label = "appledouble"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    // Complete fixed AppleDouble header, retaining version 1 and version 2.
    strings:
        $header = { 00 05 16 07 00 (01 | 02) 00 00 }
    condition:
        prefix_size >= 26 and $header at 0
}

rule taxonomy_applesingle
{
	meta:
        source_refs = "tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[2]/magic[0]"
		label = "applesingle"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    // Complete fixed AppleSingle header; filler bytes vary between historical writers.
    strings:
        $header = { 00 05 16 00 00 (01 | 02) 00 00 }
    condition:
        prefix_size >= 26 and $header at 0
}

rule taxonomy_au
{
	meta:
        source_refs = "libmagic:magic/Magdir/audio:libmagic_4fe29a65cf9987105680_line_12; libmagic:magic/Magdir/audio:libmagic_4fe29a65cf9987105680_line_14; libmagic:magic/Magdir/audio:libmagic_4fe29a65cf9987105680_line_16; libmagic:magic/Magdir/audio:libmagic_4fe29a65cf9987105680_line_18; libmagic:magic/Magdir/audio:libmagic_4fe29a65cf9987105680_line_20; libmagic:magic/Magdir/audio:libmagic_4fe29a65cf9987105680_line_22; libmagic:magic/Magdir/audio:libmagic_4fe29a65cf9987105680_line_24; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[1169]/magic[0]"
		label = "au"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    strings:
        $big = ".snd"
        $little = "dns."
    condition:
        prefix_size >= 24 and
        (
            ($big at 0 and uint32be(4) >= 24 and uint32be(16) > 0 and uint32be(20) > 0 and
             ((uint32be(12) >= 1 and uint32be(12) <= 14) or (uint32be(12) >= 16 and uint32be(12) <= 27))) or
            ($little at 0 and uint32(4) >= 24 and uint32(16) > 0 and uint32(20) > 0 and
             ((uint32(12) >= 1 and uint32(12) <= 14) or (uint32(12) >= 16 and uint32(12) <= 27)))
        )
}

rule taxonomy_avi
{
	meta:
        source_refs = "libmagic:magic/Magdir/riff:libmagic_b2aac9f8242bb4da4291_line_420; puremagic:puremagic/magic_data.json:headers[129]; puremagic:puremagic/magic_data.json:headers[148]; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[1683]/magic[0]"
		label = "avi"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    // RIFF AVI with LIST/hdrl; do not treat an isolated form name or AVF0 as AVI.
    strings:
        $header = { 52 49 46 46 ?? ?? ?? ?? 41 56 49 20 4C 49 53 54 }
        $hdrl = "hdrl"

    condition:
        prefix_size >= 24 and $header at 0 and $hdrl at 20 and uint32(4) >= 16 and uint32(16) >= 4
}

rule taxonomy_avif
{
	meta:
        source_refs = "libmagic:magic/Magdir/animation:libmagic_551bb9d95fd0095e0a3a_line_296; libmagic:magic/Magdir/animation:libmagic_551bb9d95fd0095e0a3a_line_298; puremagic:puremagic/magic_data.json:headers[1267]; puremagic:puremagic/magic_data.json:headers[1268]; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[1321]/magic[0]"
		label = "avif"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    // Ordinary 32-bit ftyp: complete major/minor version fields and aligned brand table.
    // Payload completeness is not format identity; the brand table may exceed the scan prefix.
    strings:
        $header = { 66 74 79 70 61 76 69 (66 | 73) }

    condition:
        prefix_size >= 16 and $header at 4 and
        uint32be(0) >= 16 and uint32be(0) % 4 == 0
}

rule taxonomy_avro
{
	meta:
        source_refs = "libmagic:magic/Magdir/apache:libmagic_dcfba1374d0a8a3f71c0_line_7; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:3392"
		label = "avro"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

	strings:
		$p0_0 = { 4f 62 6a 01 }
		$p1_0 = /(\x4f\x62\x6a\x01([\x00-\xff]){2})(\x61\x76\x72\x6f\x2e)(\x63\x6f\x64\x65\x63([\x00-\xff]){8,50}|\x73\x79\x6e\x63([\x00-\xff]){8,50})\x73\x63\x68\x65\x6d\x61(([\x00-\xff]){3}\x22\x74\x79\x70\x65\x22)(([\x00-\xff]){2,65}\x22\x6e\x61\x6d\x65\x22)/

	condition:
		prefix_size >= 8 and (((prefix_size >= 4 and $p0_0 at 0) or ($p1_0 at 0)))
}

rule taxonomy_bam
{
	meta:
        source_refs = "libmagic:magic/Magdir/bioinformatics:libmagic_5294848eb43d5add223c_line_43"
		label = "bam"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

	strings:
		$p0_0 = { 42 41 4d 01 }

	condition:
		prefix_size >= 8 and ((prefix_size >= 4 and $p0_0 at 0))
}

rule taxonomy_beam
{
	meta:
        source_refs = "libmagic:magic/Magdir/erlang:libmagic_035fbf116164db7a554b_line_13; libmagic:magic/Magdir/erlang:libmagic_6d109225d05319b6924c_line_8"
		label = "beam"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

	strings:
		$p0_0 = { 0f 37 42 45 41 4d 21 }
		$p1_0 = { 42 45 41 4d }
		$p2_0 = { 46 4f 52 31 }

	condition:
		prefix_size >= 8 and (((prefix_size >= 7 and $p0_0 at 0) or ((prefix_size >= 12 and $p1_0 at 8) and (prefix_size >= 4 and $p2_0 at 0))))
}

rule taxonomy_blend
{
	meta:
        source_refs = "libmagic:magic/Magdir/blender:libmagic_0e9d62e378368c6517ab_line_15; puremagic:puremagic/magic_data.json:headers[41]"
		label = "blend"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    strings:
        // libmagic's pre-v5 header fields: pointer width, byte order and three version digits.
        // Blender 5's distinct extended header remains outside this reviewed variant.
        $header = /BLENDER[_-][vV][1-4][0-9]{2}/

    condition:
        prefix_size >= 32 and $header at 0
}

rule taxonomy_bpg
{
	meta:
        source_refs = "libmagic:magic/Magdir/images:libmagic_b0d4f7aee4f73b58f192_line_2889"
		label = "bpg"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

	strings:
		$p0_0 = { 42 50 47 FB }

	condition:
		prefix_size >= 8 and ((prefix_size >= 4 and original_size >= 4 and $p0_0 at 0))
}

rule taxonomy_bzip
{
	meta:
        source_refs = "libmagic:magic/Magdir/compress:libmagic_d69c686db61d7a09058e_line_155; puremagic:puremagic/magic_data.json:headers[18]; puremagic:puremagic/magic_data.json:headers[55]; puremagic:puremagic/magic_data.json:headers[594]; puremagic:puremagic/magic_data.json:headers[595]; puremagic:puremagic/magic_data.json:headers[81]; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[902]/magic[0]"
		label = "bzip"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    strings:
        // Tika's BZh[1-9] plus bzip2 block/end markers; libmagic's bare BZh is too weak.
        $header = /BZh[1-9]/
        $block = { 31 41 59 26 53 59 }
        $empty = { 17 72 45 38 50 90 00 00 00 00 }

    condition:
        prefix_size >= 14 and $header at 0 and
        (($empty at 4) or (prefix_size >= 20 and $block at 4))
}

rule taxonomy_cram
{
	meta:
        source_refs = "libmagic:magic/Magdir/bioinformatics:libmagic_be057333cb455a73d25c_line_61"
		label = "cram"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    // CRAM file definition: complete 26-byte header and specified version pairs.
    strings:
        $header = { 43 52 41 4D (01 00 | 02 (00 | 01) | 03 (00 | 01)) }

    condition:
        prefix_size >= 26 and $header at 0
}

rule taxonomy_dex
{
	meta:
        source_refs = "puremagic:puremagic/magic_data.json:headers[1150]; puremagic:puremagic/magic_data.json:headers[329]"
		label = "dex"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    // DEX fixed header, numeric version, correct endian tag and standard/container header size.
    strings:
        $header = /dex\n[0-9]{3}\x00/

    condition:
        prefix_size >= 112 and $header at 0 and
        (
            (uint32be(4) != 0x30343100 and
             ((uint32(36) == 112 and uint32(40) == 0x12345678) or
              (uint32be(36) == 112 and uint32be(40) == 0x12345678))) or
            (uint32be(4) == 0x30343100 and prefix_size >= 120 and
             ((uint32(36) == 120 and uint32(40) == 0x12345678) or
              (uint32be(36) == 120 and uint32be(40) == 0x12345678)))
        )
}

rule taxonomy_dicom
{
	meta:
        source_refs = "libmagic:magic/Magdir/images:libmagic_89c2064c6c881a9a483b_line_1563; puremagic:puremagic/magic_data.json:headers[877]; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[34]/magic[0]"
		label = "dicom"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

	strings:
		$p0_0 = "DICM"

	condition:
		prefix_size >= 8 and ((prefix_size >= 132 and original_size >= 132 and $p0_0 at 128))
}

rule taxonomy_dsstore
{
	meta:
        source_refs = "libmagic:magic/Magdir/apple:libmagic_b54899d5c7c9126790c9_line_633"
		label = "dsstore"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

	strings:
		$p0_0 = { 00 00 00 01 42 75 64 31 00 }

	condition:
		prefix_size >= 8 and ((prefix_size >= 9 and $p0_0 at 0))
}

rule taxonomy_duckdb
{
	meta:
        source_refs = "libmagic:magic/Magdir/sql:libmagic_7b4a0cd374285fbd6a8d_line_383"
		label = "duckdb"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

	strings:
		$p0_0 = { 44 55 43 4b }

	condition:
		prefix_size >= 8 and ((prefix_size >= 12 and $p0_0 at 8))
}

rule taxonomy_dwg
{
	meta:
        source_refs = "pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:1771; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:244; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:654; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:83; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:834; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:84; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:85; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:86; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:87; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:88; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:89; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:90; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:91; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:92; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:93; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:94; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:95; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:96; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:97; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:98; puremagic:puremagic/magic_data.json:headers[512]"
		label = "dwg"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    // Modern AutoCAD version identifiers with five reserved zero bytes; legacy short ASCII prefixes are insufficient.
    strings:
        $header = /AC10(09|12|14|15|18|21|24|27|32)\x00{5}/

    condition:
        prefix_size >= 128 and $header at 0
}

rule taxonomy_ese
{
	meta:
        source_refs = "libmagic:magic/Magdir/database:libmagic_2b29fa68772129006dbb_line_701"
		label = "ese"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

	strings:
		$p0_0 = { ef cd ab 89 }
		$p1_0 = { 00 00 00 00 }

	condition:
		prefix_size >= 8 and (((prefix_size >= 8 and $p0_0 at 4) and (prefix_size >= 136 and $p1_0 at 132)))
}

rule taxonomy_fbx
{
	meta:
        source_refs = "libmagic:magic/Magdir/cad:libmagic_6357ac41d93e626ef3d0_line_378"
		label = "fbx"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    strings:
        // ufbx binary header and supported versions, including legacy 3000; little-endian variant.
        $magic = { 4B 61 79 64 61 72 61 20 46 42 58 20 42 69 6E 61 72 79 20 20 00 1A 00 }

    condition:
        prefix_size >= 27 and $magic at 0 and uint32(23) >= 3000 and uint32(23) <= 7700
}

rule taxonomy_fits
{
	meta:
        source_refs = "libmagic:magic/Magdir/images:libmagic_8f0868f843f44e8799b3_line_1478"
		label = "fits"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

	strings:
		$p0_0 = "SIMPLE  ="
		$p1_0 = { 20 20 }

	condition:
		prefix_size >= 8 and (((prefix_size >= 9 and original_size >= 9 and $p0_0 at 0) and (prefix_size >= 91 and $p1_0 at 89)))
}

rule taxonomy_flv
{
	meta:
        source_refs = "libmagic:magic/Magdir/flash:libmagic_50a83c0852acd8218669_line_54; puremagic:puremagic/magic_data.json:headers[147]; puremagic:puremagic/magic_data.json:multi-part[464c5601][0]; puremagic:puremagic/magic_data.json:multi-part[464c5601][1]; puremagic:puremagic/magic_data.json:multi-part[464c5601][2]; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[1673]/magic[0]"
		label = "flv"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    // FLV v1 standard header, legal audio/video flags, data offset and initial previous-tag size.
    strings:
        $header = { 46 4C 56 01 (00 | 01 | 04 | 05) 00 00 00 09 00 00 00 00 }

    condition:
        prefix_size >= 13 and $header at 0
}

rule taxonomy_gguf
{
	meta:
        source_refs = "libmagic:magic/Magdir/gguf:libmagic_31fb5df244790469dd6c_line_10; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:3462; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:3463; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:3464"
		label = "gguf"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    // GGUF v1 has 32-bit counts; v2/v3 have 64-bit counts. Preserve both byte orders.
    strings:
        $v1 = { 47 47 55 46 (01 00 00 00 | 00 00 00 01) }
        $v2_v3 = { 47 47 55 46 ((02 | 03) 00 00 00 | 00 00 00 (02 | 03)) }
    condition:
        prefix_size >= 16 and ($v1 at 0 or (prefix_size >= 24 and $v2_v3 at 0))
}

rule taxonomy_gltf
{
	meta:
        source_refs = "libmagic:magic/Magdir/cad:libmagic_5e370f253c9f71913267_line_370"
		label = "gltf"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    strings:
        // Khronos GLB 2.0: first chunk is aligned JSON, not a detached glTF string.
        $magic = { 67 6C 54 46 02 00 00 00 }
        $json = "JSON"
        $object = /[ \t\r\n]{0,64}\{/

    condition:
        prefix_size >= 24 and $magic at 0 and uint32(8) >= 24 and uint32(8) % 4 == 0 and
        uint32(12) >= 4 and uint32(12) % 4 == 0 and $json at 16 and $object at 20
}

rule taxonomy_gzip
{
	meta:
        source_refs = "puremagic:puremagic/magic_data.json:headers[1022]; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[956]/magic[0]"
		label = "gzip"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    // RFC 1952: complete member minimum, DEFLATE method and zero reserved flag bits.
    strings:
        $header = { 1F 8B 08 }

    condition:
        prefix_size >= 20 and $header at 0 and uint8(3) <= 31
}

rule taxonomy_hdf4
{
	meta:
        source_refs = "libmagic:magic/Magdir/images:libmagic_4344afb37c71c603cbd9_line_2556"
		label = "hdf4"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

	strings:
		$p0_0 = { 0E 03 13 01 }

	condition:
		prefix_size >= 8 and ((prefix_size >= 4 and $p0_0 at 0))
}

rule taxonomy_heif
{
	meta:
        source_refs = "libmagic:magic/Magdir/animation:libmagic_551bb9d95fd0095e0a3a_line_271; libmagic:magic/Magdir/animation:libmagic_551bb9d95fd0095e0a3a_line_273; puremagic:puremagic/magic_data.json:headers[1270]; puremagic:puremagic/magic_data.json:headers[1271]; puremagic:puremagic/magic_data.json:headers[1272]; puremagic:puremagic/magic_data.json:headers[1273]; puremagic:puremagic/magic_data.json:headers[1274]; puremagic:puremagic/magic_data.json:headers[1275]; puremagic:puremagic/magic_data.json:headers[1276]; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[1324]/magic[0]; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[1372]/magic[0]"
		label = "heif"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    // Ordinary 32-bit ftyp: complete major/minor version fields and aligned brand table.
    // Payload completeness is not format identity; the brand table may exceed the scan prefix.
    strings:
        $header = { 66 74 79 70 68 65 (69 (63 | 78 | 6D | 73) | 76 (63 | 78 | 6D | 73)) }

    condition:
        prefix_size >= 16 and $header at 4 and
        uint32be(0) >= 16 and uint32be(0) % 4 == 0
}

rule taxonomy_icc
{
	meta:
        source_refs = "puremagic:puremagic/magic_data.json:headers[959]; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[426]/magic[0]"
		label = "icc"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

	strings:
		$p0_0 = "acsp"

	condition:
		prefix_size >= 8 and ((prefix_size >= 40 and original_size >= 40 and $p0_0 at 36))
}

rule taxonomy_icns
{
	meta:
        source_refs = "libmagic:magic/Magdir/images:libmagic_caefde2b76d9a850f8b2_line_2894; puremagic:puremagic/magic_data.json:headers[110]"
		label = "icns"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    // ICNS container length, then a complete first element header unless the container is empty.
    strings:
        $header = "icns"

    condition:
        prefix_size >= 8 and $header at 0 and
        (uint32be(4) == 8 or
         (prefix_size >= 16 and uint32be(4) >= 16 and uint32be(12) >= 8))
}

rule taxonomy_llvm_bitcode
{
	meta:
        source_refs = "libmagic:magic/Magdir/llvm:libmagic_9a3c63c0003b7665804d_line_14; libmagic:magic/Magdir/llvm:libmagic_aff3e8c38b6f7bdbd36a_line_22"
		label = "llvm_bitcode"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

	strings:
		$p0_0 = { 42 43 c0 de }
		$p1_0 = { de c0 17 0b }

	condition:
		prefix_size >= 8 and (((prefix_size >= 4 and $p0_0 at 0) or (prefix_size >= 4 and $p1_0 at 0)))
}

rule taxonomy_lnk
{
	meta:
        source_refs = "libmagic:magic/Magdir/windows:libmagic_47fbb4ad20a117b8eb73_line_714"
		label = "lnk"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

	strings:
		$p0_0 = { 4c 00 00 00 01 14 02 00 00 00 00 00 c0 00 00 00 00 00 00 46 }

	condition:
		prefix_size >= 8 and ((prefix_size >= 20 and $p0_0 at 0))
}

rule taxonomy_lrz
{
	meta:
        source_refs = "libmagic:magic/Magdir/compress:libmagic_03c835c68662764d3199_line_291; puremagic:puremagic/magic_data.json:headers[712]"
		label = "lrz"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

	strings:
		$p0_0 = "LRZI"

	condition:
		prefix_size >= 8 and ((prefix_size >= 4 and original_size >= 4 and $p0_0 at 0))
}

rule taxonomy_lz
{
	meta:
        source_refs = "libmagic:magic/Magdir/compress:libmagic_c4255171492b37ee739f_line_167; puremagic:puremagic/magic_data.json:headers[1033]; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[981]/magic[0]"
		label = "lz"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    // Lzip versions 0/1 and dictionary exponent 12..29, as recognized by libarchive.
    strings:
        $header = /LZIP[\x00\x01][\x0c-\x1d\x2c-\x3d\x4c-\x5d\x6c-\x7d\x8c-\x9d\xac-\xbd\xcc-\xdd\xec-\xfd]/

    condition:
        prefix_size >= 8 and $header at 0 and
        ((uint8(4) == 0 and original_size >= 18) or (uint8(4) == 1 and original_size >= 26))
}

rule taxonomy_lz4
{
	meta:
        source_refs = "libmagic:magic/Magdir/compress:libmagic_42d7f09175b728f9aee7_line_302; libmagic:magic/Magdir/compress:libmagic_70fd0ac1784d1655ef10_line_304; libmagic:magic/Magdir/compress:libmagic_e2babc0a36b0eff2a8be_line_298; puremagic:puremagic/magic_data.json:headers[1132]; puremagic:puremagic/magic_data.json:headers[769]; puremagic:puremagic/magic_data.json:headers[770]; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[980]/magic[0]"
		label = "lz4"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

	strings:
		$p0_0 = { 02 21 4C 18 }
		$p1_0 = { 04 22 4D 18 }
		$p2_0 = { 03 21 4c 18 }

	condition:
		prefix_size >= 8 and (((prefix_size >= 4 and original_size >= 4 and $p0_0 at 0) or (prefix_size >= 4 and original_size >= 4 and $p1_0 at 0) or (prefix_size >= 4 and $p2_0 at 0)))
}

rule taxonomy_mat
{
	meta:
        source_refs = "libmagic:magic/Magdir/mathematica:libmagic_984c602b782bbdf7144d_line_98"
		label = "mat"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    // MAT-v5 text prefix and complete 128-byte header, including endian/version pair.
    strings:
        $header = "MATLAB 5"
        $version_endian = { (00 01 49 4D | 01 00 4D 49) }
    condition:
        prefix_size >= 128 and $header at 0 and $version_endian at 124
}

rule taxonomy_midi
{
	meta:
        source_refs = "libmagic:magic/Magdir/audio:libmagic_5b3643e3bc3d2a11e17d_line_86; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[1213]/magic[0]"
		label = "midi"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    strings:
        // Standard MIDI header chunk plus the first track header; extended MThd is not enforced.
        $header = { 4D 54 68 64 00 00 00 06 }
        $track = "MTrk"

    condition:
        prefix_size >= 26 and $header at 0 and uint16be(8) <= 2 and
        uint16be(10) >= 1 and $track at 14 and uint32be(18) >= 4 and
        ((uint16be(12) >= 1 and uint16be(12) <= 32767) or
         ((uint8(12) == 232 or uint8(12) == 231 or uint8(12) == 227 or uint8(12) == 226) and
          uint8(13) >= 1)) and
        ((uint16be(8) == 0 and uint16be(10) == 1) or
         (uint16be(8) >= 1 and uint16be(8) <= 2))
}

rule taxonomy_mpegts
{
	meta:
        source_refs = "libmagic:magic/Magdir/animation:libmagic_33a3d0f625e0b60f722a_line_950; libmagic:magic/Magdir/animation:libmagic_fea279d05e642cfa3dda_line_938"
		label = "mpegts"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    // Observe 21 complete TS/M2TS packets, each with sync and legal header bits.
    // Five isolated G bytes are insufficient, especially in nucleotide text.
    strings:
        $ts = /\x47[\x00-\x7f][\x00-\xff][\x10-\x3f\x50-\x7f\x90-\xbf\xd0-\xff]([\x00-\xff]{184}\x47[\x00-\x7f][\x00-\xff][\x10-\x3f\x50-\x7f\x90-\xbf\xd0-\xff]){20}/
        $m2ts = /\x47[\x00-\x7f][\x00-\xff][\x10-\x3f\x50-\x7f\x90-\xbf\xd0-\xff]([\x00-\xff]{188}\x47[\x00-\x7f][\x00-\xff][\x10-\x3f\x50-\x7f\x90-\xbf\xd0-\xff]){20}/

    condition:
        (prefix_size >= 3948 and $ts at 0)
        or (prefix_size >= 4032 and $m2ts at 4)
}

rule taxonomy_npy
{
	meta:
        source_refs = "libmagic:magic/Magdir/numpy:libmagic_4e66984eb36b9d061a6b_line_6; libmagic:magic/Magdir/python:libmagic_6a4b6a4fe571196e980c_line_316; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:4477; puremagic:puremagic/magic_data.json:headers[1025]"
		label = "npy"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    strings:
        // NumPy v1 uses a 16-bit header length; v2/v3 use 32 bits. Older files align to 16.
        // Dictionary order is unconstrained; do not require descr to be the first key.
        $v1 = { 93 4E 55 4D 50 59 01 00 }
        $v2 = { 93 4E 55 4D 50 59 ( 02 | 03 ) 00 }
        $dictionary = /\{[ \t]{0,16}['"]/

    condition:
        prefix_size >= 64 and
        (($v1 at 0 and uint16(8) >= 54 and uint16(8) % 16 == 6 and $dictionary at 10) or
         ($v2 at 0 and uint32(8) >= 52 and uint32(8) % 16 == 4 and $dictionary at 12))
}

rule taxonomy_one
{
	meta:
        source_refs = "pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:968"
		label = "one"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

	strings:
		$p0_0 = /\xe4\x52\x5c\x7b\x8c\xd8\xa7\x4d\xae\xb1\x53\x78\xd0\x29\x96\xd3/

	condition:
		prefix_size >= 8 and (($p0_0 at 0))
}



rule taxonomy_parquet
{
	meta:
        source_refs = "libmagic:magic/Magdir/apache:libmagic_db6156119e2e23b734e3_line_23; puremagic:puremagic/magic_data.json:headers[1024]"
		label = "parquet"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

	strings:
		$p0_0 = "PAR1"

	condition:
		prefix_size >= 8 and ((prefix_size >= 4 and original_size >= 4 and $p0_0 at 0))
}

rule taxonomy_postgres_dump
{
	meta:
        source_refs = "libmagic:magic/Magdir/database:libmagic_4a7770de7bf63dc622ca_line_759"
		label = "postgres_dump"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

	strings:
		$p0_0 = { 50 47 44 4d 50 }

	condition:
		prefix_size >= 8 and ((prefix_size >= 5 and $p0_0 at 0))
}

rule taxonomy_psd
{
	meta:
        source_refs = "puremagic:puremagic/magic_data.json:headers[209]; puremagic:puremagic/magic_data.json:headers[891]; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[1341]/magic[0]"
		label = "psd"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    // Adobe PSD/PSB: 26-byte header, version, reserved bytes, channels, dimensions, depth and mode.
    strings:
        $header = { 38 42 50 53 00 (01 | 02) 00 00 00 00 00 00 }

    condition:
        prefix_size >= 26 and $header at 0
        and uint16be(12) >= 1 and uint16be(12) <= 56
        and uint32be(14) >= 1 and uint32be(18) >= 1
        and ((uint16be(4) == 1 and uint32be(14) <= 30000 and uint32be(18) <= 30000)
             or (uint16be(4) == 2 and uint32be(14) <= 300000 and uint32be(18) <= 300000))
        and (uint16be(22) == 1 or uint16be(22) == 8 or uint16be(22) == 16 or uint16be(22) == 32)
        and uint16be(24) <= 9
}

rule taxonomy_qoi
{
	meta:
        source_refs = "libmagic:magic/Magdir/images:libmagic_6a665fe42be9c3f1f152_line_4228"
		label = "qoi"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    strings:
        // https://qoiformat.org/qoi-specification.pdf: complete header and minimum encoded image.
        $magic = "qoif"

    condition:
        prefix_size >= 23 and $magic at 0 and uint32be(4) > 0 and uint32be(8) > 0 and
        (uint8(12) == 3 or uint8(12) == 4) and uint8(13) <= 1
}

rule taxonomy_rar
{
	meta:
        source_refs = "libmagic:magic/Magdir/archive:libmagic_1eadff392eb6b0148880_line_1779; libmagic:magic/Magdir/archive:libmagic_4cf4f157df6b9019959b_line_1785; libmagic:magic/Magdir/archive:libmagic_7fa6fc500289dc8de349_line_1767; puremagic:puremagic/magic_data.json:headers[1253]; puremagic:puremagic/magic_data.json:headers[796]"
		label = "rar"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    // Complete RAR4/RAR5 markers, plus the pre-1.5 marker retained by libmagic.
    strings:
        $rar4 = { 52 61 72 21 1A 07 00 }
        $rar5 = { 52 61 72 21 1A 07 01 00 }
        $legacy = { 52 45 7E 5E }
    condition:
        prefix_size >= 8 and ($rar4 at 0 or $rar5 at 0 or $legacy at 0)
}

rule taxonomy_redis_rdb
{
	meta:
        source_refs = "libmagic:magic/Magdir/database:libmagic_d4babf35577f34be1a8d_line_862"
		label = "redis_rdb"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    // Redis writes REDIS followed by a four-digit, nonzero RDB version.
    strings:
        $header = /REDIS[0-9]{4}/

    condition:
        prefix_size >= 9 and $header at 0 and uint32be(5) != 0x30303030
}

rule taxonomy_rzip
{
	meta:
        source_refs = "libmagic:magic/Magdir/compress:libmagic_261539d949662e9ead7f_line_373"
		label = "rzip"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    // Rzip 2.x: complete 24-byte header; ten reserved bytes follow the two size words.
    strings:
        $header = { 52 5A 49 50 02 (00 | 01) [8] 00 00 00 00 00 00 00 00 00 00 }

    condition:
        prefix_size >= 24 and $header at 0
}

rule taxonomy_sas
{
	meta:
        source_refs = "libmagic:magic/Magdir/macintosh:libmagic_4b697b5e08176d24d0fe_line_382; libmagic:magic/Magdir/macintosh:libmagic_622cf2755dc5f02d2170_line_375"
		label = "sas"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    // SAS7BDAT: correlate the binary file magic used by pandas with Tika SAS FILE marker; no bare SAS text.
    strings:
        $header = { 00 00 00 00 00 00 00 00 00 00 00 00 C2 EA 81 60 B3 14 11 CF BD 92 08 00 09 C7 31 8C 18 1F 10 11 }
        $type = "SAS FILE"

    condition:
        prefix_size >= 288 and $header at 0 and $type at 84 and uint8(37) <= 1
}

rule taxonomy_sevenzip
{
	meta:
        source_refs = "puremagic:puremagic/magic_data.json:headers[1006]; puremagic:puremagic/magic_data.json:headers[976]; puremagic:puremagic/magic_data.json:headers[977]; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[1158]/magic[0]"
		label = "sevenzip"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    // 7z complete 32-byte signature header, exact offset and supported major version zero.
    strings:
        $header = { 37 7A BC AF 27 1C 00 }

    condition:
        prefix_size >= 32 and $header at 0
}

rule taxonomy_shapefile
{
	meta:
        source_refs = "pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:290; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:291"
		label = "shapefile"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

	strings:
		$p0_0 = /(\x00){2}\x27\x0a(\x00){20}(([\x00-\xff]){4}\xe8\x03(\x00){2})(([\x00-\xff]){68}(\x00){3}\x01)/
		$p1_0 = /(\x00){2}\x27\x0a(\x00){20}(([\x00-\xff]){4}\xe8\x03(\x00){2})(([\x00-\xff]){68}(\x00){3}\x32)/

	condition:
		prefix_size >= 8 and ((($p0_0 at 0) or ($p1_0 at 0)))
}

rule taxonomy_sketchup
{
	meta:
        source_refs = "pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:1632; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:1633; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:1634; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:1635; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:1636; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:1637; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:1638; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:1639; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:1640; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:1641; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:1642; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:1643; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:1644; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:1645; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:1646; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:241; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:243"
		label = "sketchup"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    // Version-specific PRONOM signatures extend the same Unicode header; keep the two families.
    strings:
        $unicode = { FF FE FF 0E 53 00 6B 00 65 00 74 00 63 00 68 00 55 00 70 00 20 00 4D 00 6F 00 64 00 65 00 6C 00 }
        $legacy = { 0E 53 6B 65 74 63 68 55 70 20 4D 6F 64 65 6C 08 }
    condition:
        prefix_size >= 16 and ($legacy at 0 or (prefix_size >= 32 and $unicode at 0))
}

rule taxonomy_spirv
{
	meta:
        source_refs = "libmagic:magic/Magdir/gpu:libmagic_7a4bb2d0c81dc68976d6_line_10; libmagic:magic/Magdir/gpu:libmagic_bbb365f134779a68f415_line_14"
		label = "spirv"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    // SPIR-V five-word header: version 1.0..1.6, nonzero ID bound and reserved schema zero.
    strings:
        $little = { 03 02 23 07 }
        $big = { 07 23 02 03 }

    condition:
        prefix_size >= 20 and original_size % 4 == 0 and
        (
            ($little at 0 and uint32(4) >= 0x10000 and uint32(4) <= 0x10600 and
             uint32(4) % 256 == 0 and uint32(12) > 0 and uint32(16) == 0) or
            ($big at 0 and uint32be(4) >= 0x10000 and uint32be(4) <= 0x10600 and
             uint32be(4) % 256 == 0 and uint32be(12) > 0 and uint32be(16) == 0)
        )
}

rule taxonomy_spss
{
	meta:
        source_refs = "libmagic:magic/Magdir/macintosh:libmagic_6ae5d200808844216840_line_397; libmagic:magic/Magdir/macintosh:libmagic_f7a5b496623d01e3a740_line_391; libmagic:magic/Magdir/macintosh:libmagic_fa381660522830f3ca23_line_394"
		label = "spss"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

	strings:
		$p0_0 = "$FL3"
		$p1_0 = { c9 c3 e2 c1 }
		$p2_0 = "$FL2"

	condition:
		prefix_size >= 8 and (((prefix_size >= 4 and original_size >= 4 and $p0_0 at 0) or (prefix_size >= 4 and $p1_0 at 0) or (prefix_size >= 4 and original_size >= 4 and $p2_0 at 0)))
}

rule taxonomy_swf
{
	meta:
        source_refs = "puremagic:puremagic/magic_data.json:headers[1169]; puremagic:puremagic/magic_data.json:headers[229]; puremagic:puremagic/magic_data.json:headers[230]; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[1069]/magic[0]"
		label = "swf"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    // SWF header version and declared size; FWS RECT begins with nonzero Nbits, CWS starts with a zlib header.
    strings:
        $fws = "FWS"
        $cws = "CWS"
        $zws = "ZWS"
        $zlib = { 78 (01 | 5E | 9C | DA) }

    condition:
        prefix_size >= 15 and uint32(4) >= 15 and uint8(3) >= 1 and uint8(3) <= 50
        and (($fws at 0 and uint8(8) >= 8 and uint8(8) <= 248
             and uint8(8) % 8 == 0 and uint8(9) <= 31)
             or ($cws at 0 and uint8(3) >= 6 and $zlib at 8)
             or (prefix_size >= 17 and $zws at 0 and uint8(3) >= 13
                 and uint32(8) >= 5 and uint8(12) == 93
                 and uint32(13) >= 4096 and uint32(13) <= 1073741824))
}

rule taxonomy_uf2
{
	meta:
        source_refs = "libmagic:magic/Magdir/uf2:libmagic_593953bf1fe4b0c04f55_line_10"
		label = "uf2"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    // Verify a complete first block with both starting magics and the final block magic.
    strings:
        $header = { 55 46 32 0A 57 51 5D 9E }
        $end = { 30 6F B1 0A }
    condition:
        prefix_size >= 512 and $header at 0 and $end at 508 and uint32(16) <= 476
}

rule taxonomy_vhd
{
	meta:
        source_refs = "tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[1117]/magic[0]"
		label = "vhd"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

	strings:
		$p0_0 = "conectix"

	condition:
		prefix_size >= 8 and ((prefix_size >= 8 and original_size >= 8 and $p0_0 at 0))
}

rule taxonomy_wad
{
	meta:
        source_refs = "libmagic:magic/Magdir/games:libmagic_5f1ddb326909ecffc607_line_153; libmagic:magic/Magdir/games:libmagic_9401ba2c279a4bc1ae41_line_151"
		label = "wad"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    // The IWAD/PWAD header includes the lump count and directory offset, even for an empty WAD.
    strings:
        $header = { (49 | 50) 57 41 44 }
    condition:
        prefix_size >= 12 and $header at 0
}

rule taxonomy_wasm
{
	meta:
        source_refs = "libmagic:magic/Magdir/webassembly:libmagic_90ec43ebe7a0b2de495f_line_16; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[851]/magic[0]"
		label = "wasm"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    // WebAssembly binary module: magic and version 1; an empty module is eight bytes.
    strings:
        $header = { 00 61 73 6D 01 00 00 00 }

    condition:
        prefix_size >= 8 and $header at 0
}

rule taxonomy_wav
{
	meta:
        source_refs = "libmagic:magic/Magdir/riff:libmagic_8f0be06519d9d702c0a4_line_911; libmagic:magic/Magdir/riff:libmagic_b2aac9f8242bb4da4291_line_305; puremagic:puremagic/magic_data.json:headers[196]; puremagic:puremagic/magic_data.json:headers[853]; puremagic:puremagic/magic_data.json:headers[854]"
		label = "wav"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    // Correlate WAVE form with RIFF/RIFX container and length; RF64 requires ds64.
    strings:
        $riff = "RIFF"
        $rifx = "RIFX"
        $wave = "WAVE"
        $rf64 = { 52 46 36 34 FF FF FF FF 57 41 56 45 64 73 36 34 }

    condition:
        prefix_size >= 12 and $wave at 8
        and (($riff at 0 and uint32(4) >= 4) or ($rifx at 0 and uint32be(4) >= 4)
             or (prefix_size >= 48 and $rf64 at 0 and uint32(16) >= 28))
}

rule taxonomy_woff
{
	meta:
        source_refs = "libmagic:magic/Magdir/fonts:libmagic_7bfeb01fcf0eb7d5ca50_line_440"
		label = "woff"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    strings:
        // W3C WOFF header plus one complete directory entry. Common sfnt flavors only.
        $magic = "wOFF"

    condition:
        prefix_size >= 64 and $magic at 0 and uint32be(8) >= 64 and
        uint16be(12) >= 1 and uint16be(12) <= 4095 and uint16be(14) == 0 and
        uint32be(16) >= 28 and
        (uint32be(4) == 65536 or uint32be(4) == 0x4F54544F or
         uint32be(4) == 0x74727565 or uint32be(4) == 0x74797031)
}

rule taxonomy_woff2
{
	meta:
        source_refs = "libmagic:magic/Magdir/fonts:libmagic_9edeb7fe3fa716aae642_line_446; puremagic:puremagic/magic_data.json:headers[1256]"
		label = "woff2"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    strings:
        // W3C WOFF2: header, at least one directory entry and compressed payload.
        $magic = "wOF2"

    condition:
        prefix_size >= 51 and $magic at 0 and uint32be(8) >= 51 and
        uint16be(12) >= 1 and uint16be(12) <= 4095 and uint16be(14) == 0 and
        uint32be(16) >= 28 and uint32be(20) >= 1 and
        (uint32be(4) == 65536 or uint32be(4) == 0x4F54544F or
         uint32be(4) == 0x74727565 or uint32be(4) == 0x74797031 or uint32be(4) == 0x74746366)
}

rule taxonomy_xar
{
	meta:
        source_refs = "libmagic:magic/Magdir/archive:libmagic_5ef43911251c7592c921_line_2508; puremagic:puremagic/magic_data.json:headers[772]"
		label = "xar"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    // Complete XAR header and nonzero TOC lengths; retain the reader's size/version tolerance.
    strings:
        $header = "xar!"

    condition:
        prefix_size >= 28 and $header at 0 and
        (uint32be(8) > 0 or uint32be(12) > 0) and
        (uint32be(16) > 0 or uint32be(20) > 0)
}

rule taxonomy_xcf
{
	meta:
        source_refs = "libmagic:magic/Magdir/gimp:libmagic_016ce1aee163bcd2a35b_line_28; puremagic:puremagic/magic_data.json:headers[1162]; puremagic:puremagic/magic_data.json:headers[211]; puremagic:puremagic/magic_data.json:headers[897]; puremagic:puremagic/magic_data.json:headers[898]; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[1418]/magic[0]"
		label = "xcf"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    // Complete versioned XCF header and base image type; preserve GIMP's dimension tolerance.
    strings:
        $header = /gimp xcf (file|v[0-9]{3})\x00/
    condition:
        prefix_size >= 26 and $header at 0 and uint32be(22) <= 2
}

rule taxonomy_zst
{
	meta:
        source_refs = "libmagic:magic/Magdir/compress:libmagic_3c1e821a69ef1e6c063e_line_344; libmagic:magic/Magdir/compress:libmagic_3c3f51c669c7e683e84e_line_335; libmagic:magic/Magdir/compress:libmagic_73fdbaa0f6ab1fb1e5b3_line_338; libmagic:magic/Magdir/compress:libmagic_a1474a8f7ec6448a1af9_line_341; libmagic:magic/Magdir/compress:libmagic_a4ac4a390da6374b511d_line_347; libmagic:magic/Magdir/compress:libmagic_ad94533cfef44f1f3f18_line_350; libmagic:magic/Magdir/compress:libmagic_ffc856d00fae08a2c373_line_354; puremagic:puremagic/magic_data.json:headers[1129]; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[957]/magic[0]"
		label = "zst"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

	strings:
		$p0_0 = { 28 B5 2F FD }
		$p1_0 = { 23 b5 2f fd }
		$p2_0 = { 27 b5 2f fd }
		$p3_0 = { 24 b5 2f fd }
		$p4_0 = { 25 b5 2f fd }
		$p5_0 = { 22 b5 2f fd }
		$p6_0 = { 26 b5 2f fd }

	condition:
		prefix_size >= 8 and (((prefix_size >= 4 and original_size >= 4 and $p0_0 at 0) or (prefix_size >= 4 and $p1_0 at 0) or (prefix_size >= 4 and $p2_0 at 0) or (prefix_size >= 4 and $p3_0 at 0) or (prefix_size >= 4 and $p4_0 at 0) or (prefix_size >= 4 and $p5_0 at 0) or (prefix_size >= 4 and $p6_0 at 0)))
}
