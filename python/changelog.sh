#!/bin/bash
set -euo pipefail

CHANGED_FILES=$(git diff --name-only "origin/${1:-main}"...HEAD)

if echo "$CHANGED_FILES" | grep -qE '^python/.*\.py$'; then
  if ! echo "$CHANGED_FILES" | grep -qE '^python/CHANGELOG\.md$'; then
    echo "::warning title=Changelog Missing::Some changes in the Python package are not documented in python/CHANGELOG.md"
  fi
fi
