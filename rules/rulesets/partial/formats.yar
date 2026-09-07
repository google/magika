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

rule taxonomy_apk
{
	meta:
        source_refs = "libmagic:magic/Magdir/archive:libmagic_9418b81d75bfd1980625_line_1861; libmagic:magic/Magdir/archive:libmagic_9418b81d75bfd1980625_line_1876"
		label = "apk"
		enforced = true
        class = "partial"
        fp_rate = 0
        fn_rate = 0.26000000000000001

	strings:
		$p0_0 = { 0b 00 }
		$p1_0 = { 41 6e 64 72 6f 69 64 4d 61 6e 69 66 65 73 74 2e 78 6d 6c }
		$p2_0 = { 13 00 }
		$p3_0 = { 50 4B 03 04 }
		$p4_0 = { 63 6c 61 73 73 65 73 2e 64 65 78 }

	condition:
		((prefix_size >= 4 and original_size >= 4 and $p3_0 at 0) and (((prefix_size >= 28 and $p0_0 at 26) and (prefix_size >= 41 and $p4_0 at 30)) or ((prefix_size >= 49 and $p1_0 at 30) and (prefix_size >= 28 and $p2_0 at 26))))
}

rule taxonomy_bmp
{
	meta:
        source_refs = "puremagic:puremagic/magic_data.json:headers[856]; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[1310]/magic[0]"
		label = "bmp"
		enforced = true
        class = "partial"
        fp_rate = 0
        fn_rate = 0.02

	strings:
		$p0_0 = { 18 00 }
		$p1_0 = { 08 00 }
		$p2_0 = { 01 00 }
		$p3_0 = { 01 00 }
		$p4_0 = "BMxxxx\\000\\000"
		$p5_0 = { 20 00 }
		$p6_0 = { 10 00 }
		$p7_0 = "BM"
		$p8_0 = { 00 00 }
		$p9_0 = { 04 00 }
		$p10_0 = { 00 00 00 00 }

	condition:
		(((prefix_size >= 28 and $p3_0 at 26) and ((prefix_size >= 2 and original_size >= 2 and $p7_0 at 0) and ((prefix_size >= 34 and $p10_0 at 30) and ((prefix_size >= 30 and $p0_0 at 28) or (prefix_size >= 30 and $p1_0 at 28) or (prefix_size >= 30 and $p2_0 at 28) or (prefix_size >= 30 and $p5_0 at 28) or (prefix_size >= 30 and $p6_0 at 28) or (prefix_size >= 30 and $p8_0 at 28) or (prefix_size >= 30 and $p9_0 at 28))))) or (prefix_size >= 14 and original_size >= 14 and $p4_0 at 0))
}

rule taxonomy_cab
{
	meta:
        source_refs = "puremagic:puremagic/magic_data.json:headers[240]; puremagic:puremagic/magic_data.json:headers[660]; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[525]/magic[0]; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[525]/magic[1]"
		label = "cab"
		enforced = true
        class = "partial"
        fp_rate = 0
        fn_rate = 0.44

	strings:
		$p0_0 = "MSCF"
		$p1_0 = { 4D 53 43 46 00 00 00 00 }
		$p2_0 = "MSCF\\0\\0\\0\\0"

	condition:
		((prefix_size >= 4 and original_size >= 4 and $p0_0 at 0) or (prefix_size >= 8 and $p1_0 at 0) or (prefix_size >= 12 and original_size >= 12 and $p2_0 at 0))
}

rule taxonomy_crx
{
	meta:
        source_refs = "libmagic:magic/Magdir/archive:libmagic_e974706a17bde5f0cc12_line_2655"
		label = "crx"
		enforced = true
        class = "partial"
        fp_rate = 0
        fn_rate = 0.03

	strings:
		$p0_0 = "Cr24"

	condition:
		(prefix_size >= 4 and $p0_0 at 0)
}

