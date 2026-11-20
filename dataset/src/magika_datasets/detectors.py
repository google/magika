# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Map detector output -- VirusTotal markings, TrID rankings, libmagic descriptions -- to format ids.

Filename overlap never establishes identity, and nothing here adjudicates a label: these
are the claims the label ladder weighs. The libmagic groups are disjoint, so
`description_kind` tries them in a fixed order and the first match wins.
"""

import re

# Exact names checked against the pinned TrID inventory. Unlisted names remain
# unmapped; broad containers and filename aliases must not imply a child type.
TRID_NAMES = {
    "dicom": ["DICOM medical imaging bitmap", "DICOM medical imaging bitmap (w/o header)"],
    "wmf": ["Windows Metafile", "Windows Metafile (old Win 3.x format)"],
    "iso": ["ISO 9660 CD image"],
    "ace": ["ACE compressed archive"],
    "hlp": ["Microsoft WinHelp"],
    "lha": ["LHARC/LZARK compressed archive (generic)", "LHARK compressed archive"],
    "cat": ["Microsoft Security Catalog", "Microsoft Security Catalog (DER encoded)"],
    "dwg": [
        "AutoCAD R1.0 Drawing",
        "AutoCAD R1.2 Drawing",
        "AutoCAD R1.40 Drawing",
        "AutoCAD R2.05 Drawing",
        "AutoCAD R2.05 Drawing (new)",
        "AutoCAD R2.10 Drawing",
        "AutoCAD R2.21 Drawing",
        "AutoCAD R2.22 Drawing (new)",
        "AutoCAD R2.22 Drawing (old)",
        "AutoCAD R2.5 Drawing",
        "AutoCAD R2.6 Drawing",
        "AutoCAD R9 Drawing",
        "AutoCAD R10 Drawing",
        "AutoCAD R11-12 Drawing",
        "AutoCAD R13 Drawing",
        "AutoCAD R13 Drawing (subtype 10)",
        "AutoCAD R13 Drawing (subtype 11)",
        "AutoCAD R14 Drawing",
        "AutoCAD R14 Drawing (subtype 13)",
        "AutoCAD 2000-2002 Drawing",
        "AutoCAD 2004-2006 Drawing",
        "AutoCAD 2007-2009 Drawing",
        "AutoCAD 2010-2012 Drawing",
        "AutoCAD 2013-2016 Drawing",
        "AutoCAD R2.22-20xx Drawing (generic)",
    ],
    "thumbsdb": ["Windows thumbnail Data Base"],
    "xar": ["XAR Archive"],
    "vhd": ["Virtual PC Virtual HD image", "Virtual PC Virtual HD image (dynamic)"],
    "cdf": ["Common Data Format (v2.6)", "Common Data Format (v3)"],
    "ese": ["Extensible Storage Engine DataBase", "Windows ESE database"],
    "hve": ["Windows NT Registry Hive (primary)", "Windows NT Registry Hive (generic)"],
    "lzx": ["LZX Amiga compressed archive"],
    "mssql": [
        "Microsoft SQL Server database (generic)",
        "Microsoft SQL Server backup",
        "Microsoft SQL Server Backup (compressed)",
    ],
    "palmos": ["Palm Pilot executable", "Palm Pilot executable HB++"],
    "udf": ["UDF disc image"],
    "xcoff": ["AIX Common Object File Format (COFF) executable"],
    "appledouble": ["Mac AppleDouble encoded"],
    "applesingle": ["Mac AppleSingle encoded"],
    "ar": ["ar archive"],
    "arcgrid": ["Esri grid (ASCII)"],
    "arj": ["ARJ compressed archive (standard)", "ARJ compressed archive"],
    "gltf": [
        "GL Transmission Format (v1.0)",
        "GL Transmission Format (v2.0)",
        "GL Transmission Format Binary (v1.0)",
        "GL Transmission Format Binary (v2.0)",
    ],
    "h2": ["H2 DB (MVStore)", "H2 DB (PageStore)"],
    "mapinfo": ["MapInfo Interchange Format"],
    "obj": [
        "Wavefront Object (exported by 3D Builder)",
        "Wavefront Object (exported by Houdini)",
        "Wavefront Object (created by Hexagon)",
        "Wavefront Object (mtllib)",
        "Wavefront Object (created by VXelements)",
        "Wavefront Object (generic)",
        "Wavefront Object (mtllib, with rem)",
        "Wavefront Object (created by 3D Max)",
    ],
    "ole": ["Generic OLE2 / Multistream Compound"],
    "pub": [
        "Microsoft Publisher document",
        "Microsoft Publisher document (v1)",
        "Microsoft Publisher document (v2)",
        "Microsoft Publisher document (v4)",
    ],
    "rdp": ["Remote Desktop Connection Settings", "Remote Desktop Connection Settings (Unicode)"],
    "wad": ["id Software's DOOM Patch-WAD", "id Software's DOOM Internal-WAD"],
    "3dsm": ["3D Studio mesh"],
    "3dsx": ["Nintendo 3DS Homebrew relocatable and eXecutable binary"],
    "3mf": ["3D Manufacturing Format model"],
    "access": [
        "Microsoft Access 2007 Database",
        "Microsoft Jet DB",
        "Microsoft Jet DB (encrypted)",
        "Microsoft Jet DB (old)",
    ],
    "alembic": [
        "Alembic Open Computer Graphics Exchange (HDF5)",
        "Alembic Open Computer Graphics Exchange (Ogawa) (v1.0)",
    ],
    "aout": ["ELKS 16-bit a.out executable"],
    "asf": ["Advanced Streaming Format (generic)"],
    "avro": ["Avro serialized data"],
    "bam": ["Sequence Alignment/Map format (binary)"],
    "bcad": ["bCAD Drawing"],
    "beam": ["Compiled Erlang code", "Compiled Erlang code (old)"],
    "berkeleydb": ["Berkeley DB"],
    "blend": [
        "Blender scene description (generic)",
        "Blender scene description (v1.xx)",
        "Blender scene description (v2.xx)",
        "Blender scene description (v3.xx)",
        "Blender scene description (v4.xx)",
    ],
    "bpl": ["Borland Package Library"],
    "bzip3": ["BZip3 compressed archive"],
    "cinema4d": [
        "CINEMA 4D model (generic)",
        "Maxon Cinema 4D scene (BMC4D)",
        "Maxon Cinema 4D scene (older)",
        "Maxon Cinema 4D scene (rel.6)",
        "Maxon Cinema 4D scene (v4.x)",
        "Maxon Cinema 4D scene (v5.0)",
    ],
    "collada": [
        "COLLADA Digital Asset Document",
        "COLLADA Digital Asset Document (UTF-16 LE)",
        "COLLADA Digital Asset Document (UTF-8)",
    ],
    "cram": ["Compressed Alignment format"],
    "duckdb": ["DuckDB database"],
    "fastq": ["FASTQ format"],
    "fbx": ["Autodesk - Kaydara FBX 3D format (Binary)", "Autodesk - Kaydara FBX 3D format (Text)"],
    "filemaker": [
        "FileMaker Pro database (v12)",
        "FileMaker Pro database (v5)",
        "FileMaker Pro database (v7-11)",
    ],
    "firebird": ["Firebird DataBase"],
    "fpx": ["Kodak FlashPix bitmap"],
    "gguf": ["GPT-Generated Unified Format"],
    "gml": ["Geography Markup Language"],
    "gpx": ["GPS eXchange format", "GPS eXchange format (UTF-8)"],
    "grib": ["Gridded Binary data", "Gridded Binary data 2"],
    "hdf4": ["NCSA Hierarchical Data Format", "NeXus HDF4 data format"],
    "hfs": ["HFS+ / Mac OS Extended disk image", "HFS+ / Mac OS Extended disk image (HFSX)"],
    "iges": [
        "Initial Graphics Exchange Specification (IGES) data",
        "Initial Graphics Exchange Specification (IGES) data (var.2)",
    ],
    "intelhex": ["Intel Hexadecimal object format"],
    "jng": ["JPEG Network Graphics bitmap"],
    "kml": [
        "Google Earth network link",
        "Google Earth placemark",
        "Google Earth placemark (Unicode)",
    ],
    "kmz": ["Google Earth saved working session"],
    "las": ["ASPRS Lidar Data Exchange Format", "ASPRS Lidar Data Exchange Format (compressed)"],
    "lightwave": [
        "LightWave 3D Object",
        "LightWave 3D Object (LWO2)",
        "LightWave 3D Object (LWOB)",
        "LightWave 3D Scene",
    ],
    "llvm_bitcode": ["LLVM Bitcode (generic)", "LLVM IR Bitcode"],
    "lrz": ["Long Range Zip compressed"],
    "luabytecode": [
        "Lua 4.0 bytecode",
        "Lua 5.0 bytecode",
        "Lua 5.1 bytecode",
        "Lua 5.2 bytecode",
        "Lua 5.3 bytecode",
        "Lua bytecode (alt)",
        "Lua bytecode (generic)",
        "LuaJIT 2.0 bytecode",
    ],
    "mat": [
        "Matlab Level 4 MAT-File (big-endian)",
        "Matlab Level 5 MAT-File",
        "Matlab MAT-File (generic)",
    ],
    "max": ["3D Studio Max Scene"],
    "maya": ["Maya ASCII Scene", "Maya Binary Scene (32bit)", "Maya Binary Scene (64bit)"],
    "minidump": ["Windows Minidump"],
    "mpegts": ["MPEG-2 Transport Stream", "MPEG-2 Transport Stream video"],
    "msix": ["MSIX Windows app package"],
    "mtl": ["Alias|Wavefront material library"],
    "mun": ["Windows system resource library"],
    "mysql_storage": ["MySQL MyISAM tables index"],
    "nifti": [
        "NIfTI-1 data format (big endian)",
        "NIfTI-1 data format (little endian)",
        "NIfTI-2 data format (big endian)",
        "NIfTI-2 data format (little endian)",
    ],
    "nrrd": ["Nearly Raw Raster Data"],
    "osm": ["OpenStreetMap XML Data", "ProtoBuf binary format map data"],
    "paddle": ["PaddlePaddle PreTrain Data"],
    "paradox": ["Paradox database"],
    "pcd": ["Point Cloud Data"],
    "postgres_dump": [
        "PostgreSQL database cluster dump (Mac)",
        "PostgreSQL database cluster dump (Unix)",
        "PostgreSQL database cluster dump (Win)",
        "PostgreSQL database dump",
        "PostgreSQL database dump (Mac)",
        "PostgreSQL database dump (Unix)",
        "PostgreSQL database dump (Win)",
    ],
    "qgis": ["QGIS Zipped project", "QGIS project"],
    "qoi": ["Quite OK Image Format bitmap"],
    "rdata": ["R saved work space (generic)", "R saved work space (v2)", "R saved work space (v3)"],
    "realm": ["Realm database"],
    "redis_rdb": ["Redis RDB format (generic)"],
    "rhinoceros": ["Rhinoceros 3D Model"],
    "rll": ["Microsoft Resource Library (x64)", "Microsoft Resource Library (x86)"],
    "rzip": ["rzip compressed archive"],
    "sas": ["SAS Transport (XPORT) format", "SAS v7+ Data set"],
    "sh3d": ["Sweet Home 3D Design (generic)"],
    "sketchup": ["SketchUp model"],
    "spirv": ["SPIR-V binary shaders"],
    "spss": [
        "SPSS Portable ASCII Data",
        "SPSS compressed data",
        "SPSS for Windows Data",
        "SPSS for Windows Data (IBM)",
    ],
    "stata": [
        "Stata Data format (v113, BE)",
        "Stata Data format (v113, LE)",
        "Stata Data format (v114, BE)",
        "Stata Data format (v114, LE)",
        "Stata Data format (v115, BE)",
        "Stata Data format (v115, LE)",
        "Stata Data format (v117+)",
    ],
    "step": ["ISO-10303 STEP model data"],
    "tflite": ["TensorFlow Lite format"],
    "tmdx": ["SoftMaker TextMaker text Document"],
    "uefi_fv": ["AMI Aptio UEFI firmware volume (LZX compressed)"],
    "uf2": ["USB Flashing Format"],
    "usd": [
        "Universal Scene Description (ASCII)",
        "Universal Scene Description Crate (binary)",
        "Universal Scene Description Zipped AR format (USDA)",
        "Universal Scene Description Zipped AR format (USDC)",
        "Universal Scene Description Zipped AR format (generic)",
    ],
    "visio": [
        "Microsoft Visio Drawing (generic)",
        "Visio 2013 drawing",
        "Visio Drawing (old)",
        "Visio Drawing XML",
    ],
    "vtk": [
        "VTK XML format (generic)",
        "Visualization Toolkit format (ASCII)",
        "Visualization Toolkit format (binary)",
    ],
    "wim": [
        "Windows Imaging Format (ESD)",
        "Windows Imaging Format (WIM)",
        "Windows Imaging Format (generic)",
        "Windows Imaging Format (pipable wimlib)",
    ],
    "wiredtiger": ["WiredTiger data", "WiredTiger journal"],
    "zbrush": ["ZBrush Project", "ZBrush ZTool native format"],
    "sevenzip": ["7-Zip compressed archive (v0.4)", "7-Zip compressed archive (gen)"],
    "tga": [
        "Truevision TGA/TARGA bitmap (uncompressed, RGB image)",
        "Truevision TGA/TARGA bitmap (uncompressed, B/W)",
        "Truevision TGA/TARGA bitmap (RLE encoded, RGB image)",
        "Truevision TGA/TARGA bitmap (uncompressed, color-mapped)",
        "Truevision TGA/TARGA bitmap (RLE encoded, RGB image, palette)",
        "Truevision TGA/TARGA bitmap (image id field, no palette)",
        "Truevision TGA/TARGA bitmap (image id field, palette)",
    ],
    "dsstore": ["Mac OS X folder information"],
    "wasm": ["WebAssembly module (binary)"],
    "pythonbytecode": [
        "CPython 1.x bytecode",
        "CPython 2.0 bytecode",
        "CPython 2.1 bytecode",
        "CPython 2.2 bytecode",
        "CPython 2.3 bytecode",
        "CPython 2.4 bytecode",
        "CPython 2.5 bytecode",
        "CPython 2.6 bytecode",
        "CPython 2.7 bytecode",
        "CPython 3.0 bytecode",
        "CPython 3.1 bytecode",
        "CPython 3.2 bytecode",
        "CPython 3.3 bytecode",
        "CPython 3.4 bytecode",
        "CPython 3.5 bytecode",
        "CPython 3.6 bytecode",
        "CPython 3.7 bytecode",
        "CPython 3.8 bytecode",
        "CPython 3.9 bytecode",
        "CPython 3.10 bytecode",
        "CPython 3.11 bytecode",
        "CPython 3.12 bytecode",
        "CPython 3.13 bytecode",
        "CPython 3.14 bytecode",
    ],
    "macho": [
        "Mac OS X Mach-O universal Dynamically linked shared Library",
        "Mac OS X Mach-O Universal Object code",
        "Mac OS X Mach-O 64bit Intel Dynamically linked shared Library",
        "Mac OS X Mach-O 32-bit ARM executable (little endian)",
        "Mac OS X Mach-O 32-bit Intel executable",
        "Mac OS X Mach-O 64-bit ARM executable",
        "Mac OS X Mach-O 64-bit Intel executable",
        "Mac OS X Mach-O 32-bit ARM executable (big endian)",
        "Mac OS X Mach-O 64-bit ARM executable (big endian)",
        "NeXT Mach-O m68k executable",
        "Mac OS X Mach-O 32-bit PPC executable",
        "Mac OS X Mach-O 64-bit PPC executable",
    ],
    "torrent": ["Torrent (trackerless)", "Torrent"],
    "zlibstream": [
        "ZLIB compressed data (fast comp.)",
        "ZLIB compressed data (default comp.)",
        "ZLIB compressed data (best comp.)",
        "ZLIB compressed data (low/no comp.)",
    ],
    "parquet": ["Parquet storage format"],
    "applebplist": ["Mac OS X Binary-format PList"],
    "npy": ["NumPy data"],
    "npz": ["NumPy compressed data archive format"],
    "javabytecode": ["Java bytecode"],
    "m3u": ["Extended M3U playlist", "Extended M3U playlist (UTF-8)"],
    "mum": ["Windows Update Package", "Windows Update Package (UTF-16 LE)"],
    "proteindb": ["Protein DataBank"],
    "autohotkey": ["Autohotkey script (v1.x)", "Autohotkey script (v2.x)"],
    "internetshortcut": ["Windows URL shortcut"],
    "postscript": ["PostScript document", "PostScript document (minimal)"],
    "apk": ["Android Package"],
    "dex": ["Dalvik Dex class"],
    "bmp": ["Windows Bitmap (generic)", "Run Length Encoded bitmap"],
    "elf": [
        "ELF Executable and Linkable format (Linux)",
        "ELF Executable and Linkable format (generic)",
    ],
    "epub": ["Open Publication Structure eBook"],
    "flv": ["Flash Video"],
    "ico": ["Windows Icon"],
    "jar": ["Java Archive"],
    "mkv": ["Matroska Video stream"],
    "ogg": ["OGG Vorbis audio", "Opus compressed audio"],
    "psd": ["Adobe Photoshop image"],
    "rar": ["RAR compressed archive (v5.0)", "RAR compressed archive (v-4.x)"],
    "swf": ["Macromedia Flash Player Movie", "Macromedia Flash Player Compressed Movie"],
    "ttf": ["TrueType Font"],
    "otf": ["OpenType Font"],
    "woff": ["Web Open Font Format"],
    "woff2": ["Web Open Font Format 2"],
    "doc": ["Microsoft Word document", "Microsoft Word document (old ver.)"],
    "docx": ["Word Microsoft Office Open XML Format document"],
    "xls": ["Microsoft Excel sheet", "Microsoft Excel sheet (alternate)"],
    "xlsx": ["Excel Microsoft Office Open XML Format document"],
    "xlsb": ["Excel Binary workbook"],
    "ppt": [
        "Microsoft PowerPoint document",
        "PowerPoint 97 presentation",
        "PowerPoint presentation (encrypted)",
        "PowerPoint presentation (var.)",
    ],
    "pptx": ["PowerPoint Microsoft Office Open XML Format document"],
    "rtf": ["Rich Text Format"],
    "pdf": [
        "Adobe Portable Document Format",
        "Adobe Portable Document Format (UTF-8)",
        "Adobe Portable Document Format (old header)",
        "Adobe Portable Document Format (password protected)",
    ],
    "png": ["Portable Network Graphics", "Fireworks PNG bitmap"],
    "gif": ["GIF animated bitmap", "GIF bitmap (generic)", "GIF87a bitmap", "GIF89a bitmap"],
    "webp": ["WebP bitmap"],
    "flac": ["FLAC lossless compressed audio"],
    "wav": [
        "RIFF/WAVe standard Audio",
        "RIFF/WAVe standard Audio (big-endian)",
        "Broadcast Wave File audio",
    ],
    "avi": ["Audio Video Interleave video", "OpenDML AVI video"],
    "mp3": [
        "MP3 audio",
        "MP3 audio (ID3 v1.x tag)",
        "MP3 audio (ID3 v2.x tag)",
        "LAME encoded MP3 audio",
        "LAME encoded MP3 audio (ID3 v1.x tag)",
        "LAME encoded MP3 audio (ID3 v2.x tag)",
    ],
    "mp4": [
        "MP4 v1 container video",
        "MP4 v2 container video",
        "MP4 v1 container audio",
        "MP4 v2 container audio",
        "MP4 Base Media v1 container video",
        "MP4 Base Media v2 container video",
    ],
    "gzip": ["GZipped data"],
    "tar": [
        "TAR - Tape ARchive (file)",
        "TAR - Tape ARchive (POSIX)",
        "TAR - Tape ARchive (GNU)",
        "TAR - Tape ARchive (directory)",
    ],
    "zip": ["ZIP compressed archive", "ZIP compressed archive (empty)"],
    "pcap": [
        "TCPDUMP's style capture (big-endian)",
        "TCPDUMP's style capture (little-endian)",
        "Extended TCPDUMP's style capture (big-endian)",
        "Extended TCPDUMP's style capture (little-endian)",
    ],
}


def detector_evidence(markings, formats):
    raw = markings.get("magika")
    ids = sorted(
        k
        for k, f in formats.items()
        if isinstance(raw, str)
        and raw.lower() in {k.lower(), *(a.lower() for a in f.get("aliases", []))}
    )
    magika = {
        "raw": raw,
        "format_ids": ids,
        "status": "missing" if not raw else "mapped" if len(ids) == 1 else "unmapped",
    }
    ranked = markings.get("trid", [])
    top = (
        [r for r in ranked if r["probability"] == max(x["probability"] for x in ranked)]
        if ranked
        else []
    )
    matches = sorted(
        {
            k
            for r in top
            for k, names in TRID_NAMES.items()
            if k in formats and r["file_type"] in names
        }
    )
    fully_mapped = all(
        any(r["file_type"] in names and k in formats for k, names in TRID_NAMES.items())
        for r in top
    )
    trid = {
        "raw": ranked,
        "format_ids": matches,
        "status": "missing"
        if not top
        else "mapped"
        if fully_mapped and len(matches) == 1
        else "unmapped",
    }
    return {"magika": magika, "trid": trid}


# libmagic descriptions

FILE_DESCRIPTIONS = {
    "MS Windows registry file, NT/2000 or above": "hve",
    "LZX compressed archive (Amiga)": "lzx",
    "Common Data Format (Version 3 or later) data": "cdf",
    "Common Data Format (Version 2.6 or 2.7) data": "cdf",
    "current ar archive": "ar",
    "AppleSingle encoded Macintosh file": "applesingle",
    "AppleDouble encoded Macintosh file": "appledouble",
    "H2 Database file": "h2",
    "Apache Avro version 1": "avro",
    "Apache Avro, version 1": "avro",
    "Gridded binary (GRIB) version 1": "grib",
    "Gridded binary (GRIB) version 2": "grib",
    "3D Studio model": "3dsm",
    "Nintendo 3DS Homebrew Application (3DSX)": "3dsx",
    "Microsoft Access Database": "access",
    "Erlang BEAM file": "beam",
    "COLLADA model, XML document": "collada",
    "Hierarchical Data Format (version 4) data": "hdf4",
    "LLVM IR bitcode": "llvm_bitcode",
    "OpenStreetMap XML data": "osm",
    "OpenStreetMap Protocolbuffer Binary Format": "osm",
    "Point Cloud Data": "pcd",
    "SketchUp Model": "sketchup",
    "SAS 7+ data file": "sas",
    "LIDAR point data records, version 1.2, SYSID PDAL, Generating Software Entwine": "las",
    "LIDAR point data records, version 1.2, SYSID OTHER, Generating Software Carlson PointCloud": "las",
    "USD ASCII, version 1.0": "usd",
    "USD crate, version 0.8.0": "usd",
    "USD crate, version 0.9.0": "usd",
}

FILE_PATTERNS = {
    "cram": r"CRAM version (?:2\.[01]|3\.0)(?: \(identified as [ -~]{1,20}\))?",
    "mysql_storage": (
        r"MySQL MyISAM index file Version 1, [0-9]{1,20} key parts, "
        r"[0-9]{1,20} unique key parts, [0-9]{1,20} keys, "
        r"[0-9]{1,20} records, [0-9]{1,20} deleted records"
    ),
    "gltf": r"glTF binary model, version [12], length [0-9]{1,10} bytes",
    "wad": r"doom (?:main IWAD|patch PWAD) data containing [0-9]{1,10} lumps",
    "blend": r"Blender3D, (?:pre-v5, )?saved as (?:32|64)-bits (?:little|big) endian with version [1-4]\.[0-9]{2}(?:\.[0-9]{4})?",
    "fbx": r"Kaydara FBX model, version [0-9]{1,10}",
    "redis_rdb": r"Redis RDB file, version [0-9]{4}",
}


def libmagic_kind(line):
    matches = {kind for kind, pattern in FILE_PATTERNS.items() if re.fullmatch(pattern, line)}
    if line in FILE_DESCRIPTIONS:
        matches.add(FILE_DESCRIPTIONS[line])
    return next(iter(matches)) if len(matches) == 1 else None


GENERIC_CONTINUATIONS = {
    "- data",
    "- , ASCII text",
    "- ASCII text",
    "- XML 1.0 document text",
    "- XML document text",
    "- XML document text, ASCII text",
    "- XML document text, Unicode text, UTF-8 text",
}


def generic_continuation(line):
    """Line lengths and newline styles refine text, not its format identity."""
    base, separator, qualifiers = line.partition(", with ")
    if base not in GENERIC_CONTINUATIONS:
        return False
    if not separator:
        return True
    return all(
        re.fullmatch(
            r"very long lines(?: \([0-9]+\))?|(?:CRLF|CR|LF) line terminators|no line terminators",
            qualifier,
        )
        is not None
        for qualifier in qualifiers.split(", with ")
    )


# Locally run detectors

EXTRA_TRID_NAMES = {
    "avif": {"AV1 Image File Format bitmap", "AV1 Image File Format Sequence bitmap"},
    "zst": {"Zstandard compressed data", "Zstandard compressed data (old/generic)"},
}

EXTRA_FILE_NAMES = {
    "ISO Media, AVIF Image": "avif",
    "ISO Media, AVIF Image Sequence": "avif",
    "Zstandard compressed data (v0.8+), Dictionary ID: None": "zst",
}

SPIRV_DESCRIPTION = re.compile(
    r"Khronos SPIR-V binary, (?:little|big)-endian, version 0x[0-9a-f]{6,8}, "
    r"generator (?:0x[0-9a-f]{6,8}|00000000)"
)

TRID_LINE = re.compile(r"^\s*(\d+(?:\.\d+)?)%\s+\([^\n]*?\)\s+(.+?)\s+\(\d+/\d+/\d+\)\s*$")


def local_file_kind(stdout):
    lines = stdout.splitlines()
    if not lines or any(not generic_continuation(line) for line in lines[1:]):
        return None
    first = lines[0]
    return (
        "spirv"
        if SPIRV_DESCRIPTION.fullmatch(first)
        else EXTRA_FILE_NAMES.get(first) or libmagic_kind(first)
    )


def local_trid_evidence(stdout, formats):
    ranked = []
    for line in stdout.splitlines():
        match = TRID_LINE.fullmatch(line)
        if match:
            probability = float(match[1])
            if not 0 <= probability <= 100:
                return {"status": "malformed", "format_ids": [], "raw": ranked}
            ranked.append({"probability": probability, "file_type": match[2]})
        elif re.match(r"^\s*[\d.+-]+%", line):
            return {"status": "malformed", "format_ids": [], "raw": ranked}
    if not ranked:
        return {"status": "missing", "format_ids": [], "raw": []}
    names = {k: set(v) | EXTRA_TRID_NAMES.get(k, set()) for k, v in TRID_NAMES.items()}
    for kind, values in EXTRA_TRID_NAMES.items():
        names.setdefault(kind, values)
    top = [r for r in ranked if r["probability"] == max(x["probability"] for x in ranked)]
    mapped = [
        {k for k, values in names.items() if k in formats and r["file_type"] in values} for r in top
    ]
    kinds = sorted(set().union(*mapped))
    status = "mapped" if all(mapped) and len(kinds) == 1 else "unmapped"
    return {"status": status, "format_ids": kinds, "raw": ranked}


WIM = re.compile(
    r"Windows imaging (?P<variant>\(WIM\) image|\(ESD\) image|"
    r"\(SWM [1-9][0-9]{0,4} of [1-9][0-9]{0,4}\) image|"
    r"\(Windows provisioning package\))"
    r"(?:, wimlib pipable format)? v(?P<version>1\.13|0\.14)"
    r"(?:, (?:[1-9][0-9]{0,9} images|bootable no\. [1-9][0-9]{0,9}|"
    r"(?:LZMS|LZX|XPRESS2?)(?: compressed)?|compressed|read only|resource only|"
    r"metadata only|reparse point fixup))*"
)

MAYA = re.compile(
    r"Alias Maya (?P<representation>Ascii|Binary) File, version "
    r"(?P<version>[0-9]{1,4}(?:\.[0-9]{1,2})?(?:ff[0-9]{2})?) scene"
    r"(?P<qualifiers>, .+)?"
)

BAM = re.compile(
    r"SAMtools BAM \(Binary Sequence Alignment/Map\)"
    r"(?:, with SAM header(?: version [0-9]+(?:\.[0-9]+)*)?)?"
    r"(?:, with [1-9][0-9]{0,9} reference sequences)?"
)

RZIP = re.compile(r"rzip compressed data - version 2\.1 \([0-9]{1,10} bytes\)")


def description_kind_wim(stdout):
    lines = stdout.splitlines()
    if not lines:
        return None
    first = lines[0]
    if any(
        not generic_continuation(line) and not (RZIP.fullmatch(first) and line == "- " + first)
        for line in lines[1:]
    ):
        return None
    if WIM.fullmatch(first):
        return "wim"
    maya = MAYA.fullmatch(first)
    if maya and (not maya["qualifiers"] or generic_continuation("- " + maya["qualifiers"][2:])):
        return "maya"
    if first in {
        "LRZIP compressed data - version 0.6",
        "LRZIP compressed data - version 0.6, encrypted",
    }:
        return "lrz"
    if RZIP.fullmatch(first):
        return "rzip"
    if first == "Microsoft ASF":
        return "asf"
    if BAM.fullmatch(first):
        return "bam"
    return None


PG = re.compile(r"PostgreSQL custom database dump - v1\.(?:[7-9]|1[0-6])-0")

COMPRESS = re.compile(r"compress'd data(?: block compressed)? (?:9|1[0-6]) bits")


def description_kind_database(stdout):
    lines = stdout.splitlines()
    if not lines or any(not generic_continuation(s) for s in lines[1:]):
        return None
    first = lines[0]
    if PG.fullmatch(first):
        return "postgres_dump"
    if first == "LZ4 compressed data (v1.4+)":
        return "lz4"
    if COMPRESS.fullmatch(first):
        return "unixcompress"
    return None


def description_kind(stdout):
    """The format a libmagic description names, or None. First match wins."""
    return (
        description_kind_wim(stdout) or description_kind_database(stdout) or local_file_kind(stdout)
    )
