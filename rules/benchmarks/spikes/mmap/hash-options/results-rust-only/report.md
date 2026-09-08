# Hash compiler-option audit

The same manifest and native payload bytes are hashed by every variant. Both use release opt-level=3, LTO and one codegen unit. BLAKE3 NEON and SHA-256 ARM assembly are enabled. Native additionally sets Rust target-cpu=native. CFLAGS is unset.

Ten fresh processes per cell, each hashing 21 times. First-call medians include any Rayon pool initialization (four threads); warm medians use the remaining 20 hashes. File reading is outside the timer. These are in-memory hash costs, not CLI timings.

| CPU flags | Algorithm | First hash ms | Warm hash ms |
|---|---|---:|---:|
| generic | sha256 | 0.5017 | 0.5035 |
| generic | blake3 | 0.6860 | 0.6759 |
| generic | blake3-rayon | 0.4771 | 0.3699 |
| rust-native | sha256 | 0.5019 | 0.5023 |
| rust-native | blake3 | 0.7071 | 0.6755 |
| rust-native | blake3-rayon | 0.4112 | 0.3687 |
