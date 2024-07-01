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

use crate::output::Metadata;

// DO NOT EDIT, see link below for more information:
// https://github.com/google/magika/tree/main/rust/gen

/// Content type of a file.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
pub enum Label {
    /// Adobe Illustrator Artwork
    Ai,
    /// Android package
    Apk,
    /// Apple property list
    Appleplist,
    /// Assembly
    Asm,
    /// ASP source
    Asp,
    /// DOS batch file
    Batch,
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
    /// Intel 80386 COFF
    Coff,
    /// Google Chrome extension
    Crx,
    /// C# source
    Cs,
    /// CSS source
    Css,
    /// CSV document
    Csv,
    /// Debian binary package
    Deb,
    /// Dalvik dex file
    Dex,
    /// Apple disk image
    Dmg,
    /// Microsoft Word CDF document
    Doc,
    /// Microsoft Word 2007+ document
    Docx,
    /// ELF executable
    Elf,
    /// Windows Enhanced Metafile image data
    Emf,
    /// RFC 822 mail
    Eml,
    /// EPUB document
    Epub,
    /// FLAC audio bitstream data
    Flac,
    /// GIF image data
    Gif,
    /// Golang source
    Go,
    /// gzip compressed data
    Gzip,
    /// MS Windows help
    Hlp,
    /// HTML document
    Html,
    /// MS Windows icon resource
    Ico,
    /// INI configuration file
    Ini,
    /// MS Windows Internet shortcut
    Internetshortcut,
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
    /// JPEG image data
    Jpeg,
    /// JSON document
    Json,
    /// LaTeX document
    Latex,
    /// Lisp source
    Lisp,
    /// MS Windows shortcut
    Lnk,
    /// M3U playlist
    M3u,
    /// Mach-O executable
    Macho,
    /// Makefile source
    Makefile,
    /// Markdown document
    Markdown,
    /// MHTML document
    Mht,
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
    /// ODEX ELF executable
    Odex,
    /// OpenDocument Presentation
    Odp,
    /// OpenDocument Spreadsheet
    Ods,
    /// OpenDocument Text
    Odt,
    /// Ogg data
    Ogg,
    /// MS Outlook Message
    Outlook,
    /// pcap capture file
    Pcap,
    /// PDF document
    Pdf,
    /// PE executable
    Pebin,
    /// PEM certificate
    Pem,
    /// Perl source
    Perl,
    /// PHP source
    Php,
    /// PNG image data
    Png,
    /// PostScript document
    Postscript,
    /// Powershell source
    Powershell,
    /// Microsoft PowerPoint CDF document
    Ppt,
    /// Microsoft PowerPoint 2007+ document
    Pptx,
    /// Python source
    Python,
    /// Python compiled bytecode
    Pythonbytecode,
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
    /// 7-zip archive data
    Sevenzip,
    /// Shell script
    Shell,
    /// Smali source
    Smali,
    /// SQL source
    Sql,
    /// Squash filesystem
    Squashfs,
    /// SVG Scalable Vector Graphics image data
    Svg,
    /// Macromedia Flash data
    Swf,
    /// Symbolic link (textual representation)
    Symlinktext,
    /// POSIX tar archive
    Tar,
    /// Targa image data
    Tga,
    /// TIFF image data
    Tiff,
    /// BitTorrent file
    Torrent,
    /// TrueType Font data
    Ttf,
    /// Generic text document
    Txt,
    /// Unknown binary data
    Unknown,
    /// MS Visual Basic source (VBA)
    Vba,
    /// Waveform Audio file (WAV)
    Wav,
    /// WebM data
    Webm,
    /// WebP data
    Webp,
    /// Windows Registry text
    Winregistry,
    /// Windows metafile
    Wmf,
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
    /// Zip archive data
    Zip,
    /// zlib compressed data
    Zlibstream,
}

pub(crate) const MAX_LABEL: u32 = 112;

