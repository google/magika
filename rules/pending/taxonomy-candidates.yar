// Measured prefix signatures for labels outside Magika's content types.
// Not compiled: every rule label must be a canonical Magika label, disabled rules included.
// Each rule made no wrong decision on the adjudicated combined corpus (25,421 files); its
// fn_rate is the share of its label's files it missed. Move a rule into rules/rulesets once
// its label exists in the taxonomy, then re-run the corpus gate.

rule taxonomy_minidump
{
	meta:
        source_refs = "spec:Microsoft MINIDUMP_HEADER"
		label = "minidump"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    // MDMP signature, implementation version 0xA793, at least one stream, directory after the header.
    strings:
        $header = { 4D 44 4D 50 93 A7 }
    condition:
        prefix_size >= 32 and $header at 0 and uint32(8) >= 1 and uint32(8) <= 4096 and uint32(12) >= 32
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

rule taxonomy_safetensors
{
	meta:
        source_refs = "spec:Hugging Face safetensors header"
		label = "safetensors"
		enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0

    // Little-endian 64-bit header length below 4 GiB, a JSON object, and a dtype key in the prefix.
    strings:
        $open = { 7B 22 }
        $dtype = "\"dtype\""
    condition:
        prefix_size >= 16 and uint32(0) >= 2 and uint32(4) == 0 and $open at 8 and
        $dtype in (10..4089)
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

    // Netpbm magic, optional comment lines, then width and height.
    strings:
        $header = /P[1-6][ \t\r\n]{1,8}(#[^\n]{0,256}\n[ \t\r\n]{0,8}){0,8}[0-9]{1,6}[ \t\r\n]{1,8}[0-9]{1,6}[ \t\r\n]/
    condition:
        $header at 0
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
