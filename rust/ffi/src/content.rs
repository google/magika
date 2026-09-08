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

use std::ptr;

use crate::MagikaTypeInfo;

#[rustfmt::skip] pub(crate) static _3GP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"3gp".as_ptr(),
    mime_type: c"video/3gpp".as_ptr(),
    group: c"video".as_ptr(),
    description: c"3GPP multimedia file".as_ptr(),
    extensions: [c"3gp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ACE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ace".as_ptr(),
    mime_type: c"application/x-ace-compressed".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"ACE archive".as_ptr(),
    extensions: [c"ace".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AI: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ai".as_ptr(),
    mime_type: c"application/pdf".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Adobe Illustrator Artwork".as_ptr(),
    extensions: [c"ai".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AIDL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"aidl".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"unknown".as_ptr(),
    description: c"Android Interface Definition Language".as_ptr(),
    extensions: [c"aidl".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static APK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"apk".as_ptr(),
    mime_type: c"application/vnd.android.package-archive".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Android package".as_ptr(),
    extensions: [c"apk".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static APPLEBPLIST: MagikaTypeInfo = MagikaTypeInfo {
    label: c"applebplist".as_ptr(),
    mime_type: c"application/x-bplist".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Apple binary property list".as_ptr(),
    extensions: [c"bplist".as_ptr(), c"plist".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static APPLEPLIST: MagikaTypeInfo = MagikaTypeInfo {
    label: c"appleplist".as_ptr(),
    mime_type: c"application/x-plist".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Apple property list".as_ptr(),
    extensions: [c"plist".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static ASM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"asm".as_ptr(),
    mime_type: c"text/x-asm".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Assembly".as_ptr(),
    extensions: [c"s".as_ptr(), c"S".as_ptr(), c"asm".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static ASP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"asp".as_ptr(),
    mime_type: c"text/html".as_ptr(),
    group: c"code".as_ptr(),
    description: c"ASP source".as_ptr(),
    extensions: [c"aspx".as_ptr(), c"asp".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static AUTOHOTKEY: MagikaTypeInfo = MagikaTypeInfo {
    label: c"autohotkey".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"code".as_ptr(),
    description: c"AutoHotKey script".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static AUTOIT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"autoit".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"code".as_ptr(),
    description: c"AutoIt script".as_ptr(),
    extensions: [c"au3".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static AWK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"awk".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Awk".as_ptr(),
    extensions: [c"awk".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static BATCH: MagikaTypeInfo = MagikaTypeInfo {
    label: c"batch".as_ptr(),
    mime_type: c"text/x-msdos-batch".as_ptr(),
    group: c"code".as_ptr(),
    description: c"DOS batch file".as_ptr(),
    extensions: [c"bat".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static BAZEL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bazel".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Bazel build file".as_ptr(),
    extensions: [c"bzl".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static BIB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bib".as_ptr(),
    mime_type: c"text/x-bibtex".as_ptr(),
    group: c"text".as_ptr(),
    description: c"BibTeX".as_ptr(),
    extensions: [c"bib".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static BMP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bmp".as_ptr(),
    mime_type: c"image/bmp".as_ptr(),
    group: c"image".as_ptr(),
    description: c"BMP image data".as_ptr(),
    extensions: [c"bmp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BZIP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bzip".as_ptr(),
    mime_type: c"application/x-bzip2".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"bzip2 compressed data".as_ptr(),
    extensions: [c"bz2".as_ptr(), c"tbz2".as_ptr(), c"tar.bz2".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static C: MagikaTypeInfo = MagikaTypeInfo {
    label: c"c".as_ptr(),
    mime_type: c"text/x-c".as_ptr(),
    group: c"code".as_ptr(),
    description: c"C source".as_ptr(),
    extensions: [c"c".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static CAB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cab".as_ptr(),
    mime_type: c"application/vnd.ms-cab-compressed".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Microsoft Cabinet archive data".as_ptr(),
    extensions: [c"cab".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CAT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cat".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Windows Catalog file".as_ptr(),
    extensions: [c"cat".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CHM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"chm".as_ptr(),
    mime_type: c"application/chm".as_ptr(),
    group: c"application".as_ptr(),
    description: c"MS Windows HtmlHelp Data".as_ptr(),
    extensions: [c"chm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CLOJURE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"clojure".as_ptr(),
    mime_type: c"text/x-clojure".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Clojure".as_ptr(),
    extensions: [c"clj".as_ptr(), c"cljs".as_ptr(), c"cljc".as_ptr(), c"cljr".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static CMAKE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cmake".as_ptr(),
    mime_type: c"text/x-cmake".as_ptr(),
    group: c"code".as_ptr(),
    description: c"CMake build file".as_ptr(),
    extensions: [c"cmake".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static COBOL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cobol".as_ptr(),
    mime_type: c"text/x-cobol".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Cobol".as_ptr(),
    extensions: [c"cbl".as_ptr(), c"cob".as_ptr(), c"cpy".as_ptr(), c"CBL".as_ptr(), c"COB".as_ptr(), c"CPY".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static COFF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"coff".as_ptr(),
    mime_type: c"application/x-coff".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Intel 80386 COFF".as_ptr(),
    extensions: [c"obj".as_ptr(), c"o".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static COFFEESCRIPT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"coffeescript".as_ptr(),
    mime_type: c"text/coffeescript".as_ptr(),
    group: c"code".as_ptr(),
    description: c"CoffeeScript".as_ptr(),
    extensions: [c"coffee".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static CPP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cpp".as_ptr(),
    mime_type: c"text/x-c".as_ptr(),
    group: c"code".as_ptr(),
    description: c"C++ source".as_ptr(),
    extensions: [c"cc".as_ptr(), c"cpp".as_ptr(), c"cxx".as_ptr(), c"c++".as_ptr(), c"cppm".as_ptr(), c"ixx".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static CRT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"crt".as_ptr(),
    mime_type: c"application/x-x509-ca-cert".as_ptr(),
    group: c"text".as_ptr(),
    description: c"Certificates (binary format)".as_ptr(),
    extensions: [c"der".as_ptr(), c"cer".as_ptr(), c"crt".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CRX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"crx".as_ptr(),
    mime_type: c"application/x-chrome-extension".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Google Chrome extension".as_ptr(),
    extensions: [c"crx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cs".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"code".as_ptr(),
    description: c"C# source".as_ptr(),
    extensions: [c"cs".as_ptr(), c"csx".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static CSPROJ: MagikaTypeInfo = MagikaTypeInfo {
    label: c"csproj".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"code".as_ptr(),
    description: c".NET project config".as_ptr(),
    extensions: [c"csproj".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static CSS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"css".as_ptr(),
    mime_type: c"text/css".as_ptr(),
    group: c"code".as_ptr(),
    description: c"CSS source".as_ptr(),
    extensions: [c"css".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static CSV: MagikaTypeInfo = MagikaTypeInfo {
    label: c"csv".as_ptr(),
    mime_type: c"text/csv".as_ptr(),
    group: c"code".as_ptr(),
    description: c"CSV document".as_ptr(),
    extensions: [c"csv".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static DART: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dart".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Dart source".as_ptr(),
    extensions: [c"dart".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static DEB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"deb".as_ptr(),
    mime_type: c"application/vnd.debian.binary-package".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Debian binary package".as_ptr(),
    extensions: [c"deb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DEX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dex".as_ptr(),
    mime_type: c"application/x-android-dex".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Dalvik dex file".as_ptr(),
    extensions: [c"dex".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DICOM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dicom".as_ptr(),
    mime_type: c"application/dicom".as_ptr(),
    group: c"image".as_ptr(),
    description: c"DICOM".as_ptr(),
    extensions: [c"dcm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DIFF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"diff".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"text".as_ptr(),
    description: c"Diff file".as_ptr(),
    extensions: [c"diff".as_ptr(), c"patch".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static DIRECTORY: MagikaTypeInfo = MagikaTypeInfo {
    label: c"directory".as_ptr(),
    mime_type: c"inode/directory".as_ptr(),
    group: c"inode".as_ptr(),
    description: c"A directory".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dm".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Dream Maker".as_ptr(),
    extensions: [c"dm".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static DMG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dmg".as_ptr(),
    mime_type: c"application/x-apple-diskimage".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Apple disk image".as_ptr(),
    extensions: [c"dmg".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DOC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"doc".as_ptr(),
    mime_type: c"application/msword".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Microsoft Word CDF document".as_ptr(),
    extensions: [c"doc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DOCKERFILE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dockerfile".as_ptr(),
    mime_type: c"text/x-dockerfile".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Dockerfile".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static DOCX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"docx".as_ptr(),
    mime_type: c"application/vnd.openxmlformats-officedocument.wordprocessingml.document".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Microsoft Word 2007+ document".as_ptr(),
    extensions: [c"docx".as_ptr(), c"docm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DSSTORE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dsstore".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"unknown".as_ptr(),
    description: c"Application Desktop Services Store".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DWG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dwg".as_ptr(),
    mime_type: c"image/x-dwg".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Autocad Drawing".as_ptr(),
    extensions: [c"dwg".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DXF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dxf".as_ptr(),
    mime_type: c"image/vnd.dxf".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Audocad Drawing Exchange Format".as_ptr(),
    extensions: [c"dxf".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static ELF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"elf".as_ptr(),
    mime_type: c"application/x-executable-elf".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"ELF executable".as_ptr(),
    extensions: [c"elf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ELIXIR: MagikaTypeInfo = MagikaTypeInfo {
    label: c"elixir".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Elixir script".as_ptr(),
    extensions: [c"exs".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static EMF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"emf".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Windows Enhanced Metafile image data".as_ptr(),
    extensions: [c"emf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static EML: MagikaTypeInfo = MagikaTypeInfo {
    label: c"eml".as_ptr(),
    mime_type: c"message/rfc822".as_ptr(),
    group: c"text".as_ptr(),
    description: c"RFC 822 mail".as_ptr(),
    extensions: [c"eml".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static EMPTY: MagikaTypeInfo = MagikaTypeInfo {
    label: c"empty".as_ptr(),
    mime_type: c"inode/x-empty".as_ptr(),
    group: c"inode".as_ptr(),
    description: c"Empty file".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static EPUB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"epub".as_ptr(),
    mime_type: c"application/epub+zip".as_ptr(),
    group: c"document".as_ptr(),
    description: c"EPUB document".as_ptr(),
    extensions: [c"epub".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ERB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"erb".as_ptr(),
    mime_type: c"text/x-ruby".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Embedded Ruby source".as_ptr(),
    extensions: [c"erb".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static ERLANG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"erlang".as_ptr(),
    mime_type: c"text/x-erlang".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Erlang source".as_ptr(),
    extensions: [c"erl".as_ptr(), c"hrl".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static FLAC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"flac".as_ptr(),
    mime_type: c"audio/flac".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"FLAC audio bitstream data".as_ptr(),
    extensions: [c"flac".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FLV: MagikaTypeInfo = MagikaTypeInfo {
    label: c"flv".as_ptr(),
    mime_type: c"video/x-flv".as_ptr(),
    group: c"video".as_ptr(),
    description: c"Flash Video".as_ptr(),
    extensions: [c"flv".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FORTRAN: MagikaTypeInfo = MagikaTypeInfo {
    label: c"fortran".as_ptr(),
    mime_type: c"text/x-fortran".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Fortran".as_ptr(),
    extensions: [c"f90".as_ptr(), c"f95".as_ptr(), c"f03".as_ptr(), c"F90".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static GEMFILE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"gemfile".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Gemfile file".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static GEMSPEC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"gemspec".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Gemspec file".as_ptr(),
    extensions: [c"gemspec".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static GIF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"gif".as_ptr(),
    mime_type: c"image/gif".as_ptr(),
    group: c"image".as_ptr(),
    description: c"GIF image data".as_ptr(),
    extensions: [c"gif".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GITATTRIBUTES: MagikaTypeInfo = MagikaTypeInfo {
    label: c"gitattributes".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Gitattributes file".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static GITMODULES: MagikaTypeInfo = MagikaTypeInfo {
    label: c"gitmodules".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Gitmodules file".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static GO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"go".as_ptr(),
    mime_type: c"text/x-golang".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Golang source".as_ptr(),
    extensions: [c"go".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static GRADLE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"gradle".as_ptr(),
    mime_type: c"text/x-groovy".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Gradle source".as_ptr(),
    extensions: [c"gradle".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static GROOVY: MagikaTypeInfo = MagikaTypeInfo {
    label: c"groovy".as_ptr(),
    mime_type: c"text/x-groovy".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Groovy source".as_ptr(),
    extensions: [c"groovy".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static GZIP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"gzip".as_ptr(),
    mime_type: c"application/gzip".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"gzip compressed data".as_ptr(),
    extensions: [c"gz".as_ptr(), c"gzip".as_ptr(), c"tgz".as_ptr(), c"tar.gz".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static H5: MagikaTypeInfo = MagikaTypeInfo {
    label: c"h5".as_ptr(),
    mime_type: c"application/x-hdf5".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Hierarchical Data Format v5".as_ptr(),
    extensions: [c"h5".as_ptr(), c"hdf5".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HANDLEBARS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"handlebars".as_ptr(),
    mime_type: c"text/x-handlebars-template".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Handlebars source".as_ptr(),
    extensions: [c"hbs".as_ptr(), c"handlebars".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static HASKELL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"haskell".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Haskell source".as_ptr(),
    extensions: [c"hs".as_ptr(), c"lhs".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static HCL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"hcl".as_ptr(),
    mime_type: c"text/x-hcl".as_ptr(),
    group: c"code".as_ptr(),
    description: c"HashiCorp configuration language".as_ptr(),
    extensions: [c"hcl".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static HLP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"hlp".as_ptr(),
    mime_type: c"application/winhlp".as_ptr(),
    group: c"application".as_ptr(),
    description: c"MS Windows help".as_ptr(),
    extensions: [c"hlp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HTACCESS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"htaccess".as_ptr(),
    mime_type: c"text/x-apache-conf".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Apache access configuration".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static HTML: MagikaTypeInfo = MagikaTypeInfo {
    label: c"html".as_ptr(),
    mime_type: c"text/html".as_ptr(),
    group: c"code".as_ptr(),
    description: c"HTML document".as_ptr(),
    extensions: [c"html".as_ptr(), c"htm".as_ptr(), c"xhtml".as_ptr(), c"xht".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static ICNS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"icns".as_ptr(),
    mime_type: c"image/x-icns".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Mac OS X icon".as_ptr(),
    extensions: [c"icns".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ICO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ico".as_ptr(),
    mime_type: c"image/vnd.microsoft.icon".as_ptr(),
    group: c"image".as_ptr(),
    description: c"MS Windows icon resource".as_ptr(),
    extensions: [c"ico".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ICS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ics".as_ptr(),
    mime_type: c"text/calendar".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Internet Calendaring and Scheduling".as_ptr(),
    extensions: [c"ics".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static IGNOREFILE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ignorefile".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Ignorefile".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static INI: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ini".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"text".as_ptr(),
    description: c"INI configuration file".as_ptr(),
    extensions: [c"ini".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static INTERNETSHORTCUT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"internetshortcut".as_ptr(),
    mime_type: c"application/x-mswinurl".as_ptr(),
    group: c"application".as_ptr(),
    description: c"MS Windows Internet shortcut".as_ptr(),
    extensions: [c"url".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static IPYNB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ipynb".as_ptr(),
    mime_type: c"application/json".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Jupyter notebook".as_ptr(),
    extensions: [c"ipynb".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static ISO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"iso".as_ptr(),
    mime_type: c"application/x-iso9660-image".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"ISO 9660 CD-ROM filesystem data".as_ptr(),
    extensions: [c"iso".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static JAR: MagikaTypeInfo = MagikaTypeInfo {
    label: c"jar".as_ptr(),
    mime_type: c"application/java-archive".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Java archive data (JAR)".as_ptr(),
    extensions: [c"jar".as_ptr(), c"klib".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static JAVA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"java".as_ptr(),
    mime_type: c"text/x-java".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Java source".as_ptr(),
    extensions: [c"java".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static JAVABYTECODE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"javabytecode".as_ptr(),
    mime_type: c"application/x-java-applet".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Java compiled bytecode".as_ptr(),
    extensions: [c"class".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static JAVASCRIPT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"javascript".as_ptr(),
    mime_type: c"application/javascript".as_ptr(),
    group: c"code".as_ptr(),
    description: c"JavaScript source".as_ptr(),
    extensions: [c"js".as_ptr(), c"mjs".as_ptr(), c"cjs".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static JINJA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"jinja".as_ptr(),
    mime_type: c"text/x-jinja2-template".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Jinja template".as_ptr(),
    extensions: [c"jinja".as_ptr(), c"jinja2".as_ptr(), c"j2".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static JP2: MagikaTypeInfo = MagikaTypeInfo {
    label: c"jp2".as_ptr(),
    mime_type: c"image/jpeg2000".as_ptr(),
    group: c"image".as_ptr(),
    description: c"jpeg2000".as_ptr(),
    extensions: [c"jp2".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static JPEG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"jpeg".as_ptr(),
    mime_type: c"image/jpeg".as_ptr(),
    group: c"image".as_ptr(),
    description: c"JPEG image data".as_ptr(),
    extensions: [c"jpg".as_ptr(), c"jpeg".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static JSON: MagikaTypeInfo = MagikaTypeInfo {
    label: c"json".as_ptr(),
    mime_type: c"application/json".as_ptr(),
    group: c"code".as_ptr(),
    description: c"JSON document".as_ptr(),
    extensions: [c"json".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static JSONL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"jsonl".as_ptr(),
    mime_type: c"application/json".as_ptr(),
    group: c"code".as_ptr(),
    description: c"JSONL document".as_ptr(),
    extensions: [c"jsonl".as_ptr(), c"jsonld".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static JULIA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"julia".as_ptr(),
    mime_type: c"text/x-julia".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Julia source".as_ptr(),
    extensions: [c"jl".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static KOTLIN: MagikaTypeInfo = MagikaTypeInfo {
    label: c"kotlin".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Kotlin source".as_ptr(),
    extensions: [c"kt".as_ptr(), c"kts".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static LATEX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"latex".as_ptr(),
    mime_type: c"text/x-tex".as_ptr(),
    group: c"text".as_ptr(),
    description: c"LaTeX document".as_ptr(),
    extensions: [c"tex".as_ptr(), c"sty".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static LHA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"lha".as_ptr(),
    mime_type: c"application/x-lha".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"LHarc archive".as_ptr(),
    extensions: [c"lha".as_ptr(), c"lzh".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LISP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"lisp".as_ptr(),
    mime_type: c"text/x-lisp".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Lisp source".as_ptr(),
    extensions: [c"lisp".as_ptr(), c"lsp".as_ptr(), c"l".as_ptr(), c"cl".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static LNK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"lnk".as_ptr(),
    mime_type: c"application/x-ms-shortcut".as_ptr(),
    group: c"application".as_ptr(),
    description: c"MS Windows shortcut".as_ptr(),
    extensions: [c"lnk".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LUA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"lua".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Lua".as_ptr(),
    extensions: [c"lua".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static M3U: MagikaTypeInfo = MagikaTypeInfo {
    label: c"m3u".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"application".as_ptr(),
    description: c"M3U playlist".as_ptr(),
    extensions: [c"m3u8".as_ptr(), c"m3u".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static M4: MagikaTypeInfo = MagikaTypeInfo {
    label: c"m4".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"code".as_ptr(),
    description: c"GNU Macro".as_ptr(),
    extensions: [c"m4".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static MACHO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"macho".as_ptr(),
    mime_type: c"application/x-mach-o".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Mach-O executable".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MAKEFILE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"makefile".as_ptr(),
    mime_type: c"text/x-makefile".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Makefile source".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static MARKDOWN: MagikaTypeInfo = MagikaTypeInfo {
    label: c"markdown".as_ptr(),
    mime_type: c"text/markdown".as_ptr(),
    group: c"text".as_ptr(),
    description: c"Markdown document".as_ptr(),
    extensions: [c"md".as_ptr(), c"markdown".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static MATLAB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"matlab".as_ptr(),
    mime_type: c"text/x-matlab".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Matlab Source".as_ptr(),
    extensions: [c"m".as_ptr(), c"matlab".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static MHT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mht".as_ptr(),
    mime_type: c"application/x-mimearchive".as_ptr(),
    group: c"code".as_ptr(),
    description: c"MHTML document".as_ptr(),
    extensions: [c"mht".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static MIDI: MagikaTypeInfo = MagikaTypeInfo {
    label: c"midi".as_ptr(),
    mime_type: c"audio/midi".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Midi".as_ptr(),
    extensions: [c"mid".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MKV: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mkv".as_ptr(),
    mime_type: c"video/x-matroska".as_ptr(),
    group: c"video".as_ptr(),
    description: c"Matroska".as_ptr(),
    extensions: [c"mkv".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MP3: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mp3".as_ptr(),
    mime_type: c"audio/mpeg".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"MP3 media file".as_ptr(),
    extensions: [c"mp3".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MP4: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mp4".as_ptr(),
    mime_type: c"video/mp4".as_ptr(),
    group: c"video".as_ptr(),
    description: c"MP4 media file".as_ptr(),
    extensions: [c"mp4".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MSCOMPRESS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mscompress".as_ptr(),
    mime_type: c"application/x-ms-compress-szdd".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"MS Compress archive data".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MSI: MagikaTypeInfo = MagikaTypeInfo {
    label: c"msi".as_ptr(),
    mime_type: c"application/x-msi".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Microsoft Installer file".as_ptr(),
    extensions: [c"msi".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MUM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mum".as_ptr(),
    mime_type: c"text/xml".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Windows Update Package file".as_ptr(),
    extensions: [c"mum".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static NPY: MagikaTypeInfo = MagikaTypeInfo {
    label: c"npy".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Numpy Array".as_ptr(),
    extensions: [c"npy".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NPZ: MagikaTypeInfo = MagikaTypeInfo {
    label: c"npz".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Numpy Arrays Archive".as_ptr(),
    extensions: [c"npz".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NUPKG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"nupkg".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"unknown".as_ptr(),
    description: c"NuGet Package".as_ptr(),
    extensions: [c"nupkg".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static OBJECTIVEC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"objectivec".as_ptr(),
    mime_type: c"text/x-objcsrc".as_ptr(),
    group: c"code".as_ptr(),
    description: c"ObjectiveC source".as_ptr(),
    extensions: [c"m".as_ptr(), c"mm".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static OCAML: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ocaml".as_ptr(),
    mime_type: c"text-ocaml".as_ptr(),
    group: c"code".as_ptr(),
    description: c"OCaml".as_ptr(),
    extensions: [c"ml".as_ptr(), c"mli".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static ODP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"odp".as_ptr(),
    mime_type: c"application/vnd.oasis.opendocument.presentation".as_ptr(),
    group: c"document".as_ptr(),
    description: c"OpenDocument Presentation".as_ptr(),
    extensions: [c"odp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ODS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ods".as_ptr(),
    mime_type: c"application/vnd.oasis.opendocument.spreadsheet".as_ptr(),
    group: c"document".as_ptr(),
    description: c"OpenDocument Spreadsheet".as_ptr(),
    extensions: [c"ods".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ODT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"odt".as_ptr(),
    mime_type: c"application/vnd.oasis.opendocument.text".as_ptr(),
    group: c"document".as_ptr(),
    description: c"OpenDocument Text".as_ptr(),
    extensions: [c"odt".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static OGG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ogg".as_ptr(),
    mime_type: c"audio/ogg".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Ogg data".as_ptr(),
    extensions: [c"ogg".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ONE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"one".as_ptr(),
    mime_type: c"application/msonenote".as_ptr(),
    group: c"document".as_ptr(),
    description: c"One Note".as_ptr(),
    extensions: [c"one".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ONNX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"onnx".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Open Neural Network Exchange".as_ptr(),
    extensions: [c"onnx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static OTF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"otf".as_ptr(),
    mime_type: c"font/otf".as_ptr(),
    group: c"font".as_ptr(),
    description: c"OpenType font".as_ptr(),
    extensions: [c"otf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static OUTLOOK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"outlook".as_ptr(),
    mime_type: c"application/vnd.ms-outlook".as_ptr(),
    group: c"application".as_ptr(),
    description: c"MS Outlook Message".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PARQUET: MagikaTypeInfo = MagikaTypeInfo {
    label: c"parquet".as_ptr(),
    mime_type: c"application/vnd.apache.parquet".as_ptr(),
    group: c"unknown".as_ptr(),
    description: c"Apache Parquet".as_ptr(),
    extensions: [c"pqt".as_ptr(), c"parquet".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PASCAL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pascal".as_ptr(),
    mime_type: c"text/x-pascal".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Pascal source".as_ptr(),
    extensions: [c"pas".as_ptr(), c"pp".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static PCAP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pcap".as_ptr(),
    mime_type: c"application/vnd.tcpdump.pcap".as_ptr(),
    group: c"application".as_ptr(),
    description: c"pcap capture file".as_ptr(),
    extensions: [c"pcap".as_ptr(), c"pcapng".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PDB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pdb".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Windows Program Database".as_ptr(),
    extensions: [c"pdb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PDF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pdf".as_ptr(),
    mime_type: c"application/pdf".as_ptr(),
    group: c"document".as_ptr(),
    description: c"PDF document".as_ptr(),
    extensions: [c"pdf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PEBIN: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pebin".as_ptr(),
    mime_type: c"application/x-dosexec".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"PE Windows executable".as_ptr(),
    extensions: [c"exe".as_ptr(), c"dll".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PEM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pem".as_ptr(),
    mime_type: c"application/x-pem-file".as_ptr(),
    group: c"application".as_ptr(),
    description: c"PEM certificate".as_ptr(),
    extensions: [c"pem".as_ptr(), c"pub".as_ptr(), c"gpg".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static PERL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"perl".as_ptr(),
    mime_type: c"text/x-perl".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Perl source".as_ptr(),
    extensions: [c"pl".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static PHP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"php".as_ptr(),
    mime_type: c"text/x-php".as_ptr(),
    group: c"code".as_ptr(),
    description: c"PHP source".as_ptr(),
    extensions: [c"php".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static PICKLE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pickle".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Python pickle".as_ptr(),
    extensions: [c"pickle".as_ptr(), c"pkl".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PNG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"png".as_ptr(),
    mime_type: c"image/png".as_ptr(),
    group: c"image".as_ptr(),
    description: c"PNG image".as_ptr(),
    extensions: [c"png".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"po".as_ptr(),
    mime_type: c"text/gettext-translation".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Portable Object (PO) for i18n".as_ptr(),
    extensions: [c"po".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static POSTSCRIPT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"postscript".as_ptr(),
    mime_type: c"application/postscript".as_ptr(),
    group: c"document".as_ptr(),
    description: c"PostScript document".as_ptr(),
    extensions: [c"ps".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static POWERSHELL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"powershell".as_ptr(),
    mime_type: c"application/x-powershell".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Powershell source".as_ptr(),
    extensions: [c"ps1".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static PPT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ppt".as_ptr(),
    mime_type: c"application/vnd.ms-powerpoint".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Microsoft PowerPoint CDF document".as_ptr(),
    extensions: [c"ppt".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PPTX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pptx".as_ptr(),
    mime_type: c"application/vnd.openxmlformats-officedocument.presentationml.presentation".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Microsoft PowerPoint 2007+ document".as_ptr(),
    extensions: [c"pptx".as_ptr(), c"pptm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PROLOG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"prolog".as_ptr(),
    mime_type: c"text/x-prolog".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Prolog source".as_ptr(),
    extensions: [c"pl".as_ptr(), c"pro".as_ptr(), c"P".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static PROTEINDB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"proteindb".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Protein DB".as_ptr(),
    extensions: [c"pdb".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static PROTO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"proto".as_ptr(),
    mime_type: c"text/x-proto".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Protocol buffer definition".as_ptr(),
    extensions: [c"proto".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static PSD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"psd".as_ptr(),
    mime_type: c"image/vnd.adobe.photoshop".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Adobe Photoshop".as_ptr(),
    extensions: [c"psd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PYTHON: MagikaTypeInfo = MagikaTypeInfo {
    label: c"python".as_ptr(),
    mime_type: c"text/x-python".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Python source".as_ptr(),
    extensions: [c"py".as_ptr(), c"pyi".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static PYTHONBYTECODE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pythonbytecode".as_ptr(),
    mime_type: c"application/x-bytecode.python".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Python compiled bytecode".as_ptr(),
    extensions: [c"pyc".as_ptr(), c"pyo".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PYTORCH: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pytorch".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Pytorch storage file".as_ptr(),
    extensions: [c"pt".as_ptr(), c"pth".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static QT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"qt".as_ptr(),
    mime_type: c"video/quicktime".as_ptr(),
    group: c"video".as_ptr(),
    description: c"QuickTime".as_ptr(),
    extensions: [c"mov".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static R: MagikaTypeInfo = MagikaTypeInfo {
    label: c"r".as_ptr(),
    mime_type: c"text/x-R".as_ptr(),
    group: c"code".as_ptr(),
    description: c"R (language)".as_ptr(),
    extensions: [c"R".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static RANDOMBYTES: MagikaTypeInfo = MagikaTypeInfo {
    label: c"randombytes".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"unknown".as_ptr(),
    description: c"Random bytes".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RANDOMTXT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"randomtxt".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"text".as_ptr(),
    description: c"Random text".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static RAR: MagikaTypeInfo = MagikaTypeInfo {
    label: c"rar".as_ptr(),
    mime_type: c"application/vnd.rar".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"RAR archive data".as_ptr(),
    extensions: [c"rar".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RDF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"rdf".as_ptr(),
    mime_type: c"application/rdf+xml".as_ptr(),
    group: c"text".as_ptr(),
    description: c"Resource Description Framework document (RDF)".as_ptr(),
    extensions: [c"rdf".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static RPM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"rpm".as_ptr(),
    mime_type: c"application/x-rpm".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"RedHat Package Manager archive (RPM)".as_ptr(),
    extensions: [c"rpm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RST: MagikaTypeInfo = MagikaTypeInfo {
    label: c"rst".as_ptr(),
    mime_type: c"text/x-rst".as_ptr(),
    group: c"text".as_ptr(),
    description: c"ReStructuredText document".as_ptr(),
    extensions: [c"rst".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static RTF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"rtf".as_ptr(),
    mime_type: c"text/rtf".as_ptr(),
    group: c"text".as_ptr(),
    description: c"Rich Text Format document".as_ptr(),
    extensions: [c"rtf".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static RUBY: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ruby".as_ptr(),
    mime_type: c"application/x-ruby".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Ruby source".as_ptr(),
    extensions: [c"rb".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static RUST: MagikaTypeInfo = MagikaTypeInfo {
    label: c"rust".as_ptr(),
    mime_type: c"application/x-rust".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Rust source".as_ptr(),
    extensions: [c"rs".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static SCALA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"scala".as_ptr(),
    mime_type: c"application/x-scala".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Scala source".as_ptr(),
    extensions: [c"scala".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static SCSS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"scss".as_ptr(),
    mime_type: c"text/x-scss".as_ptr(),
    group: c"code".as_ptr(),
    description: c"SCSS source".as_ptr(),
    extensions: [c"scss".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static SEVENZIP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sevenzip".as_ptr(),
    mime_type: c"application/x-7z-compressed".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"7-zip archive data".as_ptr(),
    extensions: [c"7z".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SGML: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sgml".as_ptr(),
    mime_type: c"application/sgml".as_ptr(),
    group: c"text".as_ptr(),
    description: c"sgml".as_ptr(),
    extensions: [c"sgml".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static SHELL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"shell".as_ptr(),
    mime_type: c"text/x-shellscript".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Shell script".as_ptr(),
    extensions: [c"sh".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static SMALI: MagikaTypeInfo = MagikaTypeInfo {
    label: c"smali".as_ptr(),
    mime_type: c"application/x-smali".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Smali source".as_ptr(),
    extensions: [c"smali".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static SNAP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"snap".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Snap archive".as_ptr(),
    extensions: [c"snap".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SOLIDITY: MagikaTypeInfo = MagikaTypeInfo {
    label: c"solidity".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Solidity source".as_ptr(),
    extensions: [c"sol".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static SQL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sql".as_ptr(),
    mime_type: c"application/x-sql".as_ptr(),
    group: c"code".as_ptr(),
    description: c"SQL source".as_ptr(),
    extensions: [c"sql".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static SQLITE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sqlite".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"application".as_ptr(),
    description: c"SQLITE database".as_ptr(),
    extensions: [c"sqlite".as_ptr(), c"sqlite3".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SQUASHFS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"squashfs".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Squash filesystem".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SRT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"srt".as_ptr(),
    mime_type: c"text/srt".as_ptr(),
    group: c"application".as_ptr(),
    description: c"SubRip Text Format".as_ptr(),
    extensions: [c"srt".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static STLBINARY: MagikaTypeInfo = MagikaTypeInfo {
    label: c"stlbinary".as_ptr(),
    mime_type: c"application/sla".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Stereolithography CAD (binary)".as_ptr(),
    extensions: [c"stl".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static STLTEXT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"stltext".as_ptr(),
    mime_type: c"application/sla".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Stereolithography CAD (text)".as_ptr(),
    extensions: [c"stl".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static SUM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sum".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Checksum file".as_ptr(),
    extensions: [c"sum".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static SVG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"svg".as_ptr(),
    mime_type: c"image/svg+xml".as_ptr(),
    group: c"image".as_ptr(),
    description: c"SVG Scalable Vector Graphics image data".as_ptr(),
    extensions: [c"svg".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static SWF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"swf".as_ptr(),
    mime_type: c"application/x-shockwave-flash".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Small Web File".as_ptr(),
    extensions: [c"swf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SWIFT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"swift".as_ptr(),
    mime_type: c"text/x-swift".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Swift".as_ptr(),
    extensions: [c"swift".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static SYMLINK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"symlink".as_ptr(),
    mime_type: c"inode/symlink".as_ptr(),
    group: c"inode".as_ptr(),
    description: c"Symbolic link".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TAR: MagikaTypeInfo = MagikaTypeInfo {
    label: c"tar".as_ptr(),
    mime_type: c"application/x-tar".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"POSIX tar archive".as_ptr(),
    extensions: [c"tar".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TCL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"tcl".as_ptr(),
    mime_type: c"application/x-tcl".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Tickle".as_ptr(),
    extensions: [c"tcl".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static TEXTPROTO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"textproto".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Text protocol buffer".as_ptr(),
    extensions: [c"textproto".as_ptr(), c"textpb".as_ptr(), c"pbtxt".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static TGA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"tga".as_ptr(),
    mime_type: c"image/x-tga".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Targa image data".as_ptr(),
    extensions: [c"tga".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static THUMBSDB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"thumbsdb".as_ptr(),
    mime_type: c"image/vnd.ms-thumb".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Windows thumbnail cache".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TIFF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"tiff".as_ptr(),
    mime_type: c"image/tiff".as_ptr(),
    group: c"image".as_ptr(),
    description: c"TIFF image data".as_ptr(),
    extensions: [c"tiff".as_ptr(), c"tif".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TOML: MagikaTypeInfo = MagikaTypeInfo {
    label: c"toml".as_ptr(),
    mime_type: c"application/toml".as_ptr(),
    group: c"text".as_ptr(),
    description: c"Tom's obvious, minimal language".as_ptr(),
    extensions: [c"toml".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static TORRENT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"torrent".as_ptr(),
    mime_type: c"application/x-bittorrent".as_ptr(),
    group: c"application".as_ptr(),
    description: c"BitTorrent file".as_ptr(),
    extensions: [c"torrent".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TSV: MagikaTypeInfo = MagikaTypeInfo {
    label: c"tsv".as_ptr(),
    mime_type: c"text/tsv".as_ptr(),
    group: c"code".as_ptr(),
    description: c"TSV document".as_ptr(),
    extensions: [c"tsv".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static TTF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ttf".as_ptr(),
    mime_type: c"font/sfnt".as_ptr(),
    group: c"font".as_ptr(),
    description: c"TrueType Font data".as_ptr(),
    extensions: [c"ttf".as_ptr(), c"ttc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TWIG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"twig".as_ptr(),
    mime_type: c"text/x-twig".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Twig template".as_ptr(),
    extensions: [c"twig".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static TXT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"txt".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"text".as_ptr(),
    description: c"Generic text document".as_ptr(),
    extensions: [c"txt".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static TYPESCRIPT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"typescript".as_ptr(),
    mime_type: c"application/typescript".as_ptr(),
    group: c"code".as_ptr(),
    description: c"TypeScript source".as_ptr(),
    extensions: [c"ts".as_ptr(), c"mts".as_ptr(), c"cts".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static UNDEFINED: MagikaTypeInfo = MagikaTypeInfo {
    label: c"undefined".as_ptr(),
    mime_type: c"application/undefined".as_ptr(),
    group: c"undefined".as_ptr(),
    description: c"Undefined".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static UNKNOWN: MagikaTypeInfo = MagikaTypeInfo {
    label: c"unknown".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"unknown".as_ptr(),
    description: c"Unknown binary data".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static VBA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"vba".as_ptr(),
    mime_type: c"text/vbscript".as_ptr(),
    group: c"code".as_ptr(),
    description: c"MS Visual Basic source (VBA)".as_ptr(),
    extensions: [c"vbs".as_ptr(), c"vba".as_ptr(), c"vb".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static VCXPROJ: MagikaTypeInfo = MagikaTypeInfo {
    label: c"vcxproj".as_ptr(),
    mime_type: c"application/xml".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Visual Studio MSBuild project".as_ptr(),
    extensions: [c"vcxproj".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static VERILOG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"verilog".as_ptr(),
    mime_type: c"text/x-verilog".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Verilog source".as_ptr(),
    extensions: [c"v".as_ptr(), c"verilog".as_ptr(), c"vlg".as_ptr(), c"vh".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static VHDL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"vhdl".as_ptr(),
    mime_type: c"text/x-vhdl".as_ptr(),
    group: c"code".as_ptr(),
    description: c"VHDL source".as_ptr(),
    extensions: [c"vhd".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static VTT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"vtt".as_ptr(),
    mime_type: c"text/vtt".as_ptr(),
    group: c"text".as_ptr(),
    description: c"Web Video Text Tracks".as_ptr(),
    extensions: [c"vtt".as_ptr(), c"webvtt".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static VUE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"vue".as_ptr(),
    mime_type: c"application/javascript".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Vue source".as_ptr(),
    extensions: [c"vue".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static WASM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"wasm".as_ptr(),
    mime_type: c"application/wasm".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Web Assembly".as_ptr(),
    extensions: [c"wasm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WAV: MagikaTypeInfo = MagikaTypeInfo {
    label: c"wav".as_ptr(),
    mime_type: c"audio/x-wav".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Waveform Audio file (WAV)".as_ptr(),
    extensions: [c"wav".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WEBM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"webm".as_ptr(),
    mime_type: c"video/webm".as_ptr(),
    group: c"video".as_ptr(),
    description: c"WebM media file".as_ptr(),
    extensions: [c"webm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WEBP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"webp".as_ptr(),
    mime_type: c"image/webp".as_ptr(),
    group: c"image".as_ptr(),
    description: c"WebP media file".as_ptr(),
    extensions: [c"webp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WINREGISTRY: MagikaTypeInfo = MagikaTypeInfo {
    label: c"winregistry".as_ptr(),
    mime_type: c"text/x-ms-regedit".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Windows Registry text".as_ptr(),
    extensions: [c"reg".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static WMF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"wmf".as_ptr(),
    mime_type: c"image/wmf".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Windows metafile".as_ptr(),
    extensions: [c"wmf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WOFF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"woff".as_ptr(),
    mime_type: c"font/woff".as_ptr(),
    group: c"font".as_ptr(),
    description: c"Web Open Font Format".as_ptr(),
    extensions: [c"woff".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WOFF2: MagikaTypeInfo = MagikaTypeInfo {
    label: c"woff2".as_ptr(),
    mime_type: c"font/woff2".as_ptr(),
    group: c"font".as_ptr(),
    description: c"Web Open Font Format v2".as_ptr(),
    extensions: [c"woff2".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static XAR: MagikaTypeInfo = MagikaTypeInfo {
    label: c"xar".as_ptr(),
    mime_type: c"application/x-xar".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"XAR archive compressed data".as_ptr(),
    extensions: [c"pkg".as_ptr(), c"xar".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static XLS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"xls".as_ptr(),
    mime_type: c"application/vnd.ms-excel".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Microsoft Excel CDF document".as_ptr(),
    extensions: [c"xls".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static XLSB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"xlsb".as_ptr(),
    mime_type: c"application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Microsoft Excel 2007+ document (binary format)".as_ptr(),
    extensions: [c"xlsb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static XLSX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"xlsx".as_ptr(),
    mime_type: c"application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Microsoft Excel 2007+ document".as_ptr(),
    extensions: [c"xlsx".as_ptr(), c"xlsm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static XML: MagikaTypeInfo = MagikaTypeInfo {
    label: c"xml".as_ptr(),
    mime_type: c"text/xml".as_ptr(),
    group: c"code".as_ptr(),
    description: c"XML document".as_ptr(),
    extensions: [c"xml".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static XPI: MagikaTypeInfo = MagikaTypeInfo {
    label: c"xpi".as_ptr(),
    mime_type: c"application/zip".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Compressed installation archive (XPI)".as_ptr(),
    extensions: [c"xpi".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static XZ: MagikaTypeInfo = MagikaTypeInfo {
    label: c"xz".as_ptr(),
    mime_type: c"application/x-xz".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"XZ compressed data".as_ptr(),
    extensions: [c"xz".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static YAML: MagikaTypeInfo = MagikaTypeInfo {
    label: c"yaml".as_ptr(),
    mime_type: c"application/x-yaml".as_ptr(),
    group: c"code".as_ptr(),
    description: c"YAML source".as_ptr(),
    extensions: [c"yml".as_ptr(), c"yaml".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static YARA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"yara".as_ptr(),
    mime_type: c"text/x-yara".as_ptr(),
    group: c"code".as_ptr(),
    description: c"YARA rule".as_ptr(),
    extensions: [c"yar".as_ptr(), c"yara".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static ZIG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"zig".as_ptr(),
    mime_type: c"text/zig".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Zig source".as_ptr(),
    extensions: [c"zig".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static ZIP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"zip".as_ptr(),
    mime_type: c"application/zip".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Zip archive data".as_ptr(),
    extensions: [c"zip".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ZLIBSTREAM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"zlibstream".as_ptr(),
    mime_type: c"application/zlib".as_ptr(),
    group: c"application".as_ptr(),
    description: c"zlib compressed data".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

pub(crate) fn content_type_info(content_type: magika::ContentType) -> &'static MagikaTypeInfo {
    match content_type {
        magika::ContentType::_3gp => &_3GP,
        magika::ContentType::Ace => &ACE,
        magika::ContentType::Ai => &AI,
        magika::ContentType::Aidl => &AIDL,
        magika::ContentType::Apk => &APK,
        magika::ContentType::Applebplist => &APPLEBPLIST,
        magika::ContentType::Appleplist => &APPLEPLIST,
        magika::ContentType::Asm => &ASM,
        magika::ContentType::Asp => &ASP,
        magika::ContentType::Autohotkey => &AUTOHOTKEY,
        magika::ContentType::Autoit => &AUTOIT,
        magika::ContentType::Awk => &AWK,
        magika::ContentType::Batch => &BATCH,
        magika::ContentType::Bazel => &BAZEL,
        magika::ContentType::Bib => &BIB,
        magika::ContentType::Bmp => &BMP,
        magika::ContentType::Bzip => &BZIP,
        magika::ContentType::C => &C,
        magika::ContentType::Cab => &CAB,
        magika::ContentType::Cat => &CAT,
        magika::ContentType::Chm => &CHM,
        magika::ContentType::Clojure => &CLOJURE,
        magika::ContentType::Cmake => &CMAKE,
        magika::ContentType::Cobol => &COBOL,
        magika::ContentType::Coff => &COFF,
        magika::ContentType::Coffeescript => &COFFEESCRIPT,
        magika::ContentType::Cpp => &CPP,
        magika::ContentType::Crt => &CRT,
        magika::ContentType::Crx => &CRX,
        magika::ContentType::Cs => &CS,
        magika::ContentType::Csproj => &CSPROJ,
        magika::ContentType::Css => &CSS,
        magika::ContentType::Csv => &CSV,
        magika::ContentType::Dart => &DART,
        magika::ContentType::Deb => &DEB,
        magika::ContentType::Dex => &DEX,
        magika::ContentType::Dicom => &DICOM,
        magika::ContentType::Diff => &DIFF,
        magika::ContentType::Dm => &DM,
        magika::ContentType::Dmg => &DMG,
        magika::ContentType::Doc => &DOC,
        magika::ContentType::Dockerfile => &DOCKERFILE,
        magika::ContentType::Docx => &DOCX,
        magika::ContentType::Dsstore => &DSSTORE,
        magika::ContentType::Dwg => &DWG,
        magika::ContentType::Dxf => &DXF,
        magika::ContentType::Elf => &ELF,
        magika::ContentType::Elixir => &ELIXIR,
        magika::ContentType::Emf => &EMF,
        magika::ContentType::Eml => &EML,
        magika::ContentType::Empty => &EMPTY,
        magika::ContentType::Epub => &EPUB,
        magika::ContentType::Erb => &ERB,
        magika::ContentType::Erlang => &ERLANG,
        magika::ContentType::Flac => &FLAC,
        magika::ContentType::Flv => &FLV,
        magika::ContentType::Fortran => &FORTRAN,
        magika::ContentType::Gemfile => &GEMFILE,
        magika::ContentType::Gemspec => &GEMSPEC,
        magika::ContentType::Gif => &GIF,
        magika::ContentType::Gitattributes => &GITATTRIBUTES,
        magika::ContentType::Gitmodules => &GITMODULES,
        magika::ContentType::Go => &GO,
        magika::ContentType::Gradle => &GRADLE,
        magika::ContentType::Groovy => &GROOVY,
        magika::ContentType::Gzip => &GZIP,
        magika::ContentType::H5 => &H5,
        magika::ContentType::Handlebars => &HANDLEBARS,
        magika::ContentType::Haskell => &HASKELL,
        magika::ContentType::Hcl => &HCL,
        magika::ContentType::Hlp => &HLP,
        magika::ContentType::Htaccess => &HTACCESS,
        magika::ContentType::Html => &HTML,
        magika::ContentType::Icns => &ICNS,
        magika::ContentType::Ico => &ICO,
        magika::ContentType::Ics => &ICS,
        magika::ContentType::Ignorefile => &IGNOREFILE,
        magika::ContentType::Ini => &INI,
        magika::ContentType::Internetshortcut => &INTERNETSHORTCUT,
        magika::ContentType::Ipynb => &IPYNB,
        magika::ContentType::Iso => &ISO,
        magika::ContentType::Jar => &JAR,
        magika::ContentType::Java => &JAVA,
        magika::ContentType::Javabytecode => &JAVABYTECODE,
        magika::ContentType::Javascript => &JAVASCRIPT,
        magika::ContentType::Jinja => &JINJA,
        magika::ContentType::Jp2 => &JP2,
        magika::ContentType::Jpeg => &JPEG,
        magika::ContentType::Json => &JSON,
        magika::ContentType::Jsonl => &JSONL,
        magika::ContentType::Julia => &JULIA,
        magika::ContentType::Kotlin => &KOTLIN,
        magika::ContentType::Latex => &LATEX,
        magika::ContentType::Lha => &LHA,
        magika::ContentType::Lisp => &LISP,
        magika::ContentType::Lnk => &LNK,
        magika::ContentType::Lua => &LUA,
        magika::ContentType::M3u => &M3U,
        magika::ContentType::M4 => &M4,
        magika::ContentType::Macho => &MACHO,
        magika::ContentType::Makefile => &MAKEFILE,
        magika::ContentType::Markdown => &MARKDOWN,
        magika::ContentType::Matlab => &MATLAB,
        magika::ContentType::Mht => &MHT,
        magika::ContentType::Midi => &MIDI,
        magika::ContentType::Mkv => &MKV,
        magika::ContentType::Mp3 => &MP3,
        magika::ContentType::Mp4 => &MP4,
        magika::ContentType::Mscompress => &MSCOMPRESS,
        magika::ContentType::Msi => &MSI,
        magika::ContentType::Mum => &MUM,
        magika::ContentType::Npy => &NPY,
        magika::ContentType::Npz => &NPZ,
        magika::ContentType::Nupkg => &NUPKG,
        magika::ContentType::Objectivec => &OBJECTIVEC,
        magika::ContentType::Ocaml => &OCAML,
        magika::ContentType::Odp => &ODP,
        magika::ContentType::Ods => &ODS,
        magika::ContentType::Odt => &ODT,
        magika::ContentType::Ogg => &OGG,
        magika::ContentType::One => &ONE,
        magika::ContentType::Onnx => &ONNX,
        magika::ContentType::Otf => &OTF,
        magika::ContentType::Outlook => &OUTLOOK,
        magika::ContentType::Parquet => &PARQUET,
        magika::ContentType::Pascal => &PASCAL,
        magika::ContentType::Pcap => &PCAP,
        magika::ContentType::Pdb => &PDB,
        magika::ContentType::Pdf => &PDF,
        magika::ContentType::Pebin => &PEBIN,
        magika::ContentType::Pem => &PEM,
        magika::ContentType::Perl => &PERL,
        magika::ContentType::Php => &PHP,
        magika::ContentType::Pickle => &PICKLE,
        magika::ContentType::Png => &PNG,
        magika::ContentType::Po => &PO,
        magika::ContentType::Postscript => &POSTSCRIPT,
        magika::ContentType::Powershell => &POWERSHELL,
        magika::ContentType::Ppt => &PPT,
        magika::ContentType::Pptx => &PPTX,
        magika::ContentType::Prolog => &PROLOG,
        magika::ContentType::Proteindb => &PROTEINDB,
        magika::ContentType::Proto => &PROTO,
        magika::ContentType::Psd => &PSD,
        magika::ContentType::Python => &PYTHON,
        magika::ContentType::Pythonbytecode => &PYTHONBYTECODE,
        magika::ContentType::Pytorch => &PYTORCH,
        magika::ContentType::Qt => &QT,
        magika::ContentType::R => &R,
        magika::ContentType::Randombytes => &RANDOMBYTES,
        magika::ContentType::Randomtxt => &RANDOMTXT,
        magika::ContentType::Rar => &RAR,
        magika::ContentType::Rdf => &RDF,
        magika::ContentType::Rpm => &RPM,
        magika::ContentType::Rst => &RST,
        magika::ContentType::Rtf => &RTF,
        magika::ContentType::Ruby => &RUBY,
        magika::ContentType::Rust => &RUST,
        magika::ContentType::Scala => &SCALA,
        magika::ContentType::Scss => &SCSS,
        magika::ContentType::Sevenzip => &SEVENZIP,
        magika::ContentType::Sgml => &SGML,
        magika::ContentType::Shell => &SHELL,
        magika::ContentType::Smali => &SMALI,
        magika::ContentType::Snap => &SNAP,
        magika::ContentType::Solidity => &SOLIDITY,
        magika::ContentType::Sql => &SQL,
        magika::ContentType::Sqlite => &SQLITE,
        magika::ContentType::Squashfs => &SQUASHFS,
        magika::ContentType::Srt => &SRT,
        magika::ContentType::Stlbinary => &STLBINARY,
        magika::ContentType::Stltext => &STLTEXT,
        magika::ContentType::Sum => &SUM,
        magika::ContentType::Svg => &SVG,
        magika::ContentType::Swf => &SWF,
        magika::ContentType::Swift => &SWIFT,
        magika::ContentType::Tar => &TAR,
        magika::ContentType::Tcl => &TCL,
        magika::ContentType::Textproto => &TEXTPROTO,
        magika::ContentType::Tga => &TGA,
        magika::ContentType::Thumbsdb => &THUMBSDB,
        magika::ContentType::Tiff => &TIFF,
        magika::ContentType::Toml => &TOML,
        magika::ContentType::Torrent => &TORRENT,
        magika::ContentType::Tsv => &TSV,
        magika::ContentType::Ttf => &TTF,
        magika::ContentType::Twig => &TWIG,
        magika::ContentType::Txt => &TXT,
        magika::ContentType::Typescript => &TYPESCRIPT,
        magika::ContentType::Undefined => &UNDEFINED,
        magika::ContentType::Unknown => &UNKNOWN,
        magika::ContentType::Vba => &VBA,
        magika::ContentType::Vcxproj => &VCXPROJ,
        magika::ContentType::Verilog => &VERILOG,
        magika::ContentType::Vhdl => &VHDL,
        magika::ContentType::Vtt => &VTT,
        magika::ContentType::Vue => &VUE,
        magika::ContentType::Wasm => &WASM,
        magika::ContentType::Wav => &WAV,
        magika::ContentType::Webm => &WEBM,
        magika::ContentType::Webp => &WEBP,
        magika::ContentType::Winregistry => &WINREGISTRY,
        magika::ContentType::Wmf => &WMF,
        magika::ContentType::Woff => &WOFF,
        magika::ContentType::Woff2 => &WOFF2,
        magika::ContentType::Xar => &XAR,
        magika::ContentType::Xls => &XLS,
        magika::ContentType::Xlsb => &XLSB,
        magika::ContentType::Xlsx => &XLSX,
        magika::ContentType::Xml => &XML,
        magika::ContentType::Xpi => &XPI,
        magika::ContentType::Xz => &XZ,
        magika::ContentType::Yaml => &YAML,
        magika::ContentType::Yara => &YARA,
        magika::ContentType::Zig => &ZIG,
        magika::ContentType::Zip => &ZIP,
        magika::ContentType::Zlibstream => &ZLIBSTREAM,
        _ => unreachable!(),
    }
}
