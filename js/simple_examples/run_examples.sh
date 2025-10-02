#!/bin/bash

# Exit on error
set -e

ROOT_DIR=$(pwd)
export TF_CPP_MIN_LOG_LEVEL=2
export NODE_OPTIONS='--no-warnings'

EXAMPLES=("node-commonjs-example" "node-esmodule-example" "browser-esmodule-example" "typescript-esmodule-example")

for example in "${EXAMPLES[@]}"
do
  echo "--- Running Example: $example ---"
  if [ -d "$example" ]; then
    cd "$example"
    if [ -f "package.json" ]; then
      npm run --silent start
    else
      echo "No package.json found in $(pwd)"
    fi
    cd "$ROOT_DIR"
  else
    echo "Directory $example not found"
  fi
done

