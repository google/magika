# Expected output of the CLI

The `labels` file contains the labels of the `basic` and `mitra` test directories as returned by the
CLI. The goal is to ultimately check that the labels are expected (in which case only the unexpected
ones can be stored).

The `flags` file contains the concatenated output of the CLI with different set of files and
different set of flags. The goal is to check the CLI output format.

You can update the files with the `./run.sh` script.
