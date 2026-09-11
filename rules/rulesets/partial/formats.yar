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
// Container rules on the facts stream: adjudicated combined corpus, 25,421 whole files
// (rules/QUALITY.md, "Container and PE preprocessor rules").

// A single-disk, non-zip64 archive whose central directory was held and walked, with at
// least one name in the `zip_names` view (PKWARE APPNOTE 6.3.10, sections 4.3.12-4.3.16).
private rule zip_directory_names
{
    condition:
        zip_valid == 1 and zip_entries >= 1 and zip_names_entries >= 1
}

// An OPC package whose first entry is its content types stream, held whole in the first
// block and inflated into the `zip_first_entry` view (ECMA-376 part 2, section 10.1.2.2).
private rule zip_content_types
{
    condition:
        zip_first_entry startswith "[Content_Types].xml\n"
}

rule taxonomy_3mf
{
	meta:
		label = "3mf"
        enforced = true
        class = "partial"
        fp_rate = 0
        fn_rate = 0.3591160220994475
    // 3MF: a zip whose central directory names the 3D model part.
    condition:
        zip_valid == 1 and zip_names contains "\n3D/3dmodel.model\n"
}

rule taxonomy_apk
{
	meta:
        source_refs = "libmagic:magic/Magdir/archive:libmagic_9418b81d75bfd1980625_line_1861; libmagic:magic/Magdir/archive:libmagic_9418b81d75bfd1980625_line_1876"
		label = "apk"
		enforced = true
        class = "partial"
        fp_rate = 0
        fn_rate = 0.2365415986949429

    // Android ZIP members use stored/deflated data; retain ignored ZIP versions, including 0.
    // Lengths may be deferred to a data descriptor. DEX-only ZIP/JAR files are not APKs:
    // a DEX-first package also needs an Android manifest local entry within the prefix,
    // and a manifest-first package needs a classes.dex or resources.arsc local entry
    // there, since an Android library (AAR) also opens with AndroidManifest.xml.
    // Manifest-first packages whose code follows later are decided by taxonomy_apk_names.
    strings:
        $zip = { 50 4B 03 04 }
        $dex = "classes.dex"
        $manifest = "AndroidManifest.xml"
        $manifest_entry = { 50 4B 03 04 [22] 13 00 ?? ?? 41 6E 64 72 6F 69 64 4D 61 6E 69 66 65 73 74 2E 78 6D 6C }
        $dex_entry = { 50 4B 03 04 [22] 0B 00 ?? ?? 63 6C 61 73 73 65 73 2E 64 65 78 }
        $arsc_entry = { 50 4B 03 04 [22] 0E 00 ?? ?? 72 65 73 6F 75 72 63 65 73 2E 61 72 73 63 }
    condition:
        prefix_size >= 41 and $zip at 0 and uint16(6) % 2 == 0 and
        (uint16(8) == 0 or uint16(8) == 8) and
        ((uint32(18) >= 1 and uint32(22) >= 1) or
         uint16(6) % 16 == 8 or uint16(6) % 16 == 10 or
         uint16(6) % 16 == 12 or uint16(6) % 16 == 14) and
        ((uint16(26) == 11 and $dex at 30 and $manifest_entry in (41..4047)) or
         (prefix_size >= 49 and uint16(26) == 19 and $manifest at 30 and
          ($dex_entry in (49..4055) or $arsc_entry in (49..4052))))
}

