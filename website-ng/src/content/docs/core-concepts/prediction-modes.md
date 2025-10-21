---
title: Prediction Modes
---

Magika's deep learning model returns each prediction with a confidence score (from 0.0 to 1.0). A common challenge with classification models is determining the minimum score required to trust a result.

Instead of a single, global threshold, Magika uses **per-content-type thresholds**. The rationale is that the model is naturally more confident about some types than others. For example, our experiments show that most valid PDFs are detected with over 99% confidence, so a prediction with an 80% score might be questionable. In contrast, an 80% score for a JavaScript file is often a very reliable prediction.

Magika manages these confidence levels in two ways:
- **Pre-tuned Thresholds:** Each model ships with carefully tuned, per-content-type thresholds derived from evaluating the model on our large validation dataset.
- **Prediction Modes:** Because the impact of a misidentification varies by use case, Magika allows you to select a prediction mode. This lets you balance precision (accuracy of predictions) and recall (number of identified files). The available modes are `high-confidence`, `medium-confidence`, and `best-guess`.

The `high-confidence` mode offers higher precision at the cost of lower recall. In contrast, `best-guess` provides the highest recall—potentially with lower precision—as it returns the model's prediction regardless of its confidence score. This can be selected via a command-line flag or as an option in the language bindings.