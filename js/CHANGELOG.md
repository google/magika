# CHANGELOG

## [0.3.0-dev]

- Overhaul of the API to use much cleaner abstractions.
- Removed identifyBytesFull and identifyStreamFull: identifyBytes and
  identifyStream now return all the scores as well (accessible with
  `result.prediction.scores_map`).
- Restrict the input types to `Uint8Array` and `Buffer`.

## [0.2.13] - 2024-03-26

- This is the first working (but still very experimental) version.
