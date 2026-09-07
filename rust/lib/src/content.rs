// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// DO NOT EDIT, see link below for more information:
// https://github.com/google/magika/tree/main/rust/gen

use crate::file::TypeInfo;

/// Model name (only comparable with equality).
pub const MODEL_NAME: &str = "standard_v3_3";

/// Model major version.
pub const MODEL_MAJOR_VERSION: u32 = 3;

pub(crate) static _3DSM: TypeInfo = TypeInfo {
    label: "3dsm",
    mime_type: "application/x-3ds",
    group: "image",
    description: "3D studio Max",
    extensions: &["3ds"],
    is_text: false,
};

pub(crate) static _3DSX: TypeInfo = TypeInfo {
    label: "3dsx",
    mime_type: "application/octet-stream",
    group: "unknown",
    description: "Nintendo 3DS homebrew",
    extensions: &["3dsx"],
    is_text: false,
};

pub(crate) static _3GP: TypeInfo = TypeInfo {
    label: "3gp",
    mime_type: "video/3gpp",
    group: "video",
    description: "3GPP multimedia file",
    extensions: &["3gp"],
    is_text: false,
};

pub(crate) static ACCESS: TypeInfo = TypeInfo {
    label: "access",
    mime_type: "application/octet-stream",
    group: "database",
    description: "Microsoft Access database",
    extensions: &["accdb", "mdb"],
    is_text: false,
};

pub(crate) static ACE: TypeInfo = TypeInfo {
    label: "ace",
    mime_type: "application/x-ace-compressed",
    group: "archive",
    description: "ACE archive",
    extensions: &["ace"],
    is_text: false,
};

pub(crate) static AI: TypeInfo = TypeInfo {
    label: "ai",
    mime_type: "application/pdf",
    group: "document",
    description: "Adobe Illustrator Artwork",
    extensions: &["ai"],
    is_text: false,
};

pub(crate) static AIDL: TypeInfo = TypeInfo {
    label: "aidl",
    mime_type: "text/plain",
    group: "unknown",
    description: "Android Interface Definition Language",
    extensions: &["aidl"],
    is_text: true,
};

pub(crate) static ANI: TypeInfo = TypeInfo {
    label: "ani",
    mime_type: "application/x-navi-animation",
    group: "unknown",
    description: "Animated cursor",
    extensions: &["ani"],
    is_text: false,
};

pub(crate) static AOUT: TypeInfo = TypeInfo {
    label: "aout",
    mime_type: "application/octet-stream",
    group: "executable",
    description: "a.out object / executable",
    extensions: &["out"],
    is_text: false,
};

pub(crate) static APK: TypeInfo = TypeInfo {
    label: "apk",
    mime_type: "application/vnd.android.package-archive",
    group: "executable",
    description: "Android package",
    extensions: &["apk"],
    is_text: false,
};

pub(crate) static APPLEBPLIST: TypeInfo = TypeInfo {
    label: "applebplist",
    mime_type: "application/x-bplist",
    group: "application",
    description: "Apple binary property list",
    extensions: &["bplist", "plist"],
    is_text: false,
};

pub(crate) static APPLEDOUBLE: TypeInfo = TypeInfo {
    label: "appledouble",
    mime_type: "multipart/appledouble",
    group: "unknown",
    description: "AppleDouble",
    extensions: &[],
    is_text: false,
};

pub(crate) static APPLEPLIST: TypeInfo = TypeInfo {
    label: "appleplist",
    mime_type: "application/x-plist",
    group: "application",
    description: "Apple property list",
    extensions: &["plist"],
    is_text: true,
};

pub(crate) static APPLESINGLE: TypeInfo = TypeInfo {
    label: "applesingle",
    mime_type: "application/applefile",
    group: "unknown",
    description: "AppleSingle",
    extensions: &[],
    is_text: false,
};

pub(crate) static ARC: TypeInfo = TypeInfo {
    label: "arc",
    mime_type: "application/x-arc",
    group: "archive",
    description: "Arc",
    extensions: &["arc"],
    is_text: false,
};

pub(crate) static ARJ: TypeInfo = TypeInfo {
    label: "arj",
    mime_type: "application/arj",
    group: "archive",
    description: "Arj",
    extensions: &[],
    is_text: false,
};

pub(crate) static ARROW: TypeInfo = TypeInfo {
    label: "arrow",
    mime_type: "vnd.apache.arrow.file",
    group: "unknown",
    description: "arrow",
    extensions: &[],
    is_text: false,
};

pub(crate) static ASF: TypeInfo = TypeInfo {
    label: "asf",
    mime_type: "video/x-ms-wma",
    group: "application",
    description: "Microsoft Advanced Systems Format",
    extensions: &["asf"],
    is_text: false,
};

pub(crate) static ASM: TypeInfo = TypeInfo {
    label: "asm",
    mime_type: "text/x-asm",
    group: "code",
    description: "Assembly",
    extensions: &["s", "S", "asm"],
    is_text: true,
};

pub(crate) static ASP: TypeInfo = TypeInfo {
    label: "asp",
    mime_type: "text/html",
    group: "code",
    description: "ASP source",
    extensions: &["aspx", "asp"],
    is_text: true,
};

pub(crate) static AU: TypeInfo = TypeInfo {
    label: "au",
    mime_type: "audio/basic",
    group: "audio",
    description: "NeXT/Sun AU",
    extensions: &["au"],
    is_text: false,
};

pub(crate) static AUTOHOTKEY: TypeInfo = TypeInfo {
    label: "autohotkey",
    mime_type: "text/plain",
    group: "code",
    description: "AutoHotKey script",
    extensions: &[],
    is_text: true,
};

pub(crate) static AUTOIT: TypeInfo = TypeInfo {
    label: "autoit",
    mime_type: "text/plain",
    group: "code",
    description: "AutoIt script",
    extensions: &["au3"],
    is_text: true,
};

pub(crate) static AVI: TypeInfo = TypeInfo {
    label: "avi",
    mime_type: "video/x-msvideo",
    group: "video",
    description: "Audio Video Interleave",
    extensions: &["avi"],
    is_text: false,
};

pub(crate) static AVIF: TypeInfo = TypeInfo {
    label: "avif",
    mime_type: "image/avif",
    group: "video",
    description: "AV1 Image File Format",
    extensions: &["avif", "avifs"],
    is_text: false,
};

pub(crate) static AVRO: TypeInfo = TypeInfo {
    label: "avro",
    mime_type: "application/x-avro-binary",
    group: "unknown",
    description: "Apache Avro binary",
    extensions: &["avro"],
    is_text: false,
};

pub(crate) static AWK: TypeInfo = TypeInfo {
    label: "awk",
    mime_type: "text/plain",
    group: "code",
    description: "Awk",
    extensions: &["awk"],
    is_text: true,
};

pub(crate) static BAM: TypeInfo = TypeInfo {
    label: "bam",
    mime_type: "application/octet-stream",
    group: "scientific",
    description: "BAM alignment",
    extensions: &["bam"],
    is_text: false,
};

pub(crate) static BATCH: TypeInfo = TypeInfo {
    label: "batch",
    mime_type: "text/x-msdos-batch",
    group: "code",
    description: "DOS batch file",
    extensions: &["bat"],
    is_text: true,
};

pub(crate) static BAZEL: TypeInfo = TypeInfo {
    label: "bazel",
    mime_type: "text/plain",
    group: "code",
    description: "Bazel build file",
    extensions: &["bzl"],
    is_text: true,
};

pub(crate) static BEAM: TypeInfo = TypeInfo {
    label: "beam",
    mime_type: "application/octet-stream",
    group: "executable",
    description: "Erlang BEAM",
    extensions: &["beam"],
    is_text: false,
};

pub(crate) static BERKELEYDB: TypeInfo = TypeInfo {
    label: "berkeleydb",
    mime_type: "application/octet-stream",
    group: "database",
    description: "Berkeley DB",
    extensions: &["db"],
    is_text: false,
};

pub(crate) static BIB: TypeInfo = TypeInfo {
    label: "bib",
    mime_type: "text/x-bibtex",
    group: "text",
    description: "BibTeX",
    extensions: &["bib"],
    is_text: true,
};

pub(crate) static BLEND: TypeInfo = TypeInfo {
    label: "blend",
    mime_type: "application/octet-stream",
    group: "geometry",
    description: "Blender scene",
    extensions: &["blend"],
    is_text: false,
};

pub(crate) static BMP: TypeInfo = TypeInfo {
    label: "bmp",
    mime_type: "image/bmp",
    group: "image",
    description: "BMP image data",
    extensions: &["bmp"],
    is_text: false,
};

pub(crate) static BPG: TypeInfo = TypeInfo {
    label: "bpg",
    mime_type: "image/bpg",
    group: "image",
    description: "BPG",
    extensions: &["bpg"],
    is_text: false,
};

pub(crate) static BZIP: TypeInfo = TypeInfo {
    label: "bzip",
    mime_type: "application/x-bzip2",
    group: "archive",
    description: "bzip2 compressed data",
    extensions: &["bz2", "tbz2", "tar.bz2"],
    is_text: false,
};

pub(crate) static BZIP3: TypeInfo = TypeInfo {
    label: "bzip3",
    mime_type: "application/x-bzip3",
    group: "archive",
    description: "bzip3 compressed data",
    extensions: &["bz3"],
    is_text: false,
};

pub(crate) static C: TypeInfo = TypeInfo {
    label: "c",
    mime_type: "text/x-c",
    group: "code",
    description: "C source",
    extensions: &["c"],
    is_text: true,
};

pub(crate) static CAB: TypeInfo = TypeInfo {
    label: "cab",
    mime_type: "application/vnd.ms-cab-compressed",
    group: "archive",
    description: "Microsoft Cabinet archive data",
    extensions: &["cab"],
    is_text: false,
};

pub(crate) static CAT: TypeInfo = TypeInfo {
    label: "cat",
    mime_type: "application/octet-stream",
    group: "application",
    description: "Windows Catalog file",
    extensions: &["cat"],
    is_text: false,
};

pub(crate) static CHM: TypeInfo = TypeInfo {
    label: "chm",
    mime_type: "application/chm",
    group: "application",
    description: "MS Windows HtmlHelp Data",
    extensions: &["chm"],
    is_text: false,
};

pub(crate) static CINEMA4D: TypeInfo = TypeInfo {
    label: "cinema4d",
    mime_type: "application/octet-stream",
    group: "geometry",
    description: "Cinema 4D scene",
    extensions: &["c4d"],
    is_text: false,
};

pub(crate) static CLOJURE: TypeInfo = TypeInfo {
    label: "clojure",
    mime_type: "text/x-clojure",
    group: "code",
    description: "Clojure",
    extensions: &["clj", "cljs", "cljc", "cljr"],
    is_text: true,
};

pub(crate) static CMAKE: TypeInfo = TypeInfo {
    label: "cmake",
    mime_type: "text/x-cmake",
    group: "code",
    description: "CMake build file",
    extensions: &["cmake"],
    is_text: true,
};

pub(crate) static COBOL: TypeInfo = TypeInfo {
    label: "cobol",
    mime_type: "text/x-cobol",
    group: "code",
    description: "Cobol",
    extensions: &["cbl", "cob", "cpy", "CBL", "COB", "CPY"],
    is_text: true,
};

pub(crate) static COFF: TypeInfo = TypeInfo {
    label: "coff",
    mime_type: "application/x-coff",
    group: "executable",
    description: "Intel 80386 COFF",
    extensions: &["obj", "o"],
    is_text: false,
};

pub(crate) static COFFEESCRIPT: TypeInfo = TypeInfo {
    label: "coffeescript",
    mime_type: "text/coffeescript",
    group: "code",
    description: "CoffeeScript",
    extensions: &["coffee"],
    is_text: true,
};

pub(crate) static CPP: TypeInfo = TypeInfo {
    label: "cpp",
    mime_type: "text/x-c",
    group: "code",
    description: "C++ source",
    extensions: &["cc", "cpp", "cxx", "c++", "cppm", "ixx"],
    is_text: true,
};

pub(crate) static CRAM: TypeInfo = TypeInfo {
    label: "cram",
    mime_type: "application/octet-stream",
    group: "scientific",
    description: "CRAM alignment",
    extensions: &["cram"],
    is_text: false,
};

pub(crate) static CRT: TypeInfo = TypeInfo {
    label: "crt",
    mime_type: "application/x-x509-ca-cert",
    group: "text",
    description: "Certificates (binary format)",
    extensions: &["der", "cer", "crt"],
    is_text: false,
};

pub(crate) static CRX: TypeInfo = TypeInfo {
    label: "crx",
    mime_type: "application/x-chrome-extension",
    group: "executable",
    description: "Google Chrome extension",
    extensions: &["crx"],
    is_text: false,
};

pub(crate) static CS: TypeInfo = TypeInfo {
    label: "cs",
    mime_type: "text/plain",
    group: "code",
    description: "C# source",
    extensions: &["cs", "csx"],
    is_text: true,
};

pub(crate) static CSPROJ: TypeInfo = TypeInfo {
    label: "csproj",
    mime_type: "text/plain",
    group: "code",
    description: ".NET project config",
    extensions: &["csproj"],
    is_text: true,
};

pub(crate) static CSS: TypeInfo = TypeInfo {
    label: "css",
    mime_type: "text/css",
    group: "code",
    description: "CSS source",
    extensions: &["css"],
    is_text: true,
};

