---
title: Models & Supported Content Types
---

Magika identifies files through a multi-stage detection pipeline combining **built-in heuristics** (filesystem path checks, empty-file detection, and short-file or low-confidence fallbacks), **native format resolvers**, and a **deep learning model**.

Because multiple pipeline stages contribute to the final prediction, Magika's **supported output content types** (`output.label`) are documented separately from individual **deep learning models** (`dl.label`):

- Adding or expanding native format resolvers on `main` broadens Magika's output space without modifying the underlying ML model.
- Different artifacts and released versions using the same ML model (`standard_v3_3`) can have different output spaces—for example, resolver additions on `main` ahead of package releases, or the pure-TypeScript TensorFlow.js package running byte/stream heuristics and the ML model without filesystem `stat` checks or native Rust format resolvers.

## Supported Content Types by Version

The [**Supported Content Types by Version**](/magika/content-types) index links one reference page per branch or released package: the unreleased [`main` branch](/magika/content-types/main), and a frozen snapshot for each release of the CLI, Rust library, Python module, and JavaScript package. Each page lists every content type that version can emit in `output.label`, together with its metadata (`mime_type`, `group`, `description`, `extensions`, `is_text`) and orthogonal stage-coverage flags (`in_heuristics`, `in_resolvers`, `in_ml_model`, `vetoed_by_resolvers`).

At runtime, you can also query an artifact's supported outputs programmatically via `TypeInfo::possible_output()` in Rust, `Magika.get_output_content_types()` in Python, or `magika.getOutputContentTypes()` in JavaScript.

## Deep Learning Models

Each Magika deep learning model is trained to classify a specific set of target content types (`target_labels_space`). For instance, our initial `standard_v1` model supported ~100 content types, while [`standard_v3_3`](/magika/models/standard_v3_3) covers over 200 content types while maintaining ~99% average accuracy and millisecond CPU inference.

Model documentation pages cover strictly the model itself—its trained target classes (`dl.label`), feature extraction window, per-label confidence thresholds, and `overwrite_map` remappings:

- Current default model: [`standard_v3_3` model documentation](/magika/models/standard_v3_3) (GitHub: [`assets/models/standard_v3_3/README.md`](https://github.com/google/magika/blob/main/assets/models/standard_v3_3/README.md))
- Model release history and tradeoffs: [`assets/models/CHANGELOG.md`](https://github.com/google/magika/blob/main/assets/models/CHANGELOG.md)

Clients and bindings integrate a specific default model version; see the [CLI & Bindings Overview](/magika/cli-and-bindings/overview) for a summary across artifacts.

:::caution
You may see a "content types knowledge base" ([`assets/content_types.yaml`](https://github.com/google/magika/blob/main/assets/content_types.yaml)) and an exposed content types list ([`assets/exposed_content_types.txt`](https://github.com/google/magika/blob/main/assets/exposed_content_types.txt)) in the GitHub repository. The knowledge base is a broader catalog of formats tracked for research and metadata, and the language-level `ContentType` / `ContentTypeLabel` enums are monotonic supersets of what any single Magika version outputs. To check which content types a specific Magika version or model can actually return, always consult the [per-version index](/magika/content-types) (or the model's page for raw `dl.label` values).
:::
