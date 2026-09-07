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

	strings:
		$p0_0 = /(\x4d){2}([\x00-\xff]){4}\x02\x00\x0a(\x00){3}[\x03-\x04](([\x00-\xff]){3}(\x3d){2})/
		$p1_0 = /(\x4d){2}(([\x00-\xff]){4}(\x3d){2})/

	condition:
		(($p0_0 at 0) or ($p1_0 at 0))
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
		$p0_0 = { 33 44 53 58 }

	condition:
		(prefix_size >= 4 and $p0_0 at 0)
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

	strings:
		$p0_0 = "ftyp3gg"
		$p1_0 = "ftyp3gp6"
		$p2_0 = { 33 67 67 }
		$p3_0 = "ftyp3gp4"
		$p4_0 = "ftyp3gp3"
		$p5_0 = { 33 67 70 }
		$p6_0 = "ftyp3ge6"
		$p7_0 = "ftyp3gg6"
		$p8_0 = "ftyp3gs7"
		$p9_0 = "ftyp"
		$p10_0 = "ftyp3gp2"
		$p11_0 = { 33 67 65 }
		$p12_0 = { 33 67 73 }
		$p13_0 = "ftyp3gs"
		$p14_0 = "ftyp3ge"
		$p15_0 = { 33 67 68 }
		$p16_0 = { 00 00 00 14 66 74 79 70 33 67 70 }
		$p17_0 = { 33 67 72 }
		$p18_0 = "ftyp3gp5"
		$p19_0 = { 33 67 6d }
		$p20_0 = "ftyp3ge7"
		$p21_0 = "ftyp3gp"
		$p22_0 = "ftyp3gp1"
		$p23_0 = { 33 67 74 }

	condition:
		(((prefix_size >= 8 and $p9_0 at 4) and ((prefix_size >= 11 and $p2_0 at 8) or (prefix_size >= 11 and $p5_0 at 8) or (prefix_size >= 11 and $p11_0 at 8) or (prefix_size >= 11 and $p12_0 at 8) or (prefix_size >= 11 and $p15_0 at 8) or (prefix_size >= 11 and $p17_0 at 8) or (prefix_size >= 11 and $p19_0 at 8) or (prefix_size >= 11 and $p23_0 at 8))) or ((prefix_size >= 11 and original_size >= 11 and $p0_0 at 4) or (prefix_size >= 12 and $p1_0 at 4) or (prefix_size >= 12 and $p3_0 at 4) or (prefix_size >= 12 and $p4_0 at 4) or (prefix_size >= 12 and $p6_0 at 4) or (prefix_size >= 12 and $p7_0 at 4) or (prefix_size >= 12 and $p8_0 at 4) or (prefix_size >= 12 and $p10_0 at 4) or (prefix_size >= 11 and original_size >= 11 and $p13_0 at 4) or (prefix_size >= 11 and original_size >= 11 and $p14_0 at 4) or (prefix_size >= 11 and original_size >= 11 and $p16_0 at 0) or (prefix_size >= 12 and original_size >= 12 and $p18_0 at 4) or (prefix_size >= 12 and $p20_0 at 4) or (prefix_size >= 11 and original_size >= 11 and $p21_0 at 4) or (prefix_size >= 12 and $p22_0 at 4)))
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
		((prefix_size >= 14 and original_size >= 14 and $p0_0 at 7) or ($p1_0 at 7) or ($p2_0 at 7))
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

	strings:
		$p0_0 = "bplist01"
		$p1_0 = "bplist"
		$p2_0 = { 62 70 6C 69 73 74 00 00 }
		$p3_0 = "bplist10"
		$p4_0 = { 62 70 6C 69 73 74 40 00 }
		$p5_0 = "bplist15"
		$p6_0 = "bplist00"
		$p7_0 = "bplist16"
		$p8_0 = { 62 70 6C 69 73 74 00 01 }

	condition:
		((prefix_size >= 8 and $p0_0 at 0) or (prefix_size >= 6 and original_size >= 6 and $p1_0 at 0) or (prefix_size >= 8 and $p2_0 at 0) or (prefix_size >= 8 and $p3_0 at 0) or (prefix_size >= 8 and $p4_0 at 0) or (prefix_size >= 8 and $p5_0 at 0) or (prefix_size >= 8 and $p6_0 at 0) or (prefix_size >= 8 and $p7_0 at 0) or (prefix_size >= 8 and $p8_0 at 0))
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

	strings:
		$p0_0 = { 00 05 16 07 }

	condition:
		(prefix_size >= 4 and $p0_0 at 0)
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

	strings:
		$p0_0 = { 00 05 16 00 }

	condition:
		(prefix_size >= 4 and $p0_0 at 0)
}

