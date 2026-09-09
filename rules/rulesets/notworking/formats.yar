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

rule taxonomy_access
{
	meta:
        source_refs = "pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:2143; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:2144; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:2145; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:2146; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:2147; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:270"
		label = "access"
		enforced = false
        class = "not-working"
        fp_rate = 0
        fn_rate = 1

	strings:
		$p0_0 = /\x77\x2c\x53\x20(([\x00-\xff]){1030}\x66)/
		$p1_0 = /\x01(\x00){3}([\x00-\xff]){1026}\x52\x69\x63\x68\x09/
		$p2_0 = /\x01(\x00){3}([\x00-\xff]){1026}\x52\x69\x63\x68\x08/
		$p3_0 = /\x01(\x00){3}([\x00-\xff]){1026}\x52\x69\x63\x68\x07/
		$p4_0 = /\x77\x2c\x53\x20(([\x00-\xff]){1030}\x68)/
		$p5_0 = /\x77\x2c\x53\x20(([\x00-\xff]){1030}\x69)/

	condition:
		(($p0_0 at 0) or ($p1_0 at 0) or ($p2_0 at 0) or ($p3_0 at 0) or ($p4_0 at 0) or ($p5_0 at 0))
}

rule taxonomy_ani
{
	meta:
        source_refs = "libmagic:magic/Magdir/riff:libmagic_b2aac9f8242bb4da4291_line_666; puremagic:puremagic/magic_data.json:headers[516]"
		label = "ani"
		enforced = false
        class = "not-working"
        fp_rate = 0.0101281310539374
        fn_rate = 0

	strings:
		$p0_0 = "RIFF"

	condition:
		(prefix_size >= 4 and original_size >= 4 and $p0_0 at 0)
}

rule taxonomy_aout
{
	meta:
        source_refs = "libmagic:magic/Magdir/aout:libmagic_11b4e392c9eac92690e4_line_41; libmagic:magic/Magdir/aout:libmagic_1dc19790634f9784cdc5_line_44; libmagic:magic/Magdir/aout:libmagic_4926e9be1b2eb8c724fa_line_26; libmagic:magic/Magdir/aout:libmagic_5b7dea12e239acf9e147_line_22; libmagic:magic/Magdir/aout:libmagic_dd47ef40679a9b69048c_line_38; libmagic:magic/Magdir/aout:libmagic_ebc1d5a9b11601daa6b5_line_18"
		label = "aout"
		enforced = false
        class = "not-working"
        fp_rate = 0
        fn_rate = "unmeasured"

	strings:
		$p0_0 = { 0b 01 00 00 }
		$p1_0 = { 00 00 01 0b }
		$p2_0 = { 00 00 01 08 }
		$p3_0 = { 00 00 01 07 }
		$p4_0 = { 08 01 00 00 }
		$p5_0 = { 07 01 00 00 }

	condition:
		((prefix_size >= 4 and $p0_0 at 0) or (prefix_size >= 4 and $p1_0 at 0) or (prefix_size >= 4 and $p2_0 at 0) or (prefix_size >= 4 and $p3_0 at 0) or (prefix_size >= 4 and $p4_0 at 0) or (prefix_size >= 4 and $p5_0 at 0))
}

rule taxonomy_arj
{
	meta:
        source_refs = "puremagic:puremagic/magic_data.json:headers[611]"
		label = "arj"
		enforced = false
        class = "not-working"
        fp_rate = 0
        fn_rate = "unmeasured"

	strings:
		$p0_0 = { 60 EA }

	condition:
		(prefix_size >= 2 and original_size >= 2 and $p0_0 at 0)
}

rule taxonomy_arrow
{
	meta:
        source_refs = "libmagic:magic/Magdir/apache:libmagic_c24b0fc3b5e0d2781f77_line_16"
		label = "arrow"
		enforced = false
        class = "not-working"
        fp_rate = 0
        fn_rate = "unmeasured"

	strings:
		$p0_0 = { 41 52 52 4f 57 31 }

	condition:
		(prefix_size >= 6 and $p0_0 at 0)
}

rule taxonomy_asf
{
	meta:
        source_refs = "pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:80"
		label = "asf"
		enforced = false
        class = "not-working"
        fp_rate = 0
        fn_rate = 1

	strings:
		$p0_0 = /\x30\x26\xb2\x75\x8e\x66\xcf\x11\xa6\xd9\x00\xaa\x00\x62\xce\x6c(([\x00-\xff]){12}\x01\x02)/

	condition:
		($p0_0 at 0)
}

rule taxonomy_berkeleydb
{
	meta:
        source_refs = "libmagic:magic/Magdir/database:libmagic_00186f0debd39e661df3_line_63; libmagic:magic/Magdir/database:libmagic_1641e84330e495c32c9c_line_70; libmagic:magic/Magdir/database:libmagic_191f061b53f8a2127688_line_65; libmagic:magic/Magdir/database:libmagic_1d9e9adb2cc11dedd5ff_line_77; libmagic:magic/Magdir/database:libmagic_2521f8fe9a9f04bfb34a_line_87; libmagic:magic/Magdir/database:libmagic_297d02841093879dbe9c_line_56; libmagic:magic/Magdir/database:libmagic_2e87c5991f2112dea334_line_58; libmagic:magic/Magdir/database:libmagic_41c6b9b086568846dada_line_72; libmagic:magic/Magdir/database:libmagic_42bd520b948c985d4b50_line_79; libmagic:magic/Magdir/database:libmagic_605095dd5fb581af48f2_line_67; libmagic:magic/Magdir/database:libmagic_7342e797abce01663bed_line_60; libmagic:magic/Magdir/database:libmagic_7e219a3a21cfe4187bd4_line_46; libmagic:magic/Magdir/database:libmagic_b5fc2feb318d79429e23_line_85; libmagic:magic/Magdir/database:libmagic_cdf902704174362f62e3_line_74; libmagic:magic/Magdir/database:libmagic_e1b5a9b49e9ea4326e11_line_89; libmagic:magic/Magdir/database:libmagic_e6c7da69316252f3a3ad_line_81"
		label = "berkeleydb"
		enforced = false
        class = "not-working"
        fp_rate = 0
        fn_rate = "unmeasured"

	strings:
		$p0_0 = { 61 15 06 00 }
		$p1_0 = { 00 06 15 61 }
		$p2_0 = { 53 22 04 00 }
		$p3_0 = { 00 04 09 88 }
		$p4_0 = { 62 31 05 00 }
		$p5_0 = { 00 05 31 62 }
		$p6_0 = { 00 05 31 62 }
		$p7_0 = { 62 31 05 00 }
		$p8_0 = { 00 04 22 53 }
		$p9_0 = { 88 09 04 00 }
		$p10_0 = { 00 06 15 61 }

	condition:
		((prefix_size >= 16 and $p0_0 at 12) or (prefix_size >= 4 and $p1_0 at 0) or (prefix_size >= 16 and $p2_0 at 12) or (prefix_size >= 16 and $p3_0 at 12) or (prefix_size >= 4 and $p4_0 at 0) or (prefix_size >= 16 and $p5_0 at 12) or (prefix_size >= 4 and $p6_0 at 0) or (prefix_size >= 16 and $p7_0 at 12) or (prefix_size >= 16 and $p8_0 at 12) or (prefix_size >= 16 and $p9_0 at 12) or (prefix_size >= 16 and $p10_0 at 12))
}

rule taxonomy_bzip3
{
	meta:
        source_refs = ""
		label = "bzip3"
		enforced = false
        class = "not-working"
        fp_rate = 0
        fn_rate = "unmeasured"

	condition:
		false
}

rule taxonomy_cinema4d
{
	meta:
        source_refs = "pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:1562; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:846; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:847"
		label = "cinema4d"
		enforced = false
        class = "not-working"
        fp_rate = 0
        fn_rate = "unmeasured"

	strings:
		$p0_0 = /\x46\x4f\x52\x4d(([\x00-\xff]){4}\x4d\x43\x34\x44)/
		$p1_0 = /\x43\x34\x44\x43\x34\x44\x36/
		$p2_0 = /\x4d\x43\x35\x30(([\x00-\xff]){4}\x43\x41\x54\x35|([\x00-\xff]){4}\x44\x4f\x4b\x35|([\x00-\xff]){4}\x46\x43\x56\x35|([\x00-\xff]){4}\x50\x52\x46\x35)/

	condition:
		(($p0_0 at 0) or ($p1_0 at 1) or ($p2_0 at 0))
}

rule taxonomy_coff
{
	meta:
        source_refs = ""
		label = "coff"
		enforced = false
        class = "not-working"
        fp_rate = 0
        fn_rate = 1

	condition:
		false
}

rule taxonomy_crt
{
	meta:
        source_refs = ""
		label = "crt"
		enforced = false
        class = "not-working"
        fp_rate = 0
        fn_rate = 1

	condition:
		false
}

rule taxonomy_deb
{
	meta:
        source_refs = "libmagic:magic/Magdir/archive:libmagic_702322410f7e43e3222a_line_480; libmagic:magic/Magdir/archive:libmagic_702322410f7e43e3222a_line_484; puremagic:puremagic/magic_data.json:headers[93]"
		label = "deb"
		enforced = false
        class = "not-working"
        fp_rate = 0.00339870169595215
        fn_rate = 0

	strings:
		$p0_0 = { 2d 62 69 6e 61 72 79 }
		$p1_0 = { 21 3c 61 72 63 68 3e 0a 64 65 62 69 61 6e }
		$p2_0 = "!<arch>"
		$p3_0 = { 2d 73 70 6c 69 74 }

	condition:
		(((prefix_size >= 14 and $p1_0 at 0) and ((prefix_size >= 21 and $p0_0 at 14) or (prefix_size >= 20 and $p3_0 at 14))) or (prefix_size >= 7 and original_size >= 7 and $p2_0 at 0))
}

rule taxonomy_dmg
{
	meta:
        source_refs = "libmagic:magic/Magdir/apple:libmagic_5b3bb1625131bd991d5d_line_529"
		label = "dmg"
		enforced = false
        class = "not-working"
        fp_rate = 0.00006797403391904
        fn_rate = 1

	strings:
		$p0_0 = { 45 52 }
		$p1_0 = { ( 00 | 02 | 04 | 06 | 08 | 0a | 0c | 0e ) 00 }

	condition:
		((prefix_size >= 2 and $p0_0 at 0) and (prefix_size >= 4 and $p1_0 at 2))
}

