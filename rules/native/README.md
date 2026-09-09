# ARM CRC32C for the pinned Vectorscan build

Apply `arm-crc32c.patch` to Vectorscan commit
`acd7363aadea43da9c5246542d9969db843dd132` before configuring CMake, as shown in
the rules installation instructions. The patch retains the native database CRC
check and computes the same polynomial and seed convention with ARM instructions.

Only Apple Silicon builds enable CRC instructions explicitly, and only in
`crc32.c`. Other ARM builds use the accelerated branch only when their own target
flags declare CRC32 support; otherwise they retain the software path. The x86
SSE4.2 branch is unchanged. The accelerated branch requires little endian ARM64.

`crc32c_test.c` checks 65,696 seed, alignment, short-tail, and large-buffer cases
against a bytewise polynomial oracle. The macOS rules CI job runs it with ASan
and UBSan. The implementation uses `memcpy` for unaligned words and never reads
past the supplied length. Existing native database and rule tests exercise the
library produced by the same patched build.