rule taxonomy_dbase
{
	meta:
        source_refs = "pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:381; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:382; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:384; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:385; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:544; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:545; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:546; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:547; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:548; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:549; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:550"
		label = "dbase"
		enforced = true
        class = "partial"
        fp_rate = 0
        fn_rate = 0.03

	strings:
		$p0_0 = /\x8b(([\x00-\xff]){1}[\x01-\x0c][\x01-\x1f])(([\x00-\xff]){28}(\x41|\x42|\x43|\x44|\x45|\x46|\x47|\x48|\x49|\x4a|\x4b|\x4c|\x4d|\x4e|\x4f|\x50|\x51|\x52|\x53|\x54|\x55|\x56|\x57|\x58|\x59|\x5a|\x61|\x62|\x63|\x64|\x65|\x66|\x67|\x68|\x69|\x6a|\x6b|\x6c|\x6d|\x6e|\x6f|\x70|\x71|\x72|\x73|\x74|\x75|\x76|\x77|\x78|\x79|\x7a))(([\x00-\xff]){10}(\x43|\x44|\x46|\x4c|\x4d|\x4e))/
		$p1_0 = /\x02([\x00-\xff]){2}(\x00){3}(([\x00-\xff]){1}[\x00-\x03])((\x41|\x42|\x43|\x44|\x45|\x46|\x47|\x48|\x49|\x4a|\x4b|\x4c|\x4d|\x4e|\x4f|\x50|\x51|\x52|\x53|\x54|\x55|\x56|\x57|\x58|\x59|\x5a|\x61|\x62|\x63|\x64|\x65|\x66|\x67|\x68|\x69|\x6a|\x6b|\x6c|\x6d|\x6e|\x6f|\x70|\x71|\x72|\x73|\x74|\x75|\x76|\x77|\x78|\x79|\x7a))(([\x00-\xff]){10}(\x43|\x4c|\x4e))/
		$p2_0 = /\x02(([\x00-\xff]){2}[\x01-\x1c][\x01-\x1f])(([\x00-\xff]){2}[\x00-\x03])((\x41|\x42|\x43|\x44|\x45|\x46|\x47|\x48|\x49|\x4a|\x4b|\x4c|\x4d|\x4e|\x4f|\x50|\x51|\x52|\x53|\x54|\x55|\x56|\x57|\x58|\x59|\x5a|\x61|\x62|\x63|\x64|\x65|\x66|\x67|\x68|\x69|\x6a|\x6b|\x6c|\x6d|\x6e|\x6f|\x70|\x71|\x72|\x73|\x74|\x75|\x76|\x77|\x78|\x79|\x7a))(([\x00-\xff]){10}(\x43|\x4c|\x4e))/
		$p3_0 = /\xcb(([\x00-\xff]){1}[\x01-\x0c][\x01-\x1f])(([\x00-\xff]){28}(\x41|\x42|\x43|\x44|\x45|\x46|\x47|\x48|\x49|\x4a|\x4b|\x4c|\x4d|\x4e|\x4f|\x50|\x51|\x52|\x53|\x54|\x55|\x56|\x57|\x58|\x59|\x5a|\x61|\x62|\x63|\x64|\x65|\x66|\x67|\x68|\x69|\x6a|\x6b|\x6c|\x6d|\x6e|\x6f|\x70|\x71|\x72|\x73|\x74|\x75|\x76|\x77|\x78|\x79|\x7a))(([\x00-\xff]){10}(\x43|\x44|\x46|\x4c|\x4d|\x4e))/
		$p4_0 = /\x04(([\x00-\xff]){1}[\x01-\x0c][\x01-\x1f])(([\x00-\xff]){28}(\x41|\x42|\x43|\x44|\x45|\x46|\x47|\x48|\x49|\x4a|\x4b|\x4c|\x4d|\x4e|\x4f|\x50|\x51|\x52|\x53|\x54|\x55|\x56|\x57|\x58|\x59|\x5a|\x61|\x62|\x63|\x64|\x65|\x66|\x67|\x68|\x69|\x6a|\x6b|\x6c|\x6d|\x6e|\x6f|\x70|\x71|\x72|\x73|\x74|\x75|\x76|\x77|\x78|\x79|\x7a))(([\x00-\xff]){10}(\x43|\x44|\x46|\x4c|\x4e))/
		$p5_0 = /\x03(([\x00-\xff]){1}[\x01-\x0c][\x01-\x1f])(([\x00-\xff]){28}(\x41|\x42|\x43|\x44|\x45|\x46|\x47|\x48|\x49|\x4a|\x4b|\x4c|\x4d|\x4e|\x4f|\x50|\x51|\x52|\x53|\x54|\x55|\x56|\x57|\x58|\x59|\x5a|\x61|\x62|\x63|\x64|\x65|\x66|\x67|\x68|\x69|\x6a|\x6b|\x6c|\x6d|\x6e|\x6f|\x70|\x71|\x72|\x73|\x74|\x75|\x76|\x77|\x78|\x79|\x7a))(([\x00-\xff]){10}(\x43|\x44|\x4c|\x4d|\x4e))/
		$p6_0 = /\x43(([\x00-\xff]){1}[\x01-\x0c][\x01-\x1f])(([\x00-\xff]){28}(\x41|\x42|\x43|\x44|\x45|\x46|\x47|\x48|\x49|\x4a|\x4b|\x4c|\x4d|\x4e|\x4f|\x50|\x51|\x52|\x53|\x54|\x55|\x56|\x57|\x58|\x59|\x5a|\x61|\x62|\x63|\x64|\x65|\x66|\x67|\x68|\x69|\x6a|\x6b|\x6c|\x6d|\x6e|\x6f|\x70|\x71|\x72|\x73|\x74|\x75|\x76|\x77|\x78|\x79|\x7a))(([\x00-\xff]){10}(\x43|\x44|\x46|\x4c|\x4e))/
		$p7_0 = /\x8e(([\x00-\xff]){1}[\x01-\x0c][\x01-\x1f])(([\x00-\xff]){28}(\x41|\x42|\x43|\x44|\x45|\x46|\x47|\x48|\x49|\x4a|\x4b|\x4c|\x4d|\x4e|\x4f|\x50|\x51|\x52|\x53|\x54|\x55|\x56|\x57|\x58|\x59|\x5a|\x61|\x62|\x63|\x64|\x65|\x66|\x67|\x68|\x69|\x6a|\x6b|\x6c|\x6d|\x6e|\x6f|\x70|\x71|\x72|\x73|\x74|\x75|\x76|\x77|\x78|\x79|\x7a))(([\x00-\xff]){10}(\x43|\x44|\x46|\x4c|\x4d|\x4e))/
		$p8_0 = /\x83(([\x00-\xff]){1}[\x01-\x0c][\x01-\x1f])(([\x00-\xff]){28}(\x41|\x42|\x43|\x44|\x45|\x46|\x47|\x48|\x49|\x4a|\x4b|\x4c|\x4d|\x4e|\x4f|\x50|\x51|\x52|\x53|\x54|\x55|\x56|\x57|\x58|\x59|\x5a|\x61|\x62|\x63|\x64|\x65|\x66|\x67|\x68|\x69|\x6a|\x6b|\x6c|\x6d|\x6e|\x6f|\x70|\x71|\x72|\x73|\x74|\x75|\x76|\x77|\x78|\x79|\x7a))(([\x00-\xff]){10}(\x43|\x44|\x4c|\x4d|\x4e))/
		$p9_0 = /\x7b(([\x00-\xff]){1}[\x01-\x0c][\x01-\x1f])(([\x00-\xff]){28}(\x41|\x42|\x43|\x44|\x45|\x46|\x47|\x48|\x49|\x4a|\x4b|\x4c|\x4d|\x4e|\x4f|\x50|\x51|\x52|\x53|\x54|\x55|\x56|\x57|\x58|\x59|\x5a|\x61|\x62|\x63|\x64|\x65|\x66|\x67|\x68|\x69|\x6a|\x6b|\x6c|\x6d|\x6e|\x6f|\x70|\x71|\x72|\x73|\x74|\x75|\x76|\x77|\x78|\x79|\x7a))(([\x00-\xff]){10}(\x43|\x44|\x46|\x4c|\x4d|\x4e))/
		$p10_0 = /\x63(([\x00-\xff]){1}[\x01-\x0c][\x01-\x1f])(([\x00-\xff]){28}(\x41|\x42|\x43|\x44|\x45|\x46|\x47|\x48|\x49|\x4a|\x4b|\x4c|\x4d|\x4e|\x4f|\x50|\x51|\x52|\x53|\x54|\x55|\x56|\x57|\x58|\x59|\x5a|\x61|\x62|\x63|\x64|\x65|\x66|\x67|\x68|\x69|\x6a|\x6b|\x6c|\x6d|\x6e|\x6f|\x70|\x71|\x72|\x73|\x74|\x75|\x76|\x77|\x78|\x79|\x7a))(([\x00-\xff]){10}(\x43|\x44|\x46|\x4c|\x4e))/

	condition:
		(($p0_0 at 0) or ($p1_0 at 0) or ($p2_0 at 0) or ($p3_0 at 0) or ($p4_0 at 0) or ($p5_0 at 0) or ($p6_0 at 0) or ($p7_0 at 0) or ($p8_0 at 0) or ($p9_0 at 0) or ($p10_0 at 0))
}