rule taxonomy_doc
{
	meta:
        source_refs = "libmagic:magic/Magdir/msdos:libmagic_5d161591fd29fe13e3fe_line_1831; libmagic:magic/Magdir/msdos:libmagic_5d161591fd29fe13e3fe_line_1834; libmagic:magic/Magdir/msdos:libmagic_5d161591fd29fe13e3fe_line_1837; libmagic:magic/Magdir/msdos:libmagic_5d161591fd29fe13e3fe_line_1840; puremagic:puremagic/magic_data.json:headers[212]; puremagic:puremagic/magic_data.json:headers[220]; puremagic:puremagic/magic_data.json:headers[221]; puremagic:puremagic/magic_data.json:headers[341]; puremagic:puremagic/magic_data.json:headers[346]; puremagic:puremagic/magic_data.json:headers[486]; puremagic:puremagic/magic_data.json:headers[518]; puremagic:puremagic/magic_data.json:headers[662]; puremagic:puremagic/magic_data.json:headers[663]; puremagic:puremagic/magic_data.json:headers[664]; puremagic:puremagic/magic_data.json:headers[665]; puremagic:puremagic/magic_data.json:headers[666]; puremagic:puremagic/magic_data.json:headers[667]; puremagic:puremagic/magic_data.json:headers[668]"
		label = "doc"
		enforced = false
        class = "not-working"
        fp_rate = 0.01920266458212963
        fn_rate = 0

	strings:
		$p0_0 = { EC A5 C1 00 }
		$p1_0 = { DB A5 2D 00 }
		$p2_0 = { CF 11 E0 A1 B1 1A E1 00 }
		$p3_0 = { FE 37 00 23 }
		$p4_0 = "jbjb"
		$p5_0 = "bjbj"
		$p6_0 = "\\376\\067\\0\\043"
		$p7_0 = { D0 CF 11 E0 A1 B1 1A E1 }
		$p8_0 = { 0D 44 4F 43 }
		$p9_0 = "Microsoft Word document data"
		$p10_0 = { fe 37 00 1c }
		$p11_0 = "\\333\\245-\\0\\0\\0"
		$p12_0 = "\\x31\\xbe\\x00\\x00"
		$p13_0 = { 00 00 00 00 }
		$p14_0 = "PO^Q`"
		$p15_0 = { fe 34 00 00 }
		$p16_0 = { fe 32 00 00 }

	condition:
		(((prefix_size >= 8 and $p13_0 at 4) and ((prefix_size >= 4 and $p3_0 at 0) or (prefix_size >= 4 and $p10_0 at 0) or (prefix_size >= 4 and $p15_0 at 0) or (prefix_size >= 4 and $p16_0 at 0))) or ((prefix_size >= 516 and original_size >= 516 and $p0_0 at 512) or (prefix_size >= 4 and original_size >= 4 and $p1_0 at 0) or (prefix_size >= 8 and original_size >= 8 and $p2_0 at 0) or (prefix_size >= 550 and original_size >= 550 and $p4_0 at 546) or (prefix_size >= 550 and original_size >= 550 and $p5_0 at 546) or (prefix_size >= 14 and original_size >= 14 and $p6_0 at 0) or (prefix_size >= 8 and original_size >= 8 and $p7_0 at 0) or (prefix_size >= 4 and original_size >= 4 and $p8_0 at 0) or (prefix_size >= 2140 and original_size >= 2140 and $p9_0 at 2112) or (prefix_size >= 15 and original_size >= 15 and $p11_0 at 0) or (prefix_size >= 16 and original_size >= 16 and $p12_0 at 0) or (prefix_size >= 5 and original_size >= 5 and $p14_0 at 0)))
}

rule taxonomy_docx
{
	meta:
        source_refs = "puremagic:puremagic/magic_data.json:headers[66]; puremagic:puremagic/magic_data.json:headers[967]"
		label = "docx"
		enforced = false
        class = "not-working"
        fp_rate = 0.0653910206301193
        fn_rate = 0

	strings:
		$p0_0 = { 50 4B 03 04 14 00 06 00 }
		$p1_0 = { 50 4B 03 04 }

	condition:
		((prefix_size >= 8 and original_size >= 8 and $p0_0 at 0) or (prefix_size >= 4 and original_size >= 4 and $p1_0 at 0))
}

rule taxonomy_dotx
{
	meta:
        source_refs = "puremagic:puremagic/magic_data.json:headers[74]"
		label = "dotx"
		enforced = false
        class = "not-working"
        fp_rate = 0.0653910206301193
        fn_rate = 0

	strings:
		$p0_0 = { 50 4B 03 04 }

	condition:
		(prefix_size >= 4 and original_size >= 4 and $p0_0 at 0)
}

rule taxonomy_elf
{
	meta:
        source_refs = "libmagic:magic/Magdir/elf:libmagic_4b52611fe434ecb91044_line_349"
		label = "elf"
		enforced = false
        class = "not-working"
        fp_rate = 0.00339870169595215
        fn_rate = 0.01

	strings:
		$p0_0 = { 7F 45 4C 46 }

	condition:
		(prefix_size >= 4 and original_size >= 4 and $p0_0 at 0)
}

rule taxonomy_filemaker
{
	meta:
        source_refs = "pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:1617; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:690; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:789; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:790"
		label = "filemaker"
		enforced = false
        class = "not-working"
        fp_rate = 0
        fn_rate = "unmeasured"

	strings:
		$p0_0 = /\xc0\x48\x42\x41\x4d\x37([\x00-\xff]){505}\x48\x42\x41\x4d\x32\x31\x30\x31\x4f\x43\x54(\x39){2}\xc1\x02\x48\x07\x50\x72\x6f\x20\x37\x2e\x30(\xc0){2}/
		$p1_0 = /\x00\x01(\x00){3}\x02\x00\x01\x00\x05\x00\x02\x00\x02\xc0(([\x00-\xff]){527}\x50\x72\x6f\x20\x33)/
		$p2_0 = /(\xc0\x48\x42\x41\x4d\x37([\x00-\xff]){0,512})\x48\x42\x41\x4d\x32\x31\x32\x35\x4a\x41\x4e(\x31){2}\xc1\x02\x48\x08\x50\x72\x6f\x20\x31\x32\x2e\x30(\xc0){2}/
		$p3_0 = /\x00\x01(\x00){3}\x02\x00\x01\x00\x05\x00\x02\x00\x02\xc0(([\x00-\xff]){527}\x50\x72\x6f\x20\x35)/

	condition:
		(($p0_0 at 14) or ($p1_0 at 0) or ($p2_0 in ( 0 .. 14 )) or ($p3_0 at 0))
}

rule taxonomy_flatgeobuf
{
	meta:
        source_refs = "libmagic:magic/Magdir/geo:libmagic_2678a3fdff08dc7ae6bf_line_170"
		label = "flatgeobuf"
		enforced = false
        class = "not-working"
        fp_rate = 0
        fn_rate = "unmeasured"

	strings:
		$p0_0 = { 66 67 62 03 66 67 62 01 }

	condition:
		(prefix_size >= 8 and $p0_0 at 0)
}

rule taxonomy_h5
{
	meta:
        source_refs = "libmagic:magic/Magdir/images:libmagic_424c48b2e0eff1402ff4_line_2559; libmagic:magic/Magdir/images:libmagic_6cf63e77154f73b63b0a_line_2569; libmagic:magic/Magdir/images:libmagic_7e67f9be64d21f766bb7_line_2573; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:1172; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:302; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:303; puremagic:puremagic/magic_data.json:headers[1023]"
		label = "h5"
		enforced = false
        class = "not-working"
        fp_rate = 0.00197124698365225
        fn_rate = 0

	strings:
		$p0_0 = /\x89\x48\x44\x46\x0d\x0a\x1a\x0a\x02/
		$p1_0 = { 89 48 44 46 0D 0A 1A 0A }
		$p2_0 = /\x89\x48\x44\x46\x0d\x0a\x1a\x0a\x01/
		$p3_0 = /\x89\x48\x44\x46\x0d\x0a\x1a\x0a\x00/
		$p4_0 = { 89 48 44 46 0d 0a 1a 0a }
		$p5_0 = { 89 48 44 46 0d 0a 1a 0a }

	condition:
		(($p0_0 at 0) or (prefix_size >= 8 and original_size >= 8 and $p1_0 at 0) or ($p2_0 at 0) or ($p3_0 at 0) or (prefix_size >= 2056 and $p4_0 at 2048) or (prefix_size >= 1032 and $p5_0 at 1024))
}

rule taxonomy_hwp
{
	meta:
        source_refs = "puremagic:puremagic/magic_data.json:headers[947]; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[960]/magic[0]"
		label = "hwp"
		enforced = false
        class = "not-working"
        fp_rate = 0
        fn_rate = 1

	strings:
		$p0_0 = "HWP Document File"
		$p1_0 = "HWP Document File V"

	condition:
		((prefix_size >= 17 and original_size >= 17 and $p0_0 at 0) or (prefix_size >= 19 and $p1_0 at 0))
}

rule taxonomy_iso
{
	meta:
        source_refs = ""
		label = "iso"
		enforced = false
        class = "not-working"
        fp_rate = 0
        fn_rate = 1

	condition:
		false
}

rule taxonomy_jar
{
	meta:
        source_refs = "puremagic:puremagic/magic_data.json:headers[243]; puremagic:puremagic/magic_data.json:headers[244]; puremagic:puremagic/magic_data.json:headers[308]; puremagic:puremagic/magic_data.json:headers[555]"
		label = "jar"
		enforced = false
        class = "not-working"
        fp_rate = 0.0653910206301193
        fn_rate = 0

	strings:
		$p0_0 = { 5F 27 A8 89 }
		$p1_0 = { 50 4B 03 04 14 00 08 00 }
		$p2_0 = { 50 4B 03 04 14 00 08 00 08 00 }
		$p3_0 = { 50 4B 03 04 }

	condition:
		((prefix_size >= 4 and original_size >= 4 and $p0_0 at 0) or (prefix_size >= 8 and original_size >= 8 and $p1_0 at 0) or (prefix_size >= 10 and original_size >= 10 and $p2_0 at 0) or (prefix_size >= 4 and original_size >= 4 and $p3_0 at 0))
}

rule taxonomy_javabytecode
{
	meta:
        source_refs = "pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:209"
		label = "javabytecode"
		enforced = false
        class = "not-working"
        fp_rate = 0.00030588315263569
        fn_rate = 0

	strings:
		$p0_0 = { CA FE BA BE }

	condition:
		(prefix_size >= 4 and original_size >= 4 and $p0_0 at 0)
}

rule taxonomy_jpeg
{
	meta:
        source_refs = "libmagic:magic/Magdir/jpeg:libmagic_eb4b179e9e76d7671286_line_17; puremagic:puremagic/magic_data.json:headers[140]; puremagic:puremagic/magic_data.json:headers[283]; puremagic:puremagic/magic_data.json:headers[596]; puremagic:puremagic/magic_data.json:headers[597]; puremagic:puremagic/magic_data.json:headers[82]; puremagic:puremagic/magic_data.json:headers[857]; puremagic:puremagic/magic_data.json:headers[858]; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[1329]/magic[0]"
		label = "jpeg"
		enforced = false
        class = "not-working"
        fp_rate = 0.00336471467899262
        fn_rate = 0

	strings:
		$p0_0 = { FF D8 FF }
		$p1_0 = { 00 00 00 0C 6A 50 20 20 }
		$p2_0 = { FF D8 }
		$p3_0 = "\\377\\330\\377"
		$p4_0 = { ff d8 ff ?? }

	condition:
		((prefix_size >= 3 and original_size >= 3 and $p0_0 at 0) or (prefix_size >= 8 and original_size >= 8 and $p1_0 at 0) or (prefix_size >= 2 and original_size >= 2 and $p2_0 at 0) or (prefix_size >= 12 and original_size >= 12 and $p3_0 at 0) or (prefix_size >= 4 and $p4_0 at 0))
}

