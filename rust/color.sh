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

x() { ( set -x; "$@"; ); }

color() { echo "[$1m$2:[m $3"; }
info() { color '1;36' Info "$*"; }
todo() { color '1;33' Todo "$*"; }
success() { color '1;32' Done "$*"; exit 0; }
error() { color '1;31' Error "$*"; exit 1; }