rule taxonomy_emf
{
	meta:
        source_refs = "pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:476; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:477; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:478; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:479; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:480; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:481; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:482; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:483"
		label = "emf"
		enforced = true
        class = "partial"
        fp_rate = 0
        fn_rate = 0.34999999999999998

	strings:
		$p0_0 = /\x01(\x00){3}([\x00-\xff]){36}\x20\x45\x4d\x46(\x00){2}\x01\x00(([\x00-\xff]){16}\x64(\x00){3})/
		$p1_0 = /\x01(\x00){3}\x58(\x00){3}(([\x00-\xff]){32}\x20\x45\x4d\x46(\x00){2}\x01\x00)(([\x00-\xff]){16}(\x00){4})/
		$p2_0 = /\x01(\x00){3}\x64(\x00){3}(([\x00-\xff]){32}\x20\x45\x4d\x46(\x00){2}\x01\x00)(([\x00-\xff]){12}(\x00){4})(([\x00-\xff]){28}(\x00){4})/
		$p3_0 = /\x01(\x00){3}([\x00-\xff]){36}\x20\x45\x4d\x46(\x00){2}\x01\x00(([\x00-\xff]){44}\x64(\x00){3})/
		$p4_0 = /\x01(\x00){3}([\x00-\xff]){36}\x20\x45\x4d\x46(\x00){2}\x01\x00(([\x00-\xff]){16}\x58(\x00){3})/
		$p5_0 = /\x01(\x00){3}\x6c(\x00){3}(([\x00-\xff]){32}\x20\x45\x4d\x46(\x00){2}\x01\x00)(([\x00-\xff]){12}(\x00){4})(([\x00-\xff]){28}(\x00){4})/
		$p6_0 = /\x01(\x00){3}([\x00-\xff]){36}\x20\x45\x4d\x46(\x00){2}\x01\x00(([\x00-\xff]){16}\x6c(\x00){3})/
		$p7_0 = /\x01(\x00){3}([\x00-\xff]){36}\x20\x45\x4d\x46(\x00){2}\x01\x00(([\x00-\xff]){44}\x6c(\x00){3})/

	condition:
		(($p0_0 at 0) or ($p1_0 at 0) or ($p2_0 at 0) or ($p3_0 at 0) or ($p4_0 at 0) or ($p5_0 at 0) or ($p6_0 at 0) or ($p7_0 at 0))
}

rule taxonomy_epub
{
	meta:
        source_refs = "libmagic:magic/Magdir/archive:libmagic_9418b81d75bfd1980625_line_2058; puremagic:puremagic/magic_data.json:headers[84]; puremagic:puremagic/magic_data.json:headers[85]; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[52]/magic[0]"
		label = "epub"
		enforced = true
        class = "partial"
        fp_rate = 0
        fn_rate = 0.03

	strings:
		$p0_0 = { 65 70 75 62 2b 7a 69 70 }
		$p1_0 = "mimetypeapplication/epub+zip"
		$p2_0 = "PK\\003\\004"
		$p3_0 = { 08 00 00 00 6d 69 6d 65 74 79 70 65 61 70 70 6c 69 63 61 74 69 6f 6e 2f }
		$p4_0 = { 50 4B 03 04 }

	condition:
		(((prefix_size >= 58 and $p0_0 at 50) and (prefix_size >= 50 and $p3_0 at 26) and (prefix_size >= 4 and original_size >= 4 and $p4_0 at 0)) or (prefix_size >= 58 and original_size >= 58 and $p1_0 at 30) or (prefix_size >= 10 and original_size >= 10 and $p2_0 at 0))
}

rule taxonomy_flac
{
	meta:
        source_refs = "libmagic:magic/Magdir/audio:libmagic_7dd2174c21f3a2697c4b_line_493; puremagic:puremagic/magic_data.json:headers[105]; puremagic:puremagic/magic_data.json:headers[56]"
		label = "flac"
		enforced = true
        class = "partial"
        fp_rate = 0
        fn_rate = 0.08

	strings:
		$p0_0 = "fLaC"
		$p1_0 = { 66 4C 61 43 00 00 00 22 }

	condition:
		((prefix_size >= 4 and original_size >= 4 and $p0_0 at 0) or (prefix_size >= 8 and original_size >= 8 and $p1_0 at 0))
}

rule taxonomy_gif
{
	meta:
        source_refs = "libmagic:magic/Magdir/images:libmagic_24dbfe86146e80bb7919_line_568; puremagic:puremagic/magic_data.json:headers[199]; puremagic:puremagic/magic_data.json:headers[200]; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[1320]/magic[0]"
		label = "gif"
		enforced = true
        class = "partial"
        fp_rate = 0
        fn_rate = 0.03

	strings:
		$p0_0 = { 47 49 46 38 }
		$p1_0 = "GIF87a"
		$p2_0 = "GIF89a"

	condition:
		((prefix_size >= 4 and $p0_0 at 0) or (prefix_size >= 6 and original_size >= 6 and $p1_0 at 0) or (prefix_size >= 6 and original_size >= 6 and $p2_0 at 0))
}

