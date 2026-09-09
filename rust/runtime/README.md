# Deferred inference runtimes

Build a complete release directory from the repository root:

```sh
python3 rust/build-runtime.py --gpu metal --output /absolute/path/to/new-dist
```

Omit `--gpu` for a CPU distribution; use `--gpu cuda` for CUDA. The output contains
`magika`, `lib/` and a JSON build receipt. `RUSTFLAGS` is honored and recorded;
use `RUSTFLAGS='-C target-cpu=native'` for a local-machine build, and distribute
only to compatible CPUs. The builder uses the release profile and locked dependencies.

Keep the executable and `lib/` together. Explicit CPU mode loads only the CPU
runtime; rules-only initializes neither inference backend. Auto tries the GPU
runtime before CPU fallback. A library caller can set `MAGIKA_RUNTIME_DIR` to an
absolute directory containing its installed native runtimes. Backend libraries
are executable code and must come from the same trusted distribution.

For a rules-enabled archive, bundle the built executable and runtime directory:

```sh
python3 rules/package.py bundle --binary /absolute/path/to/new-dist/magika \
  --runtime-dir /absolute/path/to/new-dist/lib \
  --library /absolute/path/to/libhs.dylib \
  --license /absolute/path/to/vectorscan-LICENSE --output /absolute/path/to/magika.tar.gz
```

The bundle command verifies CPU backend discovery and native rules compilation
from the assembled directory. Source staging includes the loader and ABI crates.
Mapped rule caches are enabled on Unix by the normal `yara-rules` feature;
`MAGIKA_RULES_MMAP=0` selects serialized caches for diagnostics. Startup spans are
available through `MAGIKA_STARTUP_TRACE=1`; historical spike commands require
their recorded source revisions.
