# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Observed TrID acceleration, independent of user-supplied configuration labels."""

import re


def enabled(version_output):
    matches = re.findall(r"^\s*Using Stringzilla: (True|False)\s*$", version_output, re.M)
    return matches[0] == "True" if len(matches) == 1 else None


MODULE_PROBE = (
    "import sys,json; sys.path.insert(0,sys.argv[1]); import stringzilla; "
    "print(json.dumps({'version':stringzilla.__version__,'path':stringzilla.__file__}))"
)