rule taxonomy_hlp
{
	meta:
        source_refs = "puremagic:puremagic/magic_data.json:multi-part[3f5f0300][0]"
		label = "hlp"
		enforced = true
        class = "partial"
        fp_rate = 0
        fn_rate = 0.02

	strings:
		$p0_0 = { 3F 5F 03 00 }
		$p1_0 = { 00 00 FF FF FF FF }

	condition:
		((prefix_size >= 4 and original_size >= 4 and $p0_0 at 0) and (prefix_size >= 12 and original_size >= 12 and $p1_0 at 6))
}

rule taxonomy_ico
{
	meta:
        source_refs = "puremagic:puremagic/magic_data.json:headers[882]; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[1359]/magic[0]"
		label = "ico"
		enforced = true
        class = "partial"
        fp_rate = 0
        fn_rate = 0.46999999999999997

	strings:
		$p0_0 = { 00 00 01 00 }
		$p1_0 = { 42 41 28 00 00 00 2E 00 00 00 00 00 00 00 }
		$p2_0 = "\\0\\0\\1\\0"

	condition:
		((prefix_size >= 4 and original_size >= 4 and $p0_0 at 0) or (prefix_size >= 14 and $p1_0 at 0) or (prefix_size >= 8 and original_size >= 8 and $p2_0 at 0))
}

rule taxonomy_jp2
{
	meta:
        source_refs = "pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:251"
		label = "jp2"
		enforced = true
        class = "partial"
        fp_rate = 0
        fn_rate = 0.03

	strings:
		$p0_0 = /(\x00){3}\x0c\x6a\x50(\x20){2}\x0d\x0a\x87\x0a(([\x00-\xff]){4}\x66\x74\x79\x70\x6a\x70\x32)/

	condition:
		($p0_0 at 0)
}

rule taxonomy_luabytecode
{
	meta:
        source_refs = "libmagic:magic/Magdir/lua:libmagic_328f668b9e354b26ac15_line_21"
		label = "luabytecode"
		enforced = true
        class = "partial"
        fp_rate = 0
        fn_rate = 0.47999999999999998

	strings:
		$p0_0 = { 1b 4c 75 61 }

	condition:
		(prefix_size >= 4 and $p0_0 at 0)
}

rule taxonomy_macho
{
	meta:
        source_refs = "tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[983]/magic[0]"
		label = "macho"
		enforced = true
        class = "partial"
        fp_rate = 0
        fn_rate = 0.09

	strings:
		$p0_0 = { FE ED FA CE }
		$p1_0 = { FE ED FA CF }
		$p2_0 = { CE FA ED FE }
		$p3_0 = { CF FA ED FE }

	condition:
		((prefix_size >= 4 and original_size >= 4 and $p0_0 at 0) or (prefix_size >= 4 and original_size >= 4 and $p1_0 at 0) or (prefix_size >= 4 and original_size >= 4 and $p2_0 at 0) or (prefix_size >= 4 and original_size >= 4 and $p3_0 at 0))
}

rule taxonomy_mkv
{
	meta:
        source_refs = "puremagic:puremagic/magic_data.json:headers[1249]; puremagic:puremagic/magic_data.json:headers[303]; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[1686]/magic[0]"
		label = "mkv"
		enforced = true
        class = "partial"
        fp_rate = 0
        fn_rate = 0.01

	strings:
		$p0_0 = { 1A 45 DF A3 93 42 82 88 6D 61 74 72 6F 73 6B 61 }
		$p1_0 = "matroska"
		$p2_0 = "matroska"

	condition:
		((prefix_size >= 16 and $p0_0 at 0) or (prefix_size >= 39 and original_size >= 39 and $p1_0 at 31) or (prefix_size >= 32 and original_size >= 32 and $p2_0 at 24))
}

rule taxonomy_mscompress
{
	meta:
        source_refs = "libmagic:magic/Magdir/archive:libmagic_ddaccc184422c32cd78b_line_950; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[998]/magic[0]"
		label = "mscompress"
		enforced = true
        class = "partial"
        fp_rate = 0
        fn_rate = 0.64000000000000001

	strings:
		$p0_0 = { 53 5a 44 44 }
		$p1_0 = { 88 f0 27 }
		$p2_0 = { 53 5A 44 44 88 F0 27 33 41 }

	condition:
		(((prefix_size >= 4 and $p0_0 at 0) and (prefix_size >= 7 and $p1_0 at 4)) or (prefix_size >= 9 and $p2_0 at 0))
}

rule taxonomy_netcdf
{
	meta:
        source_refs = "pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:298; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:299; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[1028]/magic[0]"
		label = "netcdf"
		enforced = true
        class = "partial"
        fp_rate = 0
        fn_rate = 0.23000000000000001

	strings:
		$p0_0 = { 43 44 46 02 }
		$p1_0 = { 43 44 46 01 }

	condition:
		((prefix_size >= 4 and $p0_0 at 0) or (prefix_size >= 4 and $p1_0 at 0))
}

rule taxonomy_ogg
{
	meta:
        source_refs = "libmagic:magic/Magdir/vorbis:libmagic_7c4adde2e72d289e8479_line_136; libmagic:magic/Magdir/vorbis:libmagic_7c4adde2e72d289e8479_line_31; libmagic:magic/Magdir/vorbis:libmagic_7c4adde2e72d289e8479_line_53; libmagic:magic/Magdir/vorbis:libmagic_7c4adde2e72d289e8479_line_63; puremagic:puremagic/magic_data.json:headers[99]"
		label = "ogg"
		enforced = true
        class = "partial"
        fp_rate = 0
        fn_rate = 0.56999999999999995

	strings:
		$p0_0 = "OggS"

	condition:
		(prefix_size >= 4 and original_size >= 4 and $p0_0 at 0)
}