rule taxonomy_lha
{
	meta:
        source_refs = "puremagic:puremagic/magic_data.json:headers[598]; puremagic:puremagic/magic_data.json:headers[756]; puremagic:puremagic/magic_data.json:headers[757]; puremagic:puremagic/magic_data.json:headers[758]; puremagic:puremagic/magic_data.json:headers[759]; puremagic:puremagic/magic_data.json:headers[760]; puremagic:puremagic/magic_data.json:headers[761]; puremagic:puremagic/magic_data.json:headers[762]; puremagic:puremagic/magic_data.json:headers[763]; puremagic:puremagic/magic_data.json:headers[764]; puremagic:puremagic/magic_data.json:headers[765]; puremagic:puremagic/magic_data.json:headers[766]; puremagic:puremagic/magic_data.json:headers[767]; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[978]/magic[0]"
		label = "lha"
		enforced = false
        class = "not-working"
        fp_rate = 0.00003398701695952
        fn_rate = 0

	strings:
		$p0_0 = "-lh1-"
		$p1_0 = "-lhd-"
		$p2_0 = "-lzs-"
		$p3_0 = "-lz5-"
		$p4_0 = "-lh3-"
		$p5_0 = "-lh7-"
		$p6_0 = "-lh"
		$p7_0 = "-lh2-"
		$p8_0 = "-lz4-"
		$p9_0 = "-lh5-"
		$p10_0 = "-lh40-"
		$p11_0 = "-lh4-"
		$p12_0 = "-lh6-"
		$p13_0 = "-lh0-"
		$p14_0 = "-lh -"

	condition:
		((prefix_size >= 7 and original_size >= 7 and $p0_0 at 2) or (prefix_size >= 7 and original_size >= 7 and $p1_0 at 2) or (prefix_size >= 7 and original_size >= 7 and $p2_0 at 2) or (prefix_size >= 7 and original_size >= 7 and $p3_0 at 2) or (prefix_size >= 7 and original_size >= 7 and $p4_0 at 2) or (prefix_size >= 7 and $p5_0 at 2) or (prefix_size >= 5 and original_size >= 5 and $p6_0 at 2) or (prefix_size >= 7 and original_size >= 7 and $p7_0 at 2) or (prefix_size >= 7 and original_size >= 7 and $p8_0 at 2) or (prefix_size >= 7 and original_size >= 7 and $p9_0 at 2) or (prefix_size >= 8 and original_size >= 8 and $p10_0 at 2) or (prefix_size >= 7 and original_size >= 7 and $p11_0 at 2) or (prefix_size >= 7 and $p12_0 at 2) or (prefix_size >= 7 and original_size >= 7 and $p13_0 at 2) or (prefix_size >= 7 and original_size >= 7 and $p14_0 at 2))
}

rule taxonomy_lightwave
{
	meta:
        source_refs = "pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:1583"
		label = "lightwave"
		enforced = false
        class = "not-working"
        fp_rate = 0
        fn_rate = 1

	strings:
		$p0_0 = /\x46\x4f\x52\x4d(([\x00-\xff]){4}\x4c\x57\x4f\x42)/

	condition:
		($p0_0 at 0)
}

rule taxonomy_lmdb
{
	meta:
        source_refs = "libmagic:magic/Magdir/database:libmagic_b62e7a675cb83db76567_line_1018"
		label = "lmdb"
		enforced = false
        class = "not-working"
        fp_rate = 0
        fn_rate = "unmeasured"

	strings:
		$p0_0 = { de c0 ef be }

	condition:
		(prefix_size >= 20 and $p0_0 at 16)
}

rule taxonomy_lzx
{
	meta:
        source_refs = "libmagic:magic/Magdir/amigaos:libmagic_03f6671abda59a3aee97_line_195"
		label = "lzx"
		enforced = false
        class = "not-working"
        fp_rate = 0
        fn_rate = "unmeasured"

	strings:
		$p0_0 = { 4c 5a 58 }

	condition:
		(prefix_size >= 3 and $p0_0 at 0)
}