rule taxonomy_apk_names
{
	meta:
        source_refs = "libmagic:magic/Magdir/archive:libmagic_9418b81d75bfd1980625_line_1861; libmagic:magic/Magdir/archive:libmagic_9418b81d75bfd1980625_line_1876"
		label = "apk"
		enforced = true
        class = "partial"
        fp_rate = 0
        fn_rate = 0.011419249592169658

    // Central directory evidence for packages the prefix cannot settle, manifest first or
    // not: the binary manifest plus compiled code or resources. A DEX-only archive is not
    // an APK, and an Android library (AAR) carries classes.jar rather than classes.dex.
    condition:
        zip_directory_names and zip_names contains "\nAndroidManifest.xml\n" and
        (zip_names contains "\nclasses.dex\n" or zip_names contains "\nresources.arsc\n")
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

    // BMP file header plus OS/2 BITMAPCOREHEADER or Windows uncompressed DIB.
    strings:
        $header = { 42 4D ?? ?? ?? ?? 00 00 00 00 }
    condition:
        prefix_size >= 26 and $header at 0 and uint32(2) >= 26 and uint32(10) >= 26
        and ((uint32(14) == 12 and uint16(18) >= 1 and uint16(20) >= 1
              and uint16(22) == 1
              and (uint16(24) == 1 or uint16(24) == 4 or uint16(24) == 8 or uint16(24) == 24))
             or (prefix_size >= 54 and uint32(10) >= 54
                 and (uint32(14) == 40 or uint32(14) == 52 or uint32(14) == 56
                      or uint32(14) == 108 or uint32(14) == 124)
                 and uint32(18) >= 1 and uint32(18) <= 2147483647 and uint32(22) >= 1
                 and uint16(26) == 1 and uint32(30) == 0
                 and (uint16(28) == 1 or uint16(28) == 4 or uint16(28) == 8
                      or uint16(28) == 16 or uint16(28) == 24 or uint16(28) == 32)))
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

    // Microsoft CFHEADER: signature, reserved words, version 1.3 and legal flags.
    strings:
        $header = { 4D 53 43 46 00 00 00 00 }
    condition:
        prefix_size >= 36 and $header at 0
        and uint32(8) >= 36 and uint32(12) == 0
        and uint32(16) >= 36 and uint32(20) == 0
        and uint16(24) == 259 and uint16(26) >= 1 and uint16(30) <= 7
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

    // CRX2 key/signature lengths differ from CRX3's protobuf-header length.
    strings:
        $magic = "Cr24"
    condition:
        prefix_size >= 12 and $magic at 0 and
        ((uint32(4) == 3 and uint32(8) >= 1) or
         (prefix_size >= 16 and uint32(4) == 2 and uint32(8) >= 1 and
          uint32(8) <= 65536 and uint32(12) >= 1 and uint32(12) <= 65536))
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

    // Same PRONOM version/type families, with readable character classes.
    // Observe the complete first field descriptor; DBF II has a fixed 521-byte header.
    strings:
        $memo = /[\x8b\xcb\x8e\x7b][\x00-\xff][\x01-\x0c][\x01-\x1f][\x00-\xff]{28}[A-Za-z][\x00-\xff]{10}[CDFLMN]/
        $plain = /[\x04\x43\x63][\x00-\xff][\x01-\x0c][\x01-\x1f][\x00-\xff]{28}[A-Za-z][\x00-\xff]{10}[CDFLN]/
        $iii = /[\x03\x83][\x00-\xff][\x01-\x0c][\x01-\x1f][\x00-\xff]{28}[A-Za-z][\x00-\xff]{10}[CDLMN]/
        $ii = /\x02[\x00-\xff]{2}(\x00\x00\x00|[\x01-\x0c][\x01-\x1f][\x00-\xff])[\x00-\xff][\x00-\x03][A-Za-z][\x00-\xff]{10}[CLN]/
    condition:
        (prefix_size >= 64 and uint16(8) >= 64 and uint16(10) >= 1 and
         ($memo at 0 or $plain at 0 or $iii at 0)) or
        (prefix_size >= 24 and original_size >= 521 and $ii at 0 and
         uint16(6) >= 1 and uint8(20) >= 1)
}

rule taxonomy_docx
{
	meta:
        source_refs = "puremagic:puremagic/magic_data.json:headers[66]; puremagic:puremagic/magic_data.json:headers[967]"
		label = "docx"
		enforced = true
        class = "partial"
        fp_rate = 0
        fn_rate = 0.04597701149425287

    // ECMA-376 part 2 (OPC): the main part's content type, declared in the content types
    // stream, separates a document (or its macro-enabled form) from a template, which lists
    // the same part names.
    condition:
        zip_content_types and
        (zip_first_entry contains "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml" or
         zip_first_entry contains "application/vnd.ms-word.document.macroEnabled.main+xml")
}