pub(crate) static CSV: TypeInfo = TypeInfo {
    label: "csv",
    mime_type: "text/csv",
    group: "code",
    description: "CSV document",
    extensions: &["csv"],
    is_text: true,
};

pub(crate) static DART: TypeInfo = TypeInfo {
    label: "dart",
    mime_type: "text/plain",
    group: "code",
    description: "Dart source",
    extensions: &["dart"],
    is_text: true,
};

pub(crate) static DBASE: TypeInfo = TypeInfo {
    label: "dbase",
    mime_type: "application/octet-stream",
    group: "database",
    description: "dBASE / FoxPro table",
    extensions: &["dbf"],
    is_text: false,
};

pub(crate) static DEB: TypeInfo = TypeInfo {
    label: "deb",
    mime_type: "application/vnd.debian.binary-package",
    group: "archive",
    description: "Debian binary package",
    extensions: &["deb"],
    is_text: false,
};

pub(crate) static DEX: TypeInfo = TypeInfo {
    label: "dex",
    mime_type: "application/x-android-dex",
    group: "executable",
    description: "Dalvik dex file",
    extensions: &["dex"],
    is_text: false,
};

pub(crate) static DICOM: TypeInfo = TypeInfo {
    label: "dicom",
    mime_type: "application/dicom",
    group: "image",
    description: "DICOM",
    extensions: &["dcm"],
    is_text: false,
};

pub(crate) static DIFF: TypeInfo = TypeInfo {
    label: "diff",
    mime_type: "text/plain",
    group: "text",
    description: "Diff file",
    extensions: &["diff", "patch"],
    is_text: true,
};

pub(crate) static DIRECTORY: TypeInfo = TypeInfo {
    label: "directory",
    mime_type: "inode/directory",
    group: "inode",
    description: "A directory",
    extensions: &[],
    is_text: false,
};

pub(crate) static DM: TypeInfo = TypeInfo {
    label: "dm",
    mime_type: "text/plain",
    group: "code",
    description: "Dream Maker",
    extensions: &["dm"],
    is_text: true,
};

pub(crate) static DMG: TypeInfo = TypeInfo {
    label: "dmg",
    mime_type: "application/x-apple-diskimage",
    group: "archive",
    description: "Apple disk image",
    extensions: &["dmg"],
    is_text: false,
};

pub(crate) static DOC: TypeInfo = TypeInfo {
    label: "doc",
    mime_type: "application/msword",
    group: "document",
    description: "Microsoft Word CDF document",
    extensions: &["doc"],
    is_text: false,
};

pub(crate) static DOCKERFILE: TypeInfo = TypeInfo {
    label: "dockerfile",
    mime_type: "text/x-dockerfile",
    group: "code",
    description: "Dockerfile",
    extensions: &[],
    is_text: true,
};

pub(crate) static DOCX: TypeInfo = TypeInfo {
    label: "docx",
    mime_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    group: "document",
    description: "Microsoft Word 2007+ document",
    extensions: &["docx", "docm"],
    is_text: false,
};

pub(crate) static DOTX: TypeInfo = TypeInfo {
    label: "dotx",
    mime_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.template",
    group: "document",
    description: "Office Word 2007 template",
    extensions: &["dotx"],
    is_text: false,
};

pub(crate) static DSSTORE: TypeInfo = TypeInfo {
    label: "dsstore",
    mime_type: "application/octet-stream",
    group: "unknown",
    description: "Application Desktop Services Store",
    extensions: &[],
    is_text: false,
};

pub(crate) static DUCKDB: TypeInfo = TypeInfo {
    label: "duckdb",
    mime_type: "application/octet-stream",
    group: "database",
    description: "DuckDB database",
    extensions: &["duckdb"],
    is_text: false,
};

pub(crate) static DWG: TypeInfo = TypeInfo {
    label: "dwg",
    mime_type: "image/x-dwg",
    group: "image",
    description: "Autocad Drawing",
    extensions: &["dwg"],
    is_text: false,
};

pub(crate) static DXF: TypeInfo = TypeInfo {
    label: "dxf",
    mime_type: "image/vnd.dxf",
    group: "image",
    description: "Audocad Drawing Exchange Format",
    extensions: &["dxf"],
    is_text: true,
};

pub(crate) static ELF: TypeInfo = TypeInfo {
    label: "elf",
    mime_type: "application/x-executable-elf",
    group: "executable",
    description: "ELF executable",
    extensions: &["elf"],
    is_text: false,
};

pub(crate) static ELIXIR: TypeInfo = TypeInfo {
    label: "elixir",
    mime_type: "text/plain",
    group: "code",
    description: "Elixir script",
    extensions: &["exs"],
    is_text: true,
};

pub(crate) static EMF: TypeInfo = TypeInfo {
    label: "emf",
    mime_type: "application/octet-stream",
    group: "application",
    description: "Windows Enhanced Metafile image data",
    extensions: &["emf"],
    is_text: false,
};

pub(crate) static EML: TypeInfo = TypeInfo {
    label: "eml",
    mime_type: "message/rfc822",
    group: "text",
    description: "RFC 822 mail",
    extensions: &["eml"],
    is_text: true,
};

pub(crate) static EMPTY: TypeInfo = TypeInfo {
    label: "empty",
    mime_type: "inode/x-empty",
    group: "inode",
    description: "Empty file",
    extensions: &[],
    is_text: false,
};

pub(crate) static EPUB: TypeInfo = TypeInfo {
    label: "epub",
    mime_type: "application/epub+zip",
    group: "document",
    description: "EPUB document",
    extensions: &["epub"],
    is_text: false,
};

pub(crate) static ERB: TypeInfo = TypeInfo {
    label: "erb",
    mime_type: "text/x-ruby",
    group: "code",
    description: "Embedded Ruby source",
    extensions: &["erb"],
    is_text: true,
};

pub(crate) static ERLANG: TypeInfo = TypeInfo {
    label: "erlang",
    mime_type: "text/x-erlang",
    group: "code",
    description: "Erlang source",
    extensions: &["erl", "hrl"],
    is_text: true,
};

pub(crate) static ESE: TypeInfo = TypeInfo {
    label: "ese",
    mime_type: "application/x-ms-ese",
    group: "unknown",
    description: "ESE Db",
    extensions: &["dat"],
    is_text: false,
};

pub(crate) static FBX: TypeInfo = TypeInfo {
    label: "fbx",
    mime_type: "application/octet-stream",
    group: "geometry",
    description: "Autodesk FBX",
    extensions: &["fbx"],
    is_text: false,
};

pub(crate) static FILEMAKER: TypeInfo = TypeInfo {
    label: "filemaker",
    mime_type: "application/octet-stream",
    group: "database",
    description: "FileMaker database",
    extensions: &["fmp12", "fp7"],
    is_text: false,
};

pub(crate) static FITS: TypeInfo = TypeInfo {
    label: "fits",
    mime_type: "application/octet-stream",
    group: "scientific",
    description: "FITS astronomy data",
    extensions: &["fit", "fits", "fts"],
    is_text: false,
};

pub(crate) static FLAC: TypeInfo = TypeInfo {
    label: "flac",
    mime_type: "audio/flac",
    group: "audio",
    description: "FLAC audio bitstream data",
    extensions: &["flac"],
    is_text: false,
};

pub(crate) static FLATGEOBUF: TypeInfo = TypeInfo {
    label: "flatgeobuf",
    mime_type: "application/octet-stream",
    group: "gis",
    description: "FlatGeobuf",
    extensions: &["fgb"],
    is_text: false,
};

pub(crate) static FLV: TypeInfo = TypeInfo {
    label: "flv",
    mime_type: "video/x-flv",
    group: "video",
    description: "Flash Video",
    extensions: &["flv"],
    is_text: false,
};

pub(crate) static FORTRAN: TypeInfo = TypeInfo {
    label: "fortran",
    mime_type: "text/x-fortran",
    group: "document",
    description: "Fortran",
    extensions: &["f90", "f95", "f03", "F90"],
    is_text: true,
};

pub(crate) static GEMFILE: TypeInfo = TypeInfo {
    label: "gemfile",
    mime_type: "text/plain",
    group: "code",
    description: "Gemfile file",
    extensions: &[],
    is_text: true,
};

pub(crate) static GEMSPEC: TypeInfo = TypeInfo {
    label: "gemspec",
    mime_type: "text/plain",
    group: "code",
    description: "Gemspec file",
    extensions: &["gemspec"],
    is_text: true,
};

pub(crate) static GGUF: TypeInfo = TypeInfo {
    label: "gguf",
    mime_type: "application/octet-stream",
    group: "model",
    description: "GGUF",
    extensions: &["gguf"],
    is_text: false,
};

pub(crate) static GIF: TypeInfo = TypeInfo {
    label: "gif",
    mime_type: "image/gif",
    group: "image",
    description: "GIF image data",
    extensions: &["gif"],
    is_text: false,
};

pub(crate) static GITATTRIBUTES: TypeInfo = TypeInfo {
    label: "gitattributes",
    mime_type: "text/plain",
    group: "code",
    description: "Gitattributes file",
    extensions: &[],
    is_text: true,
};

pub(crate) static GITMODULES: TypeInfo = TypeInfo {
    label: "gitmodules",
    mime_type: "text/plain",
    group: "code",
    description: "Gitmodules file",
    extensions: &[],
    is_text: true,
};

pub(crate) static GLTF: TypeInfo = TypeInfo {
    label: "gltf",
    mime_type: "application/octet-stream",
    group: "geometry",
    description: "glTF",
    extensions: &["glb", "gltf"],
    is_text: false,
};

pub(crate) static GO: TypeInfo = TypeInfo {
    label: "go",
    mime_type: "text/x-golang",
    group: "code",
    description: "Golang source",
    extensions: &["go"],
    is_text: true,
};

pub(crate) static GRADLE: TypeInfo = TypeInfo {
    label: "gradle",
    mime_type: "text/x-groovy",
    group: "code",
    description: "Gradle source",
    extensions: &["gradle"],
    is_text: true,
};

pub(crate) static GROOVY: TypeInfo = TypeInfo {
    label: "groovy",
    mime_type: "text/x-groovy",
    group: "code",
    description: "Groovy source",
    extensions: &["groovy"],
    is_text: true,
};

pub(crate) static GZIP: TypeInfo = TypeInfo {
    label: "gzip",
    mime_type: "application/gzip",
    group: "archive",
    description: "gzip compressed data",
    extensions: &["gz", "gzip", "tgz", "tar.gz"],
    is_text: false,
};

pub(crate) static H5: TypeInfo = TypeInfo {
    label: "h5",
    mime_type: "application/x-hdf5",
    group: "archive",
    description: "Hierarchical Data Format v5",
    extensions: &["h5", "hdf5"],
    is_text: false,
};

pub(crate) static HANDLEBARS: TypeInfo = TypeInfo {
    label: "handlebars",
    mime_type: "text/x-handlebars-template",
    group: "code",
    description: "Handlebars source",
    extensions: &["hbs", "handlebars"],
    is_text: true,
};

pub(crate) static HASKELL: TypeInfo = TypeInfo {
    label: "haskell",
    mime_type: "text/plain",
    group: "code",
    description: "Haskell source",
    extensions: &["hs", "lhs"],
    is_text: true,
};

pub(crate) static HCL: TypeInfo = TypeInfo {
    label: "hcl",
    mime_type: "text/x-hcl",
    group: "code",
    description: "HashiCorp configuration language",
    extensions: &["hcl"],
    is_text: true,
};

pub(crate) static HDF4: TypeInfo = TypeInfo {
    label: "hdf4",
    mime_type: "application/octet-stream",
    group: "scientific",
    description: "HDF4",
    extensions: &["h4", "hdf"],
    is_text: false,
};

pub(crate) static HEIF: TypeInfo = TypeInfo {
    label: "heif",
    mime_type: "image/heic",
    group: "image",
    description: "High Efficiency Image File",
    extensions: &["heif", "heifs", "heic", "heics"],
    is_text: false,
};

pub(crate) static HLP: TypeInfo = TypeInfo {
    label: "hlp",
    mime_type: "application/winhlp",
    group: "application",
    description: "MS Windows help",
    extensions: &["hlp"],
    is_text: false,
};

pub(crate) static HTACCESS: TypeInfo = TypeInfo {
    label: "htaccess",
    mime_type: "text/x-apache-conf",
    group: "code",
    description: "Apache access configuration",
    extensions: &[],
    is_text: true,
};

pub(crate) static HTML: TypeInfo = TypeInfo {
    label: "html",
    mime_type: "text/html",
    group: "code",
    description: "HTML document",
    extensions: &["html", "htm", "xhtml", "xht"],
    is_text: true,
};

pub(crate) static HWP: TypeInfo = TypeInfo {
    label: "hwp",
    mime_type: "application/x-hwp",
    group: "document",
    description: "Hangul Word Processor",
    extensions: &["hwp"],
    is_text: false,
};

pub(crate) static ICC: TypeInfo = TypeInfo {
    label: "icc",
    mime_type: "application/vnd.iccprofile",
    group: "unknown",
    description: "ICC profile",
    extensions: &["icc"],
    is_text: false,
};

pub(crate) static ICNS: TypeInfo = TypeInfo {
    label: "icns",
    mime_type: "image/x-icns",
    group: "image",
    description: "Mac OS X icon",
    extensions: &["icns"],
    is_text: false,
};

pub(crate) static ICO: TypeInfo = TypeInfo {
    label: "ico",
    mime_type: "image/vnd.microsoft.icon",
    group: "image",
    description: "MS Windows icon resource",
    extensions: &["ico"],
    is_text: false,
};