rule taxonomy_pcap
{
	meta:
        source_refs = "libmagic:magic/Magdir/sniffer:libmagic_11a07bcd8c17007e3d09_line_295; libmagic:magic/Magdir/sniffer:libmagic_496cf6fa510b99510d92_line_284; libmagic:magic/Magdir/sniffer:libmagic_a614b29f3dc4f565cf16_line_287; libmagic:magic/Magdir/sniffer:libmagic_f4626b156cb0e3fc04ca_line_292; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[768]/magic[0]"
		label = "pcap"
		enforced = true
        class = "partial"
        fp_rate = 0
        fn_rate = 0.17999999999999999

	strings:
		$p0_0 = { A1 B2 C3 D4 }
		$p1_0 = { D4 C3 B2 A1 }
		$p2_0 = { 4d 3c b2 a1 }
		$p3_0 = { a1 b2 3c 4d }

	condition:
		((prefix_size >= 4 and original_size >= 4 and $p0_0 at 0) or (prefix_size >= 4 and original_size >= 4 and $p1_0 at 0) or (prefix_size >= 4 and $p2_0 at 0) or (prefix_size >= 4 and $p3_0 at 0))
}

rule taxonomy_pdb
{
	meta:
        source_refs = "pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:1459; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:1460"
		label = "pdb"
		enforced = true
        class = "partial"
        fp_rate = 0
        fn_rate = 0.62

	strings:
		$p0_0 = /\x4d\x69\x63\x72\x6f\x73\x6f\x66\x74\x20\x43\x2f\x43(\x2b){2}\x20\x70\x72\x6f\x67\x72\x61\x6d\x20\x64\x61\x74\x61\x62\x61\x73\x65\x20\x32\x2e(\x30){2}/
		$p1_0 = /\x4d\x69\x63\x72\x6f\x73\x6f\x66\x74\x20\x43\x2f\x43(\x2b){2}\x20\x4d\x53\x46\x20\x37\x2e(\x30){2}/

	condition:
		(($p0_0 at 0) or ($p1_0 at 0))
}

