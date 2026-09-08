This maintenance crate regenerates two Rust source files from the selected model's
configuration and content-type metadata:

- `rust/lib/src/model.rs` contains model configuration and output-label mappings,
  generated from `rust/gen/model/config.min.json`.
- `rust/lib/src/content.rs` contains file-type metadata from
  `assets/content_types_kb.min.json`. Explicitly selected binary labels in
  `rust/gen/content_types` that are absent from the knowledge base use
  `rules/content-types.json`. Additional output identities do not add model classes
  or enable rules, and the library does not read this metadata source at runtime.

`rust/gen/model` selects the model configuration. Run `./sync.sh` from `rust/` to
regenerate these files and the CLI examples; `./sync.sh --check` checks the generated
results for differences. These sources are committed before publication.

Inference artifacts live separately in `rust/tract-runtime/models/`. The existing
[conversion script](../tract-bench/scripts/convert-model.sh) converts the checked ONNX
source into the NNEF reference, portable graph, weights and probe used by the runtime's
release workflow. `rust/gen` does not regenerate these inference artifacts.

Consumers compile the committed generated Rust sources. The library's `build.rs`
assembles the maintained YARA sources and, with `yara-rules`, checks their syntax and
metadata. It does not run model conversion, read the evaluation corpus or download
a model. Native rule compilation occurs when a compatible compiled pack is unavailable
at runtime, and requires Vectorscan.