pub(crate) const METADATA: [Metadata; 113] = [
    Metadata {
        code: "ai",
        desc: "Adobe Illustrator Artwork",
        magic: "PDF document",
        group: "document",
        mime: "application/pdf",
        extension: &["ai"],
        is_text: false,
    },
    Metadata {
        code: "apk",
        desc: "Android package",
        magic: "Java archive data",
        group: "executable",
        mime: "application/vnd.android.package-archive",
        extension: &["apk"],
        is_text: false,
    },
    Metadata {
        code: "appleplist",
        desc: "Apple property list",
        magic: "Apple binary property list",
        group: "application",
        mime: "application/x-plist",
        extension: &["bplist", "plist"],
        is_text: true,
    },
    Metadata {
        code: "asm",
        desc: "Assembly",
        magic: "assembler source",
        group: "code",
        mime: "text/x-asm",
        extension: &["S", "asm"],
        is_text: true,
    },
    Metadata {
        code: "asp",
        desc: "ASP source",
        magic: "HTML document",
        group: "code",
        mime: "text/html",
        extension: &["aspx", "asp"],
        is_text: true,
    },
    Metadata {
        code: "batch",
        desc: "DOS batch file",
        magic: "DOS batch file",
        group: "code",
        mime: "text/x-msdos-batch",
        extension: &["bat"],
        is_text: true,
    },
    Metadata {
        code: "bmp",
        desc: "BMP image data",
        magic: "PC bitmap",
        group: "image",
        mime: "image/bmp",
        extension: &["bmp"],
        is_text: false,
    },
    Metadata {
        code: "bzip",
        desc: "bzip2 compressed data",
        magic: "bzip2 compressed data",
        group: "archive",
        mime: "application/x-bzip2",
        extension: &["bz2", "tbz2", "tar.bz2"],
        is_text: false,
    },
    Metadata {
        code: "c",
        desc: "C source",
        magic: "C source",
        group: "code",
        mime: "text/x-c",
        extension: &["c", "cpp", "h", "hpp", "cc"],
        is_text: true,
    },
    Metadata {
        code: "cab",
        desc: "Microsoft Cabinet archive data",
        magic: "Microsoft Cabinet archive data",
        group: "archive",
        mime: "application/vnd.ms-cab-compressed",
        extension: &["cab"],
        is_text: false,
    },
    Metadata {
        code: "cat",
        desc: "Windows Catalog file",
        magic: "data",
        group: "application",
        mime: "application/octet-stream",
        extension: &["cat"],
        is_text: false,
    },
    Metadata {
        code: "chm",
        desc: "MS Windows HtmlHelp Data",
        magic: "MS Windows HtmlHelp Data",
        group: "application",
        mime: "application/chm",
        extension: &["chm"],
        is_text: false,
    },
    Metadata {
        code: "coff",
        desc: "Intel 80386 COFF",
        magic: "Intel 80386 COFF",
        group: "executable",
        mime: "application/x-coff",
        extension: &[],
        is_text: false,
    },
    Metadata {
        code: "crx",
        desc: "Google Chrome extension",
        magic: "Google Chrome extension",
        group: "executable",
        mime: "application/x-chrome-extension",
        extension: &["crx"],
        is_text: false,
    },
    Metadata {
        code: "cs",
        desc: "C# source",
        magic: "ASCII text",
        group: "code",
        mime: "text/plain",
        extension: &["cs"],
        is_text: true,
    },
    Metadata {
        code: "css",
        desc: "CSS source",
        magic: "ASCII text",
        group: "code",
        mime: "text/css",
        extension: &["css"],
        is_text: true,
    },
    Metadata {
        code: "csv",
        desc: "CSV document",
        magic: "CSV text",
        group: "code",
        mime: "text/csv",
        extension: &["csv"],
        is_text: true,
    },
    Metadata {
        code: "deb",
        desc: "Debian binary package",
        magic: "Debian binary package",
        group: "archive",
        mime: "application/vnd.debian.binary-package",
        extension: &["deb"],
        is_text: false,
    },
    Metadata {
        code: "dex",
        desc: "Dalvik dex file",
        magic: "Dalvik dex file",
        group: "executable",
        mime: "application/x-android-dex",
        extension: &["dex"],
        is_text: false,
    },
    Metadata {
        code: "dmg",
        desc: "Apple disk image",
        magic: "Apple disk image",
        group: "archive",
        mime: "application/x-apple-diskimage",
        extension: &["dmg"],
        is_text: false,
    },
    Metadata {
        code: "doc",
        desc: "Microsoft Word CDF document",
        magic: "Composite Document File",
        group: "document",
        mime: "application/msword",
        extension: &["doc"],
        is_text: false,
    },
    Metadata {
        code: "docx",
        desc: "Microsoft Word 2007+ document",
        magic: "Microsoft Word 2007+",
        group: "document",
        mime: "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        extension: &["docx", "docm"],
        is_text: false,
    },
    Metadata {
        code: "elf",
        desc: "ELF executable",
        magic: "ELF executable",
        group: "executable",
        mime: "application/x-executable-elf",
        extension: &["elf", "so"],
        is_text: false,
    },
    Metadata {
        code: "emf",
        desc: "Windows Enhanced Metafile image data",
        magic: "Windows Enhanced Metafile",
        group: "application",
        mime: "application/octet-stream",
        extension: &["emf"],
        is_text: false,
    },
    Metadata {
        code: "eml",
        desc: "RFC 822 mail",
        magic: "RFC 822 mail",
        group: "text",
        mime: "message/rfc822",
        extension: &["eml"],
        is_text: true,
    },
    Metadata {
        code: "epub",
        desc: "EPUB document",
        magic: "EPUB document",
        group: "document",
        mime: "application/epub+zip",
        extension: &["epub"],
        is_text: false,
    },
    Metadata {
        code: "flac",
        desc: "FLAC audio bitstream data",
        magic: "FLAC audio bitstream data",
        group: "audio",
        mime: "audio/flac",
        extension: &["flac"],
        is_text: false,
    },
    Metadata {
        code: "gif",
        desc: "GIF image data",
        magic: "GIF image data",
        group: "image",
        mime: "image/gif",
        extension: &["gif"],
        is_text: false,
    },
    Metadata {
        code: "go",
        desc: "Golang source",
        magic: "ASCII text",
        group: "code",
        mime: "text/x-golang",
        extension: &["go"],
        is_text: true,
    },
    Metadata {
        code: "gzip",
        desc: "gzip compressed data",
        magic: "gzip compressed data",
        group: "archive",
        mime: "application/gzip",
        extension: &["gz", "gzip", "tgz", "tar.gz"],
        is_text: false,
    },
    Metadata {
        code: "hlp",
        desc: "MS Windows help",
        magic: "MS Windows help",
        group: "application",
        mime: "application/winhlp",
        extension: &["hlp"],
        is_text: false,
    },
    Metadata {
        code: "html",
        desc: "HTML document",
        magic: "HTML document",
        group: "code",
        mime: "text/html",
        extension: &["html", "htm", "xhtml", "xht"],
        is_text: true,
    },
    Metadata {
        code: "ico",
        desc: "MS Windows icon resource",
        magic: "MS Windows icon resource",
        group: "image",
        mime: "image/vnd.microsoft.icon",
        extension: &["ico"],
        is_text: false,
    },
    Metadata {
        code: "ini",
        desc: "INI configuration file",
        magic: "Generic INItialization configuration",
        group: "text",
        mime: "text/plain",
        extension: &["ini"],
        is_text: true,
    },
    Metadata {
        code: "internetshortcut",
        desc: "MS Windows Internet shortcut",
        magic: "MS Windows 95 Internet shortcut",
        group: "application",
        mime: "application/x-mswinurl",
        extension: &["url"],
        is_text: true,
    },
    Metadata {
        code: "iso",
        desc: "ISO 9660 CD-ROM filesystem data",
        magic: "ISO 9660 CD-ROM filesystem data",
        group: "archive",
        mime: "application/x-iso9660-image",
        extension: &["iso"],
        is_text: false,
    },
    Metadata {
        code: "jar",
        desc: "Java archive data (JAR)",
        magic: "Java archive data (JAR)",
        group: "archive",
        mime: "application/java-archive",
        extension: &["jar"],
        is_text: false,
    },
    Metadata {
        code: "java",
        desc: "Java source",
        magic: "Java source",
        group: "code",
        mime: "text/x-java",
        extension: &["java"],
        is_text: true,
    },
    Metadata {
        code: "javabytecode",
        desc: "Java compiled bytecode",
        magic: "compiled Java class data",
        group: "executable",
        mime: "application/x-java-applet",
        extension: &["class"],
        is_text: false,
    },
    Metadata {
        code: "javascript",
        desc: "JavaScript source",
        magic: "JavaScript source",
        group: "code",
        mime: "application/javascript",
        extension: &["js"],
        is_text: true,
    },
    Metadata {
        code: "jpeg",
        desc: "JPEG image data",
        magic: "JPEG image data",
        group: "image",
        mime: "image/jpeg",
        extension: &["jpg", "jpeg"],
        is_text: false,
    },
    Metadata {
        code: "json",
        desc: "JSON document",
        magic: "JSON data",
        group: "code",
        mime: "application/json",
        extension: &["json"],
        is_text: true,
    },
    Metadata {
        code: "latex",
        desc: "LaTeX document",
        magic: "LaTeX document",
        group: "text",
        mime: "text/x-tex",
        extension: &["tex"],
        is_text: true,
    },
    Metadata {
        code: "lisp",
        desc: "Lisp source",
        magic: "Lisp/Scheme program",
        group: "code",
        mime: "text/x-lisp",
        extension: &["lisp"],
        is_text: true,
    },
    Metadata {
        code: "lnk",
        desc: "MS Windows shortcut",
        magic: "MS Windows shortcut",
        group: "application",
        mime: "application/x-ms-shortcut",
        extension: &["lnk"],
        is_text: false,
    },
    Metadata {
        code: "m3u",
        desc: "M3U playlist",
        magic: "M3U playlist",
        group: "application",
        mime: "text/plain",
        extension: &["m3u8", "m3u"],
        is_text: false,
    },
    Metadata {
        code: "macho",
        desc: "Mach-O executable",
        magic: "Mach-O executable",
        group: "executable",
        mime: "application/x-mach-o",
        extension: &[],
        is_text: false,
    },
    Metadata {
        code: "makefile",
        desc: "Makefile source",
        magic: "makefile script",
        group: "code",
        mime: "text/x-makefile",
        extension: &["=Makefile"],
        is_text: true,
    },
    Metadata {
        code: "markdown",
        desc: "Markdown document",
        magic: "ASCII text",
        group: "text",
        mime: "text/markdown",
        extension: &["md"],
        is_text: true,
    },
    Metadata {
        code: "mht",
        desc: "MHTML document",
        magic: "HTML document",
        group: "code",
        mime: "application/x-mimearchive",
        extension: &["mht"],
        is_text: true,
    },
    Metadata {
        code: "mp3",
        desc: "MP3 media file",
        magic: "Audio file with ID3",
        group: "audio",
        mime: "audio/mpeg",
        extension: &["mp3"],
        is_text: false,
    },
    Metadata {
        code: "mp4",
        desc: "MP4 media file",
        magic: "ISO Media",
        group: "video",
        mime: "video/mp4",
        extension: &["mov", "mp4"],
        is_text: false,
    },
    Metadata {
        code: "mscompress",
        desc: "MS Compress archive data",
        magic: "MS Compress archive data",
        group: "archive",
        mime: "application/x-ms-compress-szdd",
        extension: &[],
        is_text: false,
    },
    Metadata {
        code: "msi",
        desc: "Microsoft Installer file",
        magic: "Composite Document File",
        group: "archive",
        mime: "application/x-msi",
        extension: &["msi"],
        is_text: false,
    },
    Metadata {
        code: "mum",
        desc: "Windows Update Package file",
        magic: "XML document",
        group: "application",
        mime: "text/xml",
        extension: &["mum"],
        is_text: true,
    },
    Metadata {
        code: "odex",
        desc: "ODEX ELF executable",
        magic: "ELF executable",
        group: "executable",
        mime: "application/x-executable-elf",
        extension: &["odex"],
        is_text: false,
    },
    Metadata {
        code: "odp",
        desc: "OpenDocument Presentation",
        magic: "OpenDocument Presentation",
        group: "document",
        mime: "application/vnd.oasis.opendocument.presentation",
        extension: &["odp"],
        is_text: false,
    },
    Metadata {
        code: "ods",
        desc: "OpenDocument Spreadsheet",
        magic: "OpenDocument Spreadsheet",
        group: "document",
        mime: "application/vnd.oasis.opendocument.spreadsheet",
        extension: &["ods"],
        is_text: false,
    },
    Metadata {
        code: "odt",
        desc: "OpenDocument Text",
        magic: "OpenDocument Text",
        group: "document",
        mime: "application/vnd.oasis.opendocument.text",
        extension: &["odt"],
        is_text: false,
    },
    Metadata {
        code: "ogg",
        desc: "Ogg data",
        magic: "Ogg data",
        group: "audio",
        mime: "audio/ogg",
        extension: &["ogg"],
        is_text: false,
    },
    Metadata {
        code: "outlook",
        desc: "MS Outlook Message",
        magic: "CDFV2 Microsoft Outlook Message",
        group: "application",
        mime: "application/vnd.ms-outlook",
        extension: &[],
        is_text: false,
    },
    Metadata {
        code: "pcap",
        desc: "pcap capture file",
        magic: "pcap capture file",
        group: "application",
        mime: "application/vnd.tcpdump.pcap",
        extension: &["pcap", "pcapng"],
        is_text: false,
    },
    Metadata {
        code: "pdf",
        desc: "PDF document",
        magic: "PDF document",
        group: "document",
        mime: "application/pdf",
        extension: &["pdf"],
        is_text: false,
    },
    Metadata {
        code: "pebin",
        desc: "PE executable",
        magic: "PE executable",
        group: "executable",
        mime: "application/x-dosexec",
        extension: &["exe", "dll", "sys"],
        is_text: false,
    },
    Metadata {
        code: "pem",
        desc: "PEM certificate",
        magic: "PEM certificate",
        group: "application",
        mime: "application/x-pem-file",
        extension: &["pem", "pub"],
        is_text: true,
    },
    Metadata {
        code: "perl",
        desc: "Perl source",
        magic: "Perl script text executable",
        group: "code",
        mime: "text/x-perl",
        extension: &["pl"],
        is_text: true,
    },
    Metadata {
        code: "php",
        desc: "PHP source",
        magic: "PHP script",
        group: "code",
        mime: "text/x-php",
        extension: &["php"],
        is_text: true,
    },
    Metadata {
        code: "png",
        desc: "PNG image data",
        magic: "PNG image data",
        group: "image",
        mime: "image/png",
        extension: &["png"],
        is_text: false,
    },
    Metadata {
        code: "postscript",
        desc: "PostScript document",
        magic: "PostScript document text",
        group: "document",
        mime: "application/postscript",
        extension: &["ps"],
        is_text: false,
    },
    Metadata {
        code: "powershell",
        desc: "Powershell source",
        magic: "a powershell script",
        group: "code",
        mime: "application/x-powershell",
        extension: &["ps1"],
        is_text: true,
    },
    Metadata {
        code: "ppt",
        desc: "Microsoft PowerPoint CDF document",
        magic: "Composite Document File",
        group: "document",
        mime: "application/vnd.ms-powerpoint",
        extension: &["ppt"],
        is_text: false,
    },
    Metadata {
        code: "pptx",
        desc: "Microsoft PowerPoint 2007+ document",
        magic: "Microsoft PowerPoint 2007+",
        group: "document",
        mime: "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        extension: &["pptx", "pptm"],
        is_text: false,
    },
    Metadata {
        code: "python",
        desc: "Python source",
        magic: "Python script",
        group: "code",
        mime: "text/x-python",
        extension: &["py"],
        is_text: true,
    },
    Metadata {
        code: "pythonbytecode",
        desc: "Python compiled bytecode",
        magic: "python byte-compiled",
        group: "executable",
        mime: "application/x-bytecode.python",
        extension: &["pyc", "pyo"],
        is_text: false,
    },
    Metadata {
        code: "rar",
        desc: "RAR archive data",
        magic: "RAR archive data",
        group: "archive",
        mime: "application/x-rar",
        extension: &["rar"],
        is_text: false,
    },
    Metadata {
        code: "rdf",
        desc: "Resource Description Framework document (RDF)",
        magic: "XML document",
        group: "text",
        mime: "application/rdf+xml",
        extension: &["rdf"],
        is_text: true,
    },
    Metadata {
        code: "rpm",
        desc: "RedHat Package Manager archive (RPM)",
        magic: "RPM",
        group: "archive",
        mime: "application/x-rpm",
        extension: &["rpm"],
        is_text: false,
    },
    Metadata {
        code: "rst",
        desc: "ReStructuredText document",
        magic: "ReStructuredText file",
        group: "text",
        mime: "text/x-rst",
        extension: &["rst"],
        is_text: true,
    },
    Metadata {
        code: "rtf",
        desc: "Rich Text Format document",
        magic: "Rich Text Format data",
        group: "text",
        mime: "text/rtf",
        extension: &["rtf"],
        is_text: true,
    },
    Metadata {
        code: "ruby",
        desc: "Ruby source",
        magic: "Ruby script",
        group: "code",
        mime: "application/x-ruby",
        extension: &["rb"],
        is_text: true,
    },
    Metadata {
        code: "rust",
        desc: "Rust source",
        magic: "ASCII text",
        group: "code",
        mime: "application/x-rust",
        extension: &["rs"],
        is_text: true,
    },
    Metadata {
        code: "scala",
        desc: "Scala source",
        magic: "ASCII text",
        group: "code",
        mime: "application/x-scala",
        extension: &["scala"],
        is_text: true,
    },
    Metadata {
        code: "sevenzip",
        desc: "7-zip archive data",
        magic: "7-zip archive data",
        group: "archive",
        mime: "application/x-7z-compressed",
        extension: &["7z"],
        is_text: false,
    },
    Metadata {
        code: "shell",
        desc: "Shell script",
        magic: "shell script",
        group: "code",
        mime: "text/x-shellscript",
        extension: &["sh"],
        is_text: true,
    },
    Metadata {
        code: "smali",
        desc: "Smali source",
        magic: "ASCII text",
        group: "code",
        mime: "application/x-smali",
        extension: &["smali"],
        is_text: true,
    },
    Metadata {
        code: "sql",
        desc: "SQL source",
        magic: "ASCII text",
        group: "code",
        mime: "application/x-sql",
        extension: &["sql"],
        is_text: true,
    },
    Metadata {
        code: "squashfs",
        desc: "Squash filesystem",
        magic: "Squashfs filesystem",
        group: "archive",
        mime: "application/octet-stream",
        extension: &[],
        is_text: false,
    },
    Metadata {
        code: "svg",
        desc: "SVG Scalable Vector Graphics image data",
        magic: "SVG Scalable Vector Graphics image",
        group: "image",
        mime: "image/svg+xml",
        extension: &["svg"],
        is_text: true,
    },
    Metadata {
        code: "swf",
        desc: "Macromedia Flash data",
        magic: "Macromedia Flash data",
        group: "executable",
        mime: "application/x-shockwave-flash",
        extension: &["swf"],
        is_text: false,
    },
    Metadata {
        code: "symlinktext",
        desc: "Symbolic link (textual representation)",
        magic: "ASCII text",
        group: "application",
        mime: "text/plain",
        extension: &[],
        is_text: true,
    },
    Metadata {
        code: "tar",
        desc: "POSIX tar archive",
        magic: "POSIX tar archive",
        group: "archive",
        mime: "application/x-tar",
        extension: &["tar"],
        is_text: false,
    },
    Metadata {
        code: "tga",
        desc: "Targa image data",
        magic: "Targa image data",
        group: "image",
        mime: "image/x-tga",
        extension: &["tga"],
        is_text: false,
    },
    Metadata {
        code: "tiff",
        desc: "TIFF image data",
        magic: "TIFF image data",
        group: "image",
        mime: "image/tiff",
        extension: &["tiff", "tif"],
        is_text: false,
    },
    Metadata {
        code: "torrent",
        desc: "BitTorrent file",
        magic: "BitTorrent file",
        group: "application",
        mime: "application/x-bittorrent",
        extension: &["torrent"],
        is_text: false,
    },
    Metadata {
        code: "ttf",
        desc: "TrueType Font data",
        magic: "TrueType Font data",
        group: "font",
        mime: "font/sfnt",
        extension: &["ttf"],
        is_text: false,
    },
    Metadata {
        code: "txt",
        desc: "Generic text document",
        magic: "ASCII text",
        group: "text",
        mime: "text/plain",
        extension: &["txt"],
        is_text: true,
    },
    Metadata {
        code: "unknown",
        desc: "Unknown binary data",
        magic: "data",
        group: "unknown",
        mime: "application/octet-stream",
        extension: &[],
        is_text: false,
    },
    Metadata {
        code: "vba",
        desc: "MS Visual Basic source (VBA)",
        magic: "ASCII text",
        group: "code",
        mime: "text/vbscript",
        extension: &["vbs"],
        is_text: true,
    },
    Metadata {
        code: "wav",
        desc: "Waveform Audio file (WAV)",
        magic: "RIFF data",
        group: "audio",
        mime: "audio/x-wav",
        extension: &["wav"],
        is_text: false,
    },
    Metadata {
        code: "webm",
        desc: "WebM data",
        magic: "WebM",
        group: "video",
        mime: "video/webm",
        extension: &["webm"],
        is_text: false,
    },
    Metadata {
        code: "webp",
        desc: "WebP data",
        magic: "RIFF data",
        group: "image",
        mime: "image/webp",
        extension: &["webp"],
        is_text: false,
    },
    Metadata {
        code: "winregistry",
        desc: "Windows Registry text",
        magic: "Windows Registry text",
        group: "application",
        mime: "text/x-ms-regedit",
        extension: &["reg"],
        is_text: true,
    },
    Metadata {
        code: "wmf",
        desc: "Windows metafile",
        magic: "Windows metafile",
        group: "image",
        mime: "image/wmf",
        extension: &["wmf"],
        is_text: false,
    },
    Metadata {
        code: "xar",
        desc: "XAR archive compressed data",
        magic: "xar archive compressed",
        group: "archive",
        mime: "application/x-xar",
        extension: &["pkg", "xar"],
        is_text: false,
    },
    Metadata {
        code: "xls",
        desc: "Microsoft Excel CDF document",
        magic: "Composite Document File",
        group: "document",
        mime: "application/vnd.ms-excel",
        extension: &["xls"],
        is_text: false,
    },
    Metadata {
        code: "xlsb",
        desc: "Microsoft Excel 2007+ document (binary format)",
        magic: "Microsoft Excel 2007+",
        group: "document",
        mime: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        extension: &["xlsb"],
        is_text: false,
    },
    Metadata {
        code: "xlsx",
        desc: "Microsoft Excel 2007+ document",
        magic: "Microsoft Excel 2007+",
        group: "document",
        mime: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        extension: &["xlsx", "xlsm"],
        is_text: false,
    },
    Metadata {
        code: "xml",
        desc: "XML document",
        magic: "XML document",
        group: "code",
        mime: "text/xml",
        extension: &["xml"],
        is_text: true,
    },
    Metadata {
        code: "xpi",
        desc: "Compressed installation archive (XPI)",
        magic: "Zip archive data",
        group: "archive",
        mime: "application/zip",
        extension: &["xpi"],
        is_text: false,
    },
    Metadata {
        code: "xz",
        desc: "XZ compressed data",
        magic: "XZ compressed data",
        group: "archive",
        mime: "application/x-xz",
        extension: &["xz"],
        is_text: false,
    },
    Metadata {
        code: "yaml",
        desc: "YAML source",
        magic: "ASCII text",
        group: "code",
        mime: "application/x-yaml",
        extension: &["yml", "yaml"],
        is_text: true,
    },
    Metadata {
        code: "zip",
        desc: "Zip archive data",
        magic: "Zip archive data",
        group: "archive",
        mime: "application/zip",
        extension: &["zip"],
        is_text: false,
    },
    Metadata {
        code: "zlibstream",
        desc: "zlib compressed data",
        magic: "zlib compressed data",
        group: "application",
        mime: "application/zlib",
        extension: &[],
        is_text: false,
    },
];