rule taxonomy_dotx
{
	meta:
        source_refs = "puremagic:puremagic/magic_data.json:headers[74]"
		label = "dotx"
		enforced = true
        class = "partial"
        fp_rate = 0
        fn_rate = 0.03804347826086957

    // ECMA-376 part 2 (OPC): a Word template (or its macro-enabled form) declares the
    // template main part content type in the content types stream.
    condition:
        zip_content_types and
        (zip_first_entry contains "application/vnd.openxmlformats-officedocument.wordprocessingml.template.main+xml" or
         zip_first_entry contains "application/vnd.ms-word.template.macroEnabledTemplate.main+xml")
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

    // MS-EMF fixed header variants plus optional description/pixel-format data.
    // Reserved fields are ignored; each metafile includes its header and EOF record.
    strings:
        $type = { 01 00 00 00 }
        $signature = { 20 45 4D 46 00 00 01 00 }
    condition:
        prefix_size >= 88 and $type at 0 and $signature at 40 and
        uint32(4) >= 88 and uint32(4) % 4 == 0 and
        uint32(48) >= 108 and uint32(48) % 4 == 0 and uint32(52) >= 2 and (
            ((uint32(4) == 88 and uint32(64) == 0) or uint32(64) == 88) or
            (prefix_size >= 100 and uint32(4) >= 100 and uint32(48) >= 120 and
             ((uint32(4) == 100 and uint32(60) == 0 and uint32(92) == 0) or
              uint32(64) == 100 or uint32(92) == 100)) or
            (prefix_size >= 108 and uint32(4) >= 108 and uint32(48) >= 128 and
             ((uint32(4) == 108 and uint32(60) == 0 and uint32(92) == 0) or
              uint32(64) == 108 or uint32(92) == 108))
        )
}

rule taxonomy_epub
{
	meta:
        source_refs = "libmagic:magic/Magdir/archive:libmagic_9418b81d75bfd1980625_line_2058; puremagic:puremagic/magic_data.json:headers[84]; puremagic:puremagic/magic_data.json:headers[85]; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[52]/magic[0]"
		label = "epub"
		enforced = true
        class = "partial"
        fp_rate = 0
        fn_rate = 0.04

    // EPUB OCF: first local ZIP entry is uncompressed mimetype, no extra field, exact 20-byte media type.
    strings:
        $zip = { 50 4B 03 04 }
        $mime = "mimetypeapplication/epub+zip"
        $descriptor = { 50 4B 07 08 6F 61 AB 2C 14 00 00 00 14 00 00 00 }

    condition:
        prefix_size >= 58 and $zip at 0 and $mime at 30
        and uint16(6) % 2 == 0 and uint16(8) == 0
        and ((uint32(18) == 20 and uint32(22) == 20)
             or (prefix_size >= 74 and uint16(6) % 16 == 8
                 and uint32(18) == 0 and uint32(22) == 0 and $descriptor at 58))
        and uint16(26) == 8 and uint16(28) == 0
}

