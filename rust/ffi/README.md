# Magika C API

This crate provides C bindings for the Magika Rust library. It provides:
- A static library `../target/release/libmagika.a`
- A dynamic library `../target/release/libmagika.{so,dylib,dll}` depending on the platform
- A C99/C++ compatible header `include/magika.h`

You can run `cargo build --release` to build the static and dynamic libraries.

## Example

Using the static library:

```sh
gcc -Iinclude example.c ../target/release/libmagika.a -lpthread -ldl -lm -o example
```

Using the dynamic library:

```sh
gcc -Iinclude example.c -L../target/release -lmagika -Wl,-rpath,../target/release -o example
```
