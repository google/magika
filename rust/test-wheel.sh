#!/bin/bash
# Copyright 2024 Google LLC
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

set -e
cd "$(dirname "$0")"
. ./color.sh

# This script tests the wheel in the maturin container.

cd ..
set -x
python3 -m ensurepip
python3 --version
python3 -m pip install dist/*.whl
magika --version
python3 -c 'import magika; print(magika.__version__)'
magika -r tests_data/basic
python3 ./python/scripts/run_quick_test_magika_cli.py
python3 ./python/scripts/run_quick_test_magika_module.py
