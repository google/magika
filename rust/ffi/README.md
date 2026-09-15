# Magika C API

This crate provides C bindings for the Magika Rust library. It provides:
- A static library `../target/release/libmagika.a`
- A dynamic library `../target/release/libmagika.{so,dylib,dll}` depending on the platform
- A C99/C++ compatible header `include/magika.h`

You can run `cargo build --release` to build the static and dynamic libraries.

## Example

Using the dynamic library:

```sh
gcc -Iinclude example.c -L../target/release -lmagika -Wl,-rpath,../target/release -o example
```

Using the static library on Linux:

```sh
gcc -Iinclude example.c ../target/release/libmagika.a -lpthread -ldl -lm -o example
```

Using the static library on macOS:

```sh
clang -Iinclude example.c ../target/release/libmagika.a -framework Metal -framework CoreGraphics \
  -framework CoreFoundation -lobjc -liconv -lm -o example
```

The native libraries a static library needs depend on the target and the Rust toolchain. You can
list them for your build with:

```sh
cargo rustc --release --lib -- --print=native-static-libs
```
