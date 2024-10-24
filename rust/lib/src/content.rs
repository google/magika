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
pub const MODEL_NAME: &str = "standard_v2_1";

pub(crate) static _3GP: TypeInfo = TypeInfo {
    label: "3gp",
    mime_type: "video/3gpp",
    group: "video",
    description: "3GPP multimedia file",
    extensions: &["3gp"],
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

pub(crate) static APPLEPLIST: TypeInfo = TypeInfo {
    label: "appleplist",
    mime_type: "application/x-plist",
    group: "application",
    description: "Apple property list",
    extensions: &["plist"],
    is_text: true,
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

pub(crate) static AWK: TypeInfo = TypeInfo {
    label: "awk",
    mime_type: "text/plain",
    group: "code",
    description: "Awk",
    extensions: &["awk"],
    is_text: true,
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

pub(crate) static BIB: TypeInfo = TypeInfo {
    label: "bib",
    mime_type: "text/x-bibtex",
    group: "text",
    description: "BibTeX",
    extensions: &["bib"],
    is_text: true,
};

pub(crate) static BMP: TypeInfo = TypeInfo {
    label: "bmp",
    mime_type: "image/bmp",
    group: "image",
    description: "BMP image data",
    extensions: &["bmp"],
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
    group: "text",
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

pub(crate) static DSSTORE: TypeInfo = TypeInfo {
    label: "dsstore",
    mime_type: "application/octet-stream",
    group: "unknown",
    description: "Application Desktop Services Store",
    extensions: &[],
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

pub(crate) static FLAC: TypeInfo = TypeInfo {
    label: "flac",
    mime_type: "audio/flac",
    group: "audio",
    description: "FLAC audio bitstream data",
    extensions: &["flac"],
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

pub(crate) static LISP: TypeInfo = TypeInfo {
    label: "lisp",
    mime_type: "text/x-lisp",
    group: "code",
    description: "Lisp source",
    extensions: &["lisp", "lsp", "l", "cl"],
    is_text: true,
};

pub(crate) static LNK: TypeInfo = TypeInfo {
    label: "lnk",
    mime_type: "application/x-ms-shortcut",
    group: "application",
    description: "MS Windows shortcut",
    extensions: &["lnk"],
    is_text: false,
};

pub(crate) static LUA: TypeInfo = TypeInfo {
    label: "lua",
    mime_type: "text/plain",
    group: "text",
    description: "Lua",
    extensions: &["lua"],
    is_text: true,
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
    group: "text",
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

pub(crate) static RAR: TypeInfo = TypeInfo {
    label: "rar",
    mime_type: "application/x-rar",
    group: "archive",
    description: "RAR archive data",
    extensions: &["rar"],
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

pub(crate) static SHELL: TypeInfo = TypeInfo {
    label: "shell",
    mime_type: "text/x-shellscript",
    group: "code",
    description: "Shell script",
    extensions: &["sh"],
    is_text: true,
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
    group: "text",
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
    group: "text",
    description: "Typescript",
    extensions: &["ts", "mts", "cts"],
    is_text: true,
};

pub(crate) static UNDEFINED: TypeInfo = TypeInfo {
    label: "undefined",
    mime_type: "application/undefined",
    group: "undefined",
    description: "Undefined",
    extensions: &[],
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

pub(crate) static WINREGISTRY: TypeInfo = TypeInfo {
    label: "winregistry",
    mime_type: "text/x-ms-regedit",
    group: "application",
    description: "Windows Registry text",
    extensions: &["reg"],
    is_text: true,
};

pub(crate) static WMF: TypeInfo = TypeInfo {
    label: "wmf",
    mime_type: "image/wmf",
    group: "image",
    description: "Windows metafile",
    extensions: &["wmf"],
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

/// Content types for regular files.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ContentType {
    /// 3GPP multimedia file
    _3gp,
    /// ACE archive
    Ace,
    /// Adobe Illustrator Artwork
    Ai,
    /// Android Interface Definition Language
    Aidl,
    /// Android package
    Apk,
    /// Apple binary property list
    Applebplist,
    /// Apple property list
    Appleplist,
    /// Assembly
    Asm,
    /// ASP source
    Asp,
    /// AutoHotKey script
    Autohotkey,
    /// AutoIt script
    Autoit,
    /// Awk
    Awk,
    /// DOS batch file
    Batch,
    /// Bazel build file
    Bazel,
    /// BibTeX
    Bib,
    /// BMP image data
    Bmp,
    /// bzip2 compressed data
    Bzip,
    /// C source
    C,
    /// Microsoft Cabinet archive data
    Cab,
    /// Windows Catalog file
    Cat,
    /// MS Windows HtmlHelp Data
    Chm,
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
    /// Application Desktop Services Store
    Dsstore,
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
    /// FLAC audio bitstream data
    Flac,
    /// Flash Video
    Flv,
    /// Fortran
    Fortran,
    /// Gemfile file
    Gemfile,
    /// Gemspec file
    Gemspec,
    /// GIF image data
    Gif,
    /// Gitattributes file
    Gitattributes,
    /// Gitmodules file
    Gitmodules,
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
    /// MS Windows help
    Hlp,
    /// Apache access configuration
    Htaccess,
    /// HTML document
    Html,
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
    /// Kotlin source
    Kotlin,
    /// LaTeX document
    Latex,
    /// LHarc archive
    Lha,
    /// Lisp source
    Lisp,
    /// MS Windows shortcut
    Lnk,
    /// Lua
    Lua,
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
    /// MS Compress archive data
    Mscompress,
    /// Microsoft Installer file
    Msi,
    /// Windows Update Package file
    Mum,
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
    /// OpenType font
    Otf,
    /// MS Outlook Message
    Outlook,
    /// Apache Parquet
    Parquet,
    /// Pascal source
    Pascal,
    /// pcap capture file
    Pcap,
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
    /// PHP source
    Php,
    /// Python pickle
    Pickle,
    /// PNG image
    Png,
    /// Portable Object (PO) for i18n
    Po,
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
    /// Python source
    Python,
    /// Python compiled bytecode
    Pythonbytecode,
    /// Pytorch storage file
    Pytorch,
    /// QuickTime
    Qt,
    /// R (language)
    R,
    /// RAR archive data
    Rar,
    /// Resource Description Framework document (RDF)
    Rdf,
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
    /// Scala source
    Scala,
    /// SCSS source
    Scss,
    /// 7-zip archive data
    Sevenzip,
    /// sgml
    Sgml,
    /// Shell script
    Shell,
    /// Smali source
    Smali,
    /// Snap archive
    Snap,
    /// Solidity source
    Solidity,
    /// SQL source
    Sql,
    /// SQLITE database
    Sqlite,
    /// Squash filesystem
    Squashfs,
    /// SubRip Text Format
    Srt,
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
    /// Typescript
    Typescript,
    /// Undefined
    Undefined,
    /// Unknown binary data
    Unknown,
    /// MS Visual Basic source (VBA)
    Vba,
    /// Visual Studio MSBuild project
    Vcxproj,
    /// Verilog source
    Verilog,
    /// VHDL source
    Vhdl,
    /// Web Video Text Tracks
    Vtt,
    /// Vue source
    Vue,
    /// Web Assembly
    Wasm,
    /// Waveform Audio file (WAV)
    Wav,
    /// WebM media file
    Webm,
    /// WebP media file
    Webp,
    /// Windows Registry text
    Winregistry,
    /// Windows metafile
    Wmf,
    /// Web Open Font Format
    Woff,
    /// Web Open Font Format v2
    Woff2,
    /// XAR archive compressed data
    Xar,
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
}

impl ContentType {
    pub(crate) const SIZE: usize = 215;

    /// Returns the content type information.
    pub fn info(self) -> &'static TypeInfo {
        match self {
            ContentType::_3gp => &_3GP,
            ContentType::Ace => &ACE,
            ContentType::Ai => &AI,
            ContentType::Aidl => &AIDL,
            ContentType::Apk => &APK,
            ContentType::Applebplist => &APPLEBPLIST,
            ContentType::Appleplist => &APPLEPLIST,
            ContentType::Asm => &ASM,
            ContentType::Asp => &ASP,
            ContentType::Autohotkey => &AUTOHOTKEY,
            ContentType::Autoit => &AUTOIT,
            ContentType::Awk => &AWK,
            ContentType::Batch => &BATCH,
            ContentType::Bazel => &BAZEL,
            ContentType::Bib => &BIB,
            ContentType::Bmp => &BMP,
            ContentType::Bzip => &BZIP,
            ContentType::C => &C,
            ContentType::Cab => &CAB,
            ContentType::Cat => &CAT,
            ContentType::Chm => &CHM,
            ContentType::Clojure => &CLOJURE,
            ContentType::Cmake => &CMAKE,
            ContentType::Cobol => &COBOL,
            ContentType::Coff => &COFF,
            ContentType::Coffeescript => &COFFEESCRIPT,
            ContentType::Cpp => &CPP,
            ContentType::Crt => &CRT,
            ContentType::Crx => &CRX,
            ContentType::Cs => &CS,
            ContentType::Csproj => &CSPROJ,
            ContentType::Css => &CSS,
            ContentType::Csv => &CSV,
            ContentType::Dart => &DART,
            ContentType::Deb => &DEB,
            ContentType::Dex => &DEX,
            ContentType::Dicom => &DICOM,
            ContentType::Diff => &DIFF,
            ContentType::Dm => &DM,
            ContentType::Dmg => &DMG,
            ContentType::Doc => &DOC,
            ContentType::Dockerfile => &DOCKERFILE,
            ContentType::Docx => &DOCX,
            ContentType::Dsstore => &DSSTORE,
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
            ContentType::Flac => &FLAC,
            ContentType::Flv => &FLV,
            ContentType::Fortran => &FORTRAN,
            ContentType::Gemfile => &GEMFILE,
            ContentType::Gemspec => &GEMSPEC,
            ContentType::Gif => &GIF,
            ContentType::Gitattributes => &GITATTRIBUTES,
            ContentType::Gitmodules => &GITMODULES,
            ContentType::Go => &GO,
            ContentType::Gradle => &GRADLE,
            ContentType::Groovy => &GROOVY,
            ContentType::Gzip => &GZIP,
            ContentType::H5 => &H5,
            ContentType::Handlebars => &HANDLEBARS,
            ContentType::Haskell => &HASKELL,
            ContentType::Hcl => &HCL,
            ContentType::Hlp => &HLP,
            ContentType::Htaccess => &HTACCESS,
            ContentType::Html => &HTML,
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
            ContentType::Kotlin => &KOTLIN,
            ContentType::Latex => &LATEX,
            ContentType::Lha => &LHA,
            ContentType::Lisp => &LISP,
            ContentType::Lnk => &LNK,
            ContentType::Lua => &LUA,
            ContentType::M3u => &M3U,
            ContentType::M4 => &M4,
            ContentType::Macho => &MACHO,
            ContentType::Makefile => &MAKEFILE,
            ContentType::Markdown => &MARKDOWN,
            ContentType::Matlab => &MATLAB,
            ContentType::Mht => &MHT,
            ContentType::Midi => &MIDI,
            ContentType::Mkv => &MKV,
            ContentType::Mp3 => &MP3,
            ContentType::Mp4 => &MP4,
            ContentType::Mscompress => &MSCOMPRESS,
            ContentType::Msi => &MSI,
            ContentType::Mum => &MUM,
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
            ContentType::Otf => &OTF,
            ContentType::Outlook => &OUTLOOK,
            ContentType::Parquet => &PARQUET,
            ContentType::Pascal => &PASCAL,
            ContentType::Pcap => &PCAP,
            ContentType::Pdb => &PDB,
            ContentType::Pdf => &PDF,
            ContentType::Pebin => &PEBIN,
            ContentType::Pem => &PEM,
            ContentType::Perl => &PERL,
            ContentType::Php => &PHP,
            ContentType::Pickle => &PICKLE,
            ContentType::Png => &PNG,
            ContentType::Po => &PO,
            ContentType::Postscript => &POSTSCRIPT,
            ContentType::Powershell => &POWERSHELL,
            ContentType::Ppt => &PPT,
            ContentType::Pptx => &PPTX,
            ContentType::Prolog => &PROLOG,
            ContentType::Proteindb => &PROTEINDB,
            ContentType::Proto => &PROTO,
            ContentType::Psd => &PSD,
            ContentType::Python => &PYTHON,
            ContentType::Pythonbytecode => &PYTHONBYTECODE,
            ContentType::Pytorch => &PYTORCH,
            ContentType::Qt => &QT,
            ContentType::R => &R,
            ContentType::Rar => &RAR,
            ContentType::Rdf => &RDF,
            ContentType::Rpm => &RPM,
            ContentType::Rst => &RST,
            ContentType::Rtf => &RTF,
            ContentType::Ruby => &RUBY,
            ContentType::Rust => &RUST,
            ContentType::Scala => &SCALA,
            ContentType::Scss => &SCSS,
            ContentType::Sevenzip => &SEVENZIP,
            ContentType::Sgml => &SGML,
            ContentType::Shell => &SHELL,
            ContentType::Smali => &SMALI,
            ContentType::Snap => &SNAP,
            ContentType::Solidity => &SOLIDITY,
            ContentType::Sql => &SQL,
            ContentType::Sqlite => &SQLITE,
            ContentType::Squashfs => &SQUASHFS,
            ContentType::Srt => &SRT,
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
            ContentType::Undefined => &UNDEFINED,
            ContentType::Unknown => &UNKNOWN,
            ContentType::Vba => &VBA,
            ContentType::Vcxproj => &VCXPROJ,
            ContentType::Verilog => &VERILOG,
            ContentType::Vhdl => &VHDL,
            ContentType::Vtt => &VTT,
            ContentType::Vue => &VUE,
            ContentType::Wasm => &WASM,
            ContentType::Wav => &WAV,
            ContentType::Webm => &WEBM,
            ContentType::Webp => &WEBP,
            ContentType::Winregistry => &WINREGISTRY,
            ContentType::Wmf => &WMF,
            ContentType::Woff => &WOFF,
            ContentType::Woff2 => &WOFF2,
            ContentType::Xar => &XAR,
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
        }
    }
}