rule taxonomy_mp3
{
	meta:
        source_refs = "libmagic:magic/Magdir/animation:libmagic_2bb870307cef26a366c8_line_760; libmagic:magic/Magdir/animation:libmagic_4211bdc06ca9aeca9a85_line_580; libmagic:magic/Magdir/animation:libmagic_47b342bc5389d3c50a09_line_725; libmagic:magic/Magdir/animation:libmagic_b6e4748ef4f348b49fc1_line_690; libmagic:magic/Magdir/animation:libmagic_dfd9cd53d290b0f243a4_line_655; puremagic:puremagic/magic_data.json:headers[152]; puremagic:puremagic/magic_data.json:headers[153]; puremagic:puremagic/magic_data.json:headers[154]; puremagic:puremagic/magic_data.json:headers[155]; puremagic:puremagic/magic_data.json:headers[156]; puremagic:puremagic/magic_data.json:headers[157]; puremagic:puremagic/magic_data.json:headers[158]; puremagic:puremagic/magic_data.json:headers[159]; puremagic:puremagic/magic_data.json:headers[160]; puremagic:puremagic/magic_data.json:headers[161]; puremagic:puremagic/magic_data.json:headers[162]; puremagic:puremagic/magic_data.json:headers[163]; puremagic:puremagic/magic_data.json:headers[164]; puremagic:puremagic/magic_data.json:headers[165]; puremagic:puremagic/magic_data.json:headers[166]; puremagic:puremagic/magic_data.json:headers[167]; puremagic:puremagic/magic_data.json:headers[168]; puremagic:puremagic/magic_data.json:headers[169]; puremagic:puremagic/magic_data.json:headers[170]; puremagic:puremagic/magic_data.json:headers[171]; puremagic:puremagic/magic_data.json:headers[172]; puremagic:puremagic/magic_data.json:headers[173]; puremagic:puremagic/magic_data.json:headers[174]; puremagic:puremagic/magic_data.json:headers[175]; puremagic:puremagic/magic_data.json:headers[176]; puremagic:puremagic/magic_data.json:headers[177]; puremagic:puremagic/magic_data.json:headers[178]; puremagic:puremagic/magic_data.json:headers[179]; puremagic:puremagic/magic_data.json:headers[180]; puremagic:puremagic/magic_data.json:headers[181]; puremagic:puremagic/magic_data.json:headers[182]; puremagic:puremagic/magic_data.json:headers[183]; puremagic:puremagic/magic_data.json:headers[184]; puremagic:puremagic/magic_data.json:headers[185]; puremagic:puremagic/magic_data.json:headers[186]; puremagic:puremagic/magic_data.json:headers[187]; puremagic:puremagic/magic_data.json:headers[188]; puremagic:puremagic/magic_data.json:headers[189]; puremagic:puremagic/magic_data.json:headers[190]; puremagic:puremagic/magic_data.json:headers[191]; puremagic:puremagic/magic_data.json:headers[192]; puremagic:puremagic/magic_data.json:headers[193]; puremagic:puremagic/magic_data.json:headers[194]; puremagic:puremagic/magic_data.json:multi-part[4944330200][10]; puremagic:puremagic/magic_data.json:multi-part[4944330200][11]; puremagic:puremagic/magic_data.json:multi-part[4944330200][12]; puremagic:puremagic/magic_data.json:multi-part[4944330200][13]; puremagic:puremagic/magic_data.json:multi-part[4944330200][14]; puremagic:puremagic/magic_data.json:multi-part[4944330200][15]; puremagic:puremagic/magic_data.json:multi-part[4944330200][16]; puremagic:puremagic/magic_data.json:multi-part[4944330200][17]; puremagic:puremagic/magic_data.json:multi-part[4944330200][18]; puremagic:puremagic/magic_data.json:multi-part[4944330200][19]; puremagic:puremagic/magic_data.json:multi-part[4944330200][1]; puremagic:puremagic/magic_data.json:multi-part[4944330200][20]; puremagic:puremagic/magic_data.json:multi-part[4944330200][21]; puremagic:puremagic/magic_data.json:multi-part[4944330200][22]; puremagic:puremagic/magic_data.json:multi-part[4944330200][23]; puremagic:puremagic/magic_data.json:multi-part[4944330200][24]; puremagic:puremagic/magic_data.json:multi-part[4944330200][25]; puremagic:puremagic/magic_data.json:multi-part[4944330200][26]; puremagic:puremagic/magic_data.json:multi-part[4944330200][27]; puremagic:puremagic/magic_data.json:multi-part[4944330200][28]; puremagic:puremagic/magic_data.json:multi-part[4944330200][29]; puremagic:puremagic/magic_data.json:multi-part[4944330200][2]; puremagic:puremagic/magic_data.json:multi-part[4944330200][30]; puremagic:puremagic/magic_data.json:multi-part[4944330200][31]; puremagic:puremagic/magic_data.json:multi-part[4944330200][32]; puremagic:puremagic/magic_data.json:multi-part[4944330200][33]; puremagic:puremagic/magic_data.json:multi-part[4944330200][34]; puremagic:puremagic/magic_data.json:multi-part[4944330200][35]; puremagic:puremagic/magic_data.json:multi-part[4944330200][36]; puremagic:puremagic/magic_data.json:multi-part[4944330200][37]; puremagic:puremagic/magic_data.json:multi-part[4944330200][38]; puremagic:puremagic/magic_data.json:multi-part[4944330200][39]; puremagic:puremagic/magic_data.json:multi-part[4944330200][3]; puremagic:puremagic/magic_data.json:multi-part[4944330200][40]; puremagic:puremagic/magic_data.json:multi-part[4944330200][41]; puremagic:puremagic/magic_data.json:multi-part[4944330200][42]; puremagic:puremagic/magic_data.json:multi-part[4944330200][43]; puremagic:puremagic/magic_data.json:multi-part[4944330200][44]; puremagic:puremagic/magic_data.json:multi-part[4944330200][45]; puremagic:puremagic/magic_data.json:multi-part[4944330200][46]; puremagic:puremagic/magic_data.json:multi-part[4944330200][47]; puremagic:puremagic/magic_data.json:multi-part[4944330200][48]; puremagic:puremagic/magic_data.json:multi-part[4944330200][49]; puremagic:puremagic/magic_data.json:multi-part[4944330200][4]; puremagic:puremagic/magic_data.json:multi-part[4944330200][50]; puremagic:puremagic/magic_data.json:multi-part[4944330200][51]; puremagic:puremagic/magic_data.json:multi-part[4944330200][52]; puremagic:puremagic/magic_data.json:multi-part[4944330200][53]; puremagic:puremagic/magic_data.json:multi-part[4944330200][54]; puremagic:puremagic/magic_data.json:multi-part[4944330200][55]; puremagic:puremagic/magic_data.json:multi-part[4944330200][56]; puremagic:puremagic/magic_data.json:multi-part[4944330200][57]; puremagic:puremagic/magic_data.json:multi-part[4944330200][58]; puremagic:puremagic/magic_data.json:multi-part[4944330200][59]; puremagic:puremagic/magic_data.json:multi-part[4944330200][5]; puremagic:puremagic/magic_data.json:multi-part[4944330200][60]; puremagic:puremagic/magic_data.json:multi-part[4944330200][61]; puremagic:puremagic/magic_data.json:multi-part[4944330200][62]; puremagic:puremagic/magic_data.json:multi-part[4944330200][63]; puremagic:puremagic/magic_data.json:multi-part[4944330200][64]; puremagic:puremagic/magic_data.json:multi-part[4944330200][6]; puremagic:puremagic/magic_data.json:multi-part[4944330200][7]; puremagic:puremagic/magic_data.json:multi-part[4944330200][8]; puremagic:puremagic/magic_data.json:multi-part[4944330200][9]; puremagic:puremagic/magic_data.json:multi-part[4944330300][0]; puremagic:puremagic/magic_data.json:multi-part[4944330300][10]; puremagic:puremagic/magic_data.json:multi-part[4944330300][11]; puremagic:puremagic/magic_data.json:multi-part[4944330300][12]; puremagic:puremagic/magic_data.json:multi-part[4944330300][13]; puremagic:puremagic/magic_data.json:multi-part[4944330300][14]; puremagic:puremagic/magic_data.json:multi-part[4944330300][15]; puremagic:puremagic/magic_data.json:multi-part[4944330300][16]; puremagic:puremagic/magic_data.json:multi-part[4944330300][17]; puremagic:puremagic/magic_data.json:multi-part[4944330300][18]; puremagic:puremagic/magic_data.json:multi-part[4944330300][19]; puremagic:puremagic/magic_data.json:multi-part[4944330300][1]; puremagic:puremagic/magic_data.json:multi-part[4944330300][20]; puremagic:puremagic/magic_data.json:multi-part[4944330300][21]; puremagic:puremagic/magic_data.json:multi-part[4944330300][22]; puremagic:puremagic/magic_data.json:multi-part[4944330300][23]; puremagic:puremagic/magic_data.json:multi-part[4944330300][24]; puremagic:puremagic/magic_data.json:multi-part[4944330300][25]; puremagic:puremagic/magic_data.json:multi-part[4944330300][26]; puremagic:puremagic/magic_data.json:multi-part[4944330300][27]; puremagic:puremagic/magic_data.json:multi-part[4944330300][28]; puremagic:puremagic/magic_data.json:multi-part[4944330300][29]; puremagic:puremagic/magic_data.json:multi-part[4944330300][2]; puremagic:puremagic/magic_data.json:multi-part[4944330300][30]; puremagic:puremagic/magic_data.json:multi-part[4944330300][31]; puremagic:puremagic/magic_data.json:multi-part[4944330300][32]; puremagic:puremagic/magic_data.json:multi-part[4944330300][33]; puremagic:puremagic/magic_data.json:multi-part[4944330300][34]; puremagic:puremagic/magic_data.json:multi-part[4944330300][35]; puremagic:puremagic/magic_data.json:multi-part[4944330300][36]; puremagic:puremagic/magic_data.json:multi-part[4944330300][37]; puremagic:puremagic/magic_data.json:multi-part[4944330300][38]; puremagic:puremagic/magic_data.json:multi-part[4944330300][39]; puremagic:puremagic/magic_data.json:multi-part[4944330300][3]; puremagic:puremagic/magic_data.json:multi-part[4944330300][40]; puremagic:puremagic/magic_data.json:multi-part[4944330300][41]; puremagic:puremagic/magic_data.json:multi-part[4944330300][42]; puremagic:puremagic/magic_data.json:multi-part[4944330300][43]; puremagic:puremagic/magic_data.json:multi-part[4944330300][44]; puremagic:puremagic/magic_data.json:multi-part[4944330300][45]; puremagic:puremagic/magic_data.json:multi-part[4944330300][46]; puremagic:puremagic/magic_data.json:multi-part[4944330300][47]; puremagic:puremagic/magic_data.json:multi-part[4944330300][48]; puremagic:puremagic/magic_data.json:multi-part[4944330300][49]; puremagic:puremagic/magic_data.json:multi-part[4944330300][4]; puremagic:puremagic/magic_data.json:multi-part[4944330300][50]; puremagic:puremagic/magic_data.json:multi-part[4944330300][51]; puremagic:puremagic/magic_data.json:multi-part[4944330300][52]; puremagic:puremagic/magic_data.json:multi-part[4944330300][53]; puremagic:puremagic/magic_data.json:multi-part[4944330300][54]; puremagic:puremagic/magic_data.json:multi-part[4944330300][55]; puremagic:puremagic/magic_data.json:multi-part[4944330300][56]; puremagic:puremagic/magic_data.json:multi-part[4944330300][57]; puremagic:puremagic/magic_data.json:multi-part[4944330300][58]; puremagic:puremagic/magic_data.json:multi-part[4944330300][59]; puremagic:puremagic/magic_data.json:multi-part[4944330300][5]; puremagic:puremagic/magic_data.json:multi-part[4944330300][60]; puremagic:puremagic/magic_data.json:multi-part[4944330300][61]; puremagic:puremagic/magic_data.json:multi-part[4944330300][62]; puremagic:puremagic/magic_data.json:multi-part[4944330300][63]; puremagic:puremagic/magic_data.json:multi-part[4944330300][64]; puremagic:puremagic/magic_data.json:multi-part[4944330300][65]; puremagic:puremagic/magic_data.json:multi-part[4944330300][66]; puremagic:puremagic/magic_data.json:multi-part[4944330300][67]; puremagic:puremagic/magic_data.json:multi-part[4944330300][68]; puremagic:puremagic/magic_data.json:multi-part[4944330300][69]; puremagic:puremagic/magic_data.json:multi-part[4944330300][6]; puremagic:puremagic/magic_data.json:multi-part[4944330300][70]; puremagic:puremagic/magic_data.json:multi-part[4944330300][71]; puremagic:puremagic/magic_data.json:multi-part[4944330300][72]; puremagic:puremagic/magic_data.json:multi-part[4944330300][73]; puremagic:puremagic/magic_data.json:multi-part[4944330300][74]; puremagic:puremagic/magic_data.json:multi-part[4944330300][75]; puremagic:puremagic/magic_data.json:multi-part[4944330300][76]; puremagic:puremagic/magic_data.json:multi-part[4944330300][77]; puremagic:puremagic/magic_data.json:multi-part[4944330300][78]; puremagic:puremagic/magic_data.json:multi-part[4944330300][79]; puremagic:puremagic/magic_data.json:multi-part[4944330300][7]; puremagic:puremagic/magic_data.json:multi-part[4944330300][80]; puremagic:puremagic/magic_data.json:multi-part[4944330300][81]; puremagic:puremagic/magic_data.json:multi-part[4944330300][82]; puremagic:puremagic/magic_data.json:multi-part[4944330300][83]; puremagic:puremagic/magic_data.json:multi-part[4944330300][84]; puremagic:puremagic/magic_data.json:multi-part[4944330300][85]; puremagic:puremagic/magic_data.json:multi-part[4944330300][86]; puremagic:puremagic/magic_data.json:multi-part[4944330300][87]; puremagic:puremagic/magic_data.json:multi-part[4944330300][88]; puremagic:puremagic/magic_data.json:multi-part[4944330300][89]; puremagic:puremagic/magic_data.json:multi-part[4944330300][8]; puremagic:puremagic/magic_data.json:multi-part[4944330300][90]; puremagic:puremagic/magic_data.json:multi-part[4944330300][9]; puremagic:puremagic/magic_data.json:multi-part[4944330400][0]; puremagic:puremagic/magic_data.json:multi-part[4944330400][10]; puremagic:puremagic/magic_data.json:multi-part[4944330400][11]; puremagic:puremagic/magic_data.json:multi-part[4944330400][12]; puremagic:puremagic/magic_data.json:multi-part[4944330400][13]; puremagic:puremagic/magic_data.json:multi-part[4944330400][14]; puremagic:puremagic/magic_data.json:multi-part[4944330400][15]; puremagic:puremagic/magic_data.json:multi-part[4944330400][16]; puremagic:puremagic/magic_data.json:multi-part[4944330400][17]; puremagic:puremagic/magic_data.json:multi-part[4944330400][18]; puremagic:puremagic/magic_data.json:multi-part[4944330400][19]; puremagic:puremagic/magic_data.json:multi-part[4944330400][1]; puremagic:puremagic/magic_data.json:multi-part[4944330400][20]; puremagic:puremagic/magic_data.json:multi-part[4944330400][21]; puremagic:puremagic/magic_data.json:multi-part[4944330400][22]; puremagic:puremagic/magic_data.json:multi-part[4944330400][23]; puremagic:puremagic/magic_data.json:multi-part[4944330400][24]; puremagic:puremagic/magic_data.json:multi-part[4944330400][25]; puremagic:puremagic/magic_data.json:multi-part[4944330400][26]; puremagic:puremagic/magic_data.json:multi-part[4944330400][27]; puremagic:puremagic/magic_data.json:multi-part[4944330400][28]; puremagic:puremagic/magic_data.json:multi-part[4944330400][29]; puremagic:puremagic/magic_data.json:multi-part[4944330400][2]; puremagic:puremagic/magic_data.json:multi-part[4944330400][30]; puremagic:puremagic/magic_data.json:multi-part[4944330400][31]; puremagic:puremagic/magic_data.json:multi-part[4944330400][32]; puremagic:puremagic/magic_data.json:multi-part[4944330400][33]; puremagic:puremagic/magic_data.json:multi-part[4944330400][34]; puremagic:puremagic/magic_data.json:multi-part[4944330400][35]; puremagic:puremagic/magic_data.json:multi-part[4944330400][36]; puremagic:puremagic/magic_data.json:multi-part[4944330400][37]; puremagic:puremagic/magic_data.json:multi-part[4944330400][38]; puremagic:puremagic/magic_data.json:multi-part[4944330400][39]; puremagic:puremagic/magic_data.json:multi-part[4944330400][3]; puremagic:puremagic/magic_data.json:multi-part[4944330400][40]; puremagic:puremagic/magic_data.json:multi-part[4944330400][41]; puremagic:puremagic/magic_data.json:multi-part[4944330400][42]; puremagic:puremagic/magic_data.json:multi-part[4944330400][43]; puremagic:puremagic/magic_data.json:multi-part[4944330400][44]; puremagic:puremagic/magic_data.json:multi-part[4944330400][45]; puremagic:puremagic/magic_data.json:multi-part[4944330400][46]; puremagic:puremagic/magic_data.json:multi-part[4944330400][47]; puremagic:puremagic/magic_data.json:multi-part[4944330400][48]; puremagic:puremagic/magic_data.json:multi-part[4944330400][49]; puremagic:puremagic/magic_data.json:multi-part[4944330400][4]; puremagic:puremagic/magic_data.json:multi-part[4944330400][50]; puremagic:puremagic/magic_data.json:multi-part[4944330400][51]; puremagic:puremagic/magic_data.json:multi-part[4944330400][52]; puremagic:puremagic/magic_data.json:multi-part[4944330400][53]; puremagic:puremagic/magic_data.json:multi-part[4944330400][54]; puremagic:puremagic/magic_data.json:multi-part[4944330400][55]; puremagic:puremagic/magic_data.json:multi-part[4944330400][56]; puremagic:puremagic/magic_data.json:multi-part[4944330400][57]; puremagic:puremagic/magic_data.json:multi-part[4944330400][58]; puremagic:puremagic/magic_data.json:multi-part[4944330400][59]; puremagic:puremagic/magic_data.json:multi-part[4944330400][5]; puremagic:puremagic/magic_data.json:multi-part[4944330400][60]; puremagic:puremagic/magic_data.json:multi-part[4944330400][61]; puremagic:puremagic/magic_data.json:multi-part[4944330400][62]; puremagic:puremagic/magic_data.json:multi-part[4944330400][63]; puremagic:puremagic/magic_data.json:multi-part[4944330400][64]; puremagic:puremagic/magic_data.json:multi-part[4944330400][65]; puremagic:puremagic/magic_data.json:multi-part[4944330400][66]; puremagic:puremagic/magic_data.json:multi-part[4944330400][67]; puremagic:puremagic/magic_data.json:multi-part[4944330400][68]; puremagic:puremagic/magic_data.json:multi-part[4944330400][69]; puremagic:puremagic/magic_data.json:multi-part[4944330400][6]; puremagic:puremagic/magic_data.json:multi-part[4944330400][70]; puremagic:puremagic/magic_data.json:multi-part[4944330400][71]; puremagic:puremagic/magic_data.json:multi-part[4944330400][72]; puremagic:puremagic/magic_data.json:multi-part[4944330400][73]; puremagic:puremagic/magic_data.json:multi-part[4944330400][74]; puremagic:puremagic/magic_data.json:multi-part[4944330400][75]; puremagic:puremagic/magic_data.json:multi-part[4944330400][76]; puremagic:puremagic/magic_data.json:multi-part[4944330400][77]; puremagic:puremagic/magic_data.json:multi-part[4944330400][78]; puremagic:puremagic/magic_data.json:multi-part[4944330400][79]; puremagic:puremagic/magic_data.json:multi-part[4944330400][7]; puremagic:puremagic/magic_data.json:multi-part[4944330400][80]; puremagic:puremagic/magic_data.json:multi-part[4944330400][81]; puremagic:puremagic/magic_data.json:multi-part[4944330400][82]; puremagic:puremagic/magic_data.json:multi-part[4944330400][83]; puremagic:puremagic/magic_data.json:multi-part[4944330400][84]; puremagic:puremagic/magic_data.json:multi-part[4944330400][85]; puremagic:puremagic/magic_data.json:multi-part[4944330400][86]; puremagic:puremagic/magic_data.json:multi-part[4944330400][87]; puremagic:puremagic/magic_data.json:multi-part[4944330400][8]; puremagic:puremagic/magic_data.json:multi-part[4944330400][9]; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[1219]/magic[1]"
		label = "mp3"
		enforced = false
        class = "not-working"
        fp_rate = 0.00013594806783809
        fn_rate = 0

	strings:
		$p0_0 = { FF F6 }
		$p1_0 = { FF E1 }
		$p2_0 = { FF D7 }
		$p3_0 = { FF EA }
		$p4_0 = { FF E7 }
		$p5_0 = { FF E2 }
		$p6_0 = { FF D6 }
		$p7_0 = { ff ( e2 | e3 ) }
		$p8_0 = { FF FB }
		$p9_0 = { FF F7 }
		$p10_0 = { FF E0 }
		$p11_0 = { FF DA }
		$p12_0 = { FF E5 }
		$p13_0 = { ff ( fc | fd ) }
		$p14_0 = { FF FF }
		$p15_0 = { 49 44 33 03 00 }
		$p16_0 = { FF F5 }
		$p17_0 = { FF DB }
		$p18_0 = { FF F8 }
		$p19_0 = { FF F4 }
		$p20_0 = { FF E3 }
		$p21_0 = { FF F1 }
		$p22_0 = { FF ED }
		$p23_0 = { FF E4 }
		$p24_0 = { 49 44 33 04 00 }
		$p25_0 = { FF DF }
		$p26_0 = { FF F3 }
		$p27_0 = { FF F9 }
		$p28_0 = { FF E6 }
		$p29_0 = "ID3"
		$p30_0 = { FF F0 }
		$p31_0 = { FF E8 }
		$p32_0 = { FF D1 }
		$p33_0 = { FF E9 }
		$p34_0 = { FF EB }
		$p35_0 = { FF FC }
		$p36_0 = { ff ( f6 | f7 ) }
		$p37_0 = { FF FA }
		$p38_0 = { FF D0 }
		$p39_0 = { FF EF }
		$p40_0 = { FF EE }
		$p41_0 = { 49 44 33 02 00 }
		$p42_0 = { ff ( f2 | f3 ) }
		$p43_0 = { FF EC }
		$p44_0 = { FF DE }
		$p45_0 = { ff ( f4 | f5 ) }
		$p46_0 = { FF FD }
		$p47_0 = { FF F2 }

	condition:
		((prefix_size >= 2 and original_size >= 2 and $p0_0 at 0) or (prefix_size >= 2 and original_size >= 2 and $p1_0 at 0) or (prefix_size >= 2 and original_size >= 2 and $p2_0 at 0) or (prefix_size >= 2 and original_size >= 2 and $p3_0 at 0) or (prefix_size >= 2 and original_size >= 2 and $p4_0 at 0) or (prefix_size >= 2 and original_size >= 2 and $p5_0 at 0) or (prefix_size >= 2 and original_size >= 2 and $p6_0 at 0) or (prefix_size >= 2 and $p7_0 at 0) or (prefix_size >= 2 and original_size >= 2 and $p8_0 at 0) or (prefix_size >= 2 and original_size >= 2 and $p9_0 at 0) or (prefix_size >= 2 and original_size >= 2 and $p10_0 at 0) or (prefix_size >= 2 and original_size >= 2 and $p11_0 at 0) or (prefix_size >= 2 and original_size >= 2 and $p12_0 at 0) or (prefix_size >= 2 and $p13_0 at 0) or (prefix_size >= 2 and original_size >= 2 and $p14_0 at 0) or (prefix_size >= 5 and original_size >= 5 and $p15_0 at 0) or (prefix_size >= 2 and original_size >= 2 and $p16_0 at 0) or (prefix_size >= 2 and original_size >= 2 and $p17_0 at 0) or (prefix_size >= 2 and original_size >= 2 and $p18_0 at 0) or (prefix_size >= 2 and original_size >= 2 and $p19_0 at 0) or (prefix_size >= 2 and original_size >= 2 and $p20_0 at 0) or (prefix_size >= 2 and original_size >= 2 and $p21_0 at 0) or (prefix_size >= 2 and original_size >= 2 and $p22_0 at 0) or (prefix_size >= 2 and original_size >= 2 and $p23_0 at 0) or (prefix_size >= 5 and original_size >= 5 and $p24_0 at 0) or (prefix_size >= 2 and original_size >= 2 and $p25_0 at 0) or (prefix_size >= 2 and original_size >= 2 and $p26_0 at 0) or (prefix_size >= 2 and original_size >= 2 and $p27_0 at 0) or (prefix_size >= 2 and original_size >= 2 and $p28_0 at 0) or (prefix_size >= 3 and original_size >= 3 and $p29_0 at 0) or (prefix_size >= 2 and original_size >= 2 and $p30_0 at 0) or (prefix_size >= 2 and original_size >= 2 and $p31_0 at 0) or (prefix_size >= 2 and original_size >= 2 and $p32_0 at 0) or (prefix_size >= 2 and original_size >= 2 and $p33_0 at 0) or (prefix_size >= 2 and original_size >= 2 and $p34_0 at 0) or (prefix_size >= 2 and original_size >= 2 and $p35_0 at 0) or (prefix_size >= 2 and $p36_0 at 0) or (prefix_size >= 2 and original_size >= 2 and $p37_0 at 0) or (prefix_size >= 2 and original_size >= 2 and $p38_0 at 0) or (prefix_size >= 2 and original_size >= 2 and $p39_0 at 0) or (prefix_size >= 2 and original_size >= 2 and $p40_0 at 0) or (prefix_size >= 5 and original_size >= 5 and $p41_0 at 0) or (prefix_size >= 2 and $p42_0 at 0) or (prefix_size >= 2 and original_size >= 2 and $p43_0 at 0) or (prefix_size >= 2 and original_size >= 2 and $p44_0 at 0) or (prefix_size >= 2 and $p45_0 at 0) or (prefix_size >= 2 and original_size >= 2 and $p46_0 at 0) or (prefix_size >= 2 and original_size >= 2 and $p47_0 at 0))
}

