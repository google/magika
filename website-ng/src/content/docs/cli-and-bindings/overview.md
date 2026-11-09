---
title: CLI & Bindings Overview
---

Magika provides a native CLI for command-line use and official language bindings—including Python, JavaScript, and Rust—to enable seamless integration into diverse projects.


| Artifact | Status | Latest version | Default model | Supported outputs |
| --- | --- | --- | --- | --- |
| [`magika` CLI](/magika/cli-and-bindings/cli) | Stable | `1.1.0` | [`standard_v3_3`](/magika/models/standard_v3_3) | [216 content types](/magika/content-types/cli-v1_1_0) |
| [Python `Magika` module](/magika/cli-and-bindings/python) | Stable | `1.0.3` | [`standard_v3_3`](/magika/models/standard_v3_3) | [216 content types](/magika/content-types/python-v1_0_3) |
| [JavaScript / TypeScript package](/magika/cli-and-bindings/js) | Stable | `1.0.0` | [`standard_v3_3`](/magika/models/standard_v3_3) | [214 content types](/magika/content-types/js-v1_0_0) |
| [Rust `magika` library](/magika/cli-and-bindings/rust) | Stable | `1.1.0` | [`standard_v3_3`](/magika/models/standard_v3_3) | [216 content types](/magika/content-types/rust-v1_1_0) |
| [Demo Website](/magika/demo/magika-demo/) | Stable | - | [`standard_v3_3`](/magika/models/standard_v3_3) | [214 content types](/magika/content-types/js-v1_0_0) |
| [Other bindings](/magika/cli-and-bindings/other-bindings) | WIP | - | - | - |

Even when using the same default ML model, different artifacts and versions can have different output spaces depending on their active heuristics and native format resolvers. See [Supported Content Types by Version](/magika/content-types) for all releases and the unreleased [`main` branch coverage reference](/magika/content-types/main).
