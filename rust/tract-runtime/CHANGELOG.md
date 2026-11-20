# Changelog

## 0.0.1

This is the initial version of the tract inference runtime shared by the Magika library and CLI.
It loads the embedded model from its parsed graph rather than from the NNEF archive.
A CPU runtime prepares its smallest batch class at once and the others the first time a session reaches them.
A GPU runtime supports Metal on macOS and CUDA (with the `cuda` feature), checking every batch class against a CPU reference at startup.
