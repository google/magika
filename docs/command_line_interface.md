# Command Line Interface Documentation

Magika ships with a CLI (currently written in Python), but we do have bindings/libraries/APIs for automated workflows. See [here](./bindings.md) for the full list.

Magika CLI takes one or more files as arguments, and a number of options.

```help
$ magika -h
Usage: magika [OPTIONS] [FILE]...

  Magika - Determine type of FILEs with deep-learning.

Options:
  -r, --recursive                 When passing this option, magika scans every
                                  file within directories, instead of
                                  outputting "directory"
  --json                          Output in JSON format.
  --jsonl                         Output in JSONL format.
  -i, --mime-type                 Output the MIME type instead of a verbose
                                  content type description.
  -l, --label                     Output a simple label instead of a verbose
                                  content type description. Use --list-output-
                                  content-types for the list of supported
                                  output.
  -c, --compatibility-mode        Compatibility mode: output is as close as
                                  possible to `file` and colors are disabled.
  -s, --output-score              Output the prediction score in addition to
                                  the content type.
  -m, --prediction-mode [best-guess|medium-confidence|high-confidence]
  --batch-size INTEGER            How many files to process in one batch.
  --no-dereference                This option causes symlinks not to be
                                  followed. By default, symlinks are
                                  dereferenced.
  --colors / --no-colors          Enable/disable use of colors.
  -v, --verbose                   Enable more verbose output.
  -vv, --debug                    Enable debug logging.
  --generate-report               Generate report useful when reporting
                                  feedback.
  --version                       Print the version and exit.
  --list-output-content-types     Show a list of supported content types.
  --model-dir DIRECTORY           Use a custom model.
  -h, --help                      Show this message and exit.

  Magika version: "0.5.0"

  Default model: "standard_v1"

  Send any feedback to magika-dev@google.com or via GitHub issues.
```

Here are more details for some of the non-trivial options:
- `-r, --recursive`: by default, if you pass a directory as argument, magika will just print that it is a directory. This is to be backward compatible with existing tools, e.g., `file`. If you pass `-r`, magika will recursively scan every file in that directory.
- `--json`: make Magika output one single JSON with all the predictions for all files. Note that this will not output anything until the last file has been scanned; if you scan thousands of files, it could be quite slow!
- `--jsonl`: this makes Magika output one JSON per line (i.e., JSONL format). You may want to use this option if you are scanning hundreds or thousands of files.
- `-l, --label`: just output a textual content type label for each file.
- `-c, --compatibility-mode`: output a string that resembles what `file` would output (best-effort).
- `-s, --output-score`: output a string that resembles what `file` would output (best-effort).
- `-m, --prediction-mode`: Magika uses the deep learning model to perform content type inference, and it then determines whether the prediction should be considered as valid. This option tweaks the required confidence to output a prediction: `high-confidence` (the default), uses per-content-type thresholds that aim for high precision; `medium-confidence` uses a much lower number as the required confidence; `best-guess` outputs whatever content type has the highest score. You may want to use one or another mode depending on your deployment scenario.
- `--generate-report`: if you have a misdetection and would like to report it (Thank you!), you can use this option to generate a report that you can include in the github issue. **Do NOT send reports that may contain PII, the report contains (a small) part of the file!**


## Examples

```shell
$ magika -r tests_data/
tests_data/README.md: Markdown document (text)
tests_data/basic/code.asm: Assembly (code)
tests_data/basic/code.c: C source (code)
tests_data/basic/code.css: CSS source (code)
tests_data/basic/code.js: JavaScript source (code)
tests_data/basic/code.py: Python source (code)
tests_data/basic/code.rs: Rust source (code)
...
tests_data/mitra/7-zip.7z: 7-zip archive data (archive)
tests_data/mitra/bmp.bmp: BMP image data (image)
tests_data/mitra/bzip2.bz2: bzip2 compressed data (archive)
tests_data/mitra/cab.cab: Microsoft Cabinet archive data (archive)
tests_data/mitra/elf.elf: ELF executable (executable)
tests_data/mitra/flac.flac: FLAC audio bitstream data (audio)
...
```

```shell
$ cat doc.ini | magika -
-: INI configuration file (text)
```

```shell
$ magika code.py --json
[
    {
        "path": "code.py",
        "dl": {
            "ct_label": "python",
            "score": 0.9940916895866394,
            "group": "code",
            "mime_type": "text/x-python",
            "magic": "Python script",
            "description": "Python source"
        },
        "output": {
            "ct_label": "python",
            "score": 0.9940916895866394,
            "group": "code",
            "mime_type": "text/x-python",
            "magic": "Python script",
            "description": "Python source"
        }
    }
]
```

## Magika Output Documentation

See the [Magika output documentation](./magika_output.md) on how to interpret and use the output.