pub(crate) static ICS: TypeInfo = TypeInfo {
    label: "ics",
    mime_type: "text/calendar",
    group: "application",
    description: "Internet Calendaring and Scheduling",
    extensions: &["ics"],
    is_text: true,
};

pub(crate) static IGNOREFILE: TypeInfo = TypeInfo {
    label: "ignorefile",
    mime_type: "text/plain",
    group: "code",
    description: "Ignorefile",
    extensions: &[],
    is_text: true,
};

pub(crate) static INI: TypeInfo = TypeInfo {
    label: "ini",
    mime_type: "text/plain",
    group: "text",
    description: "INI configuration file",
    extensions: &["ini"],
    is_text: true,
};

pub(crate) static INTERNETSHORTCUT: TypeInfo = TypeInfo {
    label: "internetshortcut",
    mime_type: "application/x-mswinurl",
    group: "application",
    description: "MS Windows Internet shortcut",
    extensions: &["url"],
    is_text: true,
};

pub(crate) static IPYNB: TypeInfo = TypeInfo {
    label: "ipynb",
    mime_type: "application/json",
    group: "code",
    description: "Jupyter notebook",
    extensions: &["ipynb"],
    is_text: true,
};

pub(crate) static ISO: TypeInfo = TypeInfo {
    label: "iso",
    mime_type: "application/x-iso9660-image",
    group: "archive",
    description: "ISO 9660 CD-ROM filesystem data",
    extensions: &["iso"],
    is_text: false,
};

pub(crate) static JAR: TypeInfo = TypeInfo {
    label: "jar",
    mime_type: "application/java-archive",
    group: "archive",
    description: "Java archive data (JAR)",
    extensions: &["jar", "klib"],
    is_text: false,
};

pub(crate) static JAVA: TypeInfo = TypeInfo {
    label: "java",
    mime_type: "text/x-java",
    group: "code",
    description: "Java source",
    extensions: &["java"],
    is_text: true,
};

pub(crate) static JAVABYTECODE: TypeInfo = TypeInfo {
    label: "javabytecode",
    mime_type: "application/x-java-applet",
    group: "executable",
    description: "Java compiled bytecode",
    extensions: &["class"],
    is_text: false,
};

pub(crate) static JAVASCRIPT: TypeInfo = TypeInfo {
    label: "javascript",
    mime_type: "application/javascript",
    group: "code",
    description: "JavaScript source",
    extensions: &["js", "mjs", "cjs"],
    is_text: true,
};

pub(crate) static JINJA: TypeInfo = TypeInfo {
    label: "jinja",
    mime_type: "text/x-jinja2-template",
    group: "code",
    description: "Jinja template",
    extensions: &["jinja", "jinja2", "j2"],
    is_text: true,
};

pub(crate) static JP2: TypeInfo = TypeInfo {
    label: "jp2",
    mime_type: "image/jpeg2000",
    group: "image",
    description: "jpeg2000",
    extensions: &["jp2"],
    is_text: false,
};

pub(crate) static JPEG: TypeInfo = TypeInfo {
    label: "jpeg",
    mime_type: "image/jpeg",
    group: "image",
    description: "JPEG image data",
    extensions: &["jpg", "jpeg"],
    is_text: false,
};

pub(crate) static JSON: TypeInfo = TypeInfo {
    label: "json",
    mime_type: "application/json",
    group: "code",
    description: "JSON document",
    extensions: &["json"],
    is_text: true,
};

pub(crate) static JSONL: TypeInfo = TypeInfo {
    label: "jsonl",
    mime_type: "application/json",
    group: "code",
    description: "JSONL document",
    extensions: &["jsonl", "jsonld"],
    is_text: true,
};

pub(crate) static JULIA: TypeInfo = TypeInfo {
    label: "julia",
    mime_type: "text/x-julia",
    group: "code",
    description: "Julia source",
    extensions: &["jl"],
    is_text: true,
};

pub(crate) static JXL: TypeInfo = TypeInfo {
    label: "jxl",
    mime_type: "image/jxl",
    group: "image",
    description: "JPEG XL",
    extensions: &["jxl"],
    is_text: false,
};

pub(crate) static KOTLIN: TypeInfo = TypeInfo {
    label: "kotlin",
    mime_type: "text/plain",
    group: "code",
    description: "Kotlin source",
    extensions: &["kt", "kts"],
    is_text: true,
};

pub(crate) static LATEX: TypeInfo = TypeInfo {
    label: "latex",
    mime_type: "text/x-tex",
    group: "text",
    description: "LaTeX document",
    extensions: &["tex", "sty"],
    is_text: true,
};

pub(crate) static LHA: TypeInfo = TypeInfo {
    label: "lha",
    mime_type: "application/x-lha",
    group: "archive",
    description: "LHarc archive",
    extensions: &["lha", "lzh"],
    is_text: false,
};

pub(crate) static LIGHTWAVE: TypeInfo = TypeInfo {
    label: "lightwave",
    mime_type: "application/octet-stream",
    group: "geometry",
    description: "LightWave object / scene",
    extensions: &["lwo", "lws"],
    is_text: false,
};

pub(crate) static LISP: TypeInfo = TypeInfo {
    label: "lisp",
    mime_type: "text/x-lisp",
    group: "code",
    description: "Lisp source",
    extensions: &["lisp", "lsp", "l", "cl"],
    is_text: true,
};

pub(crate) static LLVM_BITCODE: TypeInfo = TypeInfo {
    label: "llvm_bitcode",
    mime_type: "application/octet-stream",
    group: "executable",
    description: "LLVM bitcode",
    extensions: &["bc"],
    is_text: false,
};

pub(crate) static LMDB: TypeInfo = TypeInfo {
    label: "lmdb",
    mime_type: "application/octet-stream",
    group: "database",
    description: "LMDB",
    extensions: &["mdb"],
    is_text: false,
};

pub(crate) static LNK: TypeInfo = TypeInfo {
    label: "lnk",
    mime_type: "application/x-ms-shortcut",
    group: "application",
    description: "MS Windows shortcut",
    extensions: &["lnk"],
    is_text: false,
};

pub(crate) static LRZ: TypeInfo = TypeInfo {
    label: "lrz",
    mime_type: "application/x-lrzip",
    group: "unknown",
    description: "LRZip",
    extensions: &["lrz"],
    is_text: false,
};

pub(crate) static LUA: TypeInfo = TypeInfo {
    label: "lua",
    mime_type: "text/plain",
    group: "code",
    description: "Lua",
    extensions: &["lua"],
    is_text: true,
};

pub(crate) static LUABYTECODE: TypeInfo = TypeInfo {
    label: "luabytecode",
    mime_type: "application/octet-stream",
    group: "executable",
    description: "Lua bytecode",
    extensions: &["luac"],
    is_text: false,
};

pub(crate) static LZ: TypeInfo = TypeInfo {
    label: "lz",
    mime_type: "application/x-lzip",
    group: "archive",
    description: "LZip",
    extensions: &["lz"],
    is_text: false,
};

pub(crate) static LZ4: TypeInfo = TypeInfo {
    label: "lz4",
    mime_type: "application/x-lz4",
    group: "archive",
    description: "LZ4",
    extensions: &["lz4"],
    is_text: false,
};

pub(crate) static LZX: TypeInfo = TypeInfo {
    label: "lzx",
    mime_type: "application/octet-stream",
    group: "unknown",
    description: "lzx",
    extensions: &[],
    is_text: false,
};

pub(crate) static M3U: TypeInfo = TypeInfo {
    label: "m3u",
    mime_type: "text/plain",
    group: "application",
    description: "M3U playlist",
    extensions: &["m3u8", "m3u"],
    is_text: true,
};

pub(crate) static M4: TypeInfo = TypeInfo {
    label: "m4",
    mime_type: "text/plain",
    group: "code",
    description: "GNU Macro",
    extensions: &["m4"],
    is_text: true,
};

pub(crate) static MACHO: TypeInfo = TypeInfo {
    label: "macho",
    mime_type: "application/x-mach-o",
    group: "executable",
    description: "Mach-O executable",
    extensions: &[],
    is_text: false,
};

pub(crate) static MAKEFILE: TypeInfo = TypeInfo {
    label: "makefile",
    mime_type: "text/x-makefile",
    group: "code",
    description: "Makefile source",
    extensions: &[],
    is_text: true,
};

pub(crate) static MARKDOWN: TypeInfo = TypeInfo {
    label: "markdown",
    mime_type: "text/markdown",
    group: "text",
    description: "Markdown document",
    extensions: &["md", "markdown"],
    is_text: true,
};

pub(crate) static MAT: TypeInfo = TypeInfo {
    label: "mat",
    mime_type: "application/octet-stream",
    group: "scientific",
    description: "MATLAB MAT data",
    extensions: &["mat"],
    is_text: false,
};

pub(crate) static MATLAB: TypeInfo = TypeInfo {
    label: "matlab",
    mime_type: "text/x-matlab",
    group: "code",
    description: "Matlab Source",
    extensions: &["m", "matlab"],
    is_text: true,
};

pub(crate) static MHT: TypeInfo = TypeInfo {
    label: "mht",
    mime_type: "application/x-mimearchive",
    group: "code",
    description: "MHTML document",
    extensions: &["mht"],
    is_text: true,
};

pub(crate) static MIDI: TypeInfo = TypeInfo {
    label: "midi",
    mime_type: "audio/midi",
    group: "audio",
    description: "Midi",
    extensions: &["mid"],
    is_text: false,
};

pub(crate) static MKV: TypeInfo = TypeInfo {
    label: "mkv",
    mime_type: "video/x-matroska",
    group: "video",
    description: "Matroska",
    extensions: &["mkv"],
    is_text: false,
};

pub(crate) static MP3: TypeInfo = TypeInfo {
    label: "mp3",
    mime_type: "audio/mpeg",
    group: "audio",
    description: "MP3 media file",
    extensions: &["mp3"],
    is_text: false,
};

pub(crate) static MP4: TypeInfo = TypeInfo {
    label: "mp4",
    mime_type: "video/mp4",
    group: "video",
    description: "MP4 media file",
    extensions: &["mp4"],
    is_text: false,
};

pub(crate) static MPEGTS: TypeInfo = TypeInfo {
    label: "mpegts",
    mime_type: "video/MP2T",
    group: "video",
    description: "MPEG Transport stream",
    extensions: &["ts", "tsv", "tsa", "m2t"],
    is_text: false,
};

pub(crate) static MSCOMPRESS: TypeInfo = TypeInfo {
    label: "mscompress",
    mime_type: "application/x-ms-compress-szdd",
    group: "archive",
    description: "MS Compress archive data",
    extensions: &[],
    is_text: false,
};

pub(crate) static MSI: TypeInfo = TypeInfo {
    label: "msi",
    mime_type: "application/x-msi",
    group: "archive",
    description: "Microsoft Installer file",
    extensions: &["msi"],
    is_text: false,
};

pub(crate) static MUM: TypeInfo = TypeInfo {
    label: "mum",
    mime_type: "text/xml",
    group: "application",
    description: "Windows Update Package file",
    extensions: &["mum"],
    is_text: true,
};

pub(crate) static NETCDF: TypeInfo = TypeInfo {
    label: "netcdf",
    mime_type: "application/octet-stream",
    group: "scientific",
    description: "NetCDF",
    extensions: &["cdf", "nc"],
    is_text: false,
};

pub(crate) static NPY: TypeInfo = TypeInfo {
    label: "npy",
    mime_type: "application/octet-stream",
    group: "archive",
    description: "Numpy Array",
    extensions: &["npy"],
    is_text: false,
};

pub(crate) static NPZ: TypeInfo = TypeInfo {
    label: "npz",
    mime_type: "application/octet-stream",
    group: "archive",
    description: "Numpy Arrays Archive",
    extensions: &["npz"],
    is_text: false,
};

pub(crate) static NUPKG: TypeInfo = TypeInfo {
    label: "nupkg",
    mime_type: "application/octet-stream",
    group: "unknown",
    description: "NuGet Package",
    extensions: &["nupkg"],
    is_text: false,
};

pub(crate) static OBJECTIVEC: TypeInfo = TypeInfo {
    label: "objectivec",
    mime_type: "text/x-objcsrc",
    group: "code",
    description: "ObjectiveC source",
    extensions: &["m", "mm"],
    is_text: true,
};

pub(crate) static OCAML: TypeInfo = TypeInfo {
    label: "ocaml",
    mime_type: "text-ocaml",
    group: "code",
    description: "OCaml",
    extensions: &["ml", "mli"],
    is_text: true,
};

pub(crate) static ODP: TypeInfo = TypeInfo {
    label: "odp",
    mime_type: "application/vnd.oasis.opendocument.presentation",
    group: "document",
    description: "OpenDocument Presentation",
    extensions: &["odp"],
    is_text: false,
};

pub(crate) static ODS: TypeInfo = TypeInfo {
    label: "ods",
    mime_type: "application/vnd.oasis.opendocument.spreadsheet",
    group: "document",
    description: "OpenDocument Spreadsheet",
    extensions: &["ods"],
    is_text: false,
};

pub(crate) static ODT: TypeInfo = TypeInfo {
    label: "odt",
    mime_type: "application/vnd.oasis.opendocument.text",
    group: "document",
    description: "OpenDocument Text",
    extensions: &["odt"],
    is_text: false,
};

