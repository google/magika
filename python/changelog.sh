#!/bin/bash
set -e

CHANGED_FILES=$(git diff --name-only origin/main...HEAD)

PYTHON_CODE_CHANGED=$(echo "$CHANGED_FILES" | grep -E '^python/.*\.py$' || true)
CHANGELOG_CHANGED=$(echo "$CHANGED_FILES" | grep -E '^python/CHANGELOG\.md$' || true)

if [[ -n "$PYTHON_CODE_CHANGED" && -z "$CHANGELOG_CHANGED" ]]; then
  echo "::warning title=Changelog Missing::Some changes in the Python package are not documented in python/CHANGELOG.md"
fi