rule taxonomy_epub_names
{
	meta:
        source_refs = "libmagic:magic/Magdir/archive:libmagic_9418b81d75bfd1980625_line_2058; puremagic:puremagic/magic_data.json:headers[84]; puremagic:puremagic/magic_data.json:headers[85]; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[52]/magic[0]"
		label = "epub"
		enforced = true
        class = "partial"
        fp_rate = 0
        fn_rate = 0.2845528455284553

    // EPUB OCF 3.3 section 4.1: the stored first `mimetype` entry names the media type, and the
    // container descriptor is a central directory entry. The exact media type line excludes
    // other OCF-style containers; a deflated or misplaced `mimetype` yields no line.
    condition:
        zip_names startswith "\nmimetype=application/epub+zip\n" and
        zip_names contains "\nMETA-INF/container.xml\n"
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

    // FLAC starts with the complete 34-byte STREAMINFO block, optionally the last metadata block.
    strings:
        $header = { 66 4C 61 43 (00 | 80) 00 00 22 }
    condition:
        prefix_size >= 42 and $header at 0 and uint16be(8) >= 16 and
        uint16be(10) >= 16 and uint32be(18) >= 4096
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
        $version = /GIF8[79]a/
        $block = { ( 21 | 2C | 3B ) }

    condition:
        prefix_size >= 14 and $version at 0 and uint16(6) > 0 and uint16(8) > 0 and
        (
            (uint8(10) < 128 and $block at 13) or
            (uint8(10) >= 128 and uint8(10) % 8 == 0 and $block at 19) or
            (uint8(10) >= 128 and uint8(10) % 8 == 1 and $block at 25) or
            (uint8(10) >= 128 and uint8(10) % 8 == 2 and $block at 37) or
            (uint8(10) >= 128 and uint8(10) % 8 == 3 and $block at 61) or
            (uint8(10) >= 128 and uint8(10) % 8 == 4 and $block at 109) or
            (uint8(10) >= 128 and uint8(10) % 8 == 5 and $block at 205) or
            (uint8(10) >= 128 and uint8(10) % 8 == 6 and $block at 397) or
            (uint8(10) >= 128 and uint8(10) % 8 == 7 and $block at 781)
        )
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

    // Complete WinHelp file header; retain the inherited no-free-chain variant.
    strings:
        $magic = { 3F 5F 03 00 }
        $no_free_chain = { 00 00 FF FF FF FF }
    condition:
        prefix_size >= 16 and $magic at 0 and $no_free_chain at 6 and
        uint32(4) >= 16 and uint32(12) >= 16
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

    // ICONDIR plus first ICONDIRENTRY: nonempty directory, reserved byte, resource size and payload offset.
    strings:
        $header = { 00 00 01 00 }

    condition:
        prefix_size >= 22 and $header at 0 and uint16(4) >= 1
        and uint8(9) == 0 and uint32(14) >= 8 and uint32(18) >= 22
}

rule taxonomy_intelhex
{
	meta:
        source_refs = "spec:Intel HEX record format"
		label = "intelhex"
		enforced = true
        class = "partial"
        fp_rate = 0
        fn_rate = 0.011363636363636364

    // Two leading Intel HEX records with uppercase digits and record types 00 to 05.
    strings:
        $records = /:[0-9A-F]{6}0[0-5][0-9A-F]{2,512}\r?\n:[0-9A-F]{6}0[0-5]/
    condition:
        $records at 0
}

rule taxonomy_jar
{
	meta:
        source_refs = "puremagic:puremagic/magic_data.json:headers[243]; puremagic:puremagic/magic_data.json:headers[244]; puremagic:puremagic/magic_data.json:headers[308]; puremagic:puremagic/magic_data.json:headers[555]"
		label = "jar"
		enforced = true
        class = "partial"
        fp_rate = 0
        fn_rate = 0.3162393162393162

    // JAR File Specification: the manifest entry plus at least one compiled class in the
    // central directory, as Tika's JarDetector examines. Resource-only archives with a
    // manifest, and APKs, WARs or Android libraries without top-level classes, abstain.
    condition:
        zip_directory_names and zip_names contains "\nMETA-INF/MANIFEST.MF\n" and
        zip_names contains ".class\n"
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

    // Signature box followed by a complete fixed FTYP header and exact JP2 brand.
    strings:
        $signature = { 00 00 00 0C 6A 50 20 20 0D 0A 87 0A }
        $brand = "ftypjp2 "
    condition:
        prefix_size >= 28 and $signature at 0 and $brand at 16 and
        uint32be(12) >= 16 and uint32be(12) % 4 == 0
}

