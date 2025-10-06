---
title: Models & Supported Content Types
---

Each Magika model is trained to detect a specific set of content types. Newer models typically expand this set, supporting a superset of the content types from previous versions, unless specified otherwise.

For instance, our initial `standard_v1` model supported approximately 100 content types. The latest model, `standard_v3_3`, supports over 200, while maintaining similar accuracy and inference speed.

The list of supported content types for each model is documented in its README file on GitHub. For example, the documentation for `standard_v3_3` is [here](https://github.com/google/magika/blob/main/assets/models/standard_v3_3/README.md#list-of-possible-models-outputs).

:::tip
The models' READMEs contain two lists: "the output space of the model" and "the output space of Magika the tool." The second list is a superset of the first, including additional labels like `empty`, `directory`, and so on.
:::

Details on the improvements and tradeoffs for each model can be found in the models' CHANGELOG on GitHub: [models/CHANGELOG.md](https://github.com/google/magika/blob/main/assets/models/CHANGELOG.md).

Clients and bindings usually integrate the latest available model, but this may not always be the case. For more information, check the [bindings section](/cli-and-bindings/overview).

:::caution
You may see a "content types knowledge base" (KB) in the GitHub source code. This is a comprehensive list of all content types we track internally for research and development. It should **not** be confused with the content types Magika currently supports. The KB is a superset of what any single model supports. To be certain which content types a specific model supports, always refer to its README file.
:::