rule taxonomy_mp4
{
	meta:
        source_refs = "libmagic:magic/Magdir/animation:libmagic_551bb9d95fd0095e0a3a_line_133; libmagic:magic/Magdir/animation:libmagic_551bb9d95fd0095e0a3a_line_155; libmagic:magic/Magdir/animation:libmagic_551bb9d95fd0095e0a3a_line_157; libmagic:magic/Magdir/animation:libmagic_551bb9d95fd0095e0a3a_line_168; libmagic:magic/Magdir/animation:libmagic_551bb9d95fd0095e0a3a_line_176; libmagic:magic/Magdir/animation:libmagic_551bb9d95fd0095e0a3a_line_198; libmagic:magic/Magdir/animation:libmagic_551bb9d95fd0095e0a3a_line_212; libmagic:magic/Magdir/animation:libmagic_551bb9d95fd0095e0a3a_line_214; libmagic:magic/Magdir/animation:libmagic_551bb9d95fd0095e0a3a_line_217; libmagic:magic/Magdir/animation:libmagic_551bb9d95fd0095e0a3a_line_219; libmagic:magic/Magdir/animation:libmagic_551bb9d95fd0095e0a3a_line_227; libmagic:magic/Magdir/animation:libmagic_551bb9d95fd0095e0a3a_line_236; libmagic:magic/Magdir/animation:libmagic_551bb9d95fd0095e0a3a_line_238; libmagic:magic/Magdir/animation:libmagic_551bb9d95fd0095e0a3a_line_240; libmagic:magic/Magdir/animation:libmagic_551bb9d95fd0095e0a3a_line_242; libmagic:magic/Magdir/animation:libmagic_551bb9d95fd0095e0a3a_line_244; libmagic:magic/Magdir/animation:libmagic_551bb9d95fd0095e0a3a_line_246; libmagic:magic/Magdir/animation:libmagic_551bb9d95fd0095e0a3a_line_248; libmagic:magic/Magdir/animation:libmagic_551bb9d95fd0095e0a3a_line_250; libmagic:magic/Magdir/animation:libmagic_551bb9d95fd0095e0a3a_line_252; libmagic:magic/Magdir/animation:libmagic_551bb9d95fd0095e0a3a_line_255; libmagic:magic/Magdir/animation:libmagic_551bb9d95fd0095e0a3a_line_39; libmagic:magic/Magdir/animation:libmagic_551bb9d95fd0095e0a3a_line_95; libmagic:magic/Magdir/animation:libmagic_551bb9d95fd0095e0a3a_line_97; libmagic:magic/Magdir/animation:libmagic_551bb9d95fd0095e0a3a_line_99; puremagic:puremagic/magic_data.json:headers[1175]; puremagic:puremagic/magic_data.json:headers[1176]; puremagic:puremagic/magic_data.json:headers[1210]; puremagic:puremagic/magic_data.json:headers[141]; puremagic:puremagic/magic_data.json:headers[142]; puremagic:puremagic/magic_data.json:headers[144]; puremagic:puremagic/magic_data.json:headers[145]; puremagic:puremagic/magic_data.json:headers[832]; puremagic:puremagic/magic_data.json:headers[833]; puremagic:puremagic/magic_data.json:headers[834]; puremagic:puremagic/magic_data.json:headers[835]; puremagic:puremagic/magic_data.json:headers[836]; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[1625]/magic[0]"
		label = "mp4"
		enforced = false
        class = "not-working"
        fp_rate = 0.00353464976379023
        fn_rate = 0.01

	strings:
		$p0_0 = { 6d 70 34 31 }
		$p1_0 = "ftypisom"
		$p2_0 = { 00 00 00 18 66 74 79 70 6D 70 34 32 }
		$p3_0 = { 6d 6d 70 34 }
		$p4_0 = { 4e 44 58 50 }
		$p5_0 = { 69 73 6d 6c }
		$p6_0 = { 69 73 6f }
		$p7_0 = { 00 00 00 14 66 74 79 70 69 73 6F 6D }
		$p8_0 = "ftypf4v "
		$p9_0 = { 58 41 56 43 }
		$p10_0 = "ftypMSNV"
		$p11_0 = { 41 52 52 49 }
		$p12_0 = "ftypM4V "
		$p13_0 = "ftyp"
		$p14_0 = { 46 34 50 }
		$p15_0 = { 4e 44 58 43 }
		$p16_0 = { 4d 34 50 }
		$p17_0 = { 6d 70 34 32 }
		$p18_0 = { 62 62 78 6d }
		$p19_0 = { 4e 44 53 48 }
		$p20_0 = "ftypmp42"
		$p21_0 = { 46 34 56 }
		$p22_0 = { 6d 6f 62 69 }
		$p23_0 = { 00 00 00 1C 66 74 79 70 }
		$p24_0 = { 4e 44 53 50 }
		$p25_0 = { 6e 69 6b 6f }
		$p26_0 = { 4e 44 58 48 }
		$p27_0 = { 00 00 00 18 66 74 79 70 33 67 70 35 }
		$p28_0 = { 4e 44 58 4d }
		$p29_0 = "iso2avc1mp4"
		$p30_0 = { 4e 44 53 53 }
		$p31_0 = "ftyp3gp5"
		$p32_0 = { 64 61 73 68 }
		$p33_0 = { 4e 44 53 43 }
		$p34_0 = { 00 00 00 1C 66 74 79 70 4D 53 4E 56 01 29 00 46 4D 53 4E 56 6D 70 34 32 }
		$p35_0 = "ftypmp41"
		$p36_0 = { 61 76 63 31 }
		$p37_0 = { 4e 44 53 4d }

	condition:
		(((prefix_size >= 8 and $p13_0 at 4) and ((prefix_size >= 12 and $p0_0 at 8) or (prefix_size >= 12 and $p3_0 at 8) or (prefix_size >= 12 and $p4_0 at 8) or (prefix_size >= 12 and $p5_0 at 8) or (prefix_size >= 11 and $p6_0 at 8) or (prefix_size >= 12 and $p9_0 at 8) or (prefix_size >= 12 and $p11_0 at 8) or (prefix_size >= 11 and $p14_0 at 8) or (prefix_size >= 12 and $p15_0 at 8) or (prefix_size >= 11 and $p16_0 at 8) or (prefix_size >= 12 and $p17_0 at 8) or (prefix_size >= 12 and $p18_0 at 8) or (prefix_size >= 12 and $p19_0 at 8) or (prefix_size >= 11 and $p21_0 at 8) or (prefix_size >= 12 and $p22_0 at 8) or (prefix_size >= 12 and $p24_0 at 8) or (prefix_size >= 12 and $p25_0 at 8) or (prefix_size >= 12 and $p26_0 at 8) or (prefix_size >= 12 and $p28_0 at 8) or (prefix_size >= 12 and $p30_0 at 8) or (prefix_size >= 12 and $p32_0 at 8) or (prefix_size >= 12 and $p33_0 at 8) or (prefix_size >= 12 and $p36_0 at 8) or (prefix_size >= 12 and $p37_0 at 8))) or ((prefix_size >= 12 and original_size >= 12 and $p1_0 at 4) or (prefix_size >= 12 and original_size >= 12 and $p2_0 at 0) or (prefix_size >= 12 and original_size >= 12 and $p7_0 at 0) or (prefix_size >= 12 and original_size >= 12 and $p8_0 at 4) or (prefix_size >= 12 and original_size >= 12 and $p10_0 at 4) or (prefix_size >= 12 and original_size >= 12 and $p12_0 at 4) or (prefix_size >= 12 and original_size >= 12 and $p20_0 at 4) or (prefix_size >= 8 and original_size >= 8 and $p23_0 at 0) or (prefix_size >= 12 and original_size >= 12 and $p27_0 at 0) or (prefix_size >= 31 and original_size >= 31 and $p29_0 at 20) or (prefix_size >= 12 and original_size >= 12 and $p31_0 at 4) or (prefix_size >= 24 and original_size >= 24 and $p34_0 at 0) or (prefix_size >= 12 and $p35_0 at 4)))
}