pub(crate) static OGG: TypeInfo = TypeInfo {
    label: "ogg",
    mime_type: "audio/ogg",
    group: "audio",
    description: "Ogg data",
    extensions: &["ogg"],
    is_text: false,
};

pub(crate) static ONE: TypeInfo = TypeInfo {
    label: "one",
    mime_type: "application/msonenote",
    group: "document",
    description: "One Note",
    extensions: &["one"],
    is_text: false,
};

pub(crate) static ONNX: TypeInfo = TypeInfo {
    label: "onnx",
    mime_type: "application/octet-stream",
    group: "archive",
    description: "Open Neural Network Exchange",
    extensions: &["onnx"],
    is_text: false,
};

pub(crate) static ORC: TypeInfo = TypeInfo {
    label: "orc",
    mime_type: "application/octet-stream",
    group: "scientific",
    description: "Apache ORC",
    extensions: &["orc"],
    is_text: false,
};

pub(crate) static OTF: TypeInfo = TypeInfo {
    label: "otf",
    mime_type: "font/otf",
    group: "font",
    description: "OpenType font",
    extensions: &["otf"],
    is_text: false,
};

pub(crate) static OUTLOOK: TypeInfo = TypeInfo {
    label: "outlook",
    mime_type: "application/vnd.ms-outlook",
    group: "application",
    description: "MS Outlook Message",
    extensions: &[],
    is_text: false,
};

pub(crate) static PARADOX: TypeInfo = TypeInfo {
    label: "paradox",
    mime_type: "application/octet-stream",
    group: "database",
    description: "Paradox database",
    extensions: &["db"],
    is_text: false,
};

pub(crate) static PARQUET: TypeInfo = TypeInfo {
    label: "parquet",
    mime_type: "application/vnd.apache.parquet",
    group: "unknown",
    description: "Apache Parquet",
    extensions: &["pqt", "parquet"],
    is_text: false,
};

pub(crate) static PASCAL: TypeInfo = TypeInfo {
    label: "pascal",
    mime_type: "text/x-pascal",
    group: "code",
    description: "Pascal source",
    extensions: &["pas", "pp"],
    is_text: true,
};

pub(crate) static PCAP: TypeInfo = TypeInfo {
    label: "pcap",
    mime_type: "application/vnd.tcpdump.pcap",
    group: "application",
    description: "pcap capture file",
    extensions: &["pcap", "pcapng"],
    is_text: false,
};

pub(crate) static PCAPNG: TypeInfo = TypeInfo {
    label: "pcapng",
    mime_type: "application/octet-stream",
    group: "binary",
    description: "Packet capture PCAPNG",
    extensions: &["pcapng"],
    is_text: false,
};

pub(crate) static PDB: TypeInfo = TypeInfo {
    label: "pdb",
    mime_type: "application/octet-stream",
    group: "application",
    description: "Windows Program Database",
    extensions: &["pdb"],
    is_text: false,
};

pub(crate) static PDF: TypeInfo = TypeInfo {
    label: "pdf",
    mime_type: "application/pdf",
    group: "document",
    description: "PDF document",
    extensions: &["pdf"],
    is_text: false,
};

pub(crate) static PEBIN: TypeInfo = TypeInfo {
    label: "pebin",
    mime_type: "application/x-dosexec",
    group: "executable",
    description: "PE Windows executable",
    extensions: &["exe", "dll"],
    is_text: false,
};

pub(crate) static PEM: TypeInfo = TypeInfo {
    label: "pem",
    mime_type: "application/x-pem-file",
    group: "application",
    description: "PEM certificate",
    extensions: &["pem", "pub", "gpg"],
    is_text: true,
};

pub(crate) static PERL: TypeInfo = TypeInfo {
    label: "perl",
    mime_type: "text/x-perl",
    group: "code",
    description: "Perl source",
    extensions: &["pl"],
    is_text: true,
};

pub(crate) static PGP: TypeInfo = TypeInfo {
    label: "pgp",
    mime_type: "application/pgp-keys",
    group: "unknown",
    description: "PGP",
    extensions: &["gpg", "pgp"],
    is_text: false,
};

pub(crate) static PHP: TypeInfo = TypeInfo {
    label: "php",
    mime_type: "text/x-php",
    group: "code",
    description: "PHP source",
    extensions: &["php"],
    is_text: true,
};

pub(crate) static PICKLE: TypeInfo = TypeInfo {
    label: "pickle",
    mime_type: "application/octet-stream",
    group: "application",
    description: "Python pickle",
    extensions: &["pickle", "pkl"],
    is_text: false,
};

pub(crate) static PNG: TypeInfo = TypeInfo {
    label: "png",
    mime_type: "image/png",
    group: "image",
    description: "PNG image",
    extensions: &["png"],
    is_text: false,
};

pub(crate) static PO: TypeInfo = TypeInfo {
    label: "po",
    mime_type: "text/gettext-translation",
    group: "application",
    description: "Portable Object (PO) for i18n",
    extensions: &["po"],
    is_text: true,
};

pub(crate) static POSTGRES_DUMP: TypeInfo = TypeInfo {
    label: "postgres_dump",
    mime_type: "application/octet-stream",
    group: "database",
    description: "PostgreSQL dump",
    extensions: &["backup", "dump", "sql"],
    is_text: false,
};

pub(crate) static POSTSCRIPT: TypeInfo = TypeInfo {
    label: "postscript",
    mime_type: "application/postscript",
    group: "document",
    description: "PostScript document",
    extensions: &["ps"],
    is_text: false,
};

pub(crate) static POWERSHELL: TypeInfo = TypeInfo {
    label: "powershell",
    mime_type: "application/x-powershell",
    group: "code",
    description: "Powershell source",
    extensions: &["ps1"],
    is_text: true,
};

pub(crate) static PPT: TypeInfo = TypeInfo {
    label: "ppt",
    mime_type: "application/vnd.ms-powerpoint",
    group: "document",
    description: "Microsoft PowerPoint CDF document",
    extensions: &["ppt"],
    is_text: false,
};

pub(crate) static PPTX: TypeInfo = TypeInfo {
    label: "pptx",
    mime_type: "application/vnd.openxmlformats-officedocument.presentationml.presentation",
    group: "document",
    description: "Microsoft PowerPoint 2007+ document",
    extensions: &["pptx", "pptm"],
    is_text: false,
};

pub(crate) static PROLOG: TypeInfo = TypeInfo {
    label: "prolog",
    mime_type: "text/x-prolog",
    group: "code",
    description: "Prolog source",
    extensions: &["pl", "pro", "P"],
    is_text: true,
};

pub(crate) static PROTEINDB: TypeInfo = TypeInfo {
    label: "proteindb",
    mime_type: "application/octet-stream",
    group: "application",
    description: "Protein DB",
    extensions: &["pdb"],
    is_text: true,
};

pub(crate) static PROTO: TypeInfo = TypeInfo {
    label: "proto",
    mime_type: "text/x-proto",
    group: "code",
    description: "Protocol buffer definition",
    extensions: &["proto"],
    is_text: true,
};

pub(crate) static PSD: TypeInfo = TypeInfo {
    label: "psd",
    mime_type: "image/vnd.adobe.photoshop",
    group: "image",
    description: "Adobe Photoshop",
    extensions: &["psd"],
    is_text: false,
};

pub(crate) static PUB: TypeInfo = TypeInfo {
    label: "pub",
    mime_type: "application/x-mspublisher",
    group: "unknown",
    description: "pub",
    extensions: &["pub"],
    is_text: false,
};

pub(crate) static PYTHON: TypeInfo = TypeInfo {
    label: "python",
    mime_type: "text/x-python",
    group: "code",
    description: "Python source",
    extensions: &["py", "pyi"],
    is_text: true,
};

pub(crate) static PYTHONBYTECODE: TypeInfo = TypeInfo {
    label: "pythonbytecode",
    mime_type: "application/x-bytecode.python",
    group: "executable",
    description: "Python compiled bytecode",
    extensions: &["pyc", "pyo"],
    is_text: false,
};

pub(crate) static PYTORCH: TypeInfo = TypeInfo {
    label: "pytorch",
    mime_type: "application/octet-stream",
    group: "application",
    description: "Pytorch storage file",
    extensions: &["pt", "pth"],
    is_text: false,
};

pub(crate) static QOI: TypeInfo = TypeInfo {
    label: "qoi",
    mime_type: "image/x-qoi",
    group: "image",
    description: "Quite Ok Image",
    extensions: &["qoi"],
    is_text: false,
};

pub(crate) static QT: TypeInfo = TypeInfo {
    label: "qt",
    mime_type: "video/quicktime",
    group: "video",
    description: "QuickTime",
    extensions: &["mov"],
    is_text: false,
};

pub(crate) static R: TypeInfo = TypeInfo {
    label: "r",
    mime_type: "text/x-R",
    group: "code",
    description: "R (language)",
    extensions: &["R"],
    is_text: true,
};

pub(crate) static RANDOMBYTES: TypeInfo = TypeInfo {
    label: "randombytes",
    mime_type: "application/octet-stream",
    group: "unknown",
    description: "Random bytes",
    extensions: &[],
    is_text: false,
};

pub(crate) static RANDOMTXT: TypeInfo = TypeInfo {
    label: "randomtxt",
    mime_type: "text/plain",
    group: "text",
    description: "Random text",
    extensions: &[],
    is_text: true,
};

pub(crate) static RAR: TypeInfo = TypeInfo {
    label: "rar",
    mime_type: "application/vnd.rar",
    group: "archive",
    description: "RAR archive data",
    extensions: &["rar"],
    is_text: false,
};

pub(crate) static RDATA: TypeInfo = TypeInfo {
    label: "rdata",
    mime_type: "application/octet-stream",
    group: "scientific",
    description: "R serialized data",
    extensions: &["rda", "rdata", "rds"],
    is_text: false,
};

pub(crate) static RDF: TypeInfo = TypeInfo {
    label: "rdf",
    mime_type: "application/rdf+xml",
    group: "text",
    description: "Resource Description Framework document (RDF)",
    extensions: &["rdf"],
    is_text: true,
};

pub(crate) static REDIS_RDB: TypeInfo = TypeInfo {
    label: "redis_rdb",
    mime_type: "application/octet-stream",
    group: "database",
    description: "Redis snapshot",
    extensions: &["rdb"],
    is_text: false,
};

pub(crate) static RHINOCEROS: TypeInfo = TypeInfo {
    label: "rhinoceros",
    mime_type: "application/octet-stream",
    group: "geometry",
    description: "Rhino 3DM",
    extensions: &["3dm"],
    is_text: false,
};

pub(crate) static RPM: TypeInfo = TypeInfo {
    label: "rpm",
    mime_type: "application/x-rpm",
    group: "archive",
    description: "RedHat Package Manager archive (RPM)",
    extensions: &["rpm"],
    is_text: false,
};

pub(crate) static RST: TypeInfo = TypeInfo {
    label: "rst",
    mime_type: "text/x-rst",
    group: "text",
    description: "ReStructuredText document",
    extensions: &["rst"],
    is_text: true,
};

pub(crate) static RTF: TypeInfo = TypeInfo {
    label: "rtf",
    mime_type: "text/rtf",
    group: "text",
    description: "Rich Text Format document",
    extensions: &["rtf"],
    is_text: true,
};

pub(crate) static RUBY: TypeInfo = TypeInfo {
    label: "ruby",
    mime_type: "application/x-ruby",
    group: "code",
    description: "Ruby source",
    extensions: &["rb"],
    is_text: true,
};

pub(crate) static RUST: TypeInfo = TypeInfo {
    label: "rust",
    mime_type: "application/x-rust",
    group: "code",
    description: "Rust source",
    extensions: &["rs"],
    is_text: true,
};

pub(crate) static RZIP: TypeInfo = TypeInfo {
    label: "rzip",
    mime_type: "application/octet-stream",
    group: "unknown",
    description: "Rzip",
    extensions: &["rz"],
    is_text: false,
};

pub(crate) static SAS: TypeInfo = TypeInfo {
    label: "sas",
    mime_type: "application/octet-stream",
    group: "scientific",
    description: "SAS dataset / transport",
    extensions: &["sas7bdat", "xpt"],
    is_text: false,
};

pub(crate) static SCALA: TypeInfo = TypeInfo {
    label: "scala",
    mime_type: "application/x-scala",
    group: "code",
    description: "Scala source",
    extensions: &["scala"],
    is_text: true,
};

pub(crate) static SCSS: TypeInfo = TypeInfo {
    label: "scss",
    mime_type: "text/x-scss",
    group: "code",
    description: "SCSS source",
    extensions: &["scss"],
    is_text: true,
};

pub(crate) static SEVENZIP: TypeInfo = TypeInfo {
    label: "sevenzip",
    mime_type: "application/x-7z-compressed",
    group: "archive",
    description: "7-zip archive data",
    extensions: &["7z"],
    is_text: false,
};

pub(crate) static SGML: TypeInfo = TypeInfo {
    label: "sgml",
    mime_type: "application/sgml",
    group: "text",
    description: "sgml",
    extensions: &["sgml"],
    is_text: true,
};

pub(crate) static SHAPEFILE: TypeInfo = TypeInfo {
    label: "shapefile",
    mime_type: "application/octet-stream",
    group: "gis",
    description: "ESRI Shapefile",
    extensions: &["dbf", "prj", "shp", "shx"],
    is_text: false,
};

pub(crate) static SHELL: TypeInfo = TypeInfo {
    label: "shell",
    mime_type: "text/x-shellscript",
    group: "code",
    description: "Shell script",
    extensions: &["sh"],
    is_text: true,
};

