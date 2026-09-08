# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Build, hydrate and inspect reproducible datasets."""

import argparse
import sys


def main(argv=None):
    argv = list(sys.argv[1:] if argv is None else argv)
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=["build", "hydrate", "stats", "archive", "validate"])
    if not argv or argv[0] in ("-h", "--help"):
        parser.print_help()
        return
    command = parser.parse_args(argv[:1]).command
    if command == "build":
        from .pipeline import main as run
    elif command == "hydrate":
        from .hydrate import main as run
    elif command == "validate":
        from .validation import main as run
    elif command == "archive":
        from .archive import main as run
    else:
        from .stats import main as run
    return run(argv[1:])