rule taxonomy_kmz
{
	meta:
		label = "kmz"
        enforced = true
        class = "partial"
        fp_rate = 0
        fn_rate = 0.2222222222222222
    // KMZ: a zip whose central directory names the KML entry doc.kml.
    condition:
        zip_valid == 1 and zip_names contains "\ndoc.kml\n"
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

    // Apple loader.h: complete thin header and nonzero file type.
    // Empty command tables have zero size; nonempty tables need an 8-byte load command.
    // CPU identifiers, flags and future nonzero file types remain open.
    strings:
        $le32 = { CE FA ED FE }
        $le64 = { CF FA ED FE }
        $be32 = { FE ED FA CE }
        $be64 = { FE ED FA CF }
    condition:
        prefix_size >= 28 and (
            (($le32 at 0 or ($le64 at 0 and prefix_size >= 32)) and uint32(12) >= 1 and
             ((uint32(16) == 0 and uint32(20) == 0) or
              (uint32(16) >= 1 and uint32(20) >= 8))) or
            (($be32 at 0 or ($be64 at 0 and prefix_size >= 32)) and uint32be(12) >= 1 and
             ((uint32be(16) == 0 and uint32be(20) == 0) or
              (uint32be(16) >= 1 and uint32be(20) >= 8)))
        )
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

    // EBML magic plus an actual DocType element, retaining the inherited name offsets.
    // Size 8 may use any legal VINT width; bare text at those offsets is insufficient.
    strings:
        $ebml = { 1A 45 DF A3 }
        $doctype = { 42 82 (88 | 40 08 | 20 00 08 | 10 00 00 08 |
                            08 00 00 00 08 | 04 00 00 00 00 08 |
                            02 00 00 00 00 00 08 | 01 00 00 00 00 00 00 08)
                     6D 61 74 72 6F 73 6B 61 }
        $name = "matroska"
    condition:
        prefix_size >= 16 and $ebml at 0 and uint8(4) >= 1 and uint8(4) <= 254 and
        $doctype in (5 .. 28) and ($name at 8 or $name at 24 or $name at 31)
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

    // Complete normal SZDD header; libmagic also records mode B in early Windows builds.
    strings:
        $header = { 53 5A 44 44 88 F0 27 33 (41 | 42) }
    condition:
        prefix_size >= 14 and $header at 0
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

    // Classic/64-bit-offset CDF: minimum empty header and first dimension-list tag.
    strings:
        $magic = { 43 44 46 (01 | 02) }
    condition:
        prefix_size >= 32 and $magic at 0 and
        (uint32be(12) == 0 or uint32be(8) == 10)
}

rule taxonomy_odp
{
	meta:
        source_refs = "puremagic:puremagic/magic_data.json:headers[1251]; puremagic:puremagic/magic_data.json:headers[561]; puremagic:puremagic/magic_data.json:headers[687]; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[608]/magic[0]"
		label = "odp"
		enforced = true
        class = "partial"
        fp_rate = 0
        fn_rate = 0.0297029702970297

    // ODF 1.3 part 2 section 2.2.4: the stored first `mimetype` entry holds the exact media
    // type, terminated so that presentation-template and other subtypes abstain, and the
    // package carries a content.xml entry.
    condition:
        zip_names startswith "\nmimetype=application/vnd.oasis.opendocument.presentation\n" and
        zip_names contains "\ncontent.xml\n"
}

rule taxonomy_ods
{
	meta:
        source_refs = "puremagic:puremagic/magic_data.json:headers[1252]; puremagic:puremagic/magic_data.json:headers[689]; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[610]/magic[0]"
		label = "ods"
		enforced = true
        class = "partial"
        fp_rate = 0
        fn_rate = 0.13274336283185842

    // ODF 1.3 part 2 section 2.2.4, as for odp: exact spreadsheet media type plus content.xml.
    condition:
        zip_names startswith "\nmimetype=application/vnd.oasis.opendocument.spreadsheet\n" and
        zip_names contains "\ncontent.xml\n"
}

rule taxonomy_odt
{
	meta:
        source_refs = "puremagic:puremagic/magic_data.json:headers[1250]; puremagic:puremagic/magic_data.json:headers[560]; puremagic:puremagic/magic_data.json:headers[681]; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[612]/magic[0]"
		label = "odt"
		enforced = true
        class = "partial"
        fp_rate = 0
        fn_rate = 0.2376237623762376

    // ODF 1.3 part 2 section 2.2.4, as for odp: exact text media type plus content.xml.
    // The terminator excludes text-template, text-master and text-web.
    condition:
        zip_names startswith "\nmimetype=application/vnd.oasis.opendocument.text\n" and
        zip_names contains "\ncontent.xml\n"
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

    // RFC 3533: complete first page header, revision zero, BOS (optionally EOS), initial sequence, nonempty segment table.
    strings:
        $header = { 4F 67 67 53 00 (02 | 06) }

    condition:
        prefix_size >= 28 and $header at 0 and uint32(18) == 0 and uint8(26) >= 1
}

rule taxonomy_osm
{
	meta:
		label = "osm"
        enforced = true
        class = "partial"
        fp_rate = 0
        fn_rate = 0.40562248995983935
    // OpenStreetMap XML: an XML document whose root element is <osm>. PBF is a separate rule.
    strings:
        $root = /<osm[ \t\r\n]/
    condition:
        uint16(0) != 0x4B50 and $root in (0 .. 256)
}

