# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Explicit detector description mappings; these functions do not adjudicate labels."""

import re

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


def description_kind(line):
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
