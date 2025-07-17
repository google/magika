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
. ../color.sh

# This script builds ONNX Runtime as a static library to be linked in the Magika CLI.
#
# This is needed when building for manylinux since the prebuilt binaries provided by the ort crate
# have too recent dependency requirements.

if [ -e runtime ]; then
  info "Using cached static libraries."
else
  info "Make sure we have Python 3.x and cmake-3.27 or higher."
  python3 -m venv venv
  source venv/bin/activate
  python3 -m pip install cmake==3.31.6

  info "Clone ONNX Runtime repository (recursively)."
  git clone --recursive https://github.com/Microsoft/onnxruntime.git runtime
  cd runtime

  info "Checkout v1.22.1 because ort v2.0.0-rc.10 needs v1.22.0 but it's broken."
  # See https://github.com/microsoft/onnxruntime/issues/25098
  git checkout v1.22.1

  # The build fails with GCC 14 due to warnings as errors.
  sed -i '/function(onnxruntime_set_compile_flags/a\
    target_compile_options(${target_name} PRIVATE "$<$<COMPILE_LANGUAGE:CXX>:-Wno-maybe-uninitialized>")\
    target_compile_options(${target_name} PRIVATE "$<$<COMPILE_LANGUAGE:CXX>:-Wno-uninitialized>")' \
    cmake/CMakeLists.txt

  info "Build the static libraries."
  x ./build.sh --config=Release --parallel $ONNX_RUNTIME_BUILD_FLAGS

  info "Only keep the static libraries to save cache space."
  find build/Linux -not -name '*.a' \( -not -type d -or -empty \) -delete
  cd ..
fi

info "Point the ort crate to the locally built static library."
cd ../..
cat >> .cargo/config.toml <<EOF

[env]
ORT_LIB_LOCATION = "$PWD/rust/onnx/runtime/build/Linux"
EOF