rule taxonomy_osm_pbf
{
	meta:
        source_refs = "spec:OpenStreetMap PBF format (OSMHeader blob)"
		label = "osm"
		enforced = true
        class = "partial"
        fp_rate = 0
        fn_rate = 0.5943775100401606

    // OSM PBF: a BlobHeader whose type is OSMHeader; OSM XML is not covered here.
    strings:
        $header = { 00 00 ?? ?? 0A 09 4F 53 4D 48 65 61 64 65 72 }
    condition:
        prefix_size >= 16 and $header at 0
}

rule taxonomy_pcap
{
	meta:
        source_refs = "libmagic:magic/Magdir/sniffer:libmagic_11a07bcd8c17007e3d09_line_295; libmagic:magic/Magdir/sniffer:libmagic_496cf6fa510b99510d92_line_284; libmagic:magic/Magdir/sniffer:libmagic_a614b29f3dc4f565cf16_line_287; libmagic:magic/Magdir/sniffer:libmagic_f4626b156cb0e3fc04ca_line_292; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[768]/magic[0]"
		label = "pcap"
		enforced = true
        class = "partial"
        fp_rate = 0
        fn_rate = 0.19

	strings:
        $little = { ( D4 C3 B2 A1 | 4D 3C B2 A1 ) 02 00 04 00 }
        $big = { ( A1 B2 C3 D4 | A1 B2 3C 4D ) 00 02 00 04 }

    condition:
        prefix_size >= 24 and
        (($little at 0 and uint32(16) > 0) or ($big at 0 and uint32be(16) > 0))
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

    // MSF7 and legacy JG fixed headers, including binary magic and valid page sizes.
    strings:
        $modern = "Microsoft C/C++ MSF 7.00\r\n\x1aDS\x00\x00\x00"
        $old = "Microsoft C/C++ program database 2.00\r\n\x1aJG\x00\x00"
        $page_size = { 00 (02 | 04 | 08 | 10 | 20 | 40 | 80) 00 00 }
    condition:
        (prefix_size >= 56 and $modern at 0 and $page_size at 32 and
         (uint32(36) == 1 or uint32(36) == 2)) or
        (prefix_size >= 60 and $old at 0 and $page_size at 44)
}

rule taxonomy_pebin
{
	meta:
        source_refs = ""
		label = "pebin"
		enforced = true
        class = "partial"
        fp_rate = 0
        fn_rate = 0.0015515903801396431

    // PE/COFF specification (COFF file header, optional header): a located image whose
    // section table is held, linked as an executable image, with a documented Windows
    // subsystem and a machine type of a shipping Windows target. DOS executables, objects
    // and images whose headers exceed the first block yield no PE facts.
    condition:
        pe_valid == 1 and pe_is_executable_image == 1 and
        pe_subsystem >= 1 and pe_subsystem <= 16 and
        (pe_machine == 0x14c or pe_machine == 0x8664 or pe_machine == 0x1c0 or
         pe_machine == 0x1c4 or pe_machine == 0xaa64 or pe_machine == 0x200 or
         pe_machine == 0x1c2 or pe_machine == 0x5032 or pe_machine == 0x5064)
}

