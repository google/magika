# Magika CLI

This binary crate implements a command-line interface (CLI) to the library crate
[magika](https://crates.io/crates/magika) which provides file type detection with deep-learning.

## Disclaimer

This project is not an official Google project. It is not supported by Google and Google
specifically disclaims all warranties as to its quality, merchantability, or fitness for a
particular purpose.

The `magika` library and this `magika-cli` binary are still unstable (as indicated by the major
version of zero) and new versions might introduce breaking changes (all changes will follow [cargo
semver compatibility](https://doc.rust-lang.org/cargo/reference/semver.html)). In particular,
version 0.1.0-rc.0 ships a new model in comparison to the Python binary and we would love feedback
on [GitHub](https://github.com/google/magika/issues).

## Installation

To install the latest version from crates.io:

```shell
cargo install --locked magika-cli
```

It is also possible to install from the git repository, in which case the version (accessible with
`magika --version`) will be suffixed by `-dev` (e.g. `0.1.0-dev`) to indicate that the binary is the
development version of the version prefix (e.g. `0.1.0` for the previous example).

To install the latest version from the git repository:

```shell
cargo install --locked --git=https://github.com/google/magika.git magika-cli
```

To install from a local clone of the git repository (possibly with custom changes):

```shell
git clone https://github.com/google/magika.git
cd magika
cargo install --locked --path=rust/cli
```

## Examples


```shell
$ cd tests_data/basic && magika -r *
asm/code.asm: Assembly (code)
batch/simple.bat: DOS batch file (code)
c/code.c: C source (code)
css/code.css: CSS source (code)
csv/magika_test.csv: CSV document (code)
dockerfile/Dockerfile: Dockerfile (code)
docx/doc.docx: Microsoft Word 2007+ document (document)
epub/doc.epub: EPUB document (document)
epub/magika_test.epub: EPUB document (document)
flac/test.flac: FLAC audio bitstream data (audio)
handlebars/example.handlebars: Handlebars source (code)
html/doc.html: HTML document (code)
ini/doc.ini: INI configuration file (text)
javascript/code.js: JavaScript source (code)
jinja/example.j2: Jinja template (code)
jpeg/magika_test.jpg: JPEG image data (image)
json/doc.json: JSON document (code)
latex/sample.tex: LaTeX document (text)
makefile/simple.Makefile: Makefile source (code)
markdown/README.md: Markdown document (text)
[...]
```

```shell
$ magika ./tests_data/basic/python/code.py --json
[
  {
    "path": "./tests_data/basic/python/code.py",
    "result": {
      "status": "ok",
      "value": {
        "dl": {
          "description": "Python source",
          "extensions": [
            "py",
            "pyi"
          ],
          "group": "code",
          "is_text": true,
          "label": "python",
          "mime_type": "text/x-python"
        },
        "output": {
          "description": "Python source",
          "extensions": [
            "py",
            "pyi"
          ],
          "group": "code",
          "is_text": true,
          "label": "python",
          "mime_type": "text/x-python"
        },
        "score": 0.753000020980835
      }
    }
  }
]
```

```shell
$ cat doc.ini | magika -
-: INI configuration file (text)
```

```help
$ magika --help
Determines the content type of files with deep-learning

Usage: magika [OPTIONS] [PATH]...

Arguments:
  [PATH]...
          List of paths to the files to analyze.

          Use a dash (-) to read from standard input (can only be used once).

Options:
  -r, --recursive
          Identifies files within directories instead of identifying the directory itself

      --no-dereference
          Identifies symbolic links as is instead of identifying their content by following them

      --colors
          Prints with colors regardless of terminal support

      --no-colors
          Prints without colors regardless of terminal support

  -s, --output-score
          Prints the prediction score in addition to the content type

  -i, --mime-type
          Prints the MIME type instead of the content type description

  -l, --label
          Prints a simple label instead of the content type description

      --json
          Prints in JSON format

      --jsonl
          Prints in JSONL format

      --format <CUSTOM>
          Prints using a custom format (use --help for details).

          The following placeholders are supported:

            %p  The file path
            %l  The unique label identifying the content type
            %d  The description of the content type
            %g  The group of the content type
            %m  The MIME type of the content type
            %e  Possible file extensions for the content type
            %s  The score of the content type for the file
            %S  The score of the content type for the file in percent
            %b  The model output if overruled (empty otherwise)
            %%  A literal %

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```


See the [docs on Magika's output](../../docs/magika_output.md) for more details about the output
format.