rule taxonomy_msi
{
	meta:
        source_refs = ""
		label = "msi"
		enforced = false
        class = "not-working"
        fp_rate = 0
        fn_rate = 1

	condition:
		false
}

rule taxonomy_odp
{
	meta:
        source_refs = "puremagic:puremagic/magic_data.json:headers[1251]; puremagic:puremagic/magic_data.json:headers[561]; puremagic:puremagic/magic_data.json:headers[687]; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[608]/magic[0]"
		label = "odp"
		enforced = false
        class = "not-working"
        fp_rate = 0.0653910206301193
        fn_rate = 0

	strings:
		$p0_0 = "PK\\003\\004"
		$p1_0 = "presentation"
		$p2_0 = "mimetypeapplication/vnd.oasis.opendocument.presentation"
		$p3_0 = { 50 4B 03 04 }
		$p4_0 = "PK"

	condition:
		((prefix_size >= 10 and original_size >= 10 and $p0_0 at 0) or (prefix_size >= 85 and original_size >= 85 and $p1_0 at 73) or ((prefix_size >= 85 and $p2_0 at 30) and (prefix_size >= 2 and $p4_0 at 0)) or (prefix_size >= 4 and original_size >= 4 and $p3_0 at 0))
}

rule taxonomy_ods
{
	meta:
        source_refs = "puremagic:puremagic/magic_data.json:headers[1252]; puremagic:puremagic/magic_data.json:headers[689]; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[610]/magic[0]"
		label = "ods"
		enforced = false
        class = "not-working"
        fp_rate = 0.00003398701695952
        fn_rate = 0.02

	strings:
		$p0_0 = "PK\\003\\004"
		$p1_0 = "spreadsheet"
		$p2_0 = "mimetypeapplication/vnd.oasis.opendocument.spreadsheet"
		$p3_0 = "PK"

	condition:
		((prefix_size >= 10 and original_size >= 10 and $p0_0 at 0) or (prefix_size >= 84 and original_size >= 84 and $p1_0 at 73) or ((prefix_size >= 84 and $p2_0 at 30) and (prefix_size >= 2 and $p3_0 at 0)))
}

rule taxonomy_odt
{
	meta:
        source_refs = "puremagic:puremagic/magic_data.json:headers[1250]; puremagic:puremagic/magic_data.json:headers[560]; puremagic:puremagic/magic_data.json:headers[681]; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[612]/magic[0]"
		label = "odt"
		enforced = false
        class = "not-working"
        fp_rate = 0.06559494273187642
        fn_rate = 0

	strings:
		$p0_0 = "PK\\003\\004"
		$p1_0 = "mimetypeapplication/vnd.oasis.opendocument.text"
		$p2_0 = "text"
		$p3_0 = { 50 4B 03 04 }
		$p4_0 = "PK"

	condition:
		((prefix_size >= 10 and original_size >= 10 and $p0_0 at 0) or ((prefix_size >= 77 and $p1_0 at 30) and (prefix_size >= 2 and $p4_0 at 0)) or (prefix_size >= 77 and original_size >= 77 and $p2_0 at 73) or (prefix_size >= 4 and original_size >= 4 and $p3_0 at 0))
}

rule taxonomy_otf
{
	meta:
        source_refs = ""
		label = "otf"
		enforced = false
        class = "not-working"
        fp_rate = 0
        fn_rate = 1

	condition:
		false
}

rule taxonomy_outlook
{
	meta:
        source_refs = "puremagic:puremagic/magic_data.json:headers[1484]"
		label = "outlook"
		enforced = false
        class = "not-working"
        fp_rate = 0.01920266458212963
        fn_rate = 0

	strings:
		$p0_0 = { D0 CF 11 E0 A1 B1 1A E1 }

	condition:
		(prefix_size >= 8 and original_size >= 8 and $p0_0 at 0)
}

rule taxonomy_paradox
{
	meta:
        source_refs = "pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:516; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:517; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:518; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:519"
		label = "paradox"
		enforced = false
        class = "not-working"
        fp_rate = 0
        fn_rate = "unmeasured"

	strings:
		$p0_0 = /\x00\x08(\x00|\x02)[\x01-\x04]([\x00-\xff]){51}[\x05-\x09]([\x00-\xff]){34}(\x00){4}/
		$p1_0 = /\x00\x08(\x00|\x02)[\x01-\x04]([\x00-\xff]){31}(\x00){4}(([\x00-\xff]){16}[\x03-\x04])/
		$p2_0 = /\x00\x08(\x00|\x02)[\x01-\x20]([\x00-\xff]){51}[\x0a-\x0b]([\x00-\xff]){34}(\x00){4}/
		$p3_0 = /\x00\x08(\x00|\x02)[\x01-\x20]([\x00-\xff]){51}\x0c([\x00-\xff]){34}(\x00){4}/

	condition:
		(($p0_0 at 2) or ($p1_0 at 2) or ($p2_0 at 2) or ($p3_0 at 2))
}

rule taxonomy_pcapng
{
	meta:
        source_refs = "libmagic:magic/Magdir/sniffer:libmagic_46f6367e781d4176898f_line_319; libmagic:magic/Magdir/sniffer:libmagic_dfb99b3410acff36a157_line_314; puremagic:puremagic/magic_data.json:headers[1069]; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[769]/magic[0]"
		label = "pcapng"
		enforced = false
        class = "not-working"
        fp_rate = 0.00061176630527139
        fn_rate = 0

	strings:
		$p0_0 = { 0A 0D 0D 0A }

	condition:
		(prefix_size >= 4 and original_size >= 4 and $p0_0 at 0)
}

rule taxonomy_pgp
{
	meta:
        source_refs = "libmagic:magic/Magdir/pgp:libmagic_7a78c07f093563b10b8a_line_28; libmagic:magic/Magdir/pgp:libmagic_e7aa59b1cca0df9e5f46_line_32; puremagic:puremagic/magic_data.json:headers[649]; puremagic:puremagic/magic_data.json:headers[650]; puremagic:puremagic/magic_data.json:headers[651]"
		label = "pgp"
		enforced = false
        class = "not-working"
        fp_rate = 0.0000677874186551
        fn_rate = 0.78947368421052633

	strings:
		$p0_0 = "-----BEGIN PGP PUBLIC KEY BLOCK-----"
		$p1_0 = { 2d 2d 2d 42 45 47 49 4e 20 50 47 50 20 50 55 42 4c 49 43 20 4b 45 59 20 42 4c 4f 43 4b 2d }
		$p2_0 = "-----BEGIN PGP PRIVATE KEY BLOCK-----"
		$p3_0 = { 99 00 }
		$p4_0 = { 2d 2d 2d 42 45 47 49 4e 20 50 47 50 20 50 52 49 56 41 54 45 20 4b 45 59 20 42 4c 4f 43 4b 2d }

	condition:
		((prefix_size >= 36 and original_size >= 36 and $p0_0 at 0) or (prefix_size >= 32 and $p1_0 at 2) or (prefix_size >= 37 and original_size >= 37 and $p2_0 at 0) or (prefix_size >= 2 and original_size >= 2 and $p3_0 at 0) or (prefix_size >= 33 and $p4_0 at 2))
}

rule taxonomy_png
{
	meta:
        source_refs = "libmagic:magic/Magdir/images:libmagic_849a015a836fc699a0f9_line_543; puremagic:puremagic/magic_data.json:headers[210]; puremagic:puremagic/magic_data.json:headers[875]; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[1334]/magic[0]"
		label = "png"
		enforced = false
        class = "not-working"
        fp_rate = 0.0004758182374333
        fn_rate = 0

	strings:
		$p0_0 = { 89 50 4E 47 0D 0A 1A 0A }
		$p1_0 = "\\x89PNG"

	condition:
		((prefix_size >= 8 and original_size >= 8 and $p0_0 at 0) or (prefix_size >= 7 and original_size >= 7 and $p1_0 at 0))
}