rule taxonomy_pptx
{
	meta:
        source_refs = "puremagic:puremagic/magic_data.json:headers[67]"
		label = "pptx"
		enforced = true
        class = "partial"
        fp_rate = 0
        fn_rate = 0.044444444444444446

    // ECMA-376 part 2 (OPC): the content types stream plus the PresentationML main part,
    // both as whole central directory names.
    condition:
        zip_directory_names and zip_names contains "\n[Content_Types].xml\n" and
        zip_names contains "\nppt/presentation.xml\n"
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

    // Preserve the existing 89 magic values; add the CPython marshal code-object tag.
    // Magic 3210 added the source-size word. These magics predate the PEP 552 header.
    strings:
        $early = { (02 09 99 00 | 03 09 99 00)
                      [4] 43 }
        $timestamp = { (03 F3 0D 0A | 04 17 0D 0A | 04 F3 0D 0A | 09 0C 0D 0A | 0A F3 0D 0A | 13 0C 0D 0A |
                      1D 0C 0D 0A | 1F 0C 0D 0A | 27 0C 0D 0A | 2A EB 0D 0A | 2B EB 0D 0A | 2D ED 0D 0A |
                      2E ED 0D 0A | 3B 0C 0D 0A | 3B F2 0D 0A | 3C F2 0D 0A | 45 0C 0D 0A | 45 F2 0D 0A |
                      4F 0C 0D 0A | 58 0C 0D 0A | 59 F2 0D 0A | 62 0C 0D 0A | 63 F2 0D 0A | 6C 0C 0D 0A |
                      6D F2 0D 0A | 6E F2 0D 0A | 76 0C 0D 0A | 77 F2 0D 0A | 80 0C 0D 0A | 81 F2 0D 0A |
                      87 C6 0D 0A | 88 C6 0D 0A | 89 2E 0D 0A | 8B F2 0D 0A | 8C F2 0D 0A | 95 F2 0D 0A |
                      99 4E 0D 0A | 9F F2 0D 0A | A9 F2 0D 0A | B3 F2 0D 0A | B4 F2 0D 0A | B8 0B 0D 0A |
                      C2 0B 0D 0A | C7 F2 0D 0A | CC 0B 0D 0A | D1 F2 0D 0A | D2 F2 0D 0A | D6 0B 0D 0A |
                      DB F2 0D 0A | E0 0B 0D 0A | E5 F2 0D 0A | EA 0B 0D 0A | EF F2 0D 0A | F4 0B 0D 0A |
                      F5 0B 0D 0A | F9 F2 0D 0A | FC C4 0D 0A | FD C4 0D 0A | FF 0B 0D 0A)
                      [4] 63 }
        $sized = { (8A 0C 0D 0A | 94 0C 0D 0A | 9E 0C 0D 0A)
                      [8] 63 }
        $references = { (02 0D 0D 0A | 0C 0D 0D 0A | 16 0D 0D 0A | 17 0D 0D 0A | 20 0D 0D 0A | 21 0D 0D 0A |
                      2A 0D 0D 0A | 2B 0D 0D 0A | 2C 0D 0D 0A | 2D 0D 0D 0A | 2F 0D 0D 0A | 30 0D 0D 0A |
                      31 0D 0D 0A | 32 0D 0D 0A | 33 0D 0D 0A | 3E 0D 0D 0A | 3F 0D 0D 0A | B2 0C 0D 0A |
                      BC 0C 0D 0A | C6 0C 0D 0A | D0 0C 0D 0A | DA 0C 0D 0A | E4 0C 0D 0A | EE 0C 0D 0A |
                      F8 0C 0D 0A)
                      [8] (63 | E3) }
    condition:
        prefix_size >= 9 and ($early at 0 or $timestamp at 0 or $sized at 0 or $references at 0)
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

    // RPM binary lead, supported major version, binary/source type and signature header; literal escaped text is not magic.
    strings:
        $header = { ED AB EE DB (03 | 04) 00 }
        $signature = { 8E AD E8 01 00 00 00 00 }

    condition:
        prefix_size >= 112 and $header at 0 and uint16be(6) <= 1 and $signature at 96
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

    // Correlate numeric release and byte order; do not freeze the version to current releases.
    strings:
        $header = /<stata_dta><header><release>[0-9]{3}<\/release><byteorder>(LSF|MSF)<\/byteorder>/
    condition:
        prefix_size >= 63 and $header at 0
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

    // BEP 3/12/52: typed tracker URLs or a torrent-specific info dictionary prefix.
    // A generic comment or info key is insufficient; long tracker lists remain recognizable.
    strings:
        $root = /d(13:announce-list|4:info|8:announce|7:comment)/
        $announce = /d8:announce([7-9]|[1-9][0-9]{1,3}):(https?|udp|wss?):\/\/[\x21-\xff]/
        $announce_list = /d13:announce-listll([7-9]|[1-9][0-9]{1,3}):(https?|udp|wss?):\/\/[\x21-\xff]/
        $info = "4:infod"
        $piece_length = /12:piece lengthi[1-9][0-9]{0,18}e/
        $name = /4:name[1-9][0-9]{0,3}:/
    condition:
        prefix_size >= 20 and $root at 0 and (
            $announce at 0 or $announce_list at 0 or
            ($info in (0 .. 4089) and $piece_length in (0 .. 4060) and $name in (0 .. 4085))
        )
}

