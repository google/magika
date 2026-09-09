# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Correct cargo-dist 0.31's library chmod path before publishing installers.

The installer moves libraries to _lib_install_temp, but chmods the final path
before moving them there. A clean installation therefore fails. This also affects
upstream 0.32. Remove this workaround when the pinned template is fixed.
"""

import sys
from pathlib import Path


def fix(path):
    old = 'ensure chmod +x "$_lib_install_dir/$_lib_name"'
    new = 'ensure chmod +x "$_lib_install_temp/$_lib_name"'
    source = path.read_text()
    if source.count(old) != 1:
        raise ValueError(
            "cargo-dist installer changed; revalidate the chmod workaround"
        )
    path.write_text(source.replace(old, new))


if __name__ == "__main__":
    fix(Path(sys.argv[1]))