rule taxonomy_postscript
{
	meta:
        source_refs = "libmagic:magic/Magdir/printer:libmagic_1ef82717f63e9c3388a8_line_19; libmagic:magic/Magdir/printer:libmagic_2cdb14a02199c304d8f2_line_8; puremagic:puremagic/magic_data.json:headers[1004]; puremagic:puremagic/magic_data.json:headers[1005]; puremagic:puremagic/magic_data.json:headers[355]; puremagic:puremagic/magic_data.json:headers[563]; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[166]/magic[0]"
		label = "postscript"
		enforced = false
        class = "not-working"
        fp_rate = 0.00006797403391904
        fn_rate = 0.17999999999999999

	strings:
		$p0_0 = { 04 25 21 }
		$p1_0 = "%!"
		$p2_0 = "%!PS-Adobe-3.0 EPSF-3.0"
		$p3_0 = { C5 D0 D3 C6 }
		$p4_0 = "\\004%!"
		$p5_0 = "%!PS-Ado"

	condition:
		((prefix_size >= 3 and $p0_0 at 0) or (prefix_size >= 2 and original_size >= 2 and $p1_0 at 0) or (prefix_size >= 23 and $p2_0 at 0) or (prefix_size >= 4 and original_size >= 4 and $p3_0 at 0) or (prefix_size >= 6 and original_size >= 6 and $p4_0 at 0) or (prefix_size >= 8 and original_size >= 8 and $p5_0 at 0))
}

rule taxonomy_ppt
{
	meta:
        source_refs = "puremagic:puremagic/magic_data.json:headers[213]; puremagic:puremagic/magic_data.json:headers[214]; puremagic:puremagic/magic_data.json:headers[219]; puremagic:puremagic/magic_data.json:headers[222]"
		label = "ppt"
		enforced = false
        class = "not-working"
        fp_rate = 0.01141963769839921
        fn_rate = 0.17000000000000001

	strings:
		$p0_0 = { 0F 00 E8 03 }
		$p1_0 = { A0 46 1D F0 }
		$p2_0 = { FD FF FF FF }
		$p3_0 = { 00 6E 1E F0 }

	condition:
		((prefix_size >= 516 and original_size >= 516 and $p0_0 at 512) or (prefix_size >= 516 and original_size >= 516 and $p1_0 at 512) or (prefix_size >= 516 and original_size >= 516 and $p2_0 at 512) or (prefix_size >= 516 and original_size >= 516 and $p3_0 at 512))
}

rule taxonomy_pptx
{
	meta:
        source_refs = "puremagic:puremagic/magic_data.json:headers[67]"
		label = "pptx"
		enforced = false
        class = "not-working"
        fp_rate = 0.0653910206301193
        fn_rate = 0

	strings:
		$p0_0 = { 50 4B 03 04 }

	condition:
		(prefix_size >= 4 and original_size >= 4 and $p0_0 at 0)
}

rule taxonomy_pub
{
	meta:
        source_refs = "pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:1882"
		label = "pub"
		enforced = false
        class = "not-working"
        fp_rate = 0
        fn_rate = "unmeasured"

	strings:
		$p0_0 = /\xe7\xac\x2c\x00/

	condition:
		($p0_0 in ( 0 .. 8 ))
}

rule taxonomy_qt
{
	meta:
        source_refs = "libmagic:magic/Magdir/animation:libmagic_551bb9d95fd0095e0a3a_line_230; libmagic:magic/Magdir/animation:libmagic_551bb9d95fd0095e0a3a_line_263; libmagic:magic/Magdir/animation:libmagic_7ccfcde9e903976423cb_line_13; libmagic:magic/Magdir/animation:libmagic_f1dc14f5fff9d2a88b65_line_21; libmagic:magic/Magdir/animation:libmagic_fe7e7b30bf5e360de973_line_19; puremagic:puremagic/magic_data.json:headers[1188]; puremagic:puremagic/magic_data.json:headers[128]; puremagic:puremagic/magic_data.json:headers[143]; puremagic:puremagic/magic_data.json:headers[150]; puremagic:puremagic/magic_data.json:headers[481]; puremagic:puremagic/magic_data.json:headers[494]; puremagic:puremagic/magic_data.json:headers[528]; puremagic:puremagic/magic_data.json:headers[537]; puremagic:puremagic/magic_data.json:headers[573]; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[1641]/magic[0]"
		label = "qt"
		enforced = false
        class = "not-working"
        fp_rate = 0.01056996227441117
        fn_rate = 0

	strings:
		$p0_0 = { 6D 6F 6F 76 00 }
		$p1_0 = { 66 72 65 65 00 }
		$p2_0 = "free"
		$p3_0 = "ftypqt"
		$p4_0 = { 00 00 00 14 66 74 79 70 71 74 20 20 }
		$p5_0 = { 00 00 00 08 77 69 64 65 }
		$p6_0 = "mdat"
		$p7_0 = "ftyp"
		$p8_0 = { 6D 64 61 74 00 }
		$p9_0 = "moov"
		$p10_0 = "pnot"
		$p11_0 = { 70 6E 6F 74 00 }
		$p12_0 = "skip"
		$p13_0 = "ftypqt  "
		$p14_0 = "wide"
		$p15_0 = { 73 6B 69 70 00 }

	condition:
		((prefix_size >= 9 and $p0_0 at 4) or (prefix_size >= 9 and $p1_0 at 4) or (prefix_size >= 8 and original_size >= 8 and $p2_0 at 4) or (prefix_size >= 10 and original_size >= 10 and $p3_0 at 4) or (prefix_size >= 12 and original_size >= 12 and $p4_0 at 0) or (prefix_size >= 8 and $p5_0 at 0) or (prefix_size >= 8 and original_size >= 8 and $p6_0 at 4) or (prefix_size >= 8 and $p7_0 at 4) or (prefix_size >= 9 and $p8_0 at 4) or (prefix_size >= 8 and original_size >= 8 and $p9_0 at 4) or (prefix_size >= 8 and original_size >= 8 and $p10_0 at 4) or (prefix_size >= 9 and $p11_0 at 4) or (prefix_size >= 8 and original_size >= 8 and $p12_0 at 4) or (prefix_size >= 12 and original_size >= 12 and $p13_0 at 4) or (prefix_size >= 8 and original_size >= 8 and $p14_0 at 4) or (prefix_size >= 9 and $p15_0 at 4))
}

rule taxonomy_rdata
{
	meta:
        source_refs = "libmagic:magic/Magdir/r:libmagic_123a83536c36eb84897d_line_80; libmagic:magic/Magdir/r:libmagic_575cd7ac3723c55fc29d_line_84"
		label = "rdata"
		enforced = false
        class = "not-working"
        fp_rate = 0
        fn_rate = "unmeasured"

	strings:
		$p0_0 = { 52 44 58 33 0a }
		$p1_0 = { 52 44 58 32 0a }

	condition:
		((prefix_size >= 5 and $p0_0 at 0) or (prefix_size >= 5 and $p1_0 at 0))
}

rule taxonomy_rhinoceros
{
	meta:
        source_refs = "pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:1225; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:214; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:215; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:216; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:217; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:3445; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:3446; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:3447"
		label = "rhinoceros"
		enforced = false
        class = "not-working"
        fp_rate = 0
        fn_rate = "unmeasured"

	strings:
		$p0_0 = /\x33\x44\x20\x47\x65\x6f\x6d\x65\x74\x72\x79\x20\x46\x69\x6c\x65\x20\x46\x6f\x72\x6d\x61\x74(\x20){7}\x35/
		$p1_0 = /\x33\x44\x20\x47\x65\x6f\x6d\x65\x74\x72\x79\x20\x46\x69\x6c\x65\x20\x46\x6f\x72\x6d\x61\x74(\x20){8}\x31\x01(\x00){3}/
		$p2_0 = /\x33\x44\x20\x47\x65\x6f\x6d\x65\x74\x72\x79\x20\x46\x69\x6c\x65\x20\x46\x6f\x72\x6d\x61\x74(\x20){7}\x38/
		$p3_0 = /\x33\x44\x20\x47\x65\x6f\x6d\x65\x74\x72\x79\x20\x46\x69\x6c\x65\x20\x46\x6f\x72\x6d\x61\x74(\x20){8}\x33\x01(\x00){3}/
		$p4_0 = /\x33\x44\x20\x47\x65\x6f\x6d\x65\x74\x72\x79\x20\x46\x69\x6c\x65\x20\x46\x6f\x72\x6d\x61\x74(\x20){8}\x34\x01(\x00){3}/
		$p5_0 = /\x33\x44\x20\x47\x65\x6f\x6d\x65\x74\x72\x79\x20\x46\x69\x6c\x65\x20\x46\x6f\x72\x6d\x61\x74(\x20){7}\x37/
		$p6_0 = /\x33\x44\x20\x47\x65\x6f\x6d\x65\x74\x72\x79\x20\x46\x69\x6c\x65\x20\x46\x6f\x72\x6d\x61\x74(\x20){7}\x36/
		$p7_0 = /\x33\x44\x20\x47\x65\x6f\x6d\x65\x74\x72\x79\x20\x46\x69\x6c\x65\x20\x46\x6f\x72\x6d\x61\x74(\x20){8}\x32\x01(\x00){3}/

	condition:
		(($p0_0 at 0) or ($p1_0 at 0) or ($p2_0 at 0) or ($p3_0 at 0) or ($p4_0 at 0) or ($p5_0 at 0) or ($p6_0 at 0) or ($p7_0 at 0))
}

rule taxonomy_squashfs
{
	meta:
        source_refs = "libmagic:magic/Magdir/filesystems:libmagic_433f12e5915c0d5c3467_line_2213; libmagic:magic/Magdir/filesystems:libmagic_8651467cdbac3bc62094_line_2216"
		label = "squashfs"
		enforced = false
        class = "not-working"
        fp_rate = 0.00023790911871665
        fn_rate = 0

	strings:
		$p0_0 = "hsqs"
		$p1_0 = "sqsh"

	condition:
		((prefix_size >= 4 and original_size >= 4 and $p0_0 at 0) or (prefix_size >= 4 and original_size >= 4 and $p1_0 at 0))
}

rule taxonomy_tar
{
	meta:
        source_refs = "puremagic:puremagic/magic_data.json:headers[232]; puremagic:puremagic/magic_data.json:headers[233]; puremagic:puremagic/magic_data.json:headers[810]; puremagic:puremagic/magic_data.json:headers[811]; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[1099]/magic[0]"
		label = "tar"
		enforced = false
        class = "not-working"
        fp_rate = 0.00006797403391904
        fn_rate = 0.02

	strings:
		$p0_0 = "ustar\\040\\040\\0"
		$p1_0 = "ustar\\0"
		$p2_0 = "ustar"
		$p3_0 = { 75 73 74 61 72 00 }

	condition:
		((prefix_size >= 272 and original_size >= 272 and $p0_0 at 257) or (prefix_size >= 264 and original_size >= 264 and $p1_0 at 257) or (prefix_size >= 262 and original_size >= 262 and $p2_0 at 257) or (prefix_size >= 263 and $p3_0 at 257))
}

rule taxonomy_tga
{
	meta:
        source_refs = "puremagic:puremagic/magic_data.json:headers[893]"
		label = "tga"
		enforced = false
        class = "not-working"
        fp_rate = 0
        fn_rate = 1

	strings:
		$p0_0 = "\\0\\2"

	condition:
		(prefix_size >= 5 and original_size >= 5 and $p0_0 at 1)
}

