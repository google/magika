# Changelog

## 0.1.0-rc.5-dev

### Minor

- Use the `standard_v3_1` model instead of `standard_v3_0` (see [model changelog])

## 0.1.0-rc.4

### Minor

- Update the model thresholds

## 0.1.0-rc.3

### Minor

- Use the `standard_v3_0` model instead of `standard_v2_1` (see [model changelog])
- Add content types `ContentType::Random{bytes,txt}` (those are only returned in
  `InferredType::content_type` and not in `RuledType::content_type`)
- Add a `MODEL_MAJOR_VERSION` integer in addition to the `MODEL_NAME` string

### Patch

- Update dependencies

## 0.1.0-rc.2

### Patch

- Update dependencies

## 0.1.0-rc.1

### Minor

- Change model from `standard_v2_0` to `standard_v2_1`

## 0.1.0-rc.0

This version is the initial implementation and should be considered unstable. In particular, it
ships a new model in comparison to the Python binary and we would love feedback on
[GitHub](https://github.com/google/magika/issues).

## 0.0.0

This version is a placeholder and does not expose anything.

[model changelog]: https://github.com/google/magika/blob/main/assets/models/CHANGELOG.md