pub(crate) static SKETCHUP: TypeInfo = TypeInfo {
    label: "sketchup",
    mime_type: "application/octet-stream",
    group: "geometry",
    description: "SketchUp model",
    extensions: &["skp"],
    is_text: false,
};

pub(crate) static SMALI: TypeInfo = TypeInfo {
    label: "smali",
    mime_type: "application/x-smali",
    group: "code",
    description: "Smali source",
    extensions: &["smali"],
    is_text: true,
};

pub(crate) static SNAP: TypeInfo = TypeInfo {
    label: "snap",
    mime_type: "application/octet-stream",
    group: "archive",
    description: "Snap archive",
    extensions: &["snap"],
    is_text: false,
};

pub(crate) static SOLIDITY: TypeInfo = TypeInfo {
    label: "solidity",
    mime_type: "text/plain",
    group: "code",
    description: "Solidity source",
    extensions: &["sol"],
    is_text: true,
};

pub(crate) static SPIRV: TypeInfo = TypeInfo {
    label: "spirv",
    mime_type: "application/octet-stream",
    group: "executable",
    description: "SPIR-V",
    extensions: &["spv"],
    is_text: false,
};

pub(crate) static SPSS: TypeInfo = TypeInfo {
    label: "spss",
    mime_type: "application/octet-stream",
    group: "scientific",
    description: "SPSS dataset",
    extensions: &["por", "sav", "zsav"],
    is_text: false,
};

pub(crate) static SQL: TypeInfo = TypeInfo {
    label: "sql",
    mime_type: "application/x-sql",
    group: "code",
    description: "SQL source",
    extensions: &["sql"],
    is_text: true,
};

pub(crate) static SQLITE: TypeInfo = TypeInfo {
    label: "sqlite",
    mime_type: "application/octet-stream",
    group: "application",
    description: "SQLITE database",
    extensions: &["sqlite", "sqlite3"],
    is_text: false,
};

pub(crate) static SQUASHFS: TypeInfo = TypeInfo {
    label: "squashfs",
    mime_type: "application/octet-stream",
    group: "archive",
    description: "Squash filesystem",
    extensions: &[],
    is_text: false,
};

pub(crate) static SRT: TypeInfo = TypeInfo {
    label: "srt",
    mime_type: "text/srt",
    group: "application",
    description: "SubRip Text Format",
    extensions: &["srt"],
    is_text: true,
};

pub(crate) static STATA: TypeInfo = TypeInfo {
    label: "stata",
    mime_type: "application/octet-stream",
    group: "scientific",
    description: "Stata dataset",
    extensions: &["dta"],
    is_text: false,
};

pub(crate) static STLBINARY: TypeInfo = TypeInfo {
    label: "stlbinary",
    mime_type: "application/sla",
    group: "image",
    description: "Stereolithography CAD (binary)",
    extensions: &["stl"],
    is_text: false,
};

pub(crate) static STLTEXT: TypeInfo = TypeInfo {
    label: "stltext",
    mime_type: "application/sla",
    group: "image",
    description: "Stereolithography CAD (text)",
    extensions: &["stl"],
    is_text: true,
};

pub(crate) static SUM: TypeInfo = TypeInfo {
    label: "sum",
    mime_type: "text/plain",
    group: "application",
    description: "Checksum file",
    extensions: &["sum"],
    is_text: true,
};

pub(crate) static SVG: TypeInfo = TypeInfo {
    label: "svg",
    mime_type: "image/svg+xml",
    group: "image",
    description: "SVG Scalable Vector Graphics image data",
    extensions: &["svg"],
    is_text: true,
};

pub(crate) static SWF: TypeInfo = TypeInfo {
    label: "swf",
    mime_type: "application/x-shockwave-flash",
    group: "executable",
    description: "Small Web File",
    extensions: &["swf"],
    is_text: false,
};

pub(crate) static SWIFT: TypeInfo = TypeInfo {
    label: "swift",
    mime_type: "text/x-swift",
    group: "code",
    description: "Swift",
    extensions: &["swift"],
    is_text: true,
};

pub(crate) static SYMLINK: TypeInfo = TypeInfo {
    label: "symlink",
    mime_type: "inode/symlink",
    group: "inode",
    description: "Symbolic link",
    extensions: &[],
    is_text: false,
};

pub(crate) static TAR: TypeInfo = TypeInfo {
    label: "tar",
    mime_type: "application/x-tar",
    group: "archive",
    description: "POSIX tar archive",
    extensions: &["tar"],
    is_text: false,
};

pub(crate) static TCL: TypeInfo = TypeInfo {
    label: "tcl",
    mime_type: "application/x-tcl",
    group: "code",
    description: "Tickle",
    extensions: &["tcl"],
    is_text: true,
};

pub(crate) static TEXTPROTO: TypeInfo = TypeInfo {
    label: "textproto",
    mime_type: "text/plain",
    group: "code",
    description: "Text protocol buffer",
    extensions: &["textproto", "textpb", "pbtxt"],
    is_text: true,
};

pub(crate) static TGA: TypeInfo = TypeInfo {
    label: "tga",
    mime_type: "image/x-tga",
    group: "image",
    description: "Targa image data",
    extensions: &["tga"],
    is_text: false,
};

pub(crate) static THUMBSDB: TypeInfo = TypeInfo {
    label: "thumbsdb",
    mime_type: "image/vnd.ms-thumb",
    group: "application",
    description: "Windows thumbnail cache",
    extensions: &[],
    is_text: false,
};

pub(crate) static TIFF: TypeInfo = TypeInfo {
    label: "tiff",
    mime_type: "image/tiff",
    group: "image",
    description: "TIFF image data",
    extensions: &["tiff", "tif"],
    is_text: false,
};

pub(crate) static TOML: TypeInfo = TypeInfo {
    label: "toml",
    mime_type: "application/toml",
    group: "text",
    description: "Tom's obvious, minimal language",
    extensions: &["toml"],
    is_text: true,
};

pub(crate) static TORRENT: TypeInfo = TypeInfo {
    label: "torrent",
    mime_type: "application/x-bittorrent",
    group: "application",
    description: "BitTorrent file",
    extensions: &["torrent"],
    is_text: false,
};

pub(crate) static TSV: TypeInfo = TypeInfo {
    label: "tsv",
    mime_type: "text/tsv",
    group: "code",
    description: "TSV document",
    extensions: &["tsv"],
    is_text: true,
};

pub(crate) static TTF: TypeInfo = TypeInfo {
    label: "ttf",
    mime_type: "font/sfnt",
    group: "font",
    description: "TrueType Font data",
    extensions: &["ttf", "ttc"],
    is_text: false,
};

pub(crate) static TWIG: TypeInfo = TypeInfo {
    label: "twig",
    mime_type: "text/x-twig",
    group: "code",
    description: "Twig template",
    extensions: &["twig"],
    is_text: true,
};

pub(crate) static TXT: TypeInfo = TypeInfo {
    label: "txt",
    mime_type: "text/plain",
    group: "text",
    description: "Generic text document",
    extensions: &["txt"],
    is_text: true,
};

pub(crate) static TYPESCRIPT: TypeInfo = TypeInfo {
    label: "typescript",
    mime_type: "application/typescript",
    group: "code",
    description: "TypeScript source",
    extensions: &["ts", "mts", "cts"],
    is_text: true,
};

pub(crate) static UF2: TypeInfo = TypeInfo {
    label: "uf2",
    mime_type: "application/octet-stream",
    group: "executable",
    description: "UF2 firmware",
    extensions: &["uf2"],
    is_text: false,
};

pub(crate) static UNDEFINED: TypeInfo = TypeInfo {
    label: "undefined",
    mime_type: "application/undefined",
    group: "undefined",
    description: "Undefined",
    extensions: &[],
    is_text: false,
};

pub(crate) static UNIXCOMPRESS: TypeInfo = TypeInfo {
    label: "unixcompress",
    mime_type: "application/x-compress",
    group: "unknown",
    description: "unixcompress",
    extensions: &["z"],
    is_text: false,
};

pub(crate) static UNKNOWN: TypeInfo = TypeInfo {
    label: "unknown",
    mime_type: "application/octet-stream",
    group: "unknown",
    description: "Unknown binary data",
    extensions: &[],
    is_text: false,
};

pub(crate) static VBA: TypeInfo = TypeInfo {
    label: "vba",
    mime_type: "text/vbscript",
    group: "code",
    description: "MS Visual Basic source (VBA)",
    extensions: &["vbs", "vba", "vb"],
    is_text: true,
};

pub(crate) static VCXPROJ: TypeInfo = TypeInfo {
    label: "vcxproj",
    mime_type: "application/xml",
    group: "code",
    description: "Visual Studio MSBuild project",
    extensions: &["vcxproj"],
    is_text: true,
};

pub(crate) static VERILOG: TypeInfo = TypeInfo {
    label: "verilog",
    mime_type: "text/x-verilog",
    group: "code",
    description: "Verilog source",
    extensions: &["v", "verilog", "vlg", "vh"],
    is_text: true,
};

pub(crate) static VHD: TypeInfo = TypeInfo {
    label: "vhd",
    mime_type: "application/x-vhd",
    group: "unknown",
    description: "Virtual Hard Disk",
    extensions: &[],
    is_text: false,
};

pub(crate) static VHDL: TypeInfo = TypeInfo {
    label: "vhdl",
    mime_type: "text/x-vhdl",
    group: "code",
    description: "VHDL source",
    extensions: &["vhd"],
    is_text: true,
};

pub(crate) static VTT: TypeInfo = TypeInfo {
    label: "vtt",
    mime_type: "text/vtt",
    group: "text",
    description: "Web Video Text Tracks",
    extensions: &["vtt", "webvtt"],
    is_text: true,
};

pub(crate) static VUE: TypeInfo = TypeInfo {
    label: "vue",
    mime_type: "application/javascript",
    group: "code",
    description: "Vue source",
    extensions: &["vue"],
    is_text: true,
};

pub(crate) static WAD: TypeInfo = TypeInfo {
    label: "wad",
    mime_type: "application/wad",
    group: "archive",
    description: "WAD",
    extensions: &["wad"],
    is_text: false,
};

pub(crate) static WASM: TypeInfo = TypeInfo {
    label: "wasm",
    mime_type: "application/wasm",
    group: "executable",
    description: "Web Assembly",
    extensions: &["wasm"],
    is_text: false,
};

pub(crate) static WAV: TypeInfo = TypeInfo {
    label: "wav",
    mime_type: "audio/x-wav",
    group: "audio",
    description: "Waveform Audio file (WAV)",
    extensions: &["wav"],
    is_text: false,
};

pub(crate) static WEBM: TypeInfo = TypeInfo {
    label: "webm",
    mime_type: "video/webm",
    group: "video",
    description: "WebM media file",
    extensions: &["webm"],
    is_text: false,
};

pub(crate) static WEBP: TypeInfo = TypeInfo {
    label: "webp",
    mime_type: "image/webp",
    group: "image",
    description: "WebP media file",
    extensions: &["webp"],
    is_text: false,
};

pub(crate) static WIM: TypeInfo = TypeInfo {
    label: "wim",
    mime_type: "application/x-ms-wim",
    group: "unknown",
    description: "Windows Imaging Format",
    extensions: &["wim", "swm", "esd"],
    is_text: false,
};

pub(crate) static WINREGISTRY: TypeInfo = TypeInfo {
    label: "winregistry",
    mime_type: "text/x-ms-regedit",
    group: "application",
    description: "Windows Registry text",
    extensions: &["reg"],
    is_text: true,
};

pub(crate) static WMA: TypeInfo = TypeInfo {
    label: "wma",
    mime_type: "audio/x-ms-wma",
    group: "audio",
    description: "Windows Media Audio",
    extensions: &["wma"],
    is_text: false,
};

pub(crate) static WMF: TypeInfo = TypeInfo {
    label: "wmf",
    mime_type: "image/wmf",
    group: "image",
    description: "Windows metafile",
    extensions: &["wmf"],
    is_text: false,
};

pub(crate) static WMV: TypeInfo = TypeInfo {
    label: "wmv",
    mime_type: "video/x-ms-wmv",
    group: "video",
    description: "Windows Media Video",
    extensions: &["wmv"],
    is_text: false,
};

pub(crate) static WOFF: TypeInfo = TypeInfo {
    label: "woff",
    mime_type: "font/woff",
    group: "font",
    description: "Web Open Font Format",
    extensions: &["woff"],
    is_text: false,
};

pub(crate) static WOFF2: TypeInfo = TypeInfo {
    label: "woff2",
    mime_type: "font/woff2",
    group: "font",
    description: "Web Open Font Format v2",
    extensions: &["woff2"],
    is_text: false,
};

pub(crate) static XAR: TypeInfo = TypeInfo {
    label: "xar",
    mime_type: "application/x-xar",
    group: "archive",
    description: "XAR archive compressed data",
    extensions: &["pkg", "xar"],
    is_text: false,
};

pub(crate) static XCF: TypeInfo = TypeInfo {
    label: "xcf",
    mime_type: "image/x-xcf",
    group: "image",
    description: "Gimp image",
    extensions: &["xcf"],
    is_text: false,
};

pub(crate) static XCOFF: TypeInfo = TypeInfo {
    label: "xcoff",
    mime_type: "application/octet-stream",
    group: "executable",
    description: "XCOFF",
    extensions: &["o"],
    is_text: false,
};

