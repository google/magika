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

    // ACE main header: fixed fields and advert-length byte precede optional data.
    strings:
        $magic = "**ACE**"
    condition:
        prefix_size >= 31 and $magic at 7 and uint16(2) >= 27 and
        uint8(4) == 0 and uint8(5) % 2 == 0
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
        source_refs = "libmagic:magic/Magdir/apache:libmagic_dcfba1374d0a8a3f71c0_line_7; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:3392; Apache Avro:1.12.0:Object Container Files"
		label = "avro"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0


    // Metadata must be nonempty (avro.schema is required); allow negative block counts.
    // Its ordering and size vary, so the schema need not appear within the scan window.
    strings:
        $magic = "Obj\x01"
    condition:
        prefix_size >= 21 and $magic at 0 and uint8(4) >= 1
}

rule taxonomy_bam
{
	meta:
        source_refs = "libmagic:magic/Magdir/bioinformatics:libmagic_5294848eb43d5add223c_line_43; HTSlib:sam.c:bam_hdr_read"
		label = "bam"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

	strings:
		$p0_0 = { 42 41 4d 01 }

	condition:
		prefix_size >= 12 and $p0_0 at 0
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

    // BPG 0.9.8: defined pixel formats/depths, color space and two nonzero ue7(32) dimensions.
    strings:
        $magic = { 42 50 47 FB }
        $format_depth = /[\x00-\x06\x10-\x16\x20-\x26\x30-\x36\x40-\x46\x50-\x56\x60-\x66\x70-\x76\x80-\x86\x90-\x96\xa0-\xa6\xb0-\xb6]/
        $dimensions = /([\x01-\x7f]|[\x81-\xff][\x80-\xff]{0,2}[\x00-\x7f]|[\x81-\x8f][\x80-\xff]{3}[\x00-\x7f]){2}[\x00-\xff]/
    condition:
        prefix_size >= 9 and $magic at 0 and $format_depth at 4 and uint8(5) < 80 and
        (uint8(4) >= 32 or uint8(5) < 16) and $dimensions at 6
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

rule taxonomy_collada
{
	meta:
		label = "collada"
        enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0
    // COLLADA digital asset: an XML document whose root element is <COLLADA>.
    strings:
        $xml = "<?xml"
        $root = "<COLLADA"
    condition:
        $xml at 0 and $root in (0 .. 256)
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

rule taxonomy_cubin
{
	meta:
        source_refs = "spec:ELF e_machine EM_CUDA (190)"
		label = "cubin"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    // An ELF image for the CUDA machine type, in either byte order.
    strings:
        $elf = { 7F 45 4C 46 }
    condition:
        prefix_size >= 20 and $elf at 0 and
        ((uint8(5) == 1 and uint16(18) == 190) or (uint8(5) == 2 and uint16be(18) == 190))
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

    // Buddy header plus space for allocator address/count fields; no fixed root address.
    strings:
        $magic = { 00 00 00 01 42 75 64 31 00 }
    condition:
        prefix_size >= 36 and $magic at 0 and uint32be(8) >= 32 and uint32be(12) >= 12
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

    // The on-disk main header occupies 4 KiB. Keep versions forward compatible, including 999.
    strings:
        $magic = "DUCK"
    condition:
        prefix_size >= 4096 and $magic at 8 and uint32(12) >= 1 and uint32(16) == 0
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

    // ESE database/stream header; keep revisions and page sizes unrestricted.
    strings:
        $magic = { EF CD AB 89 }
    condition:
        prefix_size >= 668 and $magic at 4 and uint32(8) >= 1 and
        uint32(12) <= 1 and uint32(132) == 0
}

rule taxonomy_fbx
{
	meta:
        source_refs = "libmagic:magic/Magdir/cad:libmagic_6357ac41d93e626ef3d0_line_378; spec:Autodesk FBX SDK ASCII header comment"
		label = "fbx"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    strings:
        // ufbx binary header and supported versions, including legacy 3000; little-endian variant.
        $magic = { 4B 61 79 64 61 72 61 20 46 42 58 20 42 69 6E 61 72 79 20 20 00 1A 00 }
        // ASCII export: the FBX SDK's header comment with a three-part version.
        $ascii = /; FBX [0-9]\.[0-9]\.[0-9] project file/

    condition:
        (prefix_size >= 27 and $magic at 0 and uint32(23) >= 3000 and uint32(23) <= 7700) or
        $ascii at 0
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

    // One complete FITS header block, SIMPLE value and second/third-card BITPIX.
    strings:
        $simple = /SIMPLE  = {1,20}[TF][ \/]/
        $bitpix = /BITPIX  = {1,20}(8|16|32|64|-32|-64)[ \/]/
    condition:
        prefix_size >= 2880 and $simple at 0 and uint16be(89) == 0x2020 and
        ($bitpix at 80 or $bitpix at 160)
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

rule taxonomy_geopackage
{
	meta:
        source_refs = "spec:OGC GeoPackage 1.x SQLite application_id"
		label = "geopackage"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    // A SQLite database whose application_id (offset 68) is GPKG, GP10 or GP11.
    strings:
        $sqlite = "SQLite format 3\x00"
    condition:
        prefix_size >= 100 and $sqlite at 0 and
        (uint32be(68) == 1196444487 or uint32be(68) == 1196437808 or uint32be(68) == 1196437809)
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
        source_refs = "libmagic:magic/Magdir/cad:libmagic_5e370f253c9f71913267_line_370; spec:Khronos glTF 2.0 JSON schema"
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
        // JSON glTF, anchored at the start so ordinary JSON fails at its first key: the object opens
        // with the accessor array, corroborated by a component type, or with a flat asset object
        // followed by a glTF top-level key or a KHR_/EXT_ extension list. A 3D Tiles tileset, which
        // also opens with an asset object, continues with its own keys and abstains.
        $open_accessors = /[ \t\r\n]{0,64}\{[ \t\r\n]{0,64}"accessors"[ \t\r\n]{0,16}:[ \t\r\n]{0,16}\[/
        $component_type = "\"componentType\""
        $asset_then_gltf = /[ \t\r\n]{0,64}\{[ \t\r\n]{0,64}"asset"[ \t\r\n]{0,16}:[ \t\r\n]{0,16}\{[^{}]{0,512}\}[ \t\r\n]{0,16},[ \t\r\n]{0,16}("(scene|scenes|nodes|accessors|bufferViews|buffers|meshes)"|"extensionsUsed"[ \t\r\n]{0,16}:[ \t\r\n]{0,16}\[[ \t\r\n]{0,64}"(KHR|EXT)_)/

    condition:
        (prefix_size >= 24 and $magic at 0 and uint32(8) >= 24 and uint32(8) % 4 == 0 and
         uint32(12) >= 4 and uint32(12) % 4 == 0 and $json at 16 and $object at 20) or
        ($open_accessors at 0 and $component_type in (0 .. 1024)) or
        $asset_then_gltf at 0
}

rule taxonomy_gpx
{
	meta:
		label = "gpx"
        enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0
    // GPS Exchange Format: an XML document whose root element is <gpx>.
    strings:
        $root = /<gpx[ \t\r\n]/
    condition:
        uint16(0) != 0x4B50 and $root in (0 .. 256)
}

rule taxonomy_grib
{
	meta:
        source_refs = "spec:WMO FM 92 GRIB editions 1 and 2"
		label = "grib"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    // GRIB indicator section: the GRIB magic and edition 1 or 2 at offset 7.
    strings:
        $grib = "GRIB"
    condition:
        prefix_size >= 16 and $grib at 0 and (uint8(7) == 1 or uint8(7) == 2)
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
        source_refs = "libmagic:magic/Magdir/images:libmagic_4344afb37c71c603cbd9_line_2556; HDFGroup:hdf4/hdf/src/hfiledd.c:HTPstart"
		label = "hdf4"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

	strings:
		$p0_0 = { 0E 03 13 01 }

	condition:
		prefix_size >= 22 and $p0_0 at 0 and uint16be(4) >= 1 and uint16be(4) <= 32767
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

rule taxonomy_hve
{
	meta:
        source_refs = "spec:Windows registry hive base block (regf)"
		label = "hve"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    // Registry base block: regf, major version 1, minor at most 6, primary file type, direct memory load format.
    strings:
        $regf = "regf"
    condition:
        prefix_size >= 48 and $regf at 0 and uint32(20) == 1 and uint32(24) <= 6 and
        uint32(28) == 0 and uint32(32) == 1
}

rule taxonomy_icc
{
	meta:
        source_refs = "puremagic:puremagic/magic_data.json:headers[959]; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[426]/magic[0]; LittleCMS:src/cmsio0.c:_cmsReadHeader/validDeviceClass"
		label = "icc"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0


    // Complete profile header and tag count. Retain historical zero device class.
    strings:
        $magic = "acsp"
        $device = /(\x00{4}|scnr|mntr|prtr|link|abst|spac|nmcl|cenc|mid |mlnk|mvis)/
    condition:
        prefix_size >= 132 and $magic at 36 and $device at 12
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

rule taxonomy_jng
{
	meta:
        source_refs = "spec:JNG (JPEG Network Graphics) 1.0"
		label = "jng"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    // JNG signature followed by a 16-byte JHDR chunk.
    strings:
        $header = { 8B 4A 4E 47 0D 0A 1A 0A 00 00 00 10 4A 48 44 52 }
    condition:
        prefix_size >= 16 and $header at 0
}

rule taxonomy_kml
{
	meta:
		label = "kml"
        enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0
    // Keyhole Markup Language: an XML document whose root element is <kml>.
    strings:
        $root = /<kml[ \t\r\n>]/
    condition:
        uint16(0) != 0x4B50 and $root in (0 .. 256)
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

    // Raw streams cannot begin with END_BLOCK. Wrappers carry offset/size before payload.
    strings:
        $raw = { 42 43 C0 DE }
        $wrapper = { DE C0 17 0B }
    condition:
        original_size % 4 == 0 and
        ((prefix_size >= 8 and $raw at 0 and
          (uint8(4) % 4 == 1 or uint8(4) % 4 == 2 or uint8(4) % 4 == 3)) or
         (prefix_size >= 20 and $wrapper at 0 and uint32(8) >= 16 and uint32(12) >= 4))
}

rule taxonomy_lnk
{
	meta:
        source_refs = "libmagic:magic/Magdir/windows:libmagic_47fbb4ad20a117b8eb73_line_714; Wine:dlls/shell32/shelllink.c:IPersistStream_fnLoad; MS-SHLLINK:ShellLinkHeader"
		label = "lnk"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

	strings:
		$p0_0 = { 4c 00 00 00 01 14 02 00 00 00 00 00 c0 00 00 00 00 00 00 46 }

	condition:
		prefix_size >= 76 and $p0_0 at 0
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

    // LRZIP's 24-byte version-zero header; newer minor versions reuse flag bytes.
    strings:
        $magic = "LRZI"
    condition:
        prefix_size >= 24 and $magic at 0 and uint8(4) == 0 and uint8(5) >= 1
}

rule taxonomy_luabytecode
{
	meta:
        source_refs = "libmagic:magic/Magdir/lua:libmagic_328f668b9e354b26ac15_line_21; spec:LuaJIT bytecode dump format (lj_bcdump.h)"
		label = "luabytecode"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    // Lua's own loaders: retain legacy layouts and configured numeric representations.
    // Modern chunks carry format, size and binary conversion-check fields, including LNUM modes.
    // LuaJIT dumps: ESC L J, version 1 or 2 and known flag bits, then either the first
    // stripped prototype (length, flags, parameter count, frame size) or a chunk name at @ or =.
    strings:
        $v24 = { 1B 4C 75 61 23 (12 34 | 34 12) }
        $v25 = { 1B 4C 75 61 25 02 04 ?? (12 34 | 34 12) }
        $v31 = { 1B 4C 75 61 31 (6C | 66 | 64 | 3F) }
        $v32 = { 1B 4C 75 61 32 }
        $v40 = /\x1bLua\x40[\x00\x01][\x01-\xff]{7}/
        $v50 = /\x1bLua\x50[\x00\x01][\x01-\xff]{8}/
        $v51 = /\x1bLua\x51\x00[\x00\x01][\x01-\xff]{4}[\x00\x01\x02\x04\x08\x82\x84\x88]/
        $v52 = /\x1bLua\x52\x00[\x00\x01][\x01-\xff]{4}[\x00\x01\x02\x04\x08\x82\x84\x88]\x19\x93\x0d\x0a\x1a\x0a/
        $v53 = /\x1bLua\x53\x00\x19\x93\x0d\x0a\x1a\x0a[\x01-\xff]{5}/
        $v54 = /\x1bLua\x54\x00\x19\x93\x0d\x0a\x1a\x0a[\x01-\xff]{3}/
        $v55 = /\x1bLua\x55\x00\x19\x93\x0d\x0a\x1a\x0a[\x01-\xff]/
        $luajit_stripped = /\x1bLJ[\x01\x02][\x02\x03\x06\x07\x0a\x0b\x0e\x0f]([\x08-\x7f]|[\x80-\xff][\x01-\x7f]|[\x80-\xff][\x80-\xff][\x01-\x7f])[\x00-\x1f][\x00-\xff][\x01-\xfa]/
        $luajit_named = /\x1bLJ[\x01\x02][\x00\x01\x04\x05\x08\x09\x0c\x0d]([\x02-\x7f]|[\x80-\xff][\x01-\x7f])[@=][\x20-\x7e]/
    condition:
        prefix_size >= 8 and (
            ($v24 at 0 and prefix_size >= 11) or
            ($v25 at 0 and prefix_size >= 14 and uint8(7) >= 1) or
            ($v31 at 0 and uint8(6) >= 1) or $v32 at 0 or
            ($v40 at 0 and prefix_size >= 14) or ($v50 at 0 and prefix_size >= 15) or
            $v51 at 0 or $v52 at 0 or $v53 at 0 or $v54 at 0 or $v55 at 0 or
            $luajit_stripped at 0 or $luajit_named at 0
        )
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
        $legacy = { (02 | 03) 21 4C 18 }
        // Version 1; reserved bits clear; block sizes 64 KiB through 4 MiB.
        // Each alternative observes all optional fields and the header checksum.
        $basic = { 04 22 4D 18 (40 | 44 | 50 | 54 | 60 | 64 | 70 | 74) (40 | 50 | 60 | 70) ?? }
        $dictionary = { 04 22 4D 18 (41 | 45 | 51 | 55 | 61 | 65 | 71 | 75) (40 | 50 | 60 | 70) [4] ?? }
        $size = { 04 22 4D 18 (48 | 4C | 58 | 5C | 68 | 6C | 78 | 7C) (40 | 50 | 60 | 70) [8] ?? }
        $size_dictionary = { 04 22 4D 18 (49 | 4D | 59 | 5D | 69 | 6D | 79 | 7D) (40 | 50 | 60 | 70) [12] ?? }

    condition:
        prefix_size >= 8 and (
            $legacy at 0 or
            ($basic at 0 and original_size >= 11) or
            ($dictionary at 0 and original_size >= 15) or
            ($size at 0 and original_size >= 19) or
            ($size_dictionary at 0 and original_size >= 23)
        )
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

rule taxonomy_minidump
{
	meta:
        source_refs = "spec:Microsoft MINIDUMP_HEADER"
		label = "minidump"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    // MDMP signature, implementation version 0xA793, 1 to 255 streams, directory right after the header.
    strings:
        $header = { 4D 44 4D 50 93 A7 }
    condition:
        prefix_size >= 32 and $header at 0 and uint32(8) >= 1 and uint32(8) <= 255 and uint32(12) == 32
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

rule taxonomy_nrrd
{
	meta:
        source_refs = "spec:Teem NRRD file format versions 1 to 5"
		label = "nrrd"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    // NRRD magic with a format version and a line break.
    strings:
        $nrrd = /NRRD000[1-5]\r?\n/
    condition:
        $nrrd at 0
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



rule taxonomy_palmos
{
	meta:
        source_refs = "spec:Palm OS PRC/PDB database header"
		label = "palmos"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    // Palm OS application database: NUL-terminated 32-byte name, type appl, printable creator, records present.
    strings:
        $appl = "appl"
    condition:
        prefix_size >= 78 and uint8(31) == 0 and $appl at 60 and uint16be(76) >= 1 and
        uint8(64) >= 32 and uint8(64) <= 126 and uint8(65) >= 32 and uint8(65) <= 126 and
        uint8(66) >= 32 and uint8(66) <= 126 and uint8(67) >= 32 and uint8(67) <= 126
}

rule taxonomy_parquet
{
	meta:
        source_refs = "libmagic:magic/Magdir/apache:libmagic_db6156119e2e23b734e3_line_23; puremagic:puremagic/magic_data.json:headers[1024]; Apache Parquet:File Format; Apache Arrow:parquet/file_reader.cc:ParseFooterLength"
		label = "parquet"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0


    // Leading magic plus the eight-byte footer require at least twelve bytes overall.
    strings:
        $magic = "PAR1"
    condition:
        prefix_size >= 12 and $magic at 0
}

rule taxonomy_pbm
{
	meta:
        source_refs = "spec:Netpbm P1 to P6 headers"
		label = "pbm"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    // Netpbm magic, at most two comment lines, then width and height.
    strings:
        $header = /P[1-6][ \t\r\n]{1,8}(#[^\n]{0,128}\n[ \t\r\n]{0,4}){0,2}[0-9]{1,6}[ \t\r\n]{1,4}[0-9]{1,6}[ \t\r\n]/
    condition:
        $header at 0
}

rule taxonomy_pgp
{
	meta:
        source_refs = "spec:OpenPGP ASCII armor (RFC 9580)"
		label = "pgp"
        enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0
    // OpenPGP ASCII armor header; the PEM rule excludes these, so they fall through to the model.
    strings:
        $armor = "-----BEGIN PGP "
    condition:
        $armor at 0
}

rule taxonomy_ply
{
	meta:
        source_refs = "spec:PLY polygon file format"
		label = "ply"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    // PLY magic line followed by a format line of a known encoding and version 1.0.
    strings:
        $header = /ply\r?\nformat (ascii|binary_little_endian|binary_big_endian) 1\.0[ \t\r\n]/
    condition:
        $header at 0
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

    // PostgreSQL 1.0 omits revision; 1.7 adds offset width. Keep newer minor versions.
    strings:
        $magic = "PGDMP"
        $format = { (01 | 03 | 05) }
    condition:
        prefix_size >= 9 and $magic at 0 and uint8(5) == 1 and
        ((uint8(6) == 0 and uint8(7) >= 1 and uint8(7) <= 32 and $format at 8) or
         (prefix_size >= 10 and uint8(6) >= 1 and uint8(6) <= 6 and
          uint8(8) >= 1 and uint8(8) <= 32 and $format at 9) or
         (prefix_size >= 11 and uint8(6) >= 7 and uint8(8) >= 1 and uint8(8) <= 32 and
          uint8(9) >= 1 and uint8(9) <= 32 and $format at 10))
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

rule taxonomy_safetensors
{
	meta:
        source_refs = "spec:Hugging Face safetensors header"
		label = "safetensors"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    // Little-endian 64-bit header length below 4 GiB, a JSON object, and a dtype key early in it.
    strings:
        $open = { 7B 22 }
        $dtype = "\"dtype\""
    condition:
        prefix_size >= 16 and uint32(4) == 0 and $open at 8 and $dtype in (10..512)
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

    // PRONOM main/index signatures plus complete first record/entry and legal header type.
    strings:
        $header = { 00 00 27 0A [20] ?? ?? ?? ?? E8 03 00 00 }
        $reserved = { 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 }
        $shape = { (00 | 01 | 03 | 05 | 08 | 0B | 0D | 0F | 12 | 15 | 17 | 19 | 1C | 1F) 00 00 00 }
        $first = { 00 00 00 (01 | 32) }
    condition:
        prefix_size >= 108 and $header at 0 and $reserved at 4 and $shape at 32 and
        uint32be(24) >= 54 and $first at 100
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

    // SAV/ZSAV fixed header, byte order, layout and compression. Portable header is larger.
    strings:
        $sav = { 24 46 4C (32 | 33) }
        $portable = { C9 C3 E2 C1 }
    condition:
        (prefix_size >= 176 and $sav at 0 and
         (((uint32(64) == 2 or uint32(64) == 3) and uint32(72) <= 2) or
          ((uint32be(64) == 2 or uint32be(64) == 3) and uint32be(72) <= 2))) or
        (prefix_size >= 464 and $portable at 0)
}

rule taxonomy_step
{
	meta:
        source_refs = "spec:ISO 10303-21 STEP exchange file header"
		label = "step"
        enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0
    // A STEP exchange file opens with the ISO 10303-21 header keyword.
    strings:
        $iso = "ISO-10303-21;"
    condition:
        $iso at 0
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

    // Complete leading copy of the VHD footer; fixed images with only a trailing footer abstain.
    strings:
        $magic = "conectix"
    condition:
        prefix_size >= 512 and $magic at 0 and
        (uint32be(8) == 2 or uint32be(8) == 3) and uint32be(12) == 0x10000 and
        uint32be(60) >= 2 and uint32be(60) <= 4 and uint8(84) <= 1
}

rule taxonomy_vib
{
	meta:
        source_refs = "spec:VMware Installation Bundle (ar archive, descriptor.xml VIB descriptor)"
		label = "vib"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    // A Unix ar archive whose first member is descriptor.xml holding a <vib version="..."> root.
    // The ar magic is 8 bytes and the first member header is a fixed 60 bytes, so the descriptor
    // content begins at offset 68.
    strings:
        $ar = "!<arch>\n"
        $name = "descriptor.xml"
        $vib = "<vib version"
    condition:
        prefix_size >= 80 and $ar at 0 and $name at 8 and $vib at 68
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
        $legacy = { (22 | 23 | 24 | 25 | 26 | 27) B5 2F FD }
        // Reserved bit 3 must be clear; the unused bit 4 remains unrestricted.
        $modern = { 28 B5 2F FD (?0 | ?1 | ?2 | ?3 | ?4 | ?5 | ?6 | ?7) }

    condition:
        prefix_size >= 8 and (
            $legacy at 0 or (prefix_size >= 9 and $modern at 0)
        )
}

// Prefix signatures for labels the model cannot output or often misses: adjudicated combined corpus,
// 25,421 whole files (rules/QUALITY.md, "Prefix signatures for labels the model lacks or misses").

rule taxonomy_ani
{
	meta:
        source_refs = "spec:Microsoft RIFF ACON animated cursor"
		label = "ani"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    // RIFF form type ACON; plain RIFF WAVE, AVI and WebP carry other form types.
    strings:
        $riff = "RIFF"
    condition:
        prefix_size >= 12 and $riff at 0 and uint32be(8) == 0x41434F4E
}

rule taxonomy_arrow
{
	meta:
        source_refs = "spec:Apache Arrow IPC file format (ARROW1 magic)"
		label = "arrow"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    // Arrow IPC file (Feather v2): ARROW1 magic padded to eight bytes.
    strings:
        $magic = { 41 52 52 4F 57 31 00 00 }
    condition:
        prefix_size >= 12 and $magic at 0
}

rule taxonomy_pcapng
{
	meta:
        source_refs = "spec:IETF pcapng Section Header Block"
		label = "pcapng"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    // Section Header Block in either byte order: block type, byte-order magic and major version 1.
    strings:
        $le = { 0A 0D 0D 0A ?? ?? ?? ?? 4D 3C 2B 1A 01 00 }
        $be = { 0A 0D 0D 0A ?? ?? ?? ?? 1A 2B 3C 4D 00 01 }
    condition:
        prefix_size >= 28 and ($le at 0 or $be at 0)
}

rule taxonomy_xcoff
{
	meta:
        source_refs = "spec:IBM AIX XCOFF32 and XCOFF64 file headers"
		label = "xcoff"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    // XCOFF64 (0x01F7) or XCOFF32 (0x01DF) with a sane section count and a known auxiliary header size.
    condition:
        prefix_size >= 24 and uint16be(2) >= 1 and uint16be(2) <= 1024 and
        ((uint16be(0) == 503 and (uint16be(16) == 120 or uint16be(16) == 0)) or
         (uint16be(0) == 479 and (uint16be(16) == 72 or uint16be(16) == 28 or uint16be(16) == 0)))
}
