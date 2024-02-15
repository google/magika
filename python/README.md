# Magika Python Package

Use Magika as a command line client or in your Python code!

\[Please check the full README and documentation at https://github.com/google/magika\]

## Installing Magika
```bash
$ pip install magika
```

## Using Magika as a command-line tool.

```bash
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

Run `$ magika --help` for details about the command line interface.


## Using Magika in Python

```python
from magika import Magika
magika = Magika()
result = magika.identify_bytes(b"# Example\nThis is an example of markdown!")
print(result.output.ct_label)  # Output: "markdown"
```


## More information

Please check the full README and documentation at [https://github.com/google/magika](https://github.com/google/magika).


## Citation
If you use this software for your research, please cite it as:
```bibtex
@software{magika,
author = {Fratantonio, Yanick and Bursztein, Elie and Invernizzi, Luca and Zhang, Marina and Metitieri, Giancarlo and Kurt, Thomas and Galilee, Francois and Petit-Bianco, Alexandre and Farah, Loua and Albertini, Ange},
title = {{Magika content-type scanner}},
url = {https://github.com/google/magika}
}
```
