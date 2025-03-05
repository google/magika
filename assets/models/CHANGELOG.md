# Changelog

Here we document the main changes of the various models.

Indicated inference speed calculated by averaging 100 inferences (within one invocation) on an AMD Ryzen 9 7950X 16-Core Processor CPU.

## `standard_v3_1`

- [216 possible tool's outputs](./standard_v3_1/README.md).
- Overall same average accuracy of `standard_v3_0`, ~99%, but more robust detections of short textual input and improved detection of Javascript.
- Inference speed: ~2ms (similar to `standard_v3_0`).
- Augmentation techniques used during training: CutMix, which was used for `v1` but not for `v2_1`; and "Random Snippet Selection", with which we train the model with random snippets extracted from samples in our dataset (this is only enabled for key textual content types).

## `standard_v3_0`

- [216 possible tool's outputs](./standard_v3_0/README.md).
- Overall same average accuracy of `standard_v2_1`, ~99%.
- Inference speed: ~2ms (~3x faster than `standard_v2_1`, ~20% faster than `standard_v1`).

## `standard_v2_1`

- [Support for 200+ content types](./standard_v2_1/README.md), almost double what supported in `standard_v1`.
- Overall average accuracy of ~99%.
- Inference speed: ~6.2ms, which is slower than `standard_v1`; See `fast_v2_1` in case you need something faster (at the price of less accuracy).

## `fast_v2_1`

- Similar to `standard_v2_1`, but significantly faster (about 4x faster).
- Overall average accuracy of ~98.5%.


## `standard_v1`

- Initial release.
- Support for about 100 content types.
- Average accuracy 99%+.
- Inference speed: ~2.6ms.
