# Rust metadata generation

Run `./sync.sh` from `rust/` to regenerate model configuration, content types and
CLI examples; `./sync.sh --check` checks for differences.

`model/config.min.json` supplies model configuration. Content-type metadata comes
from `assets/content_types_kb.min.json`; selected binary labels absent from the
knowledge base use `rules/content-types.json`. Extra output labels do not add ML
classes or enable rules. Fix shared MIME/group metadata in the knowledge base.

Consumers compile the committed generated sources. Cargo's rule build step
assembles the maintained YARA files and checks their syntax/metadata; it does not
read datasets, download models or compile native rule databases.

Inference artifacts are maintained separately by the
[model release tools](../tract-bench/README.md#model-release).