rule taxonomy_arc
{
	meta:
        source_refs = "libmagic:magic/Magdir/archive:libmagic_1631bb45451c23fb6aac_line_598; libmagic:magic/Magdir/archive:libmagic_243a5e9cf3cfc790f9f5_line_594; libmagic:magic/Magdir/archive:libmagic_2cf74413ac8980dea53f_line_592; libmagic:magic/Magdir/archive:libmagic_35a363d7bd396019e8f6_line_607; libmagic:magic/Magdir/archive:libmagic_610c9e502ebb6af29ccb_line_600; libmagic:magic/Magdir/archive:libmagic_a17cb6a06bbbdcf78f3a_line_605; libmagic:magic/Magdir/archive:libmagic_b419cbf5f668942ab6e5_line_602; libmagic:magic/Magdir/archive:libmagic_e1ba35801790f0f6eeae_line_609; libmagic:magic/Magdir/archive:libmagic_f1405c71130bec4d78bc_line_596; puremagic:puremagic/magic_data.json:headers[1078]; puremagic:puremagic/magic_data.json:headers[1079]; puremagic:puremagic/magic_data.json:headers[1080]; puremagic:puremagic/magic_data.json:headers[1081]; puremagic:puremagic/magic_data.json:headers[1082]; puremagic:puremagic/magic_data.json:headers[1083]"
		label = "arc"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

	strings:
		$p0_0 = { 1a 02 ( 00 | 01 | 02 | 03 | 04 | 05 | 06 | 07 | 08 | 09 | 0a | 0b | 0c | 0d | 0e | 0f | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 19 | 1a | 1b | 1c | 1d | 1e | 1f | 20 | 21 | 22 | 23 | 24 | 25 | 26 | 27 | 28 | 29 | 2a | 2b | 2c | 2d | 2e | 2f | 30 | 31 | 32 | 33 | 34 | 35 | 36 | 37 | 38 | 39 | 3a | 3b | 3c | 3d | 3e | 3f | 40 | 41 | 42 | 43 | 44 | 45 | 46 | 47 | 48 | 49 | 4a | 4b | 4c | 4d | 4e | 4f | 50 | 51 | 52 | 53 | 54 | 55 | 56 | 57 | 58 | 59 | 5a | 5b | 5c | 5d | 5e | 5f | 60 | 61 | 62 | 63 | 64 | 65 | 66 | 67 | 68 | 69 | 6a | 6b | 6c | 6d | 6e | 6f | 70 | 71 | 72 | 73 | 74 | 75 | 76 | 77 | 78 | 79 | 7a | 7b | 7c | 7d | 7e | 7f ) ( 00 | 01 | 02 | 03 | 04 | 05 | 06 | 07 | 08 | 09 | 0a | 0b | 0c | 0d | 0e | 0f | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 19 | 1a | 1b | 1c | 1d | 1e | 1f | 20 | 21 | 22 | 23 | 24 | 25 | 26 | 27 | 28 | 29 | 2a | 2b | 2c | 2d | 2e | 2f | 30 | 31 | 32 | 33 | 34 | 35 | 36 | 37 | 38 | 39 | 3a | 3b | 3c | 3d | 3e | 3f | 40 | 41 | 42 | 43 | 44 | 45 | 46 | 47 | 48 | 49 | 4a | 4b | 4c | 4d | 4e | 4f | 50 | 51 | 52 | 53 | 54 | 55 | 56 | 57 | 58 | 59 | 5a | 5b | 5c | 5d | 5e | 5f | 60 | 61 | 62 | 63 | 64 | 65 | 66 | 67 | 68 | 69 | 6a | 6b | 6c | 6d | 6e | 6f | 70 | 71 | 72 | 73 | 74 | 75 | 76 | 77 | 78 | 79 | 7a | 7b | 7c | 7d | 7e | 7f ) }
		$p1_0 = { 1a 06 ( 00 | 01 | 02 | 03 | 04 | 05 | 06 | 07 | 08 | 09 | 0a | 0b | 0c | 0d | 0e | 0f | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 19 | 1a | 1b | 1c | 1d | 1e | 1f | 20 | 21 | 22 | 23 | 24 | 25 | 26 | 27 | 28 | 29 | 2a | 2b | 2c | 2d | 2e | 2f | 30 | 31 | 32 | 33 | 34 | 35 | 36 | 37 | 38 | 39 | 3a | 3b | 3c | 3d | 3e | 3f | 40 | 41 | 42 | 43 | 44 | 45 | 46 | 47 | 48 | 49 | 4a | 4b | 4c | 4d | 4e | 4f | 50 | 51 | 52 | 53 | 54 | 55 | 56 | 57 | 58 | 59 | 5a | 5b | 5c | 5d | 5e | 5f | 60 | 61 | 62 | 63 | 64 | 65 | 66 | 67 | 68 | 69 | 6a | 6b | 6c | 6d | 6e | 6f | 70 | 71 | 72 | 73 | 74 | 75 | 76 | 77 | 78 | 79 | 7a | 7b | 7c | 7d | 7e | 7f ) ( 00 | 01 | 02 | 03 | 04 | 05 | 06 | 07 | 08 | 09 | 0a | 0b | 0c | 0d | 0e | 0f | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 19 | 1a | 1b | 1c | 1d | 1e | 1f | 20 | 21 | 22 | 23 | 24 | 25 | 26 | 27 | 28 | 29 | 2a | 2b | 2c | 2d | 2e | 2f | 30 | 31 | 32 | 33 | 34 | 35 | 36 | 37 | 38 | 39 | 3a | 3b | 3c | 3d | 3e | 3f | 40 | 41 | 42 | 43 | 44 | 45 | 46 | 47 | 48 | 49 | 4a | 4b | 4c | 4d | 4e | 4f | 50 | 51 | 52 | 53 | 54 | 55 | 56 | 57 | 58 | 59 | 5a | 5b | 5c | 5d | 5e | 5f | 60 | 61 | 62 | 63 | 64 | 65 | 66 | 67 | 68 | 69 | 6a | 6b | 6c | 6d | 6e | 6f | 70 | 71 | 72 | 73 | 74 | 75 | 76 | 77 | 78 | 79 | 7a | 7b | 7c | 7d | 7e | 7f ) }
		$p2_0 = { 1A 03 00 00 }
		$p3_0 = { 1a 08 ( 00 | 01 | 02 | 03 | 04 | 05 | 06 | 07 | 08 | 09 | 0a | 0b | 0c | 0d | 0e | 0f | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 19 | 1a | 1b | 1c | 1d | 1e | 1f | 20 | 21 | 22 | 23 | 24 | 25 | 26 | 27 | 28 | 29 | 2a | 2b | 2c | 2d | 2e | 2f | 30 | 31 | 32 | 33 | 34 | 35 | 36 | 37 | 38 | 39 | 3a | 3b | 3c | 3d | 3e | 3f | 40 | 41 | 42 | 43 | 44 | 45 | 46 | 47 | 48 | 49 | 4a | 4b | 4c | 4d | 4e | 4f | 50 | 51 | 52 | 53 | 54 | 55 | 56 | 57 | 58 | 59 | 5a | 5b | 5c | 5d | 5e | 5f | 60 | 61 | 62 | 63 | 64 | 65 | 66 | 67 | 68 | 69 | 6a | 6b | 6c | 6d | 6e | 6f | 70 | 71 | 72 | 73 | 74 | 75 | 76 | 77 | 78 | 79 | 7a | 7b | 7c | 7d | 7e | 7f ) ( 00 | 01 | 02 | 03 | 04 | 05 | 06 | 07 | 08 | 09 | 0a | 0b | 0c | 0d | 0e | 0f | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 19 | 1a | 1b | 1c | 1d | 1e | 1f | 20 | 21 | 22 | 23 | 24 | 25 | 26 | 27 | 28 | 29 | 2a | 2b | 2c | 2d | 2e | 2f | 30 | 31 | 32 | 33 | 34 | 35 | 36 | 37 | 38 | 39 | 3a | 3b | 3c | 3d | 3e | 3f | 40 | 41 | 42 | 43 | 44 | 45 | 46 | 47 | 48 | 49 | 4a | 4b | 4c | 4d | 4e | 4f | 50 | 51 | 52 | 53 | 54 | 55 | 56 | 57 | 58 | 59 | 5a | 5b | 5c | 5d | 5e | 5f | 60 | 61 | 62 | 63 | 64 | 65 | 66 | 67 | 68 | 69 | 6a | 6b | 6c | 6d | 6e | 6f | 70 | 71 | 72 | 73 | 74 | 75 | 76 | 77 | 78 | 79 | 7a | 7b | 7c | 7d | 7e | 7f ) }
		$p4_0 = { 1a 09 ( 00 | 01 | 02 | 03 | 04 | 05 | 06 | 07 | 08 | 09 | 0a | 0b | 0c | 0d | 0e | 0f | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 19 | 1a | 1b | 1c | 1d | 1e | 1f | 20 | 21 | 22 | 23 | 24 | 25 | 26 | 27 | 28 | 29 | 2a | 2b | 2c | 2d | 2e | 2f | 30 | 31 | 32 | 33 | 34 | 35 | 36 | 37 | 38 | 39 | 3a | 3b | 3c | 3d | 3e | 3f | 40 | 41 | 42 | 43 | 44 | 45 | 46 | 47 | 48 | 49 | 4a | 4b | 4c | 4d | 4e | 4f | 50 | 51 | 52 | 53 | 54 | 55 | 56 | 57 | 58 | 59 | 5a | 5b | 5c | 5d | 5e | 5f | 60 | 61 | 62 | 63 | 64 | 65 | 66 | 67 | 68 | 69 | 6a | 6b | 6c | 6d | 6e | 6f | 70 | 71 | 72 | 73 | 74 | 75 | 76 | 77 | 78 | 79 | 7a | 7b | 7c | 7d | 7e | 7f ) ( 00 | 01 | 02 | 03 | 04 | 05 | 06 | 07 | 08 | 09 | 0a | 0b | 0c | 0d | 0e | 0f | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 19 | 1a | 1b | 1c | 1d | 1e | 1f | 20 | 21 | 22 | 23 | 24 | 25 | 26 | 27 | 28 | 29 | 2a | 2b | 2c | 2d | 2e | 2f | 30 | 31 | 32 | 33 | 34 | 35 | 36 | 37 | 38 | 39 | 3a | 3b | 3c | 3d | 3e | 3f | 40 | 41 | 42 | 43 | 44 | 45 | 46 | 47 | 48 | 49 | 4a | 4b | 4c | 4d | 4e | 4f | 50 | 51 | 52 | 53 | 54 | 55 | 56 | 57 | 58 | 59 | 5a | 5b | 5c | 5d | 5e | 5f | 60 | 61 | 62 | 63 | 64 | 65 | 66 | 67 | 68 | 69 | 6a | 6b | 6c | 6d | 6e | 6f | 70 | 71 | 72 | 73 | 74 | 75 | 76 | 77 | 78 | 79 | 7a | 7b | 7c | 7d | 7e | 7f ) }
		$p5_0 = { 1A 08 00 00 }
		$p6_0 = { 1a 03 ( 00 | 01 | 02 | 03 | 04 | 05 | 06 | 07 | 08 | 09 | 0a | 0b | 0c | 0d | 0e | 0f | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 19 | 1a | 1b | 1c | 1d | 1e | 1f | 20 | 21 | 22 | 23 | 24 | 25 | 26 | 27 | 28 | 29 | 2a | 2b | 2c | 2d | 2e | 2f | 30 | 31 | 32 | 33 | 34 | 35 | 36 | 37 | 38 | 39 | 3a | 3b | 3c | 3d | 3e | 3f | 40 | 41 | 42 | 43 | 44 | 45 | 46 | 47 | 48 | 49 | 4a | 4b | 4c | 4d | 4e | 4f | 50 | 51 | 52 | 53 | 54 | 55 | 56 | 57 | 58 | 59 | 5a | 5b | 5c | 5d | 5e | 5f | 60 | 61 | 62 | 63 | 64 | 65 | 66 | 67 | 68 | 69 | 6a | 6b | 6c | 6d | 6e | 6f | 70 | 71 | 72 | 73 | 74 | 75 | 76 | 77 | 78 | 79 | 7a | 7b | 7c | 7d | 7e | 7f ) ( 00 | 01 | 02 | 03 | 04 | 05 | 06 | 07 | 08 | 09 | 0a | 0b | 0c | 0d | 0e | 0f | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 19 | 1a | 1b | 1c | 1d | 1e | 1f | 20 | 21 | 22 | 23 | 24 | 25 | 26 | 27 | 28 | 29 | 2a | 2b | 2c | 2d | 2e | 2f | 30 | 31 | 32 | 33 | 34 | 35 | 36 | 37 | 38 | 39 | 3a | 3b | 3c | 3d | 3e | 3f | 40 | 41 | 42 | 43 | 44 | 45 | 46 | 47 | 48 | 49 | 4a | 4b | 4c | 4d | 4e | 4f | 50 | 51 | 52 | 53 | 54 | 55 | 56 | 57 | 58 | 59 | 5a | 5b | 5c | 5d | 5e | 5f | 60 | 61 | 62 | 63 | 64 | 65 | 66 | 67 | 68 | 69 | 6a | 6b | 6c | 6d | 6e | 6f | 70 | 71 | 72 | 73 | 74 | 75 | 76 | 77 | 78 | 79 | 7a | 7b | 7c | 7d | 7e | 7f ) }
		$p7_0 = { 1A 02 00 00 }
		$p8_0 = { 1a 04 ( 00 | 01 | 02 | 03 | 04 | 05 | 06 | 07 | 08 | 09 | 0a | 0b | 0c | 0d | 0e | 0f | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 19 | 1a | 1b | 1c | 1d | 1e | 1f | 20 | 21 | 22 | 23 | 24 | 25 | 26 | 27 | 28 | 29 | 2a | 2b | 2c | 2d | 2e | 2f | 30 | 31 | 32 | 33 | 34 | 35 | 36 | 37 | 38 | 39 | 3a | 3b | 3c | 3d | 3e | 3f | 40 | 41 | 42 | 43 | 44 | 45 | 46 | 47 | 48 | 49 | 4a | 4b | 4c | 4d | 4e | 4f | 50 | 51 | 52 | 53 | 54 | 55 | 56 | 57 | 58 | 59 | 5a | 5b | 5c | 5d | 5e | 5f | 60 | 61 | 62 | 63 | 64 | 65 | 66 | 67 | 68 | 69 | 6a | 6b | 6c | 6d | 6e | 6f | 70 | 71 | 72 | 73 | 74 | 75 | 76 | 77 | 78 | 79 | 7a | 7b | 7c | 7d | 7e | 7f ) ( 00 | 01 | 02 | 03 | 04 | 05 | 06 | 07 | 08 | 09 | 0a | 0b | 0c | 0d | 0e | 0f | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 19 | 1a | 1b | 1c | 1d | 1e | 1f | 20 | 21 | 22 | 23 | 24 | 25 | 26 | 27 | 28 | 29 | 2a | 2b | 2c | 2d | 2e | 2f | 30 | 31 | 32 | 33 | 34 | 35 | 36 | 37 | 38 | 39 | 3a | 3b | 3c | 3d | 3e | 3f | 40 | 41 | 42 | 43 | 44 | 45 | 46 | 47 | 48 | 49 | 4a | 4b | 4c | 4d | 4e | 4f | 50 | 51 | 52 | 53 | 54 | 55 | 56 | 57 | 58 | 59 | 5a | 5b | 5c | 5d | 5e | 5f | 60 | 61 | 62 | 63 | 64 | 65 | 66 | 67 | 68 | 69 | 6a | 6b | 6c | 6d | 6e | 6f | 70 | 71 | 72 | 73 | 74 | 75 | 76 | 77 | 78 | 79 | 7a | 7b | 7c | 7d | 7e | 7f ) }
		$p9_0 = { 1a 48 ( 00 | 01 | 02 | 03 | 04 | 05 | 06 | 07 | 08 | 09 | 0a | 0b | 0c | 0d | 0e | 0f | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 19 | 1a | 1b | 1c | 1d | 1e | 1f | 20 | 21 | 22 | 23 | 24 | 25 | 26 | 27 | 28 | 29 | 2a | 2b | 2c | 2d | 2e | 2f | 30 | 31 | 32 | 33 | 34 | 35 | 36 | 37 | 38 | 39 | 3a | 3b | 3c | 3d | 3e | 3f | 40 | 41 | 42 | 43 | 44 | 45 | 46 | 47 | 48 | 49 | 4a | 4b | 4c | 4d | 4e | 4f | 50 | 51 | 52 | 53 | 54 | 55 | 56 | 57 | 58 | 59 | 5a | 5b | 5c | 5d | 5e | 5f | 60 | 61 | 62 | 63 | 64 | 65 | 66 | 67 | 68 | 69 | 6a | 6b | 6c | 6d | 6e | 6f | 70 | 71 | 72 | 73 | 74 | 75 | 76 | 77 | 78 | 79 | 7a | 7b | 7c | 7d | 7e | 7f ) ( 00 | 01 | 02 | 03 | 04 | 05 | 06 | 07 | 08 | 09 | 0a | 0b | 0c | 0d | 0e | 0f | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 19 | 1a | 1b | 1c | 1d | 1e | 1f | 20 | 21 | 22 | 23 | 24 | 25 | 26 | 27 | 28 | 29 | 2a | 2b | 2c | 2d | 2e | 2f | 30 | 31 | 32 | 33 | 34 | 35 | 36 | 37 | 38 | 39 | 3a | 3b | 3c | 3d | 3e | 3f | 40 | 41 | 42 | 43 | 44 | 45 | 46 | 47 | 48 | 49 | 4a | 4b | 4c | 4d | 4e | 4f | 50 | 51 | 52 | 53 | 54 | 55 | 56 | 57 | 58 | 59 | 5a | 5b | 5c | 5d | 5e | 5f | 60 | 61 | 62 | 63 | 64 | 65 | 66 | 67 | 68 | 69 | 6a | 6b | 6c | 6d | 6e | 6f | 70 | 71 | 72 | 73 | 74 | 75 | 76 | 77 | 78 | 79 | 7a | 7b | 7c | 7d | 7e | 7f ) }
		$p10_0 = { 1A 06 00 00 }
		$p11_0 = { 1A 09 00 00 }
		$p12_0 = { 1a 14 ( 00 | 01 | 02 | 03 | 04 | 05 | 06 | 07 | 08 | 09 | 0a | 0b | 0c | 0d | 0e | 0f | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 19 | 1a | 1b | 1c | 1d | 1e | 1f | 20 | 21 | 22 | 23 | 24 | 25 | 26 | 27 | 28 | 29 | 2a | 2b | 2c | 2d | 2e | 2f | 30 | 31 | 32 | 33 | 34 | 35 | 36 | 37 | 38 | 39 | 3a | 3b | 3c | 3d | 3e | 3f | 40 | 41 | 42 | 43 | 44 | 45 | 46 | 47 | 48 | 49 | 4a | 4b | 4c | 4d | 4e | 4f | 50 | 51 | 52 | 53 | 54 | 55 | 56 | 57 | 58 | 59 | 5a | 5b | 5c | 5d | 5e | 5f | 60 | 61 | 62 | 63 | 64 | 65 | 66 | 67 | 68 | 69 | 6a | 6b | 6c | 6d | 6e | 6f | 70 | 71 | 72 | 73 | 74 | 75 | 76 | 77 | 78 | 79 | 7a | 7b | 7c | 7d | 7e | 7f ) ( 00 | 01 | 02 | 03 | 04 | 05 | 06 | 07 | 08 | 09 | 0a | 0b | 0c | 0d | 0e | 0f | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 19 | 1a | 1b | 1c | 1d | 1e | 1f | 20 | 21 | 22 | 23 | 24 | 25 | 26 | 27 | 28 | 29 | 2a | 2b | 2c | 2d | 2e | 2f | 30 | 31 | 32 | 33 | 34 | 35 | 36 | 37 | 38 | 39 | 3a | 3b | 3c | 3d | 3e | 3f | 40 | 41 | 42 | 43 | 44 | 45 | 46 | 47 | 48 | 49 | 4a | 4b | 4c | 4d | 4e | 4f | 50 | 51 | 52 | 53 | 54 | 55 | 56 | 57 | 58 | 59 | 5a | 5b | 5c | 5d | 5e | 5f | 60 | 61 | 62 | 63 | 64 | 65 | 66 | 67 | 68 | 69 | 6a | 6b | 6c | 6d | 6e | 6f | 70 | 71 | 72 | 73 | 74 | 75 | 76 | 77 | 78 | 79 | 7a | 7b | 7c | 7d | 7e | 7f ) }
		$p13_0 = { 1A 04 00 00 }
		$p14_0 = { 1a 0a ( 00 | 01 | 02 | 03 | 04 | 05 | 06 | 07 | 08 | 09 | 0a | 0b | 0c | 0d | 0e | 0f | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 19 | 1a | 1b | 1c | 1d | 1e | 1f | 20 | 21 | 22 | 23 | 24 | 25 | 26 | 27 | 28 | 29 | 2a | 2b | 2c | 2d | 2e | 2f | 30 | 31 | 32 | 33 | 34 | 35 | 36 | 37 | 38 | 39 | 3a | 3b | 3c | 3d | 3e | 3f | 40 | 41 | 42 | 43 | 44 | 45 | 46 | 47 | 48 | 49 | 4a | 4b | 4c | 4d | 4e | 4f | 50 | 51 | 52 | 53 | 54 | 55 | 56 | 57 | 58 | 59 | 5a | 5b | 5c | 5d | 5e | 5f | 60 | 61 | 62 | 63 | 64 | 65 | 66 | 67 | 68 | 69 | 6a | 6b | 6c | 6d | 6e | 6f | 70 | 71 | 72 | 73 | 74 | 75 | 76 | 77 | 78 | 79 | 7a | 7b | 7c | 7d | 7e | 7f ) ( 00 | 01 | 02 | 03 | 04 | 05 | 06 | 07 | 08 | 09 | 0a | 0b | 0c | 0d | 0e | 0f | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 19 | 1a | 1b | 1c | 1d | 1e | 1f | 20 | 21 | 22 | 23 | 24 | 25 | 26 | 27 | 28 | 29 | 2a | 2b | 2c | 2d | 2e | 2f | 30 | 31 | 32 | 33 | 34 | 35 | 36 | 37 | 38 | 39 | 3a | 3b | 3c | 3d | 3e | 3f | 40 | 41 | 42 | 43 | 44 | 45 | 46 | 47 | 48 | 49 | 4a | 4b | 4c | 4d | 4e | 4f | 50 | 51 | 52 | 53 | 54 | 55 | 56 | 57 | 58 | 59 | 5a | 5b | 5c | 5d | 5e | 5f | 60 | 61 | 62 | 63 | 64 | 65 | 66 | 67 | 68 | 69 | 6a | 6b | 6c | 6d | 6e | 6f | 70 | 71 | 72 | 73 | 74 | 75 | 76 | 77 | 78 | 79 | 7a | 7b | 7c | 7d | 7e | 7f ) }

	condition:
		((prefix_size >= 4 and $p0_0 at 0) or (prefix_size >= 4 and $p1_0 at 0) or (prefix_size >= 4 and original_size >= 4 and $p2_0 at 0) or (prefix_size >= 4 and $p3_0 at 0) or (prefix_size >= 4 and $p4_0 at 0) or (prefix_size >= 4 and original_size >= 4 and $p5_0 at 0) or (prefix_size >= 4 and $p6_0 at 0) or (prefix_size >= 4 and original_size >= 4 and $p7_0 at 0) or (prefix_size >= 4 and $p8_0 at 0) or (prefix_size >= 4 and $p9_0 at 0) or (prefix_size >= 4 and original_size >= 4 and $p10_0 at 0) or (prefix_size >= 4 and original_size >= 4 and $p11_0 at 0) or (prefix_size >= 4 and $p12_0 at 0) or (prefix_size >= 4 and original_size >= 4 and $p13_0 at 0) or (prefix_size >= 4 and $p14_0 at 0))
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
		$p0_0 = ".snd"
		$p1_0 = { 00 00 00 01 }
		$p2_0 = { 00 00 00 03 }
		$p3_0 = { 2E 73 6E 64 00 00 00 }
		$p4_0 = { 00 00 00 06 }
		$p5_0 = { 00 00 00 07 }
		$p6_0 = { 00 00 00 04 }
		$p7_0 = { 00 00 00 02 }
		$p8_0 = { 00 00 00 05 }

	condition:
		(((prefix_size >= 4 and original_size >= 4 and $p0_0 at 0) and ((prefix_size >= 16 and $p1_0 at 12) or (prefix_size >= 16 and $p2_0 at 12) or (prefix_size >= 16 and $p4_0 at 12) or (prefix_size >= 16 and $p5_0 at 12) or (prefix_size >= 16 and $p6_0 at 12) or (prefix_size >= 16 and $p7_0 at 12) or (prefix_size >= 16 and $p8_0 at 12))) or (prefix_size >= 7 and $p3_0 at 0))
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

	strings:
		$p0_0 = { 52 49 46 46 ?? ?? ?? ?? 41 56 49 20 }
		$p1_0 = "AVI "
		$p2_0 = "AVF0"
		$p3_0 = "AVI LIST"

	condition:
		((prefix_size >= 12 and $p0_0 at 0) or (prefix_size >= 12 and original_size >= 12 and $p1_0 at 8) or (prefix_size >= 4 and original_size >= 4 and $p2_0 at 0) or (prefix_size >= 16 and original_size >= 16 and $p3_0 at 8))
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

	strings:
		$p0_0 = { 61 76 69 66 }
		$p1_0 = { 61 76 69 73 }
		$p2_0 = "ftyp"
		$p3_0 = "ftypavif"
		$p4_0 = "ftypavis"

	condition:
		(((prefix_size >= 8 and $p2_0 at 4) and ((prefix_size >= 12 and $p0_0 at 8) or (prefix_size >= 12 and $p1_0 at 8))) or ((prefix_size >= 12 and original_size >= 12 and $p3_0 at 4) or (prefix_size >= 12 and original_size >= 12 and $p4_0 at 4)))
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
		((prefix_size >= 4 and $p0_0 at 0) or ($p1_0 at 0))
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
		(prefix_size >= 4 and $p0_0 at 0)
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
		((prefix_size >= 7 and $p0_0 at 0) or ((prefix_size >= 12 and $p1_0 at 8) and (prefix_size >= 4 and $p2_0 at 0)))
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
		$p0_0 = "BLENDER"

	condition:
		(prefix_size >= 7 and original_size >= 7 and $p0_0 at 0)
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
		(prefix_size >= 4 and original_size >= 4 and $p0_0 at 0)
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
		$p0_0 = /[\x42][\x5a][\x68][\x31\x32\x33\x34\x35\x36\x37\x38\x39]/
		$p1_0 = "BZh"

	condition:
		((prefix_size >= 4 and $p0_0 at 0) or (prefix_size >= 3 and original_size >= 3 and $p1_0 at 0))
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

	strings:
		$p0_0 = { 43 52 41 4d }

	condition:
		(prefix_size >= 4 and $p0_0 at 0)
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

	strings:
		$p0_0 = { 64 65 78 0A }
		$p1_0 = { 64 65 78 0A 30 30 39 00 }

	condition:
		((prefix_size >= 4 and original_size >= 4 and $p0_0 at 0) or (prefix_size >= 8 and original_size >= 8 and $p1_0 at 0))
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
		(prefix_size >= 132 and original_size >= 132 and $p0_0 at 128)
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
		(prefix_size >= 9 and $p0_0 at 0)
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
		(prefix_size >= 12 and $p0_0 at 8)
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

	strings:
		$p0_0 = { 41 43 31 30 31 38 }
		$p1_0 = "AC2.10"
		$p2_0 = { 41 43 31 30 31 35 }
		$p3_0 = /\x41\x43\x31(\x30){2}\x33/
		$p4_0 = /\x41\x43\x31(\x30){2}\x36/
		$p5_0 = "MC0.0"
		$p6_0 = /\x41\x43\x31(\x30){2}\x32/
		$p7_0 = { 41 43 31 2e 33 }
		$p8_0 = "AC1.50"
		$p9_0 = { 41 43 31 30 33 32 }
		$p10_0 = { 41 43 31 30 31 34 }
		$p11_0 = /\x41\x43(\x31(\x30){2}\x31|\x32\x2e\x32\x31|\x32\x2e(\x32){2})/
		$p12_0 = "AC10"
		$p13_0 = /\x41\x43\x31\x30\x32\x34(\x00){2}/
		$p14_0 = /\x41\x43\x31\x30\x32\x31(\x00){2}/
		$p15_0 = "AC1.2"
		$p16_0 = { 41 43 31 30 32 37 }
		$p17_0 = "AC1.40"
		$p18_0 = /\x41\x43\x31(\x30){2}\x34/
		$p19_0 = /\x41\x43\x31(\x30){2}\x39/
		$p20_0 = { 41 43 31 30 31 32 }

	condition:
		((prefix_size >= 6 and $p0_0 at 0) or (prefix_size >= 6 and $p1_0 at 0) or (prefix_size >= 6 and $p2_0 at 0) or ($p3_0 at 0) or ($p4_0 at 0) or (prefix_size >= 5 and $p5_0 at 0) or ($p6_0 at 0) or (prefix_size >= 5 and $p7_0 at 0) or (prefix_size >= 6 and $p8_0 at 0) or (prefix_size >= 6 and $p9_0 at 0) or (prefix_size >= 6 and $p10_0 at 0) or ($p11_0 at 0) or (prefix_size >= 4 and original_size >= 4 and $p12_0 at 0) or ($p13_0 at 0) or ($p14_0 at 0) or (prefix_size >= 5 and $p15_0 at 0) or (prefix_size >= 6 and $p16_0 at 0) or (prefix_size >= 6 and $p17_0 at 0) or ($p18_0 at 0) or ($p19_0 at 0) or (prefix_size >= 6 and $p20_0 at 0))
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
		((prefix_size >= 8 and $p0_0 at 4) and (prefix_size >= 136 and $p1_0 at 132))
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
		$p0_0 = { 4b 61 79 64 61 72 61 20 46 42 58 20 42 69 6e 61 72 79 20 20 00 }

	condition:
		(prefix_size >= 21 and $p0_0 at 0)
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
		((prefix_size >= 9 and original_size >= 9 and $p0_0 at 0) and (prefix_size >= 91 and $p1_0 at 89))
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

	strings:
		$p0_0 = { 46 4C 56 01 }
		$p1_0 = "FLV"

	condition:
		((prefix_size >= 4 and original_size >= 4 and $p0_0 at 0) or (prefix_size >= 3 and $p1_0 at 0))
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

	strings:
		$p0_0 = /(\x47){2}\x55\x46\x02(\x00){3}/
		$p1_0 = /(\x47){2}\x55\x46\x01(\x00){3}/
		$p2_0 = { 47 47 55 46 }
		$p3_0 = /(\x47){2}\x55\x46\x03(\x00){3}/

	condition:
		(($p0_0 at 0) or ($p1_0 at 0) or (prefix_size >= 4 and $p2_0 at 0) or ($p3_0 at 0))
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
		$p0_0 = { 67 6c 54 46 }

	condition:
		(prefix_size >= 4 and $p0_0 at 0)
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

	strings:
		$p0_0 = "\\037\\213"
		$p1_0 = { 1F 8B }

	condition:
		((prefix_size >= 8 and original_size >= 8 and $p0_0 at 0) or (prefix_size >= 2 and $p1_0 at 0))
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
		(prefix_size >= 4 and $p0_0 at 0)
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

	strings:
		$p0_0 = "ftypheim"
		$p1_0 = "ftyp"
		$p2_0 = "ftyphevc"
		$p3_0 = { 68 65 69 63 }
		$p4_0 = "ftyphevm"
		$p5_0 = "ftyphevx"
		$p6_0 = "ftyphevs"
		$p7_0 = "ftypheix"
		$p8_0 = "ftypheic"
		$p9_0 = { 68 65 69 78 }
		$p10_0 = "ftypheis"

	condition:
		(((prefix_size >= 8 and $p1_0 at 4) and ((prefix_size >= 12 and $p3_0 at 8) or (prefix_size >= 12 and $p9_0 at 8))) or ((prefix_size >= 12 and original_size >= 12 and $p0_0 at 4) or (prefix_size >= 12 and original_size >= 12 and $p2_0 at 4) or (prefix_size >= 12 and original_size >= 12 and $p4_0 at 4) or (prefix_size >= 12 and original_size >= 12 and $p5_0 at 4) or (prefix_size >= 12 and original_size >= 12 and $p6_0 at 4) or (prefix_size >= 12 and original_size >= 12 and $p7_0 at 4) or (prefix_size >= 12 and original_size >= 12 and $p8_0 at 4) or (prefix_size >= 12 and original_size >= 12 and $p10_0 at 4)))
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
		(prefix_size >= 40 and original_size >= 40 and $p0_0 at 36)
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

	strings:
		$p0_0 = "icns"

	condition:
		(prefix_size >= 4 and original_size >= 4 and $p0_0 at 0)
}

