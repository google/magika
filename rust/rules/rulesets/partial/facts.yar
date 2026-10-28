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

// Rules that read the zip and PE facts of an input; see src/facts.

private rule zip_directory_names
{
    condition:
        zip_valid == 1 and zip_entries >= 1 and zip_names_entries >= 1
}

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

rule taxonomy_msix
{
	meta:
        source_refs = "spec:MSIX/AppX package (OPC zip with AppxManifest.xml and AppxBlockMap.xml)"
		label = "msix"
        enforced = true
        class = "partial"
        fp_rate = 0
        fn_rate = 0.016574585635359115
    // MSIX/AppX package: a zip whose central directory names the AppX manifest and block map.
    condition:
        zip_valid == 1 and zip_names contains "\nAppxManifest.xml\n" and zip_names contains "\nAppxBlockMap.xml\n"
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
    // A DLL without an entry point holds only resources, as a MUI file does: its headers
    // alone do not say whether it is an executable, so the rule abstains.
    condition:
        pe_valid == 1 and pe_is_executable_image == 1 and
        not (pe_is_dll == 1 and pe_entry_point == 0) and
        pe_subsystem >= 1 and pe_subsystem <= 16 and
        (pe_machine == 0x14c or pe_machine == 0x8664 or pe_machine == 0x1c0 or
         pe_machine == 0x1c4 or pe_machine == 0xaa64 or pe_machine == 0x200 or
         pe_machine == 0x1c2 or pe_machine == 0x5032 or pe_machine == 0x5064)
}