rule taxonomy_pythonbytecode
{
	meta:
        source_refs = "libmagic:magic/Magdir/python:libmagic_02a4b429b096ebde149d_line_161; libmagic:magic/Magdir/python:libmagic_03329d1ce697cca70023_line_65; libmagic:magic/Magdir/python:libmagic_052aa2277fd4b437c801_line_185; libmagic:magic/Magdir/python:libmagic_0c7d0926d35973d86f71_line_25; libmagic:magic/Magdir/python:libmagic_150b6b0110ca3346ba92_line_87; libmagic:magic/Magdir/python:libmagic_1573bd8972cd286484b1_line_167; libmagic:magic/Magdir/python:libmagic_16016513a513eabb1be3_line_121; libmagic:magic/Magdir/python:libmagic_17489124e5c12c200a13_line_103; libmagic:magic/Magdir/python:libmagic_18397b59a214c6596bd6_line_131; libmagic:magic/Magdir/python:libmagic_1a91e3e30b8d545a17a5_line_91; libmagic:magic/Magdir/python:libmagic_1abe92062aa884d47d76_line_49; libmagic:magic/Magdir/python:libmagic_1d45cd74474c890f2d2b_line_19; libmagic:magic/Magdir/python:libmagic_21e03fb9d3ac66d45322_line_71; libmagic:magic/Magdir/python:libmagic_248554770e7d1dca97b5_line_45; libmagic:magic/Magdir/python:libmagic_2c29e1029fc7529ca6c3_line_75; libmagic:magic/Magdir/python:libmagic_2d19782fab6b70dd23d0_line_15; libmagic:magic/Magdir/python:libmagic_2d866c774b962f1b53ab_line_145; libmagic:magic/Magdir/python:libmagic_2dae9ae8f655d9e90aec_line_63; libmagic:magic/Magdir/python:libmagic_303b1d9e34c840e5dfb0_line_169; libmagic:magic/Magdir/python:libmagic_3c6e1bc564343517bfc1_line_113; libmagic:magic/Magdir/python:libmagic_42e058401c9dcd7a13f9_line_97; libmagic:magic/Magdir/python:libmagic_43c690b1f34f5529ed41_line_111; libmagic:magic/Magdir/python:libmagic_48e15f3c9a7e1c593024_line_41; libmagic:magic/Magdir/python:libmagic_490a2693c905d49ac785_line_57; libmagic:magic/Magdir/python:libmagic_4f28a2fba84b4f690d1a_line_159; libmagic:magic/Magdir/python:libmagic_504d57ef02c345a1e288_line_101; libmagic:magic/Magdir/python:libmagic_55a2df06883d0f694d86_line_59; libmagic:magic/Magdir/python:libmagic_5d72509835322de2fcd6_line_175; libmagic:magic/Magdir/python:libmagic_6260e405840f107fa766_line_163; libmagic:magic/Magdir/python:libmagic_629b83b47e9271cdcffc_line_53; libmagic:magic/Magdir/python:libmagic_639c7a9486f94eb11049_line_79; libmagic:magic/Magdir/python:libmagic_6638d77b97a0dc1da5b7_line_129; libmagic:magic/Magdir/python:libmagic_6993e8a48f3cd337fc67_line_177; libmagic:magic/Magdir/python:libmagic_6a7d81dd676d1ebb5958_line_95; libmagic:magic/Magdir/python:libmagic_6a833b21994e3d36362f_line_115; libmagic:magic/Magdir/python:libmagic_6c736bceee8b8a4f072a_line_171; libmagic:magic/Magdir/python:libmagic_6ec378338c8964a89801_line_109; libmagic:magic/Magdir/python:libmagic_7003633d96862a80fd9d_line_143; libmagic:magic/Magdir/python:libmagic_709a8d221033a98f635a_line_61; libmagic:magic/Magdir/python:libmagic_71e3acd3d2f391366d8d_line_99; libmagic:magic/Magdir/python:libmagic_75ce08cdf2217efc0bef_line_107; libmagic:magic/Magdir/python:libmagic_7814d288a25077543bf8_line_21; libmagic:magic/Magdir/python:libmagic_790c80498ef60abccb24_line_125; libmagic:magic/Magdir/python:libmagic_7b3548cca50ba3cb8daf_line_155; libmagic:magic/Magdir/python:libmagic_7d7ae9217ea6f3019715_line_149; libmagic:magic/Magdir/python:libmagic_800606df829695281381_line_81; libmagic:magic/Magdir/python:libmagic_83417878fa777e21c814_line_173; libmagic:magic/Magdir/python:libmagic_85f146505c3f91d0504b_line_31; libmagic:magic/Magdir/python:libmagic_8a002f2656e1543e00cd_line_77; libmagic:magic/Magdir/python:libmagic_8b024c72abad3b233981_line_89; libmagic:magic/Magdir/python:libmagic_8cd8d04351229ef30b35_line_179; libmagic:magic/Magdir/python:libmagic_8d0ead699403b8274fc3_line_69; libmagic:magic/Magdir/python:libmagic_9060156e881818d4e0bb_line_47; libmagic:magic/Magdir/python:libmagic_93a17c478b3ddbcd8b44_line_51; libmagic:magic/Magdir/python:libmagic_95c076848096a7980059_line_105; libmagic:magic/Magdir/python:libmagic_9674d19b07360437e8c4_line_93; libmagic:magic/Magdir/python:libmagic_9a9b8d7ad4658abf2c27_line_139; libmagic:magic/Magdir/python:libmagic_9e9c0e624c7b956895c9_line_127; libmagic:magic/Magdir/python:libmagic_a661d7ba42f65e020580_line_137; libmagic:magic/Magdir/python:libmagic_af14b29354d25bdc8659_line_39; libmagic:magic/Magdir/python:libmagic_b4ce7b4716285ef743c7_line_117; libmagic:magic/Magdir/python:libmagic_b6028cb2b5218af94ecc_line_23; libmagic:magic/Magdir/python:libmagic_b6b96b14ad643ce84306_line_157; libmagic:magic/Magdir/python:libmagic_b87ae209962ffa4a8e9f_line_27; libmagic:magic/Magdir/python:libmagic_bebbaa165af8ed7645ad_line_13; libmagic:magic/Magdir/python:libmagic_c0bbd72dcad6520c6138_line_183; libmagic:magic/Magdir/python:libmagic_c383b6bb7cf79bb3a1ed_line_135; libmagic:magic/Magdir/python:libmagic_c44959203055a79c2f00_line_133; libmagic:magic/Magdir/python:libmagic_ce08ab9fea41c1c09475_line_151; libmagic:magic/Magdir/python:libmagic_ce25b9943495bec80e26_line_55; libmagic:magic/Magdir/python:libmagic_ce7f5954320bcd72dc2a_line_165; libmagic:magic/Magdir/python:libmagic_ceff6792f85bdc88a208_line_187; libmagic:magic/Magdir/python:libmagic_d06baf48548d2a4c04c4_line_35; libmagic:magic/Magdir/python:libmagic_d299141faec41c9749b3_line_67; libmagic:magic/Magdir/python:libmagic_d9191e2619faaa472849_line_29; libmagic:magic/Magdir/python:libmagic_d9c1261297fa92934f53_line_85; libmagic:magic/Magdir/python:libmagic_dd83e241d509442fd840_line_37; libmagic:magic/Magdir/python:libmagic_e0b3a02c8ba058815a12_line_33; libmagic:magic/Magdir/python:libmagic_e60e0d56d249866f2d2e_line_123; libmagic:magic/Magdir/python:libmagic_ea8f9543291a87a44e4d_line_181; libmagic:magic/Magdir/python:libmagic_ef0c382467319d7afa2b_line_147; libmagic:magic/Magdir/python:libmagic_f000305a9899f3e1ea49_line_17; libmagic:magic/Magdir/python:libmagic_f1b23542b3e6d7aebb9c_line_83; libmagic:magic/Magdir/python:libmagic_f29ff3226bcc3a0049ed_line_141; libmagic:magic/Magdir/python:libmagic_f51800de09e20a6f9dbe_line_43; libmagic:magic/Magdir/python:libmagic_f681442145396edfa4a5_line_189; libmagic:magic/Magdir/python:libmagic_fa6c3f31d497e5f054af_line_73; libmagic:magic/Magdir/python:libmagic_fbe46c1c4bf2a11f8e05_line_119; libmagic:magic/Magdir/python:libmagic_fd0430dab0cca4b38bea_line_153"
		label = "pythonbytecode"
		enforced = true
        class = "partial"
        fp_rate = 0
        fn_rate = 0.97999999999999998

	strings:
		$p0_0 = { 87 c6 0d 0a }
		$p1_0 = { 02 09 99 00 }
		$p2_0 = { 09 0c 0d 0a }
		$p3_0 = { 1f 0c 0d 0a }
		$p4_0 = { e5 f2 0d 0a }
		$p5_0 = { 76 0c 0d 0a }
		$p6_0 = { 27 0c 0d 0a }
		$p7_0 = { 2d 0d 0d 0a }
		$p8_0 = { 03 f3 0d 0a }
		$p9_0 = { 45 0c 0d 0a }
		$p10_0 = { 33 0d 0d 0a }
		$p11_0 = { 63 f2 0d 0a }
		$p12_0 = { fc c4 0d 0a }
		$p13_0 = { c2 0b 0d 0a }
		$p14_0 = { 45 f2 0d 0a }
		$p15_0 = { 02 0d 0d 0a }
		$p16_0 = { 31 0d 0d 0a }
		$p17_0 = { bc 0c 0d 0a }
		$p18_0 = { 6e f2 0d 0a }
		$p19_0 = { 8b f2 0d 0a }
		$p20_0 = { e0 0b 0d 0a }
		$p21_0 = { 2a eb 0d 0a }
		$p22_0 = { 2c 0d 0d 0a }
		$p23_0 = { cc 0b 0d 0a }
		$p24_0 = { 95 f2 0d 0a }
		$p25_0 = { b3 f2 0d 0a }
		$p26_0 = { a9 f2 0d 0a }
		$p27_0 = { 20 0d 0d 0a }
		$p28_0 = { c7 f2 0d 0a }
		$p29_0 = { 3b 0c 0d 0a }
		$p30_0 = { d0 0c 0d 0a }
		$p31_0 = { b8 0b 0d 0a }
		$p32_0 = { 3e 0d 0d 0a }
		$p33_0 = { 17 0d 0d 0a }
		$p34_0 = { 80 0c 0d 0a }
		$p35_0 = { 89 2e 0d 0a }
		$p36_0 = { 2e ed 0d 0a }
		$p37_0 = { c6 0c 0d 0a }
		$p38_0 = { 1d 0c 0d 0a }
		$p39_0 = { 94 0c 0d 0a }
		$p40_0 = { e4 0c 0d 0a }
		$p41_0 = { 04 17 0d 0a }
		$p42_0 = { 3b f2 0d 0a }
		$p43_0 = { 16 0d 0d 0a }
		$p44_0 = { 03 09 99 00 }
		$p45_0 = { 13 0c 0d 0a }
		$p46_0 = { d2 f2 0d 0a }
		$p47_0 = { 2d ed 0d 0a }
		$p48_0 = { ea 0b 0d 0a }
		$p49_0 = { d6 0b 0d 0a }
		$p50_0 = { 4f 0c 0d 0a }
		$p51_0 = { f8 0c 0d 0a }
		$p52_0 = { 21 0d 0d 0a }
		$p53_0 = { 04 f3 0d 0a }
		$p54_0 = { 58 0c 0d 0a }
		$p55_0 = { 2a 0d 0d 0a }
		$p56_0 = { ee 0c 0d 0a }
		$p57_0 = { 62 0c 0d 0a }
		$p58_0 = { 6d f2 0d 0a }
		$p59_0 = { db f2 0d 0a }
		$p60_0 = { f4 0b 0d 0a }
		$p61_0 = { 88 c6 0d 0a }
		$p62_0 = { 9f f2 0d 0a }
		$p63_0 = { 99 4E 0D 0A }
		$p64_0 = { 3c f2 0d 0a }
		$p65_0 = { 3f 0d 0d 0a }
		$p66_0 = { f5 0b 0d 0a }
		$p67_0 = { 77 f2 0d 0a }
		$p68_0 = { f9 f2 0d 0a }
		$p69_0 = { 6c 0c 0d 0a }
		$p70_0 = { 0a f3 0d 0a }
		$p71_0 = { b4 f2 0d 0a }
		$p72_0 = { 0c 0d 0d 0a }
		$p73_0 = { b2 0c 0d 0a }
		$p74_0 = { 9e 0c 0d 0a }
		$p75_0 = { ff 0b 0d 0a }
		$p76_0 = { 2b 0d 0d 0a }
		$p77_0 = { 30 0d 0d 0a }
		$p78_0 = { 2b eb 0d 0a }
		$p79_0 = { 59 f2 0d 0a }
		$p80_0 = { 8a 0c 0d 0a }
		$p81_0 = { ef f2 0d 0a }
		$p82_0 = { 2f 0d 0d 0a }
		$p83_0 = { 8c f2 0d 0a }
		$p84_0 = { 81 f2 0d 0a }
		$p85_0 = { 32 0d 0d 0a }
		$p86_0 = { da 0c 0d 0a }
		$p87_0 = { fd c4 0d 0a }
		$p88_0 = { d1 f2 0d 0a }

	condition:
		((prefix_size >= 4 and $p0_0 at 0) or (prefix_size >= 4 and $p1_0 at 0) or (prefix_size >= 4 and $p2_0 at 0) or (prefix_size >= 4 and $p3_0 at 0) or (prefix_size >= 4 and $p4_0 at 0) or (prefix_size >= 4 and $p5_0 at 0) or (prefix_size >= 4 and $p6_0 at 0) or (prefix_size >= 4 and $p7_0 at 0) or (prefix_size >= 4 and $p8_0 at 0) or (prefix_size >= 4 and $p9_0 at 0) or (prefix_size >= 4 and $p10_0 at 0) or (prefix_size >= 4 and $p11_0 at 0) or (prefix_size >= 4 and $p12_0 at 0) or (prefix_size >= 4 and $p13_0 at 0) or (prefix_size >= 4 and $p14_0 at 0) or (prefix_size >= 4 and $p15_0 at 0) or (prefix_size >= 4 and $p16_0 at 0) or (prefix_size >= 4 and $p17_0 at 0) or (prefix_size >= 4 and $p18_0 at 0) or (prefix_size >= 4 and $p19_0 at 0) or (prefix_size >= 4 and $p20_0 at 0) or (prefix_size >= 4 and $p21_0 at 0) or (prefix_size >= 4 and $p22_0 at 0) or (prefix_size >= 4 and $p23_0 at 0) or (prefix_size >= 4 and $p24_0 at 0) or (prefix_size >= 4 and $p25_0 at 0) or (prefix_size >= 4 and $p26_0 at 0) or (prefix_size >= 4 and $p27_0 at 0) or (prefix_size >= 4 and $p28_0 at 0) or (prefix_size >= 4 and $p29_0 at 0) or (prefix_size >= 4 and $p30_0 at 0) or (prefix_size >= 4 and $p31_0 at 0) or (prefix_size >= 4 and $p32_0 at 0) or (prefix_size >= 4 and $p33_0 at 0) or (prefix_size >= 4 and $p34_0 at 0) or (prefix_size >= 4 and $p35_0 at 0) or (prefix_size >= 4 and $p36_0 at 0) or (prefix_size >= 4 and $p37_0 at 0) or (prefix_size >= 4 and $p38_0 at 0) or (prefix_size >= 4 and $p39_0 at 0) or (prefix_size >= 4 and $p40_0 at 0) or (prefix_size >= 4 and $p41_0 at 0) or (prefix_size >= 4 and $p42_0 at 0) or (prefix_size >= 4 and $p43_0 at 0) or (prefix_size >= 4 and $p44_0 at 0) or (prefix_size >= 4 and $p45_0 at 0) or (prefix_size >= 4 and $p46_0 at 0) or (prefix_size >= 4 and $p47_0 at 0) or (prefix_size >= 4 and $p48_0 at 0) or (prefix_size >= 4 and $p49_0 at 0) or (prefix_size >= 4 and $p50_0 at 0) or (prefix_size >= 4 and $p51_0 at 0) or (prefix_size >= 4 and $p52_0 at 0) or (prefix_size >= 4 and $p53_0 at 0) or (prefix_size >= 4 and $p54_0 at 0) or (prefix_size >= 4 and $p55_0 at 0) or (prefix_size >= 4 and $p56_0 at 0) or (prefix_size >= 4 and $p57_0 at 0) or (prefix_size >= 4 and $p58_0 at 0) or (prefix_size >= 4 and $p59_0 at 0) or (prefix_size >= 4 and $p60_0 at 0) or (prefix_size >= 4 and $p61_0 at 0) or (prefix_size >= 4 and $p62_0 at 0) or (prefix_size >= 4 and original_size >= 4 and $p63_0 at 0) or (prefix_size >= 4 and $p64_0 at 0) or (prefix_size >= 4 and $p65_0 at 0) or (prefix_size >= 4 and $p66_0 at 0) or (prefix_size >= 4 and $p67_0 at 0) or (prefix_size >= 4 and $p68_0 at 0) or (prefix_size >= 4 and $p69_0 at 0) or (prefix_size >= 4 and $p70_0 at 0) or (prefix_size >= 4 and $p71_0 at 0) or (prefix_size >= 4 and $p72_0 at 0) or (prefix_size >= 4 and $p73_0 at 0) or (prefix_size >= 4 and $p74_0 at 0) or (prefix_size >= 4 and $p75_0 at 0) or (prefix_size >= 4 and $p76_0 at 0) or (prefix_size >= 4 and $p77_0 at 0) or (prefix_size >= 4 and $p78_0 at 0) or (prefix_size >= 4 and $p79_0 at 0) or (prefix_size >= 4 and $p80_0 at 0) or (prefix_size >= 4 and $p81_0 at 0) or (prefix_size >= 4 and $p82_0 at 0) or (prefix_size >= 4 and $p83_0 at 0) or (prefix_size >= 4 and $p84_0 at 0) or (prefix_size >= 4 and $p85_0 at 0) or (prefix_size >= 4 and $p86_0 at 0) or (prefix_size >= 4 and $p87_0 at 0) or (prefix_size >= 4 and $p88_0 at 0))
}