pub(crate) static XLS: TypeInfo = TypeInfo {
    label: "xls",
    mime_type: "application/vnd.ms-excel",
    group: "document",
    description: "Microsoft Excel CDF document",
    extensions: &["xls"],
    is_text: false,
};

pub(crate) static XLSB: TypeInfo = TypeInfo {
    label: "xlsb",
    mime_type: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    group: "document",
    description: "Microsoft Excel 2007+ document (binary format)",
    extensions: &["xlsb"],
    is_text: false,
};

pub(crate) static XLSX: TypeInfo = TypeInfo {
    label: "xlsx",
    mime_type: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    group: "document",
    description: "Microsoft Excel 2007+ document",
    extensions: &["xlsx", "xlsm"],
    is_text: false,
};

pub(crate) static XML: TypeInfo = TypeInfo {
    label: "xml",
    mime_type: "text/xml",
    group: "code",
    description: "XML document",
    extensions: &["xml"],
    is_text: true,
};

pub(crate) static XPI: TypeInfo = TypeInfo {
    label: "xpi",
    mime_type: "application/zip",
    group: "archive",
    description: "Compressed installation archive (XPI)",
    extensions: &["xpi"],
    is_text: false,
};

pub(crate) static XZ: TypeInfo = TypeInfo {
    label: "xz",
    mime_type: "application/x-xz",
    group: "archive",
    description: "XZ compressed data",
    extensions: &["xz"],
    is_text: false,
};

pub(crate) static YAML: TypeInfo = TypeInfo {
    label: "yaml",
    mime_type: "application/x-yaml",
    group: "code",
    description: "YAML source",
    extensions: &["yml", "yaml"],
    is_text: true,
};

pub(crate) static YARA: TypeInfo = TypeInfo {
    label: "yara",
    mime_type: "text/x-yara",
    group: "code",
    description: "YARA rule",
    extensions: &["yar", "yara"],
    is_text: true,
};

pub(crate) static ZIG: TypeInfo = TypeInfo {
    label: "zig",
    mime_type: "text/zig",
    group: "code",
    description: "Zig source",
    extensions: &["zig"],
    is_text: true,
};

pub(crate) static ZIP: TypeInfo = TypeInfo {
    label: "zip",
    mime_type: "application/zip",
    group: "archive",
    description: "Zip archive data",
    extensions: &["zip"],
    is_text: false,
};

pub(crate) static ZLIBSTREAM: TypeInfo = TypeInfo {
    label: "zlibstream",
    mime_type: "application/zlib",
    group: "application",
    description: "zlib compressed data",
    extensions: &[],
    is_text: false,
};

pub(crate) static ZST: TypeInfo = TypeInfo {
    label: "zst",
    mime_type: "application/zstd",
    group: "archive",
    description: "Zstandard",
    extensions: &["zst"],
    is_text: false,
};

/// Content types for regular files.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ContentType {
    /// 3D studio Max
    _3dsm,
    /// Nintendo 3DS homebrew
    _3dsx,
    /// 3GPP multimedia file
    _3gp,
    /// Microsoft Access database
    Access,
    /// ACE archive
    Ace,
    /// Adobe Illustrator Artwork
    Ai,
    /// Android Interface Definition Language
    Aidl,
    /// Animated cursor
    Ani,
    /// a.out object / executable
    Aout,
    /// Android package
    Apk,
    /// Apple binary property list
    Applebplist,
    /// AppleDouble
    Appledouble,
    /// Apple property list
    Appleplist,
    /// AppleSingle
    Applesingle,
    /// Arc
    Arc,
    /// Arj
    Arj,
    /// arrow
    Arrow,
    /// Microsoft Advanced Systems Format
    Asf,
    /// Assembly
    Asm,
    /// ASP source
    Asp,
    /// NeXT/Sun AU
    Au,
    /// AutoHotKey script
    Autohotkey,
    /// AutoIt script
    Autoit,
    /// Audio Video Interleave
    Avi,
    /// AV1 Image File Format
    Avif,
    /// Apache Avro binary
    Avro,
    /// Awk
    Awk,
    /// BAM alignment
    Bam,
    /// DOS batch file
    Batch,
    /// Bazel build file
    Bazel,
    /// Erlang BEAM
    Beam,
    /// Berkeley DB
    Berkeleydb,
    /// BibTeX
    Bib,
    /// Blender scene
    Blend,
    /// BMP image data
    Bmp,
    /// BPG
    Bpg,
    /// bzip2 compressed data
    Bzip,
    /// bzip3 compressed data
    Bzip3,
    /// C source
    C,
    /// Microsoft Cabinet archive data
    Cab,
    /// Windows Catalog file
    Cat,
    /// MS Windows HtmlHelp Data
    Chm,
    /// Cinema 4D scene
    Cinema4d,
    /// Clojure
    Clojure,
    /// CMake build file
    Cmake,
    /// Cobol
    Cobol,
    /// Intel 80386 COFF
    Coff,
    /// CoffeeScript
    Coffeescript,
    /// C++ source
    Cpp,
    /// CRAM alignment
    Cram,
    /// Certificates (binary format)
    Crt,
    /// Google Chrome extension
    Crx,
    /// C# source
    Cs,
    /// .NET project config
    Csproj,
    /// CSS source
    Css,
    /// CSV document
    Csv,
    /// Dart source
    Dart,
    /// dBASE / FoxPro table
    Dbase,
    /// Debian binary package
    Deb,
    /// Dalvik dex file
    Dex,
    /// DICOM
    Dicom,
    /// Diff file
    Diff,
    /// Dream Maker
    Dm,
    /// Apple disk image
    Dmg,
    /// Microsoft Word CDF document
    Doc,
    /// Dockerfile
    Dockerfile,
    /// Microsoft Word 2007+ document
    Docx,
    /// Office Word 2007 template
    Dotx,
    /// Application Desktop Services Store
    Dsstore,
    /// DuckDB database
    Duckdb,
    /// Autocad Drawing
    Dwg,
    /// Audocad Drawing Exchange Format
    Dxf,
    /// ELF executable
    Elf,
    /// Elixir script
    Elixir,
    /// Windows Enhanced Metafile image data
    Emf,
    /// RFC 822 mail
    Eml,
    /// Empty file
    Empty,
    /// EPUB document
    Epub,
    /// Embedded Ruby source
    Erb,
    /// Erlang source
    Erlang,
    /// ESE Db
    Ese,
    /// Autodesk FBX
    Fbx,
    /// FileMaker database
    Filemaker,
    /// FITS astronomy data
    Fits,
    /// FLAC audio bitstream data
    Flac,
    /// FlatGeobuf
    Flatgeobuf,
    /// Flash Video
    Flv,
    /// Fortran
    Fortran,
    /// Gemfile file
    Gemfile,
    /// Gemspec file
    Gemspec,
    /// GGUF
    Gguf,
    /// GIF image data
    Gif,
    /// Gitattributes file
    Gitattributes,
    /// Gitmodules file
    Gitmodules,
    /// glTF
    Gltf,
    /// Golang source
    Go,
    /// Gradle source
    Gradle,
    /// Groovy source
    Groovy,
    /// gzip compressed data
    Gzip,
    /// Hierarchical Data Format v5
    H5,
    /// Handlebars source
    Handlebars,
    /// Haskell source
    Haskell,
    /// HashiCorp configuration language
    Hcl,
    /// HDF4
    Hdf4,
    /// High Efficiency Image File
    Heif,
    /// MS Windows help
    Hlp,
    /// Apache access configuration
    Htaccess,
    /// HTML document
    Html,
    /// Hangul Word Processor
    Hwp,
    /// ICC profile
    Icc,
    /// Mac OS X icon
    Icns,
    /// MS Windows icon resource
    Ico,
    /// Internet Calendaring and Scheduling
    Ics,
    /// Ignorefile
    Ignorefile,
    /// INI configuration file
    Ini,
    /// MS Windows Internet shortcut
    Internetshortcut,
    /// Jupyter notebook
    Ipynb,
    /// ISO 9660 CD-ROM filesystem data
    Iso,
    /// Java archive data (JAR)
    Jar,
    /// Java source
    Java,
    /// Java compiled bytecode
    Javabytecode,
    /// JavaScript source
    Javascript,
    /// Jinja template
    Jinja,
    /// jpeg2000
    Jp2,
    /// JPEG image data
    Jpeg,
    /// JSON document
    Json,
    /// JSONL document
    Jsonl,
    /// Julia source
    Julia,
    /// JPEG XL
    Jxl,
    /// Kotlin source
    Kotlin,
    /// LaTeX document
    Latex,
    /// LHarc archive
    Lha,
    /// LightWave object / scene
    Lightwave,
    /// Lisp source
    Lisp,
    /// LLVM bitcode
    LlvmBitcode,
    /// LMDB
    Lmdb,
    /// MS Windows shortcut
    Lnk,
    /// LRZip
    Lrz,
    /// Lua
    Lua,
    /// Lua bytecode
    Luabytecode,
    /// LZip
    Lz,
    /// LZ4
    Lz4,
    /// lzx
    Lzx,
    /// M3U playlist
    M3u,
    /// GNU Macro
    M4,
    /// Mach-O executable
    Macho,
    /// Makefile source
    Makefile,
    /// Markdown document
    Markdown,
    /// MATLAB MAT data
    Mat,
    /// Matlab Source
    Matlab,
    /// MHTML document
    Mht,
    /// Midi
    Midi,
    /// Matroska
    Mkv,
    /// MP3 media file
    Mp3,
    /// MP4 media file
    Mp4,
    /// MPEG Transport stream
    Mpegts,
    /// MS Compress archive data
    Mscompress,
    /// Microsoft Installer file
    Msi,
    /// Windows Update Package file
    Mum,
    /// NetCDF
    Netcdf,
    /// Numpy Array
    Npy,
    /// Numpy Arrays Archive
    Npz,
    /// NuGet Package
    Nupkg,
    /// ObjectiveC source
    Objectivec,
    /// OCaml
    Ocaml,
    /// OpenDocument Presentation
    Odp,
    /// OpenDocument Spreadsheet
    Ods,
    /// OpenDocument Text
    Odt,
    /// Ogg data
    Ogg,
    /// One Note
    One,
    /// Open Neural Network Exchange
    Onnx,
    /// Apache ORC
    Orc,
    /// OpenType font
    Otf,
    /// MS Outlook Message
    Outlook,
    /// Paradox database
    Paradox,
    /// Apache Parquet
    Parquet,
    /// Pascal source
    Pascal,
    /// pcap capture file
    Pcap,
    /// Packet capture PCAPNG
    Pcapng,
    /// Windows Program Database
    Pdb,
    /// PDF document
    Pdf,
    /// PE Windows executable
    Pebin,
    /// PEM certificate
    Pem,
    /// Perl source
    Perl,
    /// PGP
    Pgp,
    /// PHP source
    Php,
    /// Python pickle
    Pickle,
    /// PNG image
    Png,
    /// Portable Object (PO) for i18n
    Po,
    /// PostgreSQL dump
    PostgresDump,
    /// PostScript document
    Postscript,
    /// Powershell source
    Powershell,
    /// Microsoft PowerPoint CDF document
    Ppt,
    /// Microsoft PowerPoint 2007+ document
    Pptx,
    /// Prolog source
    Prolog,
    /// Protein DB
    Proteindb,
    /// Protocol buffer definition
    Proto,
    /// Adobe Photoshop
    Psd,
    /// pub
    Pub,
    /// Python source
    Python,
    /// Python compiled bytecode
    Pythonbytecode,
    /// Pytorch storage file
    Pytorch,
    /// Quite Ok Image
    Qoi,
    /// QuickTime
    Qt,
    /// R (language)
    R,
    /// Random bytes
    Randombytes,
    /// Random text
    Randomtxt,
    /// RAR archive data
    Rar,
    /// R serialized data
    Rdata,
    /// Resource Description Framework document (RDF)
    Rdf,
    /// Redis snapshot
    RedisRdb,
    /// Rhino 3DM
    Rhinoceros,
    /// RedHat Package Manager archive (RPM)
    Rpm,
    /// ReStructuredText document
    Rst,
    /// Rich Text Format document
    Rtf,
    /// Ruby source
    Ruby,
    /// Rust source
    Rust,
    /// Rzip
    Rzip,
    /// SAS dataset / transport
    Sas,
    /// Scala source
    Scala,
    /// SCSS source
    Scss,
    /// 7-zip archive data
    Sevenzip,
    /// sgml
    Sgml,
    /// ESRI Shapefile
    Shapefile,
    /// Shell script
    Shell,
    /// SketchUp model
    Sketchup,
    /// Smali source
    Smali,
    /// Snap archive
    Snap,
    /// Solidity source
    Solidity,
    /// SPIR-V
    Spirv,
    /// SPSS dataset
    Spss,
    /// SQL source
    Sql,
    /// SQLITE database
    Sqlite,
    /// Squash filesystem
    Squashfs,
    /// SubRip Text Format
    Srt,
    /// Stata dataset
    Stata,
    /// Stereolithography CAD (binary)
    Stlbinary,
    /// Stereolithography CAD (text)
    Stltext,
    /// Checksum file
    Sum,
    /// SVG Scalable Vector Graphics image data
    Svg,
    /// Small Web File
    Swf,
    /// Swift
    Swift,
    /// POSIX tar archive
    Tar,
    /// Tickle
    Tcl,
    /// Text protocol buffer
    Textproto,
    /// Targa image data
    Tga,
    /// Windows thumbnail cache
    Thumbsdb,
    /// TIFF image data
    Tiff,
    /// Tom's obvious, minimal language
    Toml,
    /// BitTorrent file
    Torrent,
    /// TSV document
    Tsv,
    /// TrueType Font data
    Ttf,
    /// Twig template
    Twig,
    /// Generic text document
    Txt,
    /// TypeScript source
    Typescript,
    /// UF2 firmware
    Uf2,
    /// Undefined
    Undefined,
    /// unixcompress
    Unixcompress,
    /// Unknown binary data
    Unknown,
    /// MS Visual Basic source (VBA)
    Vba,
    /// Visual Studio MSBuild project
    Vcxproj,
    /// Verilog source
    Verilog,
    /// Virtual Hard Disk
    Vhd,
    /// VHDL source
    Vhdl,
    /// Web Video Text Tracks
    Vtt,
    /// Vue source
    Vue,
    /// WAD
    Wad,
    /// Web Assembly
    Wasm,
    /// Waveform Audio file (WAV)
    Wav,
    /// WebM media file
    Webm,
    /// WebP media file
    Webp,
    /// Windows Imaging Format
    Wim,
    /// Windows Registry text
    Winregistry,
    /// Windows Media Audio
    Wma,
    /// Windows metafile
    Wmf,
    /// Windows Media Video
    Wmv,
    /// Web Open Font Format
    Woff,
    /// Web Open Font Format v2
    Woff2,
    /// XAR archive compressed data
    Xar,
    /// Gimp image
    Xcf,
    /// XCOFF
    Xcoff,
    /// Microsoft Excel CDF document
    Xls,
    /// Microsoft Excel 2007+ document (binary format)
    Xlsb,
    /// Microsoft Excel 2007+ document
    Xlsx,
    /// XML document
    Xml,
    /// Compressed installation archive (XPI)
    Xpi,
    /// XZ compressed data
    Xz,
    /// YAML source
    Yaml,
    /// YARA rule
    Yara,
    /// Zig source
    Zig,
    /// Zip archive data
    Zip,
    /// zlib compressed data
    Zlibstream,
    /// Zstandard
    Zst,
}

