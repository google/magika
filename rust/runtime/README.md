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

The standard cargo-dist archives and binary Python wheels install runtime libraries
beside `magika`. The loader supports both layouts. The platform GPU library in
cargo-dist archives is named `magika_runtime_gpu` (with the platform prefix and
extension); its ABI identifies Metal or CUDA. Python wheels include CPU on every
platform and Metal on macOS. Rules still require a separately installed Vectorscan
library unless supplied by `rules/package.py bundle`.

Release CI builds the native payload with `rust/distribution/build.py` and tests
installation before publishing. The shell installer needs a narrowly checked
postprocessing fix for cargo-dist 0.31's library chmod path; see `fix_installer.py`.

For an editable Python checkout (`uv sync`) or a library development build, build
the backend once and point the process to it:

```sh
python3 rust/build-runtime.py --backends-only --output "$PWD/tmp/dev-runtime"
export MAGIKA_RUNTIME_DIR="$PWD/tmp/dev-runtime/lib"
```

Rebuild that directory after changing the runtime or model; the helper requires a
new output directory so an old backend cannot silently masquerade as a new build.
