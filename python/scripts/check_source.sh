#!/bin/bash

# From https://stackoverflow.com/a/246128
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

PYTHON_ROOT_DIR=$SCRIPT_DIR/..

pushd $PYTHON_ROOT_DIR > /dev/null

echo "Running ruff..."
ruff check

echo "Running mypy..."
mypy magika tests

popd > /dev/null
