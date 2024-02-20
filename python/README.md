# Magika Python Package

Magika is a novel AI powered file type detection tool that rely on the recent advance of deep learning to provide accurate detection. Under the hood, Magika employs a custom, highly optimized Keras model that only weighs about 1MB, and enables precise file identification within milliseconds, even when running on a single CPU.

Use Magika as a command line client or in your Python code!

Please check out Magika on GitHub for more information and documentation: [https://github.com/google/magika](https://github.com/google/magika).


## Installing Magika

```shell
$ pip install magika
```

If you intend to use Magika only as a command line, you may want to use `$ pipx install magika` instead.


## Using Magika as a command-line tool

```shell
$ magika examples/*
code.asm: Assembly (code)
code.py: Python source (code)
doc.docx: Microsoft Word 2007+ document (document)
doc.ini: INI configuration file (text)
elf64.elf: ELF executable (executable)
flac.flac: FLAC audio bitstream data (audio)
image.bmp: BMP image data (image)
java.class: Java compiled bytecode (executable)
jpg.jpg: JPEG image data (image)
pdf.pdf: PDF document (document)
pe32.exe: PE executable (executable)
png.png: PNG image data (image)
README.md: Markdown document (text)
tar.tar: POSIX tar archive (archive)
webm.webm: WebM data (video)
```

```help
$ magika --help
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


## Using Magika as a Python module

```python
from magika import Magika
magika = Magika()
result = magika.identify_bytes(b"# Example\nThis is an example of markdown!")
print(result.output.ct_label)  # Output: "markdown"
```


## Citation
If you use this software for your research, please cite it as:
```bibtex
@software{magika,
author = {Fratantonio, Yanick and Bursztein, Elie and Invernizzi, Luca and Zhang, Marina and Metitieri, Giancarlo and Kurt, Thomas and Galilee, Francois and Petit-Bianco, Alexandre and Farah, Loua and Albertini, Ange},
title = {{Magika content-type scanner}},
url = {https://github.com/google/magika}
}
```
