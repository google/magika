# Model documentation

## Table of Contents

1. [List of possible outputs](#list-of-possible-outputs)
1. [List of possible model's outputs](#list-of-possible-models-outputs)

## List of possible outputs

This is the full list of all possible tool's outputs (which are different than the possible raw output of the model, see table below). E.g., this is the list of all possible values for Magika python module's `MagikaResult.prediction.output.label`.

| Index   |      Content Type Label      | Description |
|----------|:-------------:|------|
| 1 | 3gp | 3GPP multimedia file |
| 2 | ace | ACE archive |
| 3 | ai | Adobe Illustrator Artwork |
| 4 | aidl | Android Interface Definition Language |
| 5 | apk | Android package |
| 6 | applebplist | Apple binary property list |
| 7 | appleplist | Apple property list |
| 8 | asm | Assembly |
| 9 | asp | ASP source |
| 10 | autohotkey | AutoHotKey script |
| 11 | autoit | AutoIt script |
| 12 | awk | Awk |
| 13 | batch | DOS batch file |
| 14 | bazel | Bazel build file |
| 15 | bib | BibTeX |
| 16 | bmp | BMP image data |
| 17 | bzip | bzip2 compressed data |
| 18 | c | C source |
| 19 | cab | Microsoft Cabinet archive data |
| 20 | cat | Windows Catalog file |
| 21 | chm | MS Windows HtmlHelp Data |
| 22 | clojure | Clojure |
| 23 | cmake | CMake build file |
| 24 | cobol | Cobol |
| 25 | coff | Intel 80386 COFF |
| 26 | coffeescript | CoffeeScript |
| 27 | cpp | C++ source |
| 28 | crt | Certificates (binary format) |
| 29 | crx | Google Chrome extension |
| 30 | cs | C# source |
| 31 | csproj | .NET project config |
| 32 | css | CSS source |
| 33 | csv | CSV document |
| 34 | dart | Dart source |
| 35 | deb | Debian binary package |
| 36 | dex | Dalvik dex file |
| 37 | dicom | DICOM |
| 38 | diff | Diff file |
| 39 | directory | A directory |
| 40 | dm | Dream Maker |
| 41 | dmg | Apple disk image |
| 42 | doc | Microsoft Word CDF document |
| 43 | dockerfile | Dockerfile |
| 44 | docx | Microsoft Word 2007+ document |
| 45 | dsstore | Application Desktop Services Store |
| 46 | dwg | Autocad Drawing |
| 47 | dxf | Audocad Drawing Exchange Format |
| 48 | elf | ELF executable |
| 49 | elixir | Elixir script |
| 50 | emf | Windows Enhanced Metafile image data |
| 51 | eml | RFC 822 mail |
| 52 | empty | Empty file |
| 53 | epub | EPUB document |
| 54 | erb | Embedded Ruby source |
| 55 | erlang | Erlang source |
| 56 | flac | FLAC audio bitstream data |
| 57 | flv | Flash Video |
| 58 | fortran | Fortran |
| 59 | gemfile | Gemfile file |
| 60 | gemspec | Gemspec file |
| 61 | gif | GIF image data |
| 62 | gitattributes | Gitattributes file |
| 63 | gitmodules | Gitmodules file |
| 64 | go | Golang source |
| 65 | gradle | Gradle source |
| 66 | groovy | Groovy source |
| 67 | gzip | gzip compressed data |
| 68 | h5 | Hierarchical Data Format v5 |
| 69 | handlebars | Handlebars source |
| 70 | haskell | Haskell source |
| 71 | hcl | HashiCorp configuration language |
| 72 | hlp | MS Windows help |
| 73 | htaccess | Apache access configuration |
| 74 | html | HTML document |
| 75 | icns | Mac OS X icon |
| 76 | ico | MS Windows icon resource |
| 77 | ics | Internet Calendaring and Scheduling |
| 78 | ignorefile | Ignorefile |
| 79 | ini | INI configuration file |
| 80 | internetshortcut | MS Windows Internet shortcut |
| 81 | ipynb | Jupyter notebook |
| 82 | iso | ISO 9660 CD-ROM filesystem data |
| 83 | jar | Java archive data (JAR) |
| 84 | java | Java source |
| 85 | javabytecode | Java compiled bytecode |
| 86 | javascript | JavaScript source |
| 87 | jinja | Jinja template |
| 88 | jp2 | jpeg2000 |
| 89 | jpeg | JPEG image data |
| 90 | json | JSON document |
| 91 | jsonl | JSONL document |
| 92 | julia | Julia source |
| 93 | kotlin | Kotlin source |
| 94 | latex | LaTeX document |
| 95 | lha | LHarc archive |
| 96 | lisp | Lisp source |
| 97 | lnk | MS Windows shortcut |
| 98 | lua | Lua |
| 99 | m3u | M3U playlist |
| 100 | m4 | GNU Macro |
| 101 | macho | Mach-O executable |
| 102 | makefile | Makefile source |
| 103 | markdown | Markdown document |
| 104 | matlab | Matlab Source |
| 105 | mht | MHTML document |
| 106 | midi | Midi |
| 107 | mkv | Matroska |
| 108 | mp3 | MP3 media file |
| 109 | mp4 | MP4 media file |
| 110 | mscompress | MS Compress archive data |
| 111 | msi | Microsoft Installer file |
| 112 | mum | Windows Update Package file |
| 113 | npy | Numpy Array |
| 114 | npz | Numpy Arrays Archive |
| 115 | nupkg | NuGet Package |
| 116 | objectivec | ObjectiveC source |
| 117 | ocaml | OCaml |
| 118 | odp | OpenDocument Presentation |
| 119 | ods | OpenDocument Spreadsheet |
| 120 | odt | OpenDocument Text |
| 121 | ogg | Ogg data |
| 122 | one | One Note |
| 123 | onnx | Open Neural Network Exchange |
| 124 | otf | OpenType font |
| 125 | outlook | MS Outlook Message |
| 126 | parquet | Apache Parquet |
| 127 | pascal | Pascal source |
| 128 | pcap | pcap capture file |
| 129 | pdb | Windows Program Database |
| 130 | pdf | PDF document |
| 131 | pebin | PE Windows executable |
| 132 | pem | PEM certificate |
| 133 | perl | Perl source |
| 134 | php | PHP source |
| 135 | pickle | Python pickle |
| 136 | png | PNG image |
| 137 | po | Portable Object (PO) for i18n |
| 138 | postscript | PostScript document |
| 139 | powershell | Powershell source |
| 140 | ppt | Microsoft PowerPoint CDF document |
| 141 | pptx | Microsoft PowerPoint 2007+ document |
| 142 | prolog | Prolog source |
| 143 | proteindb | Protein DB |
| 144 | proto | Protocol buffer definition |
| 145 | psd | Adobe Photoshop |
| 146 | python | Python source |
| 147 | pythonbytecode | Python compiled bytecode |
| 148 | pytorch | Pytorch storage file |
| 149 | qt | QuickTime |
| 150 | r | R (language) |
| 151 | rar | RAR archive data |
| 152 | rdf | Resource Description Framework document (RDF) |
| 153 | rpm | RedHat Package Manager archive (RPM) |
| 154 | rst | ReStructuredText document |
| 155 | rtf | Rich Text Format document |
| 156 | ruby | Ruby source |
| 157 | rust | Rust source |
| 158 | scala | Scala source |
| 159 | scss | SCSS source |
| 160 | sevenzip | 7-zip archive data |
| 161 | sgml | sgml |
| 162 | shell | Shell script |
| 163 | smali | Smali source |
| 164 | snap | Snap archive |
| 165 | solidity | Solidity source |
| 166 | sql | SQL source |
| 167 | sqlite | SQLITE database |
| 168 | squashfs | Squash filesystem |
| 169 | srt | SubRip Text Format |
| 170 | stlbinary | Stereolithography CAD (binary) |
| 171 | stltext | Stereolithography CAD (text) |
| 172 | sum | Checksum file |
| 173 | svg | SVG Scalable Vector Graphics image data |
| 174 | swf | Small Web File |
| 175 | swift | Swift |
| 176 | symlink | Symbolic link |
| 177 | tar | POSIX tar archive |
| 178 | tcl | Tickle |
| 179 | textproto | Text protocol buffer |
| 180 | tga | Targa image data |
| 181 | thumbsdb | Windows thumbnail cache |
| 182 | tiff | TIFF image data |
| 183 | toml | Tom's obvious, minimal language |
| 184 | torrent | BitTorrent file |
| 185 | tsv | TSV document |
| 186 | ttf | TrueType Font data |
| 187 | twig | Twig template |
| 188 | txt | Generic text document |
| 189 | typescript | TypeScript source |
| 190 | unknown | Unknown binary data |
| 191 | vba | MS Visual Basic source (VBA) |
| 192 | vcxproj | Visual Studio MSBuild project |
| 193 | verilog | Verilog source |
| 194 | vhdl | VHDL source |
| 195 | vtt | Web Video Text Tracks |
| 196 | vue | Vue source |
| 197 | wasm | Web Assembly |
| 198 | wav | Waveform Audio file (WAV) |
| 199 | webm | WebM media file |
| 200 | webp | WebP media file |
| 201 | winregistry | Windows Registry text |
| 202 | wmf | Windows metafile |
| 203 | woff | Web Open Font Format |
| 204 | woff2 | Web Open Font Format v2 |
| 205 | xar | XAR archive compressed data |
| 206 | xls | Microsoft Excel CDF document |
| 207 | xlsb | Microsoft Excel 2007+ document (binary format) |
| 208 | xlsx | Microsoft Excel 2007+ document |
| 209 | xml | XML document |
| 210 | xpi | Compressed installation archive (XPI) |
| 211 | xz | XZ compressed data |
| 212 | yaml | YAML source |
| 213 | yara | YARA rule |
| 214 | zig | Zig source |
| 215 | zip | Zip archive data |
| 216 | zlibstream | zlib compressed data |


## List of possible model's outputs

This is the full list of all possible model's output. E.g., this is the list of all possible values for Magika python module's `MagikaResult.prediction.dl.label`. Note that, in general, the list of "model outputs" is different than the "tool outputs" as in some cases the model is not even used, or the model's output is overwritten due to a low-confidence score or other reasons. This list is useful mostly for debugging purposes; the vast majority of client should just consult the table above.

| Index   |      Content Type Label      | Description |
|----------|:-------------:|------|
| 1 | 3gp | 3GPP multimedia file |
| 2 | ace | ACE archive |
| 3 | ai | Adobe Illustrator Artwork |
| 4 | aidl | Android Interface Definition Language |
| 5 | apk | Android package |
| 6 | applebplist | Apple binary property list |
| 7 | appleplist | Apple property list |
| 8 | asm | Assembly |
| 9 | asp | ASP source |
| 10 | autohotkey | AutoHotKey script |
| 11 | autoit | AutoIt script |
| 12 | awk | Awk |
| 13 | batch | DOS batch file |
| 14 | bazel | Bazel build file |
| 15 | bib | BibTeX |
| 16 | bmp | BMP image data |
| 17 | bzip | bzip2 compressed data |
| 18 | c | C source |
| 19 | cab | Microsoft Cabinet archive data |
| 20 | cat | Windows Catalog file |
| 21 | chm | MS Windows HtmlHelp Data |
| 22 | clojure | Clojure |
| 23 | cmake | CMake build file |
| 24 | cobol | Cobol |
| 25 | coff | Intel 80386 COFF |
| 26 | coffeescript | CoffeeScript |
| 27 | cpp | C++ source |
| 28 | crt | Certificates (binary format) |
| 29 | crx | Google Chrome extension |
| 30 | cs | C# source |
| 31 | csproj | .NET project config |
| 32 | css | CSS source |
| 33 | csv | CSV document |
| 34 | dart | Dart source |
| 35 | deb | Debian binary package |
| 36 | dex | Dalvik dex file |
| 37 | dicom | DICOM |
| 38 | diff | Diff file |
| 39 | dm | Dream Maker |
| 40 | dmg | Apple disk image |
| 41 | doc | Microsoft Word CDF document |
| 42 | dockerfile | Dockerfile |
| 43 | docx | Microsoft Word 2007+ document |
| 44 | dsstore | Application Desktop Services Store |
| 45 | dwg | Autocad Drawing |
| 46 | dxf | Audocad Drawing Exchange Format |
| 47 | elf | ELF executable |
| 48 | elixir | Elixir script |
| 49 | emf | Windows Enhanced Metafile image data |
| 50 | eml | RFC 822 mail |
| 51 | epub | EPUB document |
| 52 | erb | Embedded Ruby source |
| 53 | erlang | Erlang source |
| 54 | flac | FLAC audio bitstream data |
| 55 | flv | Flash Video |
| 56 | fortran | Fortran |
| 57 | gemfile | Gemfile file |
| 58 | gemspec | Gemspec file |
| 59 | gif | GIF image data |
| 60 | gitattributes | Gitattributes file |
| 61 | gitmodules | Gitmodules file |
| 62 | go | Golang source |
| 63 | gradle | Gradle source |
| 64 | groovy | Groovy source |
| 65 | gzip | gzip compressed data |
| 66 | h5 | Hierarchical Data Format v5 |
| 67 | handlebars | Handlebars source |
| 68 | haskell | Haskell source |
| 69 | hcl | HashiCorp configuration language |
| 70 | hlp | MS Windows help |
| 71 | htaccess | Apache access configuration |
| 72 | html | HTML document |
| 73 | icns | Mac OS X icon |
| 74 | ico | MS Windows icon resource |
| 75 | ics | Internet Calendaring and Scheduling |
| 76 | ignorefile | Ignorefile |
| 77 | ini | INI configuration file |
| 78 | internetshortcut | MS Windows Internet shortcut |
| 79 | ipynb | Jupyter notebook |
| 80 | iso | ISO 9660 CD-ROM filesystem data |
| 81 | jar | Java archive data (JAR) |
| 82 | java | Java source |
| 83 | javabytecode | Java compiled bytecode |
| 84 | javascript | JavaScript source |
| 85 | jinja | Jinja template |
| 86 | jp2 | jpeg2000 |
| 87 | jpeg | JPEG image data |
| 88 | json | JSON document |
| 89 | jsonl | JSONL document |
| 90 | julia | Julia source |
| 91 | kotlin | Kotlin source |
| 92 | latex | LaTeX document |
| 93 | lha | LHarc archive |
| 94 | lisp | Lisp source |
| 95 | lnk | MS Windows shortcut |
| 96 | lua | Lua |
| 97 | m3u | M3U playlist |
| 98 | m4 | GNU Macro |
| 99 | macho | Mach-O executable |
| 100 | makefile | Makefile source |
| 101 | markdown | Markdown document |
| 102 | matlab | Matlab Source |
| 103 | mht | MHTML document |
| 104 | midi | Midi |
| 105 | mkv | Matroska |
| 106 | mp3 | MP3 media file |
| 107 | mp4 | MP4 media file |
| 108 | mscompress | MS Compress archive data |
| 109 | msi | Microsoft Installer file |
| 110 | mum | Windows Update Package file |
| 111 | npy | Numpy Array |
| 112 | npz | Numpy Arrays Archive |
| 113 | nupkg | NuGet Package |
| 114 | objectivec | ObjectiveC source |
| 115 | ocaml | OCaml |
| 116 | odp | OpenDocument Presentation |
| 117 | ods | OpenDocument Spreadsheet |
| 118 | odt | OpenDocument Text |
| 119 | ogg | Ogg data |
| 120 | one | One Note |
| 121 | onnx | Open Neural Network Exchange |
| 122 | otf | OpenType font |
| 123 | outlook | MS Outlook Message |
| 124 | parquet | Apache Parquet |
| 125 | pascal | Pascal source |
| 126 | pcap | pcap capture file |
| 127 | pdb | Windows Program Database |
| 128 | pdf | PDF document |
| 129 | pebin | PE Windows executable |
| 130 | pem | PEM certificate |
| 131 | perl | Perl source |
| 132 | php | PHP source |
| 133 | pickle | Python pickle |
| 134 | png | PNG image |
| 135 | po | Portable Object (PO) for i18n |
| 136 | postscript | PostScript document |
| 137 | powershell | Powershell source |
| 138 | ppt | Microsoft PowerPoint CDF document |
| 139 | pptx | Microsoft PowerPoint 2007+ document |
| 140 | prolog | Prolog source |
| 141 | proteindb | Protein DB |
| 142 | proto | Protocol buffer definition |
| 143 | psd | Adobe Photoshop |
| 144 | python | Python source |
| 145 | pythonbytecode | Python compiled bytecode |
| 146 | pytorch | Pytorch storage file |
| 147 | qt | QuickTime |
| 148 | r | R (language) |
| 149 | randombytes | Random bytes |
| 150 | randomtxt | Random text |
| 151 | rar | RAR archive data |
| 152 | rdf | Resource Description Framework document (RDF) |
| 153 | rpm | RedHat Package Manager archive (RPM) |
| 154 | rst | ReStructuredText document |
| 155 | rtf | Rich Text Format document |
| 156 | ruby | Ruby source |
| 157 | rust | Rust source |
| 158 | scala | Scala source |
| 159 | scss | SCSS source |
| 160 | sevenzip | 7-zip archive data |
| 161 | sgml | sgml |
| 162 | shell | Shell script |
| 163 | smali | Smali source |
| 164 | snap | Snap archive |
| 165 | solidity | Solidity source |
| 166 | sql | SQL source |
| 167 | sqlite | SQLITE database |
| 168 | squashfs | Squash filesystem |
| 169 | srt | SubRip Text Format |
| 170 | stlbinary | Stereolithography CAD (binary) |
| 171 | stltext | Stereolithography CAD (text) |
| 172 | sum | Checksum file |
| 173 | svg | SVG Scalable Vector Graphics image data |
| 174 | swf | Small Web File |
| 175 | swift | Swift |
| 176 | tar | POSIX tar archive |
| 177 | tcl | Tickle |
| 178 | textproto | Text protocol buffer |
| 179 | tga | Targa image data |
| 180 | thumbsdb | Windows thumbnail cache |
| 181 | tiff | TIFF image data |
| 182 | toml | Tom's obvious, minimal language |
| 183 | torrent | BitTorrent file |
| 184 | tsv | TSV document |
| 185 | ttf | TrueType Font data |
| 186 | twig | Twig template |
| 187 | txt | Generic text document |
| 188 | typescript | TypeScript source |
| 189 | undefined | Undefined |
| 190 | vba | MS Visual Basic source (VBA) |
| 191 | vcxproj | Visual Studio MSBuild project |
| 192 | verilog | Verilog source |
| 193 | vhdl | VHDL source |
| 194 | vtt | Web Video Text Tracks |
| 195 | vue | Vue source |
| 196 | wasm | Web Assembly |
| 197 | wav | Waveform Audio file (WAV) |
| 198 | webm | WebM media file |
| 199 | webp | WebP media file |
| 200 | winregistry | Windows Registry text |
| 201 | wmf | Windows metafile |
| 202 | woff | Web Open Font Format |
| 203 | woff2 | Web Open Font Format v2 |
| 204 | xar | XAR archive compressed data |
| 205 | xls | Microsoft Excel CDF document |
| 206 | xlsb | Microsoft Excel 2007+ document (binary format) |
| 207 | xlsx | Microsoft Excel 2007+ document |
| 208 | xml | XML document |
| 209 | xpi | Compressed installation archive (XPI) |
| 210 | xz | XZ compressed data |
| 211 | yaml | YAML source |
| 212 | yara | YARA rule |
| 213 | zig | Zig source |
| 214 | zip | Zip archive data |
| 215 | zlibstream | zlib compressed data |>