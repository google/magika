# Tests Data

We use these files for regressions testing.

These files were not (and should not) be used for training purposes.

They are organized by directory:
- `basic/`: a number of simple files of various content types.
- `mitra/`: a selection of the files available at [https://github.com/corkami/mitra](https://github.com/corkami/mitra/tree/master/input).
- `resolvers/`: test inputs for native format resolvers:
  - `resolvers/fixtures/`: minimal, shareable, PII-free header fixtures organized into `<category>/` (standalone content types) and `_<metaformat>/` (container/framing families and subformats). Unprefixed files (`<name>.<ext>`) are hollowed and truncated from real files (with unread payload bytes zeroed); `"!"`-prefixed files (`!<name>.<ext>`) are synthesized from format specifications when no redistributable wild sample is available, or forcibly synthesized whenever wild samples carry PII or security risk.
  - `resolvers/positive/`, `resolvers/negative/`, and `resolvers/regressions.json`: targeted positive/negative edge cases and recorded resolver regression vectors.
