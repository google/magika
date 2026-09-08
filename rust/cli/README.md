# Magika CLI

This binary crate implements a command-line interface (CLI) to the library crate
[magika](https://crates.io/crates/magika) which provides file content type detection using AI.

For piped input, pass the literal path `-`, for example `cat sample.bin | magika -`.
Named input paths must be regular files (or directories for traversal); device paths
such as `/dev/stdin` and process-substitution paths are not substitutes for `-`.

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

You can install the latest version from the `magika` python package:

```shell
pipx install magika
```

You can install the latest version from a shell using `curl`:

```shell
curl -LsSf https://securityresearch.google/magika/install.sh | sh
```

You can install the latest version from a shell using `wget`:

```shell
wget -qO- https://securityresearch.google/magika/install.sh | sh
```

You can install the latest version from a powershell:

```shell
powershell -ExecutionPolicy Bypass -c "irm https://securityresearch.google/magika/install.ps1 | iex"
```

You can install the latest version from crates.io:

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
% cd tests_data/basic && magika -r * | head
asm/code.asm: Assembly (code)
awk/weekly_totals.awk: Awk (code)
batch/simple.bat: DOS batch file (code)
bib/references.bib: BibTeX (text)
c/code.c: C source (code)
clojure/weather_summary.clj: Clojure (code)
css/code.css: CSS source (code)
csv/magika_test.csv: CSV document (code)
dart/inventory_report.dart: Dart source (code)
diff/weather-station.patch: Diff file (text)
```

```shell
% magika ./tests_data/basic/python/code.py --json
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
        "score": 0.996999979019165
      }
    }
  }
]
```

```shell
% cat tests_data/basic/ini/doc.ini | magika -
-: INI configuration file (text)
```

```shell
% magika --help
Determines file content types using AI

Usage: magika [OPTIONS] [PATH]...

Arguments:
  [PATH]...
          List of paths to the files to analyze.

          Use a dash (-) to read from standard input (can only be used once).

Options:
      --rules <RULES>
          Enables the selected ruleset (requires the yara-rules feature)

          [default: off]
          [possible values: off, enforce]

      --rules-file <RULES_FILE>
          Loads a YARA pack with per-rule enforcement metadata. Requires --rules=enforce

      --compile-rules <COMPILE_RULES>
          Compiles a YARA file to a sibling .hsdb file and exits; refuses to overwrite

      --write-default-rules <WRITE_DEFAULT_RULES>
          Writes the bundled YARA source to a new file and exits

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


See the [docs on Magika's core
concepts](https://securityresearch.google/magika/core-concepts/how-magika-works/) for more details
about the output format and other important aspects.

## Rule and inference concurrency

The CLI prepares the model on a background coordinator while its readers extract and classify
files. Rule hits can reach ordered output before model preparation completes; misses queue in
bounded inference batches. Once the backend is ready, the coordinator starts the normal CPU or
GPU worker count (`--threads` overrides it). Each worker owns its inference session. An earlier
ML miss can still hold up later rule hits because output preserves input order.

On CPU, a declared maximum batch retains one model plan. Partial batches are padded through that
plan and padding outputs are discarded, avoiding extra unfused plans and their retained buffers.
When traversal fills the bounded output window, it requests a batch flush after the outstanding
reads complete. Scheduling delays never trigger partial inference batches.