rule taxonomy_jxl
{
	meta:
        source_refs = "libmagic:magic/Magdir/jpeg:libmagic_09c181c8a4d299652a29_line_265; libmagic:magic/Magdir/jpeg:libmagic_75914ba77f68d4a72f4e_line_256; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:1856; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:1857; puremagic:puremagic/magic_data.json:headers[13]; puremagic:puremagic/magic_data.json:headers[14]; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[1384]/magic[0]"
		label = "jxl"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

	strings:
		$p0_0 = { FF 0A }
		$p1_0 = /(\x00){3}\x0c\x4a\x58\x4c\x20\x0d\x0a\x87\x0a/
		$p2_0 = { 00 00 00 0C 4A 58 4C 20 0D 0A 87 0A }

	condition:
		((prefix_size >= 2 and original_size >= 2 and $p0_0 at 0) or ($p1_0 at 0) or (prefix_size >= 12 and original_size >= 12 and $p2_0 at 0))
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
		((prefix_size >= 4 and $p0_0 at 0) or (prefix_size >= 4 and $p1_0 at 0))
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
		(prefix_size >= 20 and $p0_0 at 0)
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
		(prefix_size >= 4 and original_size >= 4 and $p0_0 at 0)
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

	strings:
		$p0_0 = "LZIP"

	condition:
		(prefix_size >= 4 and original_size >= 4 and $p0_0 at 0)
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
		((prefix_size >= 4 and original_size >= 4 and $p0_0 at 0) or (prefix_size >= 4 and original_size >= 4 and $p1_0 at 0) or (prefix_size >= 4 and $p2_0 at 0))
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

	strings:
		$p0_0 = "MATLAB"
		$p1_0 = { 35 }

	condition:
		((prefix_size >= 6 and $p0_0 at 0) and (prefix_size >= 8 and $p1_0 at 7))
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
		$p0_0 = "MThd"

	condition:
		(prefix_size >= 4 and original_size >= 4 and $p0_0 at 0)
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

	strings:
		$p0_0 = { 47 }
		$p1_0 = { 47 }
		$p2_0 = { 47 }
		$p3_0 = { 47 }
		$p4_0 = { 47 }
		$p5_0 = { 47 }
		$p6_0 = { 47 }
		$p7_0 = { 47 }
		$p8_0 = { 47 }
		$p9_0 = { 47 }

	condition:
		(((prefix_size >= 389 and $p0_0 at 388) and (prefix_size >= 773 and $p2_0 at 772) and (prefix_size >= 5 and $p3_0 at 4) and (prefix_size >= 197 and $p6_0 at 196) and (prefix_size >= 581 and $p8_0 at 580)) or ((prefix_size >= 189 and $p1_0 at 188) and (prefix_size >= 565 and $p4_0 at 564) and (prefix_size >= 1 and $p5_0 at 0) and (prefix_size >= 753 and $p7_0 at 752) and (prefix_size >= 377 and $p9_0 at 376)))
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
		$p0_0 = { 93 4E 55 4D 50 59 }

	condition:
		(prefix_size >= 6 and original_size >= 6 and $p0_0 at 0)
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
		($p0_0 at 0)
}

