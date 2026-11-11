---
title: Supported Content Types (Rust 1.1.0)
---

<!-- DO NOT EDIT: frozen release snapshot of Rust `magika` library 1.1.0, reconstructed from the content types KB and standard_v3_3 config.min.json at git tag cli/v1.1.0 (5e2f437f, which released the Rust library and CLI together), which predates automatic snapshots. Its sha256 is pinned in assets/content_types_snapshots.sha256 and enforced by 'just sync-kb --check'. -->

This reference is the frozen output-space (`output.label`) and detection stage-coverage snapshot for **Rust `magika` library `1.1.0`** (default model: [`standard_v3_3`](/magika/models/standard_v3_3)). For other versions and the `main` development branch, see the [per-version index](/magika/content-types).

## Pipeline Coverage Summary

- **Total supported outputs**: **216** content types.
  - **Heuristics (`in_heuristics`)**: **5** content types (`directory`, `empty`, `symlink`, `txt`, `unknown`).
  - **Format Resolvers (`in_resolvers`)**: **0** content types (this release predates native format resolvers).
  - **ML Model (`in_ml_model`)**: **212** content types predicted by [`standard_v3_3`](/magika/models/standard_v3_3) (after `overwrite_map`).

## Supported Content Types

| Label | Description | MIME Type | Group | Extensions | Text | Heuristics | Resolvers | ML Model | Vetoed |
| --- | --- | --- | --- | --- | :---: | :---: | :---: | :---: | :---: |
| `3gp` | 3GPP multimedia file | `video/3gpp` | `video` | `3gp` |  |  |  | ✓ |  |
| `ace` | ACE archive | `application/x-ace-compressed` | `archive` | `ace` |  |  |  | ✓ |  |
| `ai` | Adobe Illustrator Artwork | `application/pdf` | `document` | `ai` |  |  |  | ✓ |  |
| `aidl` | Android Interface Definition Language | `text/plain` | `unknown` | `aidl` | ✓ |  |  | ✓ |  |
| `apk` | Android package | `application/vnd.android.package-archive` | `executable` | `apk` |  |  |  | ✓ |  |
| `applebplist` | Apple binary property list | `application/x-bplist` | `application` | `bplist`, `plist` |  |  |  | ✓ |  |
| `appleplist` | Apple property list | `application/x-plist` | `application` | `plist` | ✓ |  |  | ✓ |  |
| `asm` | Assembly | `text/x-asm` | `code` | `s`, `S`, `asm` | ✓ |  |  | ✓ |  |
| `asp` | ASP source | `text/html` | `code` | `aspx`, `asp` | ✓ |  |  | ✓ |  |
| `autohotkey` | AutoHotKey script | `text/plain` | `code` | — | ✓ |  |  | ✓ |  |
| `autoit` | AutoIt script | `text/plain` | `code` | `au3` | ✓ |  |  | ✓ |  |
| `awk` | Awk | `text/plain` | `code` | `awk` | ✓ |  |  | ✓ |  |
| `batch` | DOS batch file | `text/x-msdos-batch` | `code` | `bat` | ✓ |  |  | ✓ |  |
| `bazel` | Bazel build file | `text/plain` | `code` | `bzl` | ✓ |  |  | ✓ |  |
| `bib` | BibTeX | `text/x-bibtex` | `text` | `bib` | ✓ |  |  | ✓ |  |
| `bmp` | BMP image data | `image/bmp` | `image` | `bmp` |  |  |  | ✓ |  |
| `bzip` | bzip2 compressed data | `application/x-bzip2` | `archive` | `bz2`, `tbz2`, `tar.bz2` |  |  |  | ✓ |  |
| `c` | C source | `text/x-c` | `code` | `c` | ✓ |  |  | ✓ |  |
| `cab` | Microsoft Cabinet archive data | `application/vnd.ms-cab-compressed` | `archive` | `cab` |  |  |  | ✓ |  |
| `cat` | Windows Catalog file | `application/octet-stream` | `application` | `cat` |  |  |  | ✓ |  |
| `chm` | MS Windows HtmlHelp Data | `application/chm` | `application` | `chm` |  |  |  | ✓ |  |
| `clojure` | Clojure | `text/x-clojure` | `code` | `clj`, `cljs`, `cljc`, `cljr` | ✓ |  |  | ✓ |  |
| `cmake` | CMake build file | `text/x-cmake` | `code` | `cmake` | ✓ |  |  | ✓ |  |
| `cobol` | Cobol | `text/x-cobol` | `code` | `cbl`, `cob`, `cpy`, `CBL`, `COB`, `CPY` | ✓ |  |  | ✓ |  |
| `coff` | Intel 80386 COFF | `application/x-coff` | `executable` | `obj`, `o` |  |  |  | ✓ |  |
| `coffeescript` | CoffeeScript | `text/coffeescript` | `code` | `coffee` | ✓ |  |  | ✓ |  |
| `cpp` | C++ source | `text/x-c` | `code` | `cc`, `cpp`, `cxx`, `c++`, `cppm`, `ixx` | ✓ |  |  | ✓ |  |
| `crt` | Certificates (binary format) | `application/x-x509-ca-cert` | `text` | `der`, `cer`, `crt` |  |  |  | ✓ |  |
| `crx` | Google Chrome extension | `application/x-chrome-extension` | `executable` | `crx` |  |  |  | ✓ |  |
| `cs` | C# source | `text/plain` | `code` | `cs`, `csx` | ✓ |  |  | ✓ |  |
| `csproj` | .NET project config | `text/plain` | `code` | `csproj` | ✓ |  |  | ✓ |  |
| `css` | CSS source | `text/css` | `code` | `css` | ✓ |  |  | ✓ |  |
| `csv` | CSV document | `text/csv` | `code` | `csv` | ✓ |  |  | ✓ |  |
| `dart` | Dart source | `text/plain` | `code` | `dart` | ✓ |  |  | ✓ |  |
| `deb` | Debian binary package | `application/vnd.debian.binary-package` | `archive` | `deb` |  |  |  | ✓ |  |
| `dex` | Dalvik dex file | `application/x-android-dex` | `executable` | `dex` |  |  |  | ✓ |  |
| `dicom` | DICOM | `application/dicom` | `image` | `dcm` |  |  |  | ✓ |  |
| `diff` | Diff file | `text/plain` | `text` | `diff`, `patch` | ✓ |  |  | ✓ |  |
| `directory` | A directory | `inode/directory` | `inode` | — |  | ✓ |  |  |  |
| `dm` | Dream Maker | `text/plain` | `code` | `dm` | ✓ |  |  | ✓ |  |
| `dmg` | Apple disk image | `application/x-apple-diskimage` | `archive` | `dmg` |  |  |  | ✓ |  |
| `doc` | Microsoft Word CDF document | `application/msword` | `document` | `doc` |  |  |  | ✓ |  |
| `dockerfile` | Dockerfile | `text/x-dockerfile` | `code` | — | ✓ |  |  | ✓ |  |
| `docx` | Microsoft Word 2007+ document | `application/vnd.openxmlformats-officedocument.wordprocessingml.document` | `document` | `docx`, `docm` |  |  |  | ✓ |  |
| `dsstore` | Application Desktop Services Store | `application/octet-stream` | `unknown` | — |  |  |  | ✓ |  |
| `dwg` | Autocad Drawing | `image/x-dwg` | `image` | `dwg` |  |  |  | ✓ |  |
| `dxf` | Audocad Drawing Exchange Format | `image/vnd.dxf` | `image` | `dxf` | ✓ |  |  | ✓ |  |
| `elf` | ELF executable | `application/x-executable-elf` | `executable` | `elf` |  |  |  | ✓ |  |
| `elixir` | Elixir script | `text/plain` | `code` | `exs` | ✓ |  |  | ✓ |  |
| `emf` | Windows Enhanced Metafile image data | `application/octet-stream` | `application` | `emf` |  |  |  | ✓ |  |
| `eml` | RFC 822 mail | `message/rfc822` | `text` | `eml` | ✓ |  |  | ✓ |  |
| `empty` | Empty file | `inode/x-empty` | `inode` | — |  | ✓ |  |  |  |
| `epub` | EPUB document | `application/epub+zip` | `document` | `epub` |  |  |  | ✓ |  |
| `erb` | Embedded Ruby source | `text/x-ruby` | `code` | `erb` | ✓ |  |  | ✓ |  |
| `erlang` | Erlang source | `text/x-erlang` | `code` | `erl`, `hrl` | ✓ |  |  | ✓ |  |
| `flac` | FLAC audio bitstream data | `audio/flac` | `audio` | `flac` |  |  |  | ✓ |  |
| `flv` | Flash Video | `video/x-flv` | `video` | `flv` |  |  |  | ✓ |  |
| `fortran` | Fortran | `text/x-fortran` | `document` | `f90`, `f95`, `f03`, `F90` | ✓ |  |  | ✓ |  |
| `gemfile` | Gemfile file | `text/plain` | `code` | — | ✓ |  |  | ✓ |  |
| `gemspec` | Gemspec file | `text/plain` | `code` | `gemspec` | ✓ |  |  | ✓ |  |
| `gif` | GIF image data | `image/gif` | `image` | `gif` |  |  |  | ✓ |  |
| `gitattributes` | Gitattributes file | `text/plain` | `code` | — | ✓ |  |  | ✓ |  |
| `gitmodules` | Gitmodules file | `text/plain` | `code` | — | ✓ |  |  | ✓ |  |
| `go` | Golang source | `text/x-golang` | `code` | `go` | ✓ |  |  | ✓ |  |
| `gradle` | Gradle source | `text/x-groovy` | `code` | `gradle` | ✓ |  |  | ✓ |  |
| `groovy` | Groovy source | `text/x-groovy` | `code` | `groovy` | ✓ |  |  | ✓ |  |
| `gzip` | gzip compressed data | `application/gzip` | `archive` | `gz`, `gzip`, `tgz`, `tar.gz` |  |  |  | ✓ |  |
| `h5` | Hierarchical Data Format v5 | `application/x-hdf5` | `archive` | `h5`, `hdf5` |  |  |  | ✓ |  |
| `handlebars` | Handlebars source | `text/x-handlebars-template` | `code` | `hbs`, `handlebars` | ✓ |  |  | ✓ |  |
| `haskell` | Haskell source | `text/plain` | `code` | `hs`, `lhs` | ✓ |  |  | ✓ |  |
| `hcl` | HashiCorp configuration language | `text/x-hcl` | `code` | `hcl` | ✓ |  |  | ✓ |  |
| `hlp` | MS Windows help | `application/winhlp` | `application` | `hlp` |  |  |  | ✓ |  |
| `htaccess` | Apache access configuration | `text/x-apache-conf` | `code` | — | ✓ |  |  | ✓ |  |
| `html` | HTML document | `text/html` | `code` | `html`, `htm`, `xhtml`, `xht` | ✓ |  |  | ✓ |  |
| `icns` | Mac OS X icon | `image/x-icns` | `image` | `icns` |  |  |  | ✓ |  |
| `ico` | MS Windows icon resource | `image/vnd.microsoft.icon` | `image` | `ico` |  |  |  | ✓ |  |
| `ics` | Internet Calendaring and Scheduling | `text/calendar` | `application` | `ics` | ✓ |  |  | ✓ |  |
| `ignorefile` | Ignorefile | `text/plain` | `code` | — | ✓ |  |  | ✓ |  |
| `ini` | INI configuration file | `text/plain` | `text` | `ini` | ✓ |  |  | ✓ |  |
| `internetshortcut` | MS Windows Internet shortcut | `application/x-mswinurl` | `application` | `url` | ✓ |  |  | ✓ |  |
| `ipynb` | Jupyter notebook | `application/json` | `code` | `ipynb` | ✓ |  |  | ✓ |  |
| `iso` | ISO 9660 CD-ROM filesystem data | `application/x-iso9660-image` | `archive` | `iso` |  |  |  | ✓ |  |
| `jar` | Java archive data (JAR) | `application/java-archive` | `archive` | `jar`, `klib` |  |  |  | ✓ |  |
| `java` | Java source | `text/x-java` | `code` | `java` | ✓ |  |  | ✓ |  |
| `javabytecode` | Java compiled bytecode | `application/x-java-applet` | `executable` | `class` |  |  |  | ✓ |  |
| `javascript` | JavaScript source | `application/javascript` | `code` | `js`, `mjs`, `cjs` | ✓ |  |  | ✓ |  |
| `jinja` | Jinja template | `text/x-jinja2-template` | `code` | `jinja`, `jinja2`, `j2` | ✓ |  |  | ✓ |  |
| `jp2` | jpeg2000 | `image/jpeg2000` | `image` | `jp2` |  |  |  | ✓ |  |
| `jpeg` | JPEG image data | `image/jpeg` | `image` | `jpg`, `jpeg` |  |  |  | ✓ |  |
| `json` | JSON document | `application/json` | `code` | `json` | ✓ |  |  | ✓ |  |
| `jsonl` | JSONL document | `application/json` | `code` | `jsonl`, `jsonld` | ✓ |  |  | ✓ |  |
| `julia` | Julia source | `text/x-julia` | `code` | `jl` | ✓ |  |  | ✓ |  |
| `kotlin` | Kotlin source | `text/plain` | `code` | `kt`, `kts` | ✓ |  |  | ✓ |  |
| `latex` | LaTeX document | `text/x-tex` | `text` | `tex`, `sty` | ✓ |  |  | ✓ |  |
| `lha` | LHarc archive | `application/x-lha` | `archive` | `lha`, `lzh` |  |  |  | ✓ |  |
| `lisp` | Lisp source | `text/x-lisp` | `code` | `lisp`, `lsp`, `l`, `cl` | ✓ |  |  | ✓ |  |
| `lnk` | MS Windows shortcut | `application/x-ms-shortcut` | `application` | `lnk` |  |  |  | ✓ |  |
| `lua` | Lua | `text/plain` | `code` | `lua` | ✓ |  |  | ✓ |  |
| `m3u` | M3U playlist | `text/plain` | `application` | `m3u8`, `m3u` | ✓ |  |  | ✓ |  |
| `m4` | GNU Macro | `text/plain` | `code` | `m4` | ✓ |  |  | ✓ |  |
| `macho` | Mach-O executable | `application/x-mach-o` | `executable` | — |  |  |  | ✓ |  |
| `makefile` | Makefile source | `text/x-makefile` | `code` | — | ✓ |  |  | ✓ |  |
| `markdown` | Markdown document | `text/markdown` | `text` | `md`, `markdown` | ✓ |  |  | ✓ |  |
| `matlab` | Matlab Source | `text/x-matlab` | `code` | `m`, `matlab` | ✓ |  |  | ✓ |  |
| `mht` | MHTML document | `application/x-mimearchive` | `code` | `mht` | ✓ |  |  | ✓ |  |
| `midi` | Midi | `audio/midi` | `audio` | `mid` |  |  |  | ✓ |  |
| `mkv` | Matroska | `video/x-matroska` | `video` | `mkv` |  |  |  | ✓ |  |
| `mp3` | MP3 media file | `audio/mpeg` | `audio` | `mp3` |  |  |  | ✓ |  |
| `mp4` | MP4 media file | `video/mp4` | `video` | `mp4` |  |  |  | ✓ |  |
| `mscompress` | MS Compress archive data | `application/x-ms-compress-szdd` | `archive` | — |  |  |  | ✓ |  |
| `msi` | Microsoft Installer file | `application/x-msi` | `archive` | `msi` |  |  |  | ✓ |  |
| `mum` | Windows Update Package file | `text/xml` | `application` | `mum` | ✓ |  |  | ✓ |  |
| `npy` | Numpy Array | `application/octet-stream` | `archive` | `npy` |  |  |  | ✓ |  |
| `npz` | Numpy Arrays Archive | `application/octet-stream` | `archive` | `npz` |  |  |  | ✓ |  |
| `nupkg` | NuGet Package | `application/octet-stream` | `unknown` | `nupkg` |  |  |  | ✓ |  |
| `objectivec` | ObjectiveC source | `text/x-objcsrc` | `code` | `m`, `mm` | ✓ |  |  | ✓ |  |
| `ocaml` | OCaml | `text-ocaml` | `code` | `ml`, `mli` | ✓ |  |  | ✓ |  |
| `odp` | OpenDocument Presentation | `application/vnd.oasis.opendocument.presentation` | `document` | `odp` |  |  |  | ✓ |  |
| `ods` | OpenDocument Spreadsheet | `application/vnd.oasis.opendocument.spreadsheet` | `document` | `ods` |  |  |  | ✓ |  |
| `odt` | OpenDocument Text | `application/vnd.oasis.opendocument.text` | `document` | `odt` |  |  |  | ✓ |  |
| `ogg` | Ogg data | `audio/ogg` | `audio` | `ogg` |  |  |  | ✓ |  |
| `one` | One Note | `application/msonenote` | `document` | `one` |  |  |  | ✓ |  |
| `onnx` | Open Neural Network Exchange | `application/octet-stream` | `archive` | `onnx` |  |  |  | ✓ |  |
| `otf` | OpenType font | `font/otf` | `font` | `otf` |  |  |  | ✓ |  |
| `outlook` | MS Outlook Message | `application/vnd.ms-outlook` | `application` | — |  |  |  | ✓ |  |
| `parquet` | Apache Parquet | `application/vnd.apache.parquet` | `unknown` | `pqt`, `parquet` |  |  |  | ✓ |  |
| `pascal` | Pascal source | `text/x-pascal` | `code` | `pas`, `pp` | ✓ |  |  | ✓ |  |
| `pcap` | pcap capture file | `application/vnd.tcpdump.pcap` | `application` | `pcap`, `pcapng` |  |  |  | ✓ |  |
| `pdb` | Windows Program Database | `application/octet-stream` | `application` | `pdb` |  |  |  | ✓ |  |
| `pdf` | PDF document | `application/pdf` | `document` | `pdf` |  |  |  | ✓ |  |
| `pebin` | PE Windows executable | `application/x-dosexec` | `executable` | `exe`, `dll` |  |  |  | ✓ |  |
| `pem` | PEM certificate | `application/x-pem-file` | `application` | `pem`, `pub`, `gpg` | ✓ |  |  | ✓ |  |
| `perl` | Perl source | `text/x-perl` | `code` | `pl` | ✓ |  |  | ✓ |  |
| `php` | PHP source | `text/x-php` | `code` | `php` | ✓ |  |  | ✓ |  |
| `pickle` | Python pickle | `application/octet-stream` | `application` | `pickle`, `pkl` |  |  |  | ✓ |  |
| `png` | PNG image | `image/png` | `image` | `png` |  |  |  | ✓ |  |
| `po` | Portable Object (PO) for i18n | `text/gettext-translation` | `application` | `po` | ✓ |  |  | ✓ |  |
| `postscript` | PostScript document | `application/postscript` | `document` | `ps` |  |  |  | ✓ |  |
| `powershell` | Powershell source | `application/x-powershell` | `code` | `ps1` | ✓ |  |  | ✓ |  |
| `ppt` | Microsoft PowerPoint CDF document | `application/vnd.ms-powerpoint` | `document` | `ppt` |  |  |  | ✓ |  |
| `pptx` | Microsoft PowerPoint 2007+ document | `application/vnd.openxmlformats-officedocument.presentationml.presentation` | `document` | `pptx`, `pptm` |  |  |  | ✓ |  |
| `prolog` | Prolog source | `text/x-prolog` | `code` | `pl`, `pro`, `P` | ✓ |  |  | ✓ |  |
| `proteindb` | Protein DB | `application/octet-stream` | `application` | `pdb` | ✓ |  |  | ✓ |  |
| `proto` | Protocol buffer definition | `text/x-proto` | `code` | `proto` | ✓ |  |  | ✓ |  |
| `psd` | Adobe Photoshop | `image/vnd.adobe.photoshop` | `image` | `psd` |  |  |  | ✓ |  |
| `python` | Python source | `text/x-python` | `code` | `py`, `pyi` | ✓ |  |  | ✓ |  |
| `pythonbytecode` | Python compiled bytecode | `application/x-bytecode.python` | `executable` | `pyc`, `pyo` |  |  |  | ✓ |  |
| `pytorch` | Pytorch storage file | `application/octet-stream` | `application` | `pt`, `pth` |  |  |  | ✓ |  |
| `qt` | QuickTime | `video/quicktime` | `video` | `mov` |  |  |  | ✓ |  |
| `r` | R (language) | `text/x-R` | `code` | `R` | ✓ |  |  | ✓ |  |
| `rar` | RAR archive data | `application/x-rar` | `archive` | `rar` |  |  |  | ✓ |  |
| `rdf` | Resource Description Framework document (RDF) | `application/rdf+xml` | `text` | `rdf` | ✓ |  |  | ✓ |  |
| `rpm` | RedHat Package Manager archive (RPM) | `application/x-rpm` | `archive` | `rpm` |  |  |  | ✓ |  |
| `rst` | ReStructuredText document | `text/x-rst` | `text` | `rst` | ✓ |  |  | ✓ |  |
| `rtf` | Rich Text Format document | `text/rtf` | `text` | `rtf` | ✓ |  |  | ✓ |  |
| `ruby` | Ruby source | `application/x-ruby` | `code` | `rb` | ✓ |  |  | ✓ |  |
| `rust` | Rust source | `application/x-rust` | `code` | `rs` | ✓ |  |  | ✓ |  |
| `scala` | Scala source | `application/x-scala` | `code` | `scala` | ✓ |  |  | ✓ |  |
| `scss` | SCSS source | `text/x-scss` | `code` | `scss` | ✓ |  |  | ✓ |  |
| `sevenzip` | 7-zip archive data | `application/x-7z-compressed` | `archive` | `7z` |  |  |  | ✓ |  |
| `sgml` | sgml | `application/sgml` | `text` | `sgml` | ✓ |  |  | ✓ |  |
| `shell` | Shell script | `text/x-shellscript` | `code` | `sh` | ✓ |  |  | ✓ |  |
| `smali` | Smali source | `application/x-smali` | `code` | `smali` | ✓ |  |  | ✓ |  |
| `snap` | Snap archive | `application/octet-stream` | `archive` | `snap` |  |  |  | ✓ |  |
| `solidity` | Solidity source | `text/plain` | `code` | `sol` | ✓ |  |  | ✓ |  |
| `sql` | SQL source | `application/x-sql` | `code` | `sql` | ✓ |  |  | ✓ |  |
| `sqlite` | SQLITE database | `application/octet-stream` | `application` | `sqlite`, `sqlite3` |  |  |  | ✓ |  |
| `squashfs` | Squash filesystem | `application/octet-stream` | `archive` | — |  |  |  | ✓ |  |
| `srt` | SubRip Text Format | `text/srt` | `application` | `srt` | ✓ |  |  | ✓ |  |
| `stlbinary` | Stereolithography CAD (binary) | `application/sla` | `image` | `stl` |  |  |  | ✓ |  |
| `stltext` | Stereolithography CAD (text) | `application/sla` | `image` | `stl` | ✓ |  |  | ✓ |  |
| `sum` | Checksum file | `text/plain` | `application` | `sum` | ✓ |  |  | ✓ |  |
| `svg` | SVG Scalable Vector Graphics image data | `image/svg+xml` | `image` | `svg` | ✓ |  |  | ✓ |  |
| `swf` | Small Web File | `application/x-shockwave-flash` | `executable` | `swf` |  |  |  | ✓ |  |
| `swift` | Swift | `text/x-swift` | `code` | `swift` | ✓ |  |  | ✓ |  |
| `symlink` | Symbolic link | `inode/symlink` | `inode` | — |  | ✓ |  |  |  |
| `tar` | POSIX tar archive | `application/x-tar` | `archive` | `tar` |  |  |  | ✓ |  |
| `tcl` | Tickle | `application/x-tcl` | `code` | `tcl` | ✓ |  |  | ✓ |  |
| `textproto` | Text protocol buffer | `text/plain` | `code` | `textproto`, `textpb`, `pbtxt` | ✓ |  |  | ✓ |  |
| `tga` | Targa image data | `image/x-tga` | `image` | `tga` |  |  |  | ✓ |  |
| `thumbsdb` | Windows thumbnail cache | `image/vnd.ms-thumb` | `application` | — |  |  |  | ✓ |  |
| `tiff` | TIFF image data | `image/tiff` | `image` | `tiff`, `tif` |  |  |  | ✓ |  |
| `toml` | Tom's obvious, minimal language | `application/toml` | `text` | `toml` | ✓ |  |  | ✓ |  |
| `torrent` | BitTorrent file | `application/x-bittorrent` | `application` | `torrent` |  |  |  | ✓ |  |
| `tsv` | TSV document | `text/tsv` | `code` | `tsv` | ✓ |  |  | ✓ |  |
| `ttf` | TrueType Font data | `font/sfnt` | `font` | `ttf`, `ttc` |  |  |  | ✓ |  |
| `twig` | Twig template | `text/x-twig` | `code` | `twig` | ✓ |  |  | ✓ |  |
| `txt` | Generic text document | `text/plain` | `text` | `txt` | ✓ | ✓ |  | ✓ |  |
| `typescript` | TypeScript source | `application/typescript` | `code` | `ts`, `mts`, `cts` | ✓ |  |  | ✓ |  |
| `unknown` | Unknown binary data | `application/octet-stream` | `unknown` | — |  | ✓ |  |  |  |
| `vba` | MS Visual Basic source (VBA) | `text/vbscript` | `code` | `vbs`, `vba`, `vb` | ✓ |  |  | ✓ |  |
| `vcxproj` | Visual Studio MSBuild project | `application/xml` | `code` | `vcxproj` | ✓ |  |  | ✓ |  |
| `verilog` | Verilog source | `text/x-verilog` | `code` | `v`, `verilog`, `vlg`, `vh` | ✓ |  |  | ✓ |  |
| `vhdl` | VHDL source | `text/x-vhdl` | `code` | `vhd` | ✓ |  |  | ✓ |  |
| `vtt` | Web Video Text Tracks | `text/vtt` | `text` | `vtt`, `webvtt` | ✓ |  |  | ✓ |  |
| `vue` | Vue source | `application/javascript` | `code` | `vue` | ✓ |  |  | ✓ |  |
| `wasm` | Web Assembly | `application/wasm` | `executable` | `wasm` |  |  |  | ✓ |  |
| `wav` | Waveform Audio file (WAV) | `audio/x-wav` | `audio` | `wav` |  |  |  | ✓ |  |
| `webm` | WebM media file | `video/webm` | `video` | `webm` |  |  |  | ✓ |  |
| `webp` | WebP media file | `image/webp` | `image` | `webp` |  |  |  | ✓ |  |
| `winregistry` | Windows Registry text | `text/x-ms-regedit` | `application` | `reg` | ✓ |  |  | ✓ |  |
| `wmf` | Windows metafile | `image/wmf` | `image` | `wmf` |  |  |  | ✓ |  |
| `woff` | Web Open Font Format | `font/woff` | `font` | `woff` |  |  |  | ✓ |  |
| `woff2` | Web Open Font Format v2 | `font/woff2` | `font` | `woff2` |  |  |  | ✓ |  |
| `xar` | XAR archive compressed data | `application/x-xar` | `archive` | `pkg`, `xar` |  |  |  | ✓ |  |
| `xls` | Microsoft Excel CDF document | `application/vnd.ms-excel` | `document` | `xls` |  |  |  | ✓ |  |
| `xlsb` | Microsoft Excel 2007+ document (binary format) | `application/vnd.openxmlformats-officedocument.spreadsheetml.sheet` | `document` | `xlsb` |  |  |  | ✓ |  |
| `xlsx` | Microsoft Excel 2007+ document | `application/vnd.openxmlformats-officedocument.spreadsheetml.sheet` | `document` | `xlsx`, `xlsm` |  |  |  | ✓ |  |
| `xml` | XML document | `text/xml` | `code` | `xml` | ✓ |  |  | ✓ |  |
| `xpi` | Compressed installation archive (XPI) | `application/zip` | `archive` | `xpi` |  |  |  | ✓ |  |
| `xz` | XZ compressed data | `application/x-xz` | `archive` | `xz` |  |  |  | ✓ |  |
| `yaml` | YAML source | `application/x-yaml` | `code` | `yml`, `yaml` | ✓ |  |  | ✓ |  |
| `yara` | YARA rule | `text/x-yara` | `code` | `yar`, `yara` | ✓ |  |  | ✓ |  |
| `zig` | Zig source | `text/zig` | `code` | `zig` | ✓ |  |  | ✓ |  |
| `zip` | Zip archive data | `application/zip` | `archive` | `zip` |  |  |  | ✓ |  |
| `zlibstream` | zlib compressed data | `application/zlib` | `application` | — |  |  |  | ✓ |  |