rule taxonomy_rpm
{
	meta:
        source_refs = "libmagic:magic/Magdir/rpm:libmagic_0f79d098479628a1e0d6_line_7; libmagic:magic/Magdir/rpm:libmagic_380bab26065062d1f1c2_line_39; puremagic:puremagic/magic_data.json:headers[245]; puremagic:puremagic/magic_data.json:headers[798]; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[1045]/magic[0]"
		label = "rpm"
		enforced = true
        class = "partial"
        fp_rate = 0
        fn_rate = 0.82999999999999996

	strings:
		$p0_0 = "\\xed\\xab\\xee\\xdb"
		$p1_0 = { 64 72 70 6d }
		$p2_0 = { ED AB EE DB }

	condition:
		((prefix_size >= 16 and original_size >= 16 and $p0_0 at 0) or (prefix_size >= 4 and $p1_0 at 0) or (prefix_size >= 4 and original_size >= 4 and $p2_0 at 0))
}

rule taxonomy_stata
{
	meta:
        source_refs = "libmagic:magic/Magdir/statistics:libmagic_abd0b55d37f30fab6e91_line_44"
		label = "stata"
		enforced = true
        class = "partial"
        fp_rate = 0
        fn_rate = 0.40999999999999998

	strings:
		$p0_0 = "<stata_dta><header><release>"

	condition:
		(prefix_size >= 28 and $p0_0 at 0)
}