rule taxonomy_orc
{
	meta:
        source_refs = "libmagic:magic/Magdir/apache:libmagic_11a425bf4c85f8783ee1_line_11"
		label = "orc"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

	strings:
		$p0_0 = { 4f 52 43 }

	condition:
		(prefix_size >= 3 and $p0_0 at 0)
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
		(prefix_size >= 4 and original_size >= 4 and $p0_0 at 0)
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
		(prefix_size >= 5 and $p0_0 at 0)
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

	strings:
		$p0_0 = { 38 42 50 53 00 01 }
		$p1_0 = "8BPS"
		$p2_0 = { 38 42 50 53 00 02 }
		$p3_0 = "8BPS  \\000\\000\\000\\000"

	condition:
		((prefix_size >= 6 and $p0_0 at 0) or (prefix_size >= 4 and original_size >= 4 and $p1_0 at 0) or (prefix_size >= 6 and $p2_0 at 0) or (prefix_size >= 22 and original_size >= 22 and $p3_0 at 0))
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
		$p0_0 = "qoif"

	condition:
		(prefix_size >= 4 and original_size >= 4 and $p0_0 at 0)
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

	strings:
		$p0_0 = { 52 61 72 21 1A 07 01 00 }
		$p1_0 = { 52 61 72 21 1A 07 00 }
		$p2_0 = "Rar!"
		$p3_0 = { 52 45 7e 5e }

	condition:
		((prefix_size >= 8 and original_size >= 8 and $p0_0 at 0) or (prefix_size >= 7 and original_size >= 7 and $p1_0 at 0) or (prefix_size >= 4 and original_size >= 4 and $p2_0 at 0) or (prefix_size >= 4 and $p3_0 at 0))
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

	strings:
		$p0_0 = { 52 45 44 49 53 }

	condition:
		(prefix_size >= 5 and $p0_0 at 0)
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

	strings:
		$p0_0 = { 52 5a 49 50 }

	condition:
		(prefix_size >= 4 and $p0_0 at 0)
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

	strings:
		$p0_0 = { 53 41 53 }
		$p1_0 = { 53 41 53 }

	condition:
		((prefix_size >= 87 and $p0_0 at 84) or (prefix_size >= 3 and $p1_0 at 0))
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

	strings:
		$p0_0 = "7z\\274\\257\\047\\034"
		$p1_0 = { 37 7A BC AF 27 1C }
		$p2_0 = "7z"
		$p3_0 = { BC AF 27 1C }

	condition:
		((prefix_size >= 18 and original_size >= 18 and $p0_0 at 0) or (prefix_size >= 6 and original_size >= 6 and $p1_0 at 0) or ((prefix_size >= 2 and $p2_0 in ( 0 .. 1 )) and (prefix_size >= 6 and $p3_0 in ( 2 .. 5 ))))
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
		(($p0_0 at 0) or ($p1_0 at 0))
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

	strings:
		$p0_0 = /\xff\xfe\xff\x0e\x53\x00\x6b\x00\x65\x00\x74\x00\x63\x00\x68\x00\x55\x00\x70\x00\x20\x00\x4d\x00\x6f\x00\x64\x00\x65\x00\x6c\x00\xff\xfe\xff(([\x00-\xff]){1}\x7b\x00\x32)/
		$p1_0 = /\xff\xfe\xff\x0e\x53\x00\x6b\x00\x65\x00\x74\x00\x63\x00\x68\x00\x55\x00\x70\x00\x20\x00\x4d\x00\x6f\x00\x64\x00\x65\x00\x6c\x00\xff\xfe\xff(([\x00-\xff]){1}\x7b\x00\x34)/
		$p2_0 = /\xff\xfe\xff\x0e\x53\x00\x6b\x00\x65\x00\x74\x00\x63\x00\x68\x00\x55\x00\x70\x00\x20\x00\x4d\x00\x6f\x00\x64\x00\x65\x00\x6c\x00\xff\xfe\xff(([\x00-\xff]){1}\x7b\x00\x31\x00\x34)/
		$p3_0 = /\xff\xfe\xff\x0e\x53\x00\x6b\x00\x65\x00\x74\x00\x63\x00\x68\x00\x55\x00\x70\x00\x20\x00\x4d\x00\x6f\x00\x64\x00\x65\x00\x6c\x00\xff\xfe\xff(([\x00-\xff]){1}\x7b\x00\x38)/
		$p4_0 = { ff fe ff 0e 53 00 6b 00 65 00 74 00 63 00 68 00 55 00 70 00 20 00 4d 00 6f 00 64 00 65 00 6c 00 }
		$p5_0 = /\xff\xfe\xff\x0e\x53\x00\x6b\x00\x65\x00\x74\x00\x63\x00\x68\x00\x55\x00\x70\x00\x20\x00\x4d\x00\x6f\x00\x64\x00\x65\x00\x6c\x00\xff\xfe\xff(([\x00-\xff]){1}\x7b\x00\x31)/
		$p6_0 = /\xff\xfe\xff\x0e\x53\x00\x6b\x00\x65\x00\x74\x00\x63\x00\x68\x00\x55\x00\x70\x00\x20\x00\x4d\x00\x6f\x00\x64\x00\x65\x00\x6c\x00\xff\xfe\xff(([\x00-\xff]){1}\x7b\x00\x33)/
		$p7_0 = /\xff\xfe\xff\x0e\x53\x00\x6b\x00\x65\x00\x74\x00\x63\x00\x68\x00\x55\x00\x70\x00\x20\x00\x4d\x00\x6f\x00\x64\x00\x65\x00\x6c\x00\xff\xfe\xff(([\x00-\xff]){1}\x7b\x00\x31\x00\x33)/
		$p8_0 = /\xff\xfe\xff\x0e\x53\x00\x6b\x00\x65\x00\x74\x00\x63\x00\x68\x00\x55\x00\x70\x00\x20\x00\x4d\x00\x6f\x00\x64\x00\x65\x00\x6c\x00\xff\xfe\xff(([\x00-\xff]){1}\x7b\x00\x31\x00\x35)/
		$p9_0 = /\xff\xfe\xff\x0e\x53\x00\x6b\x00\x65\x00\x74\x00\x63\x00\x68\x00\x55\x00\x70\x00\x20\x00\x4d\x00\x6f\x00\x64\x00\x65\x00\x6c\x00\xff\xfe\xff(([\x00-\xff]){1}\x7b\x00\x31\x00\x36)/
		$p10_0 = /\xff\xfe\xff\x0e\x53\x00\x6b\x00\x65\x00\x74\x00\x63\x00\x68\x00\x55\x00\x70\x00\x20\x00\x4d\x00\x6f\x00\x64\x00\x65\x00\x6c\x00\xff\xfe\xff(([\x00-\xff]){1}\x7b\x00\x36)/
		$p11_0 = /\xff\xfe\xff\x0e\x53\x00\x6b\x00\x65\x00\x74\x00\x63\x00\x68\x00\x55\x00\x70\x00\x20\x00\x4d\x00\x6f\x00\x64\x00\x65\x00\x6c\x00\xff\xfe\xff(([\x00-\xff]){1}\x7b\x00\x31\x00\x39)/
		$p12_0 = /\xff\xfe\xff\x0e\x53\x00\x6b\x00\x65\x00\x74\x00\x63\x00\x68\x00\x55\x00\x70\x00\x20\x00\x4d\x00\x6f\x00\x64\x00\x65\x00\x6c\x00\xff\xfe\xff(([\x00-\xff]){1}\x7b\x00\x31\x00\x37)/
		$p13_0 = /\xff\xfe\xff\x0e\x53\x00\x6b\x00\x65\x00\x74\x00\x63\x00\x68\x00\x55\x00\x70\x00\x20\x00\x4d\x00\x6f\x00\x64\x00\x65\x00\x6c\x00\xff\xfe\xff(([\x00-\xff]){1}\x7b\x00\x37)/
		$p14_0 = /\x0e\x53\x6b\x65\x74\x63\x68\x55\x70\x20\x4d\x6f\x64\x65\x6c\x08/
		$p15_0 = /\xff\xfe\xff\x0e\x53\x00\x6b\x00\x65\x00\x74\x00\x63\x00\x68\x00\x55\x00\x70\x00\x20\x00\x4d\x00\x6f\x00\x64\x00\x65\x00\x6c\x00\xff\xfe\xff(([\x00-\xff]){1}\x7b\x00\x35)/
		$p16_0 = /\xff\xfe\xff\x0e\x53\x00\x6b\x00\x65\x00\x74\x00\x63\x00\x68\x00\x55\x00\x70\x00\x20\x00\x4d\x00\x6f\x00\x64\x00\x65\x00\x6c\x00\xff\xfe\xff(([\x00-\xff]){1}\x7b\x00\x31\x00\x38)/

	condition:
		(($p0_0 at 0) or ($p1_0 at 0) or ($p2_0 at 0) or ($p3_0 at 0) or (prefix_size >= 32 and $p4_0 at 0) or ($p5_0 at 0) or ($p6_0 at 0) or ($p7_0 at 0) or ($p8_0 at 0) or ($p9_0 at 0) or ($p10_0 at 0) or ($p11_0 at 0) or ($p12_0 at 0) or ($p13_0 at 0) or ($p14_0 at 0) or ($p15_0 at 0) or ($p16_0 at 0))
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

	strings:
		$p0_0 = { 07 23 02 03 }
		$p1_0 = { 03 02 23 07 }

	condition:
		((prefix_size >= 4 and $p0_0 at 0) or (prefix_size >= 4 and $p1_0 at 0))
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
		((prefix_size >= 4 and original_size >= 4 and $p0_0 at 0) or (prefix_size >= 4 and $p1_0 at 0) or (prefix_size >= 4 and original_size >= 4 and $p2_0 at 0))
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

	strings:
		$p0_0 = "FWS"
		$p1_0 = "CWS"
		$p2_0 = "ZWS"

	condition:
		((prefix_size >= 3 and original_size >= 3 and $p0_0 at 0) or (prefix_size >= 3 and original_size >= 3 and $p1_0 at 0) or (prefix_size >= 3 and original_size >= 3 and $p2_0 at 0))
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

	strings:
		$p0_0 = { 55 46 32 0a }

	condition:
		(prefix_size >= 4 and $p0_0 at 0)
}

