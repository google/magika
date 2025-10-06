---
title: How Magika Works
---

Magika's command-line tool and language bindings are essentially wrappers around a compact deep learning model, optimized for efficient inference on standard CPUs.

The identification process is highly efficient because Magika primarily inspects a few hundred bytes of a file (depending on the model, usually up to 2K bytes). This approach ensures a fast, constant-time inference that is independent of the overall file size.

The core process works as follows:

1. Magika reads a few chunks of the input file (or byte stream). This is fast and memory efficient even for big files, as they are never fully read in memory.
2. It extracts "features" from these initial bytes, which are then processed by the deep learning model to predict the content type.
3. After the model makes a prediction, Magika evaluates its confidence score.
4. If this score exceeds a predefined threshold for the predicted type, Magika accepts the model's prediction. If the confidence is too low, Magika returns a more generic label, such as `txt` (for text files) or `unknown` (for binary files).

This distinction is important: Magika internally manages two content type labels—one from the deep learning model and one from **"Magika the tool."** While they are often the same, they can differ when the model's confidence is low or in certain edge cases.

The model is not used in all situations. Specifically:
- If the input file is **empty**, Magika returns `empty`.
- If the input is not a regular file, such as a **directory** or a **symlink**, Magika returns `directory` or `symlink`.
- If the file is **too small** for the model (e.g., under ~8 bytes), Magika uses simple heuristics to return a generic answer like `txt` or `unknown`.

In these cases, the model is not run, and its internal content type label is set to `undefined`. By default, users only see the final, processed prediction, but the model's raw output can be inspected for debugging. See the [Understanding the Output](/core-concepts/understanding-the-output) section for details.