rule taxonomy_torrent
{
	meta:
        source_refs = "libmagic:magic/Magdir/archive:libmagic_04da80c1a8b308d1ca2b_line_2354; libmagic:magic/Magdir/archive:libmagic_3efbcd9ff003a4e6507c_line_2347; libmagic:magic/Magdir/archive:libmagic_f935cb57d32135cc0208_line_2351; libmagic:magic/Magdir/archive:libmagic_ff3aca023d9cf134f701_line_2357; puremagic:puremagic/magic_data.json:headers[19]; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[893]/magic[0]"
		label = "torrent"
		enforced = true
        class = "partial"
        fp_rate = 0
        fn_rate = 0.01

	strings:
		$p0_0 = { 64 31 33 3a 61 6e 6e 6f 75 6e 63 65 2d 6c 69 73 74 }
		$p1_0 = { 64 34 3a 69 6e 66 6f }
		$p2_0 = "d8:announce"
		$p3_0 = { 64 37 3a 63 6f 6d 6d 65 6e 74 }

	condition:
		((prefix_size >= 17 and $p0_0 at 0) or (prefix_size >= 7 and $p1_0 at 0) or (prefix_size >= 11 and original_size >= 11 and $p2_0 at 0) or (prefix_size >= 10 and $p3_0 at 0))
}

rule taxonomy_xcoff
{
	meta:
        source_refs = "libmagic:magic/Magdir/ibm6000:libmagic_72f92e57f4bd2f191a75_line_28"
		label = "xcoff"
		enforced = true
        class = "partial"
        fp_rate = 0
        fn_rate = 0.01

	strings:
		$p0_0 = { 01 f7 }

	condition:
		(prefix_size >= 2 and $p0_0 at 0)
}