rule taxonomy_unixcompress
{
	meta:
        source_refs = "libmagic:magic/Magdir/compress:libmagic_d3a761d2dade747b519c_line_12; puremagic:puremagic/magic_data.json:headers[1047]; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[907]/magic[0]"
		label = "unixcompress"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

	strings:
		$p0_0 = "\\037\\235"
		$p1_0 = { 1F 9D }

	condition:
		((prefix_size >= 8 and original_size >= 8 and $p0_0 at 0) or (prefix_size >= 2 and original_size >= 2 and $p1_0 at 0))
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
		(prefix_size >= 8 and original_size >= 8 and $p0_0 at 0)
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

	strings:
		$p0_0 = "IWAD"
		$p1_0 = "PWAD"

	condition:
		((prefix_size >= 4 and original_size >= 4 and $p0_0 at 0) or (prefix_size >= 4 and original_size >= 4 and $p1_0 at 0))
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

	strings:
		$p0_0 = { 6D 73 61 00 }
		$p1_0 = { 00 61 73 6D }

	condition:
		((prefix_size >= 4 and $p0_0 at 0) or (prefix_size >= 4 and $p1_0 at 0))
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

	strings:
		$p0_0 = "WAVE"
		$p1_0 = "WAV "
		$p2_0 = "WAVEfmt "
		$p3_0 = { 52 46 36 34 ff ff ff ff 57 41 56 45 64 73 36 34 }

	condition:
		((prefix_size >= 12 and original_size >= 12 and $p0_0 at 8) or (prefix_size >= 12 and original_size >= 12 and $p1_0 at 8) or (prefix_size >= 16 and original_size >= 16 and $p2_0 at 8) or (prefix_size >= 16 and $p3_0 at 0))
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
		$p0_0 = "wOFF"

	condition:
		(prefix_size >= 4 and original_size >= 4 and $p0_0 at 0)
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
		$p0_0 = "wOF2"

	condition:
		(prefix_size >= 4 and original_size >= 4 and $p0_0 at 0)
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

	strings:
		$p0_0 = "xar!"

	condition:
		(prefix_size >= 4 and original_size >= 4 and $p0_0 at 0)
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

	strings:
		$p0_0 = "gimp xcf v"
		$p1_0 = "gimp xcf file"
		$p2_0 = "gimp xcf "
		$p3_0 = "gimp xcf"

	condition:
		((prefix_size >= 10 and original_size >= 10 and $p0_0 at 0) or (prefix_size >= 13 and original_size >= 13 and $p1_0 at 0) or (prefix_size >= 9 and original_size >= 9 and $p2_0 at 0) or (prefix_size >= 8 and original_size >= 8 and $p3_0 at 0))
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
		((prefix_size >= 4 and original_size >= 4 and $p0_0 at 0) or (prefix_size >= 4 and $p1_0 at 0) or (prefix_size >= 4 and $p2_0 at 0) or (prefix_size >= 4 and $p3_0 at 0) or (prefix_size >= 4 and $p4_0 at 0) or (prefix_size >= 4 and $p5_0 at 0) or (prefix_size >= 4 and $p6_0 at 0))
}