impl ContentType {
    pub(crate) const SIZE: usize = 293;

    /// Looks up an exact, canonical Magika label.
    pub fn from_label(label: &str) -> Option<Self> {
        Some(match label {
            "3dsm" => Self::_3dsm,
            "3dsx" => Self::_3dsx,
            "3gp" => Self::_3gp,
            "access" => Self::Access,
            "ace" => Self::Ace,
            "ai" => Self::Ai,
            "aidl" => Self::Aidl,
            "ani" => Self::Ani,
            "aout" => Self::Aout,
            "apk" => Self::Apk,
            "applebplist" => Self::Applebplist,
            "appledouble" => Self::Appledouble,
            "appleplist" => Self::Appleplist,
            "applesingle" => Self::Applesingle,
            "arc" => Self::Arc,
            "arj" => Self::Arj,
            "arrow" => Self::Arrow,
            "asf" => Self::Asf,
            "asm" => Self::Asm,
            "asp" => Self::Asp,
            "au" => Self::Au,
            "autohotkey" => Self::Autohotkey,
            "autoit" => Self::Autoit,
            "avi" => Self::Avi,
            "avif" => Self::Avif,
            "avro" => Self::Avro,
            "awk" => Self::Awk,
            "bam" => Self::Bam,
            "batch" => Self::Batch,
            "bazel" => Self::Bazel,
            "beam" => Self::Beam,
            "berkeleydb" => Self::Berkeleydb,
            "bib" => Self::Bib,
            "blend" => Self::Blend,
            "bmp" => Self::Bmp,
            "bpg" => Self::Bpg,
            "bzip" => Self::Bzip,
            "bzip3" => Self::Bzip3,
            "c" => Self::C,
            "cab" => Self::Cab,
            "cat" => Self::Cat,
            "chm" => Self::Chm,
            "cinema4d" => Self::Cinema4d,
            "clojure" => Self::Clojure,
            "cmake" => Self::Cmake,
            "cobol" => Self::Cobol,
            "coff" => Self::Coff,
            "coffeescript" => Self::Coffeescript,
            "cpp" => Self::Cpp,
            "cram" => Self::Cram,
            "crt" => Self::Crt,
            "crx" => Self::Crx,
            "cs" => Self::Cs,
            "csproj" => Self::Csproj,
            "css" => Self::Css,
            "csv" => Self::Csv,
            "dart" => Self::Dart,
            "dbase" => Self::Dbase,
            "deb" => Self::Deb,
            "dex" => Self::Dex,
            "dicom" => Self::Dicom,
            "diff" => Self::Diff,
            "dm" => Self::Dm,
            "dmg" => Self::Dmg,
            "doc" => Self::Doc,
            "dockerfile" => Self::Dockerfile,
            "docx" => Self::Docx,
            "dotx" => Self::Dotx,
            "dsstore" => Self::Dsstore,
            "duckdb" => Self::Duckdb,
            "dwg" => Self::Dwg,
            "dxf" => Self::Dxf,
            "elf" => Self::Elf,
            "elixir" => Self::Elixir,
            "emf" => Self::Emf,
            "eml" => Self::Eml,
            "empty" => Self::Empty,
            "epub" => Self::Epub,
            "erb" => Self::Erb,
            "erlang" => Self::Erlang,
            "ese" => Self::Ese,
            "fbx" => Self::Fbx,
            "filemaker" => Self::Filemaker,
            "fits" => Self::Fits,
            "flac" => Self::Flac,
            "flatgeobuf" => Self::Flatgeobuf,
            "flv" => Self::Flv,
            "fortran" => Self::Fortran,
            "gemfile" => Self::Gemfile,
            "gemspec" => Self::Gemspec,
            "gguf" => Self::Gguf,
            "gif" => Self::Gif,
            "gitattributes" => Self::Gitattributes,
            "gitmodules" => Self::Gitmodules,
            "gltf" => Self::Gltf,
            "go" => Self::Go,
            "gradle" => Self::Gradle,
            "groovy" => Self::Groovy,
            "gzip" => Self::Gzip,
            "h5" => Self::H5,
            "handlebars" => Self::Handlebars,
            "haskell" => Self::Haskell,
            "hcl" => Self::Hcl,
            "hdf4" => Self::Hdf4,
            "heif" => Self::Heif,
            "hlp" => Self::Hlp,
            "htaccess" => Self::Htaccess,
            "html" => Self::Html,
            "hwp" => Self::Hwp,
            "icc" => Self::Icc,
            "icns" => Self::Icns,
            "ico" => Self::Ico,
            "ics" => Self::Ics,
            "ignorefile" => Self::Ignorefile,
            "ini" => Self::Ini,
            "internetshortcut" => Self::Internetshortcut,
            "ipynb" => Self::Ipynb,
            "iso" => Self::Iso,
            "jar" => Self::Jar,
            "java" => Self::Java,
            "javabytecode" => Self::Javabytecode,
            "javascript" => Self::Javascript,
            "jinja" => Self::Jinja,
            "jp2" => Self::Jp2,
            "jpeg" => Self::Jpeg,
            "json" => Self::Json,
            "jsonl" => Self::Jsonl,
            "julia" => Self::Julia,
            "jxl" => Self::Jxl,
            "kotlin" => Self::Kotlin,
            "latex" => Self::Latex,
            "lha" => Self::Lha,
            "lightwave" => Self::Lightwave,
            "lisp" => Self::Lisp,
            "llvm_bitcode" => Self::LlvmBitcode,
            "lmdb" => Self::Lmdb,
            "lnk" => Self::Lnk,
            "lrz" => Self::Lrz,
            "lua" => Self::Lua,
            "luabytecode" => Self::Luabytecode,
            "lz" => Self::Lz,
            "lz4" => Self::Lz4,
            "lzx" => Self::Lzx,
            "m3u" => Self::M3u,
            "m4" => Self::M4,
            "macho" => Self::Macho,
            "makefile" => Self::Makefile,
            "markdown" => Self::Markdown,
            "mat" => Self::Mat,
            "matlab" => Self::Matlab,
            "mht" => Self::Mht,
            "midi" => Self::Midi,
            "mkv" => Self::Mkv,
            "mp3" => Self::Mp3,
            "mp4" => Self::Mp4,
            "mpegts" => Self::Mpegts,
            "mscompress" => Self::Mscompress,
            "msi" => Self::Msi,
            "mum" => Self::Mum,
            "netcdf" => Self::Netcdf,
            "npy" => Self::Npy,
            "npz" => Self::Npz,
            "nupkg" => Self::Nupkg,
            "objectivec" => Self::Objectivec,
            "ocaml" => Self::Ocaml,
            "odp" => Self::Odp,
            "ods" => Self::Ods,
            "odt" => Self::Odt,
            "ogg" => Self::Ogg,
            "one" => Self::One,
            "onnx" => Self::Onnx,
            "orc" => Self::Orc,
            "otf" => Self::Otf,
            "outlook" => Self::Outlook,
            "paradox" => Self::Paradox,
            "parquet" => Self::Parquet,
            "pascal" => Self::Pascal,
            "pcap" => Self::Pcap,
            "pcapng" => Self::Pcapng,
            "pdb" => Self::Pdb,
            "pdf" => Self::Pdf,
            "pebin" => Self::Pebin,
            "pem" => Self::Pem,
            "perl" => Self::Perl,
            "pgp" => Self::Pgp,
            "php" => Self::Php,
            "pickle" => Self::Pickle,
            "png" => Self::Png,
            "po" => Self::Po,
            "postgres_dump" => Self::PostgresDump,
            "postscript" => Self::Postscript,
            "powershell" => Self::Powershell,
            "ppt" => Self::Ppt,
            "pptx" => Self::Pptx,
            "prolog" => Self::Prolog,
            "proteindb" => Self::Proteindb,
            "proto" => Self::Proto,
            "psd" => Self::Psd,
            "pub" => Self::Pub,
            "python" => Self::Python,
            "pythonbytecode" => Self::Pythonbytecode,
            "pytorch" => Self::Pytorch,
            "qoi" => Self::Qoi,
            "qt" => Self::Qt,
            "r" => Self::R,
            "randombytes" => Self::Randombytes,
            "randomtxt" => Self::Randomtxt,
            "rar" => Self::Rar,
            "rdata" => Self::Rdata,
            "rdf" => Self::Rdf,
            "redis_rdb" => Self::RedisRdb,
            "rhinoceros" => Self::Rhinoceros,
            "rpm" => Self::Rpm,
            "rst" => Self::Rst,
            "rtf" => Self::Rtf,
            "ruby" => Self::Ruby,
            "rust" => Self::Rust,
            "rzip" => Self::Rzip,
            "sas" => Self::Sas,
            "scala" => Self::Scala,
            "scss" => Self::Scss,
            "sevenzip" => Self::Sevenzip,
            "sgml" => Self::Sgml,
            "shapefile" => Self::Shapefile,
            "shell" => Self::Shell,
            "sketchup" => Self::Sketchup,
            "smali" => Self::Smali,
            "snap" => Self::Snap,
            "solidity" => Self::Solidity,
            "spirv" => Self::Spirv,
            "spss" => Self::Spss,
            "sql" => Self::Sql,
            "sqlite" => Self::Sqlite,
            "squashfs" => Self::Squashfs,
            "srt" => Self::Srt,
            "stata" => Self::Stata,
            "stlbinary" => Self::Stlbinary,
            "stltext" => Self::Stltext,
            "sum" => Self::Sum,
            "svg" => Self::Svg,
            "swf" => Self::Swf,
            "swift" => Self::Swift,
            "tar" => Self::Tar,
            "tcl" => Self::Tcl,
            "textproto" => Self::Textproto,
            "tga" => Self::Tga,
            "thumbsdb" => Self::Thumbsdb,
            "tiff" => Self::Tiff,
            "toml" => Self::Toml,
            "torrent" => Self::Torrent,
            "tsv" => Self::Tsv,
            "ttf" => Self::Ttf,
            "twig" => Self::Twig,
            "txt" => Self::Txt,
            "typescript" => Self::Typescript,
            "uf2" => Self::Uf2,
            "undefined" => Self::Undefined,
            "unixcompress" => Self::Unixcompress,
            "unknown" => Self::Unknown,
            "vba" => Self::Vba,
            "vcxproj" => Self::Vcxproj,
            "verilog" => Self::Verilog,
            "vhd" => Self::Vhd,
            "vhdl" => Self::Vhdl,
            "vtt" => Self::Vtt,
            "vue" => Self::Vue,
            "wad" => Self::Wad,
            "wasm" => Self::Wasm,
            "wav" => Self::Wav,
            "webm" => Self::Webm,
            "webp" => Self::Webp,
            "wim" => Self::Wim,
            "winregistry" => Self::Winregistry,
            "wma" => Self::Wma,
            "wmf" => Self::Wmf,
            "wmv" => Self::Wmv,
            "woff" => Self::Woff,
            "woff2" => Self::Woff2,
            "xar" => Self::Xar,
            "xcf" => Self::Xcf,
            "xcoff" => Self::Xcoff,
            "xls" => Self::Xls,
            "xlsb" => Self::Xlsb,
            "xlsx" => Self::Xlsx,
            "xml" => Self::Xml,
            "xpi" => Self::Xpi,
            "xz" => Self::Xz,
            "yaml" => Self::Yaml,
            "yara" => Self::Yara,
            "zig" => Self::Zig,
            "zip" => Self::Zip,
            "zlibstream" => Self::Zlibstream,
            "zst" => Self::Zst,
            _ => return None,
        })
    }