rule taxonomy_jxl
{
	meta:
        source_refs = "libmagic:magic/Magdir/jpeg:libmagic_09c181c8a4d299652a29_line_265; libmagic:magic/Magdir/jpeg:libmagic_75914ba77f68d4a72f4e_line_256; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:1856; pronom-binary:DROID_SignatureFile_V125.xml:InternalSignature:1857; puremagic:puremagic/magic_data.json:headers[13]; puremagic:puremagic/magic_data.json:headers[14]; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[1384]/magic[0]"
		label = "jxl"
		enforced = true
        class = "partial"
        fp_rate = 0
        fn_rate = 0.1764705882352941

    // JPEG XL container signature plus ftyp jxl brand. Raw two-byte codestream needs a parser and is not enforced.
    strings:
        $header = { 00 00 00 0C 4A 58 4C 20 0D 0A 87 0A ?? ?? ?? ?? 66 74 79 70 6A 78 6C 20 }

    condition:
        prefix_size >= 32 and $header at 0 and uint32be(12) >= 20
}

rule taxonomy_unixcompress
{
	meta:
        source_refs = "libmagic:magic/Magdir/compress:libmagic_d3a761d2dade747b519c_line_12; puremagic:puremagic/magic_data.json:headers[1047]; tika:tika-core/src/main/resources/org/apache/tika/mime/tika-mimetypes.xml:mime[907]/magic[0]"
		label = "unixcompress"
		enforced = true
        class = "partial"
        fp_rate = 0
        fn_rate = 0.02083333333333333

    // LZW maxbits 9..16, optional block-mode bit; reserved flag bits stay zero.
    strings:
        $header = { 1F 9D (09 | 0A | 0B | 0C | 0D | 0E | 0F | 10 | 89 | 8A | 8B | 8C | 8D | 8E | 8F | 90) }
    condition:
        prefix_size >= 8 and $header at 0
}

rule taxonomy_xlsb
{
	meta:
        source_refs = "puremagic:puremagic/magic_data.json:headers[69]"
		label = "xlsb"
		enforced = true
        class = "partial"
        fp_rate = 0
        fn_rate = 0.07058823529411765

    // ECMA-376 part 2 (OPC) as used by Excel: a binary workbook declares its main part
    // content type in the content types stream.
    condition:
        zip_content_types and
        zip_first_entry contains "application/vnd.ms-excel.sheet.binary.macroEnabled.main"
}

// Prefix signatures for labels the model cannot output or often misses: adjudicated combined corpus,
// 25,421 whole files (rules/QUALITY.md, "Prefix signatures for labels the model lacks or misses").

rule taxonomy_pem
{
	meta:
        source_refs = "spec:RFC 7468 textual encodings"
		label = "pem"
		enforced = true
        class = "partial"
        fp_rate = 0
        fn_rate = 0.36257309941520466

    // An RFC 7468 label from the certificate and key families at the start; PGP armor is excluded.
    strings:
        $pem = /-----BEGIN (CERTIFICATE|TRUSTED CERTIFICATE|CERTIFICATE REQUEST|NEW CERTIFICATE REQUEST|X509 CRL|PUBLIC KEY|PRIVATE KEY|ENCRYPTED PRIVATE KEY|RSA PRIVATE KEY|RSA PUBLIC KEY|DSA PRIVATE KEY|DSA PARAMETERS|EC PRIVATE KEY|EC PARAMETERS|DH PARAMETERS|PKCS7|CMS)-----\r?\n/
    condition:
        $pem at 0
}

rule taxonomy_postscript
{
	meta:
        source_refs = "spec:Adobe PostScript Document Structuring Conventions"
		label = "postscript"
		enforced = true
        class = "partial"
        fp_rate = 0
        fn_rate = 0.6273291925465838

    // The conforming DSC header %!PS-Adobe; a bare %!PS or %! opening is too short to establish identity.
    strings:
        $ps = "%!PS-Adobe"
    condition:
        $ps at 0
}
