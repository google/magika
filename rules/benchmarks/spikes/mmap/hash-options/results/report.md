# Hash compiler-option audit

The same manifest and native payload bytes are hashed by every variant. Both use release opt-level=3, LTO and one codegen unit. BLAKE3 NEON and SHA-256 ARM assembly are enabled. Native additionally sets Rust target-cpu=native and C -mcpu=native.

Ten fresh processes per cell, each hashing 21 times. First-call medians include any Rayon pool initialization (four threads); warm medians use the remaining 20 hashes. File reading is outside the timer. These are in-memory hash costs, not CLI timings.

| CPU flags | Algorithm | First hash ms | Warm hash ms |
|---|---|---:|---:|
| generic | sha256 | 0.5025 | 0.5012 |
| generic | blake3 | 0.6773 | 0.6702 |
| generic | blake3-rayon | 0.4487 | 0.3657 |
| native | sha256 | 0.4974 | 0.5005 |
| native | blake3 | 0.6909 | 0.6722 |
| native | blake3-rayon | 0.4195 | 0.3289 |
