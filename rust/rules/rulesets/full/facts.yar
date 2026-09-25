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

rule taxonomy_keras
{
	meta:
		label = "keras"
        enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0
    // Keras v3 model archive: a zip whose central directory names the weights and metadata parts.
    condition:
        zip_valid == 1 and zip_names contains "\nmodel.weights.h5\n" and zip_names contains "\nmetadata.json\n"
}

rule taxonomy_qgis
{
	meta:
		label = "qgis"
        enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0
    // QGIS project archive: a zip whose central directory names a .qgs project.
    condition:
        zip_valid == 1 and zip_names contains ".qgs\n"
}

rule taxonomy_visio
{
	meta:
		label = "visio"
        enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0
    // Visio drawing: an Office Open XML package whose central directory names visio/document.xml.
    condition:
        zip_valid == 1 and zip_names contains "\nvisio/document.xml\n"
}

rule taxonomy_xlsx
{
	meta:
        source_refs = "puremagic:puremagic/magic_data.json:headers[68]"
		label = "xlsx"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    // ECMA-376 part 2 (OPC): the content types stream plus the SpreadsheetML workbook part,
    // named in a held single-disk central directory (rules/QUALITY.md, container rules).
    // A binary workbook (xlsb) carries xl/workbook.bin instead and abstains.
    condition:
        zip_valid == 1 and zip_entries >= 1 and zip_names_entries >= 1 and
        zip_names contains "\n[Content_Types].xml\n" and
        zip_names contains "\nxl/workbook.xml\n"
}