rule taxonomy_tiff
{
	meta:
        source_refs = "libmagic:magic/Magdir/images:libmagic_9197a5d25a78c07e90d4_line_480; libmagic:magic/Magdir/images:libmagic_b36d98418f01d87a7b58_line_482; libmagic:magic/Magdir/images:libmagic_c8a071e170bc86042246_line_321; libmagic:magic/Magdir/images:libmagic_e733d2c5ff0bf3288a06_line_326; puremagic:puremagic/magic_data.json:headers[136]; puremagic:puremagic/magic_data.json:headers[137]; puremagic:puremagic/magic_data.json:headers[138]; puremagic:puremagic/magic_data.json:headers[139]; puremagic:puremagic/magic_data.json:headers[524]; puremagic:puremagic/magic_data.json:headers[586]; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[1339]/magic[0]"
		label = "tiff"
		enforced = false
        class = "not-working"
        fp_rate = 0.00356863678074975
        fn_rate = 0

	strings:
		$p0_0 = { 4D 4D 00 2B }
		$p1_0 = "I I"
		$p2_0 = { 49 49 2b 00 }
		$p3_0 = { 4D 4D 00 2A }
		$p4_0 = { 49 49 2A 00 }

	condition:
		((prefix_size >= 4 and original_size >= 4 and $p0_0 at 0) or (prefix_size >= 3 and original_size >= 3 and $p1_0 at 0) or (prefix_size >= 4 and $p2_0 at 0) or (prefix_size >= 4 and original_size >= 4 and $p3_0 at 0) or (prefix_size >= 4 and original_size >= 4 and $p4_0 at 0))
}

rule taxonomy_webm
{
	meta:
        source_refs = "puremagic:puremagic/magic_data.json:headers[98]"
		label = "webm"
		enforced = false
        class = "not-working"
        fp_rate = 0.00339870169595215
        fn_rate = 0

	strings:
		$p0_0 = { 1A 45 DF A3 }

	condition:
		(prefix_size >= 4 and original_size >= 4 and $p0_0 at 0)
}

rule taxonomy_webp
{
	meta:
        source_refs = "libmagic:magic/Magdir/riff:libmagic_b2aac9f8242bb4da4291_line_698; puremagic:puremagic/magic_data.json:headers[1191]; puremagic:puremagic/magic_data.json:multi-part[52494646][14]; puremagic:puremagic/magic_data.json:multi-part[52494646][15]; puremagic:puremagic/magic_data.json:multi-part[52494646][16]; puremagic:puremagic/magic_data.json:multi-part[52494646][17]; puremagic:puremagic/magic_data.json:multi-part[52494646][2]; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[1371]/magic[0]"
		label = "webp"
		enforced = false
        class = "not-working"
        fp_rate = 0.01019610508785644
        fn_rate = 0.02

	strings:
		$p0_0 = { 52 49 46 46 ?? ?? ?? ?? 57 45 42 50 }
		$p1_0 = "RIFF"

	condition:
		((prefix_size >= 12 and $p0_0 at 0) or (prefix_size >= 4 and original_size >= 4 and $p1_0 at 0))
}

rule taxonomy_wim
{
	meta:
        source_refs = "puremagic:puremagic/magic_data.json:headers[815]"
		label = "wim"
		enforced = false
        class = "not-working"
        fp_rate = 0
        fn_rate = 1

	strings:
		$p0_0 = "MSWIM\\000\\000\\000"

	condition:
		(prefix_size >= 17 and original_size >= 17 and $p0_0 at 0)
}

rule taxonomy_wma
{
	meta:
        source_refs = "puremagic:puremagic/magic_data.json:headers[151]"
		label = "wma"
		enforced = false
        class = "not-working"
        fp_rate = 0.00006774379297497
        fn_rate = "unmeasured"

	strings:
		$p0_0 = { 30 26 B2 75 8E 66 CF 11 A6 D9 00 AA 00 62 CE 6C }

	condition:
		(prefix_size >= 16 and original_size >= 16 and $p0_0 at 0)
}

rule taxonomy_wmf
{
	meta:
        source_refs = "puremagic:puremagic/magic_data.json:headers[896]; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[1376]/magic[0]"
		label = "wmf"
		enforced = false
        class = "not-working"
        fp_rate = 0.00227713013628794
        fn_rate = 0

	strings:
		$p0_0 = { 01 00 09 00 00 03 }
		$p1_0 = { D7 CD C6 9A 00 00 }
		$p2_0 = { 01 00 }

	condition:
		((prefix_size >= 6 and $p0_0 at 0) or (prefix_size >= 6 and $p1_0 at 0) or (prefix_size >= 2 and original_size >= 2 and $p2_0 at 0))
}

rule taxonomy_wmv
{
	meta:
        source_refs = "puremagic:puremagic/magic_data.json:headers[146]; puremagic:puremagic/magic_data.json:headers[269]"
		label = "wmv"
		enforced = false
        class = "not-working"
        fp_rate = 0.00006774379297497
        fn_rate = "unmeasured"

	strings:
		$p0_0 = { 30 26 B2 75 8E 66 CF 11 A6 D9 00 AA 00 62 CE 6C }
		$p1_0 = { 30 26 B2 75 8E 66 CF 11 }

	condition:
		((prefix_size >= 16 and original_size >= 16 and $p0_0 at 0) or (prefix_size >= 8 and original_size >= 8 and $p1_0 at 0))
}

rule taxonomy_xls
{
	meta:
        source_refs = "puremagic:puremagic/magic_data.json:headers[661]"
		label = "xls"
		enforced = false
        class = "not-working"
        fp_rate = 0
        fn_rate = 1

	strings:
		$p0_0 = "Microsoft Excel 5.0 Worksheet"

	condition:
		(prefix_size >= 2109 and original_size >= 2109 and $p0_0 at 2080)
}

rule taxonomy_xlsb
{
	meta:
        source_refs = "puremagic:puremagic/magic_data.json:headers[69]"
		label = "xlsb"
		enforced = false
        class = "not-working"
        fp_rate = 0.0653910206301193
        fn_rate = 0

	strings:
		$p0_0 = { 50 4B 03 04 }

	condition:
		(prefix_size >= 4 and original_size >= 4 and $p0_0 at 0)
}

rule taxonomy_xlsx
{
	meta:
        source_refs = "puremagic:puremagic/magic_data.json:headers[68]"
		label = "xlsx"
		enforced = false
        class = "not-working"
        fp_rate = 0.0653910206301193
        fn_rate = 0

	strings:
		$p0_0 = { 50 4B 03 04 }

	condition:
		(prefix_size >= 4 and original_size >= 4 and $p0_0 at 0)
}

rule xz_v1 {
    meta:
        format_id = "xz"
        label = "xz"
        enforced = false
        class = "not-working"
        fp_rate = "unmeasured"
        fn_rate = "unmeasured"
        status = "blocked_ambiguous_polyglot"
        minimum_prefix = 12

    condition:
        prefix_size >= 12 and original_size >= 32 and original_size % 4 == 0 and
        uint32be(0) == 0xfd377a58 and uint16be(4) == 0x5a00 and uint8(6) == 0 and
        (
            (uint8(7) == 0 and uint32(8) == 0x41d912ff) or
            (uint8(7) == 1 and uint32(8) == 0x36de2269) or
            (uint8(7) == 4 and uint32(8) == 0x46b4d6e6) or
            (uint8(7) == 10 and uint32(8) == 0xa10cfbe1)
        )
}

rule taxonomy_zlibstream
{
	meta:
        source_refs = "tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[1157]/magic[0]"
		label = "zlibstream"
		enforced = false
        class = "not-working"
        fp_rate = 0.00288889644155932
        fn_rate = 0

	strings:
		$p0_0 = { 78 9C }
		$p1_0 = "x^"
		$p2_0 = { 78 DA }
		$p3_0 = { 78 01 }

	condition:
		((prefix_size >= 2 and $p0_0 at 0) or (prefix_size >= 2 and $p1_0 at 0) or (prefix_size >= 2 and $p2_0 at 0) or (prefix_size >= 2 and $p3_0 at 0))
}

rule sqlite_v1 {
    meta:
        format_id = "sqlite"
        label = "sqlite"
        enforced = false
        class = "not-working"
        fp_rate = "unmeasured"
        fn_rate = "unmeasured"
        status = "blocked_unresolved_subtypes"
        minimum_prefix = 100

    condition:
        prefix_size >= 100 and original_size >= 512 and
        uint32be(0) == 0x53514c69 and uint32be(4) == 0x74652066 and
        uint32be(8) == 0x6f726d61 and uint32be(12) == 0x74203300 and
        (uint8(18) == 1 or uint8(18) == 2) and (uint8(19) == 1 or uint8(19) == 2) and
        uint8(21) == 64 and uint8(22) == 32 and uint8(23) == 32 and
        uint32be(44) >= 1 and uint32be(44) <= 4 and
        uint32be(56) >= 1 and uint32be(56) <= 3 and
        uint32be(72) == 0 and uint32be(76) == 0 and uint32be(80) == 0 and
        uint32be(84) == 0 and uint32be(88) == 0 and uint32be(96) > 0 and
        (
            (uint16be(16) == 512 and original_size % 512 == 0) or
            (uint16be(16) == 1024 and original_size % 1024 == 0) or
            (uint16be(16) == 2048 and original_size % 2048 == 0) or
            (uint16be(16) == 4096 and original_size % 4096 == 0) or
            (uint16be(16) == 8192 and original_size % 8192 == 0) or
            (uint16be(16) == 16384 and original_size % 16384 == 0) or
            (uint16be(16) == 32768 and original_size % 32768 == 0) or
            (uint16be(16) == 1 and original_size % 65536 == 0)
        )
}


// ARC method plus filename bytes is too weak without archive record validation.
rule taxonomy_arc
{
	meta:
        source_refs = "libmagic:magic/Magdir/archive:libmagic_1631bb45451c23fb6aac_line_598; libmagic:magic/Magdir/archive:libmagic_243a5e9cf3cfc790f9f5_line_594; libmagic:magic/Magdir/archive:libmagic_2cf74413ac8980dea53f_line_592; libmagic:magic/Magdir/archive:libmagic_35a363d7bd396019e8f6_line_607; libmagic:magic/Magdir/archive:libmagic_610c9e502ebb6af29ccb_line_600; libmagic:magic/Magdir/archive:libmagic_a17cb6a06bbbdcf78f3a_line_605; libmagic:magic/Magdir/archive:libmagic_b419cbf5f668942ab6e5_line_602; libmagic:magic/Magdir/archive:libmagic_e1ba35801790f0f6eeae_line_609; libmagic:magic/Magdir/archive:libmagic_f1405c71130bec4d78bc_line_596; puremagic:puremagic/magic_data.json:headers[1078]; puremagic:puremagic/magic_data.json:headers[1079]; puremagic:puremagic/magic_data.json:headers[1080]; puremagic:puremagic/magic_data.json:headers[1081]; puremagic:puremagic/magic_data.json:headers[1082]; puremagic:puremagic/magic_data.json:headers[1083]"
		label = "arc"
		enforced = false
        class = "not-working"
        fp_rate = "unmeasured"
        fn_rate = "unmeasured"

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

// ORC identifying postscript is at EOF; three ASCII bytes cannot support an enforced prefix decision.
rule taxonomy_orc
{
	meta:
        source_refs = "libmagic:magic/Magdir/apache:libmagic_11a425bf4c85f8783ee1_line_11"
		label = "orc"
		enforced = false
        class = "not-working"
        fp_rate = "unmeasured"
        fn_rate = "unmeasured"

	strings:
		$p0_0 = { 4f 52 43 }

	condition:
		(prefix_size >= 3 and $p0_0 at 0)
}

// Two-byte XCOFF magic is insufficient; section/symbol table validation remains pending.
rule taxonomy_xcoff
{
	meta:
        source_refs = "libmagic:magic/Magdir/ibm6000:libmagic_72f92e57f4bd2f191a75_line_28"
		label = "xcoff"
		enforced = false
        class = "not-working"
        fp_rate = "unmeasured"
        fn_rate = "unmeasured"

	strings:
		$p0_0 = { 01 f7 }

	condition:
		(prefix_size >= 2 and $p0_0 at 0)
}
