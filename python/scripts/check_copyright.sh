#!/bin/bash
# Copyright 2023-2024 Google LLC
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     https://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.


# Taken from https://github.com/google/scaaml/blob/main/tools/check_copyright.sh


errors=0
e() {
  echo -e "$(tput bold)$(tput setaf 1)Error:$(tput sgr0) $*"
  errors=$(( $error + 1 ))
}

# Files we want to check for copyright
EXTENSIONS="py\|sh"


for file in $(git ls-files | \
  grep -e '\.\('"${EXTENSIONS}"'\)$')
do
  sed -n 'N;/Copyright/q;q1' $file || e "No copyright notice in $file"
done

if [ $errors -gt 0 ]
then
  exit 1
fi
exit 0