    /// Returns the content type information.
    pub fn info(self) -> &'static TypeInfo {
        match self {
            ContentType::_3dsm => &_3DSM,
            ContentType::_3dsx => &_3DSX,
            ContentType::_3gp => &_3GP,
            ContentType::Access => &ACCESS,
            ContentType::Ace => &ACE,
            ContentType::Ai => &AI,
            ContentType::Aidl => &AIDL,
            ContentType::Ani => &ANI,
            ContentType::Aout => &AOUT,
            ContentType::Apk => &APK,
            ContentType::Applebplist => &APPLEBPLIST,
            ContentType::Appledouble => &APPLEDOUBLE,
            ContentType::Appleplist => &APPLEPLIST,
            ContentType::Applesingle => &APPLESINGLE,
            ContentType::Arc => &ARC,
            ContentType::Arj => &ARJ,
            ContentType::Arrow => &ARROW,
            ContentType::Asf => &ASF,
            ContentType::Asm => &ASM,
            ContentType::Asp => &ASP,
            ContentType::Au => &AU,
            ContentType::Autohotkey => &AUTOHOTKEY,
            ContentType::Autoit => &AUTOIT,
            ContentType::Avi => &AVI,
            ContentType::Avif => &AVIF,
            ContentType::Avro => &AVRO,
            ContentType::Awk => &AWK,
            ContentType::Bam => &BAM,
            ContentType::Batch => &BATCH,
            ContentType::Bazel => &BAZEL,
            ContentType::Beam => &BEAM,
            ContentType::Berkeleydb => &BERKELEYDB,
            ContentType::Bib => &BIB,
            ContentType::Blend => &BLEND,
            ContentType::Bmp => &BMP,
            ContentType::Bpg => &BPG,
            ContentType::Bzip => &BZIP,
            ContentType::Bzip3 => &BZIP3,
            ContentType::C => &C,
            ContentType::Cab => &CAB,
            ContentType::Cat => &CAT,
            ContentType::Chm => &CHM,
            ContentType::Cinema4d => &CINEMA4D,
            ContentType::Clojure => &CLOJURE,
            ContentType::Cmake => &CMAKE,
            ContentType::Cobol => &COBOL,
            ContentType::Coff => &COFF,
            ContentType::Coffeescript => &COFFEESCRIPT,
            ContentType::Cpp => &CPP,
            ContentType::Cram => &CRAM,
            ContentType::Crt => &CRT,
            ContentType::Crx => &CRX,
            ContentType::Cs => &CS,
            ContentType::Csproj => &CSPROJ,
            ContentType::Css => &CSS,
            ContentType::Csv => &CSV,
            ContentType::Dart => &DART,
            ContentType::Dbase => &DBASE,
            ContentType::Deb => &DEB,
            ContentType::Dex => &DEX,
            ContentType::Dicom => &DICOM,
            ContentType::Diff => &DIFF,
            ContentType::Dm => &DM,
            ContentType::Dmg => &DMG,
            ContentType::Doc => &DOC,
            ContentType::Dockerfile => &DOCKERFILE,
            ContentType::Docx => &DOCX,
            ContentType::Dotx => &DOTX,
            ContentType::Dsstore => &DSSTORE,
            ContentType::Duckdb => &DUCKDB,
            ContentType::Dwg => &DWG,
            ContentType::Dxf => &DXF,
            ContentType::Elf => &ELF,
            ContentType::Elixir => &ELIXIR,
            ContentType::Emf => &EMF,
            ContentType::Eml => &EML,
            ContentType::Empty => &EMPTY,
            ContentType::Epub => &EPUB,
            ContentType::Erb => &ERB,
            ContentType::Erlang => &ERLANG,
            ContentType::Ese => &ESE,
            ContentType::Fbx => &FBX,
            ContentType::Filemaker => &FILEMAKER,
            ContentType::Fits => &FITS,
            ContentType::Flac => &FLAC,
            ContentType::Flatgeobuf => &FLATGEOBUF,
            ContentType::Flv => &FLV,
            ContentType::Fortran => &FORTRAN,
            ContentType::Gemfile => &GEMFILE,
            ContentType::Gemspec => &GEMSPEC,
            ContentType::Gguf => &GGUF,
            ContentType::Gif => &GIF,
            ContentType::Gitattributes => &GITATTRIBUTES,
            ContentType::Gitmodules => &GITMODULES,
            ContentType::Gltf => &GLTF,
            ContentType::Go => &GO,
            ContentType::Gradle => &GRADLE,
            ContentType::Groovy => &GROOVY,
            ContentType::Gzip => &GZIP,
            ContentType::H5 => &H5,
            ContentType::Handlebars => &HANDLEBARS,
            ContentType::Haskell => &HASKELL,
            ContentType::Hcl => &HCL,
            ContentType::Hdf4 => &HDF4,
            ContentType::Heif => &HEIF,
            ContentType::Hlp => &HLP,
            ContentType::Htaccess => &HTACCESS,
            ContentType::Html => &HTML,
            ContentType::Hwp => &HWP,
            ContentType::Icc => &ICC,
            ContentType::Icns => &ICNS,
            ContentType::Ico => &ICO,
            ContentType::Ics => &ICS,
            ContentType::Ignorefile => &IGNOREFILE,
            ContentType::Ini => &INI,
            ContentType::Internetshortcut => &INTERNETSHORTCUT,
            ContentType::Ipynb => &IPYNB,
            ContentType::Iso => &ISO,
            ContentType::Jar => &JAR,
            ContentType::Java => &JAVA,
            ContentType::Javabytecode => &JAVABYTECODE,
            ContentType::Javascript => &JAVASCRIPT,
            ContentType::Jinja => &JINJA,
            ContentType::Jp2 => &JP2,
            ContentType::Jpeg => &JPEG,
            ContentType::Json => &JSON,
            ContentType::Jsonl => &JSONL,
            ContentType::Julia => &JULIA,
            ContentType::Jxl => &JXL,
            ContentType::Kotlin => &KOTLIN,
            ContentType::Latex => &LATEX,
            ContentType::Lha => &LHA,
            ContentType::Lightwave => &LIGHTWAVE,
            ContentType::Lisp => &LISP,
            ContentType::LlvmBitcode => &LLVM_BITCODE,
            ContentType::Lmdb => &LMDB,
            ContentType::Lnk => &LNK,
            ContentType::Lrz => &LRZ,
            ContentType::Lua => &LUA,
            ContentType::Luabytecode => &LUABYTECODE,
            ContentType::Lz => &LZ,
            ContentType::Lz4 => &LZ4,
            ContentType::Lzx => &LZX,
            ContentType::M3u => &M3U,
            ContentType::M4 => &M4,
            ContentType::Macho => &MACHO,
            ContentType::Makefile => &MAKEFILE,
            ContentType::Markdown => &MARKDOWN,
            ContentType::Mat => &MAT,
            ContentType::Matlab => &MATLAB,
            ContentType::Mht => &MHT,
            ContentType::Midi => &MIDI,
            ContentType::Mkv => &MKV,
            ContentType::Mp3 => &MP3,
            ContentType::Mp4 => &MP4,
            ContentType::Mpegts => &MPEGTS,
            ContentType::Mscompress => &MSCOMPRESS,
            ContentType::Msi => &MSI,
            ContentType::Mum => &MUM,
            ContentType::Netcdf => &NETCDF,
            ContentType::Npy => &NPY,
            ContentType::Npz => &NPZ,
            ContentType::Nupkg => &NUPKG,
            ContentType::Objectivec => &OBJECTIVEC,
            ContentType::Ocaml => &OCAML,
            ContentType::Odp => &ODP,
            ContentType::Ods => &ODS,
            ContentType::Odt => &ODT,
            ContentType::Ogg => &OGG,
            ContentType::One => &ONE,
            ContentType::Onnx => &ONNX,
            ContentType::Orc => &ORC,
            ContentType::Otf => &OTF,
            ContentType::Outlook => &OUTLOOK,
            ContentType::Paradox => &PARADOX,
            ContentType::Parquet => &PARQUET,
            ContentType::Pascal => &PASCAL,
            ContentType::Pcap => &PCAP,
            ContentType::Pcapng => &PCAPNG,
            ContentType::Pdb => &PDB,
            ContentType::Pdf => &PDF,
            ContentType::Pebin => &PEBIN,
            ContentType::Pem => &PEM,
            ContentType::Perl => &PERL,
            ContentType::Pgp => &PGP,
            ContentType::Php => &PHP,
            ContentType::Pickle => &PICKLE,
            ContentType::Png => &PNG,
            ContentType::Po => &PO,
            ContentType::PostgresDump => &POSTGRES_DUMP,
            ContentType::Postscript => &POSTSCRIPT,
            ContentType::Powershell => &POWERSHELL,
            ContentType::Ppt => &PPT,
            ContentType::Pptx => &PPTX,
            ContentType::Prolog => &PROLOG,
            ContentType::Proteindb => &PROTEINDB,
            ContentType::Proto => &PROTO,
            ContentType::Psd => &PSD,
            ContentType::Pub => &PUB,
            ContentType::Python => &PYTHON,
            ContentType::Pythonbytecode => &PYTHONBYTECODE,
            ContentType::Pytorch => &PYTORCH,
            ContentType::Qoi => &QOI,
            ContentType::Qt => &QT,
            ContentType::R => &R,
            ContentType::Randombytes => &RANDOMBYTES,
            ContentType::Randomtxt => &RANDOMTXT,
            ContentType::Rar => &RAR,
            ContentType::Rdata => &RDATA,
            ContentType::Rdf => &RDF,
            ContentType::RedisRdb => &REDIS_RDB,
            ContentType::Rhinoceros => &RHINOCEROS,
            ContentType::Rpm => &RPM,
            ContentType::Rst => &RST,
            ContentType::Rtf => &RTF,
            ContentType::Ruby => &RUBY,
            ContentType::Rust => &RUST,
            ContentType::Rzip => &RZIP,
            ContentType::Sas => &SAS,
            ContentType::Scala => &SCALA,
            ContentType::Scss => &SCSS,
            ContentType::Sevenzip => &SEVENZIP,
            ContentType::Sgml => &SGML,
            ContentType::Shapefile => &SHAPEFILE,
            ContentType::Shell => &SHELL,
            ContentType::Sketchup => &SKETCHUP,
            ContentType::Smali => &SMALI,
            ContentType::Snap => &SNAP,
            ContentType::Solidity => &SOLIDITY,
            ContentType::Spirv => &SPIRV,
            ContentType::Spss => &SPSS,
            ContentType::Sql => &SQL,
            ContentType::Sqlite => &SQLITE,
            ContentType::Squashfs => &SQUASHFS,
            ContentType::Srt => &SRT,
            ContentType::Stata => &STATA,
            ContentType::Stlbinary => &STLBINARY,
            ContentType::Stltext => &STLTEXT,
            ContentType::Sum => &SUM,
            ContentType::Svg => &SVG,
            ContentType::Swf => &SWF,
            ContentType::Swift => &SWIFT,
            ContentType::Tar => &TAR,
            ContentType::Tcl => &TCL,
            ContentType::Textproto => &TEXTPROTO,
            ContentType::Tga => &TGA,
            ContentType::Thumbsdb => &THUMBSDB,
            ContentType::Tiff => &TIFF,
            ContentType::Toml => &TOML,
            ContentType::Torrent => &TORRENT,
            ContentType::Tsv => &TSV,
            ContentType::Ttf => &TTF,
            ContentType::Twig => &TWIG,
            ContentType::Txt => &TXT,
            ContentType::Typescript => &TYPESCRIPT,
            ContentType::Uf2 => &UF2,
            ContentType::Undefined => &UNDEFINED,
            ContentType::Unixcompress => &UNIXCOMPRESS,
            ContentType::Unknown => &UNKNOWN,
            ContentType::Vba => &VBA,
            ContentType::Vcxproj => &VCXPROJ,
            ContentType::Verilog => &VERILOG,
            ContentType::Vhd => &VHD,
            ContentType::Vhdl => &VHDL,
            ContentType::Vtt => &VTT,
            ContentType::Vue => &VUE,
            ContentType::Wad => &WAD,
            ContentType::Wasm => &WASM,
            ContentType::Wav => &WAV,
            ContentType::Webm => &WEBM,
            ContentType::Webp => &WEBP,
            ContentType::Wim => &WIM,
            ContentType::Winregistry => &WINREGISTRY,
            ContentType::Wma => &WMA,
            ContentType::Wmf => &WMF,
            ContentType::Wmv => &WMV,
            ContentType::Woff => &WOFF,
            ContentType::Woff2 => &WOFF2,
            ContentType::Xar => &XAR,
            ContentType::Xcf => &XCF,
            ContentType::Xcoff => &XCOFF,
            ContentType::Xls => &XLS,
            ContentType::Xlsb => &XLSB,
            ContentType::Xlsx => &XLSX,
            ContentType::Xml => &XML,
            ContentType::Xpi => &XPI,
            ContentType::Xz => &XZ,
            ContentType::Yaml => &YAML,
            ContentType::Yara => &YARA,
            ContentType::Zig => &ZIG,
            ContentType::Zip => &ZIP,
            ContentType::Zlibstream => &ZLIBSTREAM,
            ContentType::Zst => &ZST,
        }
    }
}
