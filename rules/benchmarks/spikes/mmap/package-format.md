# Compiled Magika data package: initial design

Ship one immutable, versioned data file containing the compiled model, compiled
rules, labels and provenance. Compilation happens during release packaging or
when a user changes the rules. Runtime selects sections without rebuilding the
whole package. This is a format direction, not an implemented model loader.

## Proposed layout

A fixed little-endian header carries magic, container version, package length,
section count and a bounded section table. Each entry contains a typed section ID,
payload-format version, 64-bit offset and length, required alignment, compatibility
identifier and digest. Offsets are relative to the start of the file. Check bounds,
overlaps, alignment and supported required sections before exposing any slice to C.

Initial sections:

| Section | Contents | When accessed |
|---|---|---|
| Identity | Package ID, source/compiler/runtime IDs, content taxonomy version | Open |
| Labels | Rule output IDs and model output labels mapped to the same taxonomy | Selected detector |
| Rules image | Scan-ready Vectorscan image, uncompressed and aligned | Rules enabled |
| Rules source | Optional editable source and attribution | Export or rule compilation |
| Model graphs | Existing prepared CPU and GPU graph variants, indexed by batch class | ML requested |
| Model weights | Aligned tensor data with shapes, types and offsets | ML requested |
| Notices | Source and dependency attribution | Inspection/export |

Use section-relative offsets and an alignment contract; 64 KiB section boundaries
are a conservative starting point across current target mapping granularities.
The spike's rules payload only needs 64-byte native alignment and sits at a 4 KiB
file offset. Package compatibility and native image compatibility are separate:
the wrapper is portable, while a native image is tied to its engine build, ABI,
endianness, architecture and supported CPU features. One package may contain
multiple explicitly tagged variants; unsupported targets fail or rebuild from
source, never guess a compatible native representation.

## Independent loading

Rules-only opens metadata and the rules section. It does not parse model graphs,
construct tensors, initialize tract/Metal, or verify the entire model payload on
every invocation. Validate the container metadata and the sections actually used;
packaging/install verification may cover the complete package. Keep a mapping
owner alive through every database, tensor and worker borrowing it.

The existing model artifact is already prepared graph JSON plus binary weights,
but `artifact::Bundle` borrows `&'static [u8]`, reconstructs tract graphs, and builds
runtime tensors. Combining files alone does not make this model directly runnable
from mmap. A model-loading follow-up must introduce owned mapping lifetimes and
establish which tensor storage and prepared runtime state can be reused without
copying. Device allocations and transfers remain separate from disk packaging.

## Build and update behavior

Release packaging creates the final aligned model/rules sections once. Bundle
identity is derived from content and compiler/runtime inputs, not filenames.
Custom rule edits produce a new rules section and a cached composed package that
reuses the unchanged model sections. Building that package may copy or reflink
those sections once; it does not repeat on startup. An optional rules-only cache
can avoid duplicating model bytes when a composed package is unnecessary.

Write to a private temporary inode, validate, then atomically publish. Existing
processes retain their old mapping. Never truncate or rewrite a mapped inode.
Bound the format and map only trusted installed/cache artifacts; checksums detect
corruption, not authenticity. Keep compiler access separate from the runtime
library so normal launches need not load Vectorscan's compiler dependencies.

## Compilation optimization opportunities

The current Vectorscan compiler already receives detected CPU tuning/features and
compiles the whole enabled rule program together. Mmap is a loading optimization,
not an additional matching optimization. Candidate packaging work to measure:

- Precompute source/compiler identities and compact rule-to-label metadata.
- Emit target-specific database variants using supported compiler tuning knobs.
- Precompute model graph transformations and CPU weight packing where the runtime
  exposes a stable reusable representation.
- Keep predicates unchanged and compare decisions plus sustained throughput before
  accepting any compiler transformation. A faster startup does not establish
  faster matching or equivalent model predictions.

Unresolved before production: native ABI/build fingerprinting, model mapping
ownership, cross-platform image tests, package discovery/fallback policy, signing
integration and reproducible binary output across build hosts. The rules mmap
spike tests the central image-lifetime and relocation premise independently